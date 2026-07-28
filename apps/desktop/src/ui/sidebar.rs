use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use std::io::BufRead;
use std::ops::Range;

use gpui::{
    div, prelude::*, px, Bounds, ClickEvent, Context, EntityInputHandler, FocusHandle, Focusable,
    IntoElement, KeyDownEvent, ParentElement, Pixels, Point, Render, SharedString, Styled,
    UTF16Selection, Window,
};

use crate::state::EditorState;
use crate::ui::theme::Theme;

#[derive(Clone, Copy, PartialEq)]
enum SidebarTab {
    Media,
    VideoGen,
    Studio,
    Effects,
    Properties,
}

struct VideoGenState {
    status: String,
    progress: f32,
    prompt: String,
    duration: u32,
    style: String,
    platform: String,
    language: String,
    output_path: Option<String>,
    job_id: Option<String>,
    is_generating: bool,
}

impl Default for VideoGenState {
    fn default() -> Self {
        Self {
            status: "Ready".to_string(),
            progress: 0.0,
            prompt: String::new(),
            duration: 30,
            style: "cinematic".to_string(),
            platform: "youtube".to_string(),
            language: "id".to_string(),
            output_path: None,
            job_id: None,
            is_generating: false,
        }
    }
}

pub struct Sidebar {
    editor: Rc<RefCell<EditorState>>,
    active_tab: SidebarTab,
    gen_state: Arc<Mutex<VideoGenState>>,
    prompt_input: String,
    focus_handle: FocusHandle,
    cursor_offset: usize,
    studio_prompt: String,
    
}

const STYLES: &[&str] = &["cinematic", "vlog", "tutorial", "product", "music"];
const PLATFORMS: &[&str] = &["youtube", "tiktok", "instagram", "reels", "shorts"];
const DURATIONS: &[u32] = &[15, 30, 45, 60, 90, 120];

impl Sidebar {
    pub fn new(editor: Rc<RefCell<EditorState>>, cx: &mut Context<Self>) -> Self {
        Self {
            editor,
            active_tab: SidebarTab::Media,
            gen_state: Arc::new(Mutex::new(VideoGenState::default())),
            prompt_input: String::new(),
            focus_handle: cx.focus_handle(),
            cursor_offset: 0,
            studio_prompt: String::new(),
            
        }
    }

    fn start_generation(gen_state: &Arc<Mutex<VideoGenState>>) {
        let state_clone = gen_state.clone();
        let prompt;
        let duration;
        let style;
        let platform;
        let language;
        {
            let state = state_clone.lock().unwrap();
            prompt = state.prompt.clone();
            duration = state.duration;
            style = state.style.clone();
            platform = state.platform.clone();
            language = state.language.clone();
        }

        if prompt.trim().is_empty() {
            return;
        }

        {
            let mut state = state_clone.lock().unwrap();
            state.is_generating = true;
            state.progress = 0.0;
            state.status = "Starting...".to_string();
            state.output_path = None;
            state.job_id = None;
        }

        std::thread::spawn(move || {
            let url = "http://localhost:8765/api/generate";
            let payload = serde_json::json!({
                "prompt": prompt,
                "duration": duration,
                "style": style,
                "platform": platform,
                "language": language,
            });

            let resp = match ureq::post(url).send_json(&payload) {
                Ok(r) => r,
                Err(_) => {
                    let mut s = state_clone.lock().unwrap();
                    s.status = "Error: AI Engine Offline".to_string();
                    s.is_generating = false;
                    s.progress = 0.0;
                    return;
                }
            };

            let body: serde_json::Value = match resp.into_json() {
                Ok(b) => b,
                Err(_) => {
                    let mut s = state_clone.lock().unwrap();
                    s.status = "Error: Invalid response from API".to_string();
                    s.is_generating = false;
                    return;
                }
            };

            let job_id = body["job_id"].as_str().unwrap_or("").to_string();
            let poll_url = format!("http://localhost:8765/api/generate/{}", job_id);

            {
                let mut s = state_clone.lock().unwrap();
                s.job_id = Some(job_id.clone());
                s.status = "Generating EDL...".to_string();
                s.progress = 0.05;
            }

            loop {
                std::thread::sleep(Duration::from_millis(1000));

                let poll_resp = match ureq::get(&poll_url).call() {
                    Ok(r) => r,
                    Err(_) => {
                        let mut s = state_clone.lock().unwrap();
                        s.status = "Polling error".to_string();
                        break;
                    }
                };

                let poll_body: serde_json::Value = match poll_resp.into_json() {
                    Ok(b) => b,
                    Err(_) => {
                        let mut s = state_clone.lock().unwrap();
                        s.status = "Failed to parse poll data".to_string();
                        break;
                    }
                };

                let poll_status = poll_body["status"].as_str().unwrap_or("unknown").to_string();
                let progress = poll_body["progress"].as_f64().unwrap_or(0.0) as f32;

                {
                    let mut s = state_clone.lock().unwrap();
                    s.status = format!("{} ({:.0}%)", poll_status, progress * 100.0);
                    s.progress = progress;
                }

                match poll_status.as_str() {
                    "completed" => {
                        let output_path = poll_body["output_path"].as_str().map(|s| s.to_string());
                        let mut s = state_clone.lock().unwrap();
                        s.status = "Completed!".to_string();
                        s.progress = 1.0;
                        s.output_path = output_path;
                        s.is_generating = false;
                        break;
                    }
                    "failed" => {
                        let error = poll_body["error"].as_str().unwrap_or("Unknown error");
                        let mut s = state_clone.lock().unwrap();
                        s.status = format!("Failed: {}", error);
                        s.is_generating = false;
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    fn open_video(output_path: &str) {
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(output_path)
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(output_path)
                .spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "", output_path])
                .spawn();
        }
    }

    /// Spawn a background thread to send user message to AI and get reply.
    fn send_chat_message(
        editor: &Rc<RefCell<EditorState>>,
        user_text: String,
    ) {
        use crate::state::editor::{ChatSender, AiConversationState};
        editor.borrow_mut().push_chat(ChatSender::User, user_text.clone());

        let pending_arc = { editor.borrow().pending_chat_reply.clone() };
        let ai_state_str = editor.borrow().ai_state.as_str().to_string();
        let plan_json = editor.borrow().pending_plan_json.clone();
        let episodes: Vec<serde_json::Value> = editor.borrow().story_episodes.iter().map(|ep| {
            serde_json::json!({ "part_number": ep.part_number, "title": ep.title, "summary": ep.summary })
        }).collect();

        let input = serde_json::json!({
            "user_message": user_text,
            "ai_state": ai_state_str,
            "story_context": serde_json::from_str::<serde_json::Value>(&plan_json).unwrap_or_default(),
            "past_episodes": episodes,
        });
        let input_str = input.to_string();

        std::thread::spawn(move || {
            let script_path = Self::gemini_chat_path();
            let child_result = std::process::Command::new(Self::python_path())
                .arg(&script_path)
                .arg("reply")
                .arg(&input_str)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit())
                .spawn();
            let output = match child_result {
                Ok(mut child) => {
                    let timeout = std::time::Duration::from_secs(180);
                    let start = std::time::Instant::now();
                    loop {
                        if start.elapsed() > timeout {
                            eprintln!("[Chat] Timeout (180s) — killing Python subprocess");
                            let _ = child.kill();
                            let _ = child.wait();
                            let fallback = serde_json::json!({
                                "ai_reply": "Maaf, koneksi AI timeout. Coba lagi?",
                                "next_state": ai_state_str,
                                "story_context": {},
                                "quick_replies": ["Coba lagi"]
                            });
                            if let Ok(mut guard) = pending_arc.lock() {
                                *guard = Some(fallback.to_string());
                            }
                            return;
                        }
                        match child.try_wait() {
                            Ok(Some(_)) => break,
                            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                            Err(e) => {
                                eprintln!("[Chat] Error waiting for child: {}", e);
                                let fallback = serde_json::json!({
                                    "ai_reply": "Maaf, ada error sistem. Coba lagi?",
                                    "next_state": ai_state_str,
                                    "story_context": {},
                                    "quick_replies": ["Coba lagi"]
                                });
                                if let Ok(mut guard) = pending_arc.lock() {
                                    *guard = Some(fallback.to_string());
                                }
                                return;
                            }
                        }
                    }
                    child.wait_with_output()
                }
                Err(e) => {
                    eprintln!("[Chat] Failed to spawn gemini_chat.py: {}", e);
                    let fallback = serde_json::json!({
                        "ai_reply": "Maaf, ada gangguan koneksi AI. Coba lagi?",
                        "next_state": ai_state_str,
                        "story_context": {},
                        "quick_replies": ["Coba lagi"]
                    });
                    if let Ok(mut guard) = pending_arc.lock() {
                        *guard = Some(fallback.to_string());
                    }
                    return;
                }
            };
            match output {
                Ok(out) if out.status.success() => {
                    let json_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if let Ok(mut guard) = pending_arc.lock() {
                        *guard = Some(json_str);
                    }
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    eprintln!("[Chat] Error from gemini_chat.py: {}", &err[..err.len().min(200)]);
                    let fallback = serde_json::json!({
                        "ai_reply": "Maaf, ada gangguan koneksi AI. Coba lagi?",
                        "next_state": ai_state_str,
                        "story_context": {},
                        "quick_replies": ["Coba lagi"]
                    });
                    if let Ok(mut guard) = pending_arc.lock() {
                        *guard = Some(fallback.to_string());
                    }
                }
                Err(e) => {
                    eprintln!("[Chat] Failed to read output: {}", e);
                    let fallback = serde_json::json!({
                        "ai_reply": "Maaf, ada error sistem. Coba lagi?",
                        "next_state": ai_state_str,
                        "story_context": {},
                        "quick_replies": ["Coba lagi"]
                    });
                    if let Ok(mut guard) = pending_arc.lock() {
                        *guard = Some(fallback.to_string());
                    }
                }
            }
        });
    }

    /// Spawn a background thread to call gemini_chat.py greet and start the chat.
    fn start_ai_chat(editor: &Rc<RefCell<EditorState>>) {
        use crate::state::editor::{ChatSender, AiConversationState};
        // Only greet once
        if editor.borrow().ai_state != AiConversationState::Idle {
            return;
        }
        editor.borrow_mut().ai_state = AiConversationState::WaitingForStoryIdea;

        let pending_arc = { editor.borrow().pending_chat_reply.clone() };
        std::thread::spawn(move || {
            let script_path = Self::gemini_chat_path();
            let child_result = std::process::Command::new(Self::python_path())
                .arg(&script_path)
                .arg("greet")
                .arg("{}")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit())
                .spawn();
            let output = match child_result {
                Ok(mut child) => {
                    let timeout = std::time::Duration::from_secs(120);
                    let start = std::time::Instant::now();
                    loop {
                        if start.elapsed() > timeout {
                            eprintln!("[Chat] Timeout (120s) on greet");
                            let _ = child.kill();
                            let _ = child.wait();
                            break;
                        }
                        match child.try_wait() {
                            Ok(Some(_)) => break,
                            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                            Err(_) => break,
                        }
                    }
                    child.wait_with_output()
                }
                Err(_) => return, // fall through to fallback
            };
            let json_str = output.ok().and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });
            if let Some(s) = json_str {
                if let Ok(mut guard) = pending_arc.lock() {
                    *guard = Some(s);
                }
            } else {
                let fallback = serde_json::json!({
                    "ai_reply": "Halo! 🎬 Ceritakan ide cerita kamu!",
                    "next_state": "waiting_for_story_idea",
                    "story_context": {},
                    "quick_replies": ["Polisi vs Teroris", "Penculikan Anak", "Kejar-kejaran Mobil"]
                });
                if let Ok(mut guard) = pending_arc.lock() {
                    *guard = Some(fallback.to_string());
                }
            }
        });
    }

    /// Spawn background thread to generate the movie from the confirmed plan.
    pub fn trigger_generate(editor: &Rc<RefCell<EditorState>>) {
        use crate::state::editor::{ChatSender, AiConversationState};
        {
            let mut ed = editor.borrow_mut();
            if ed.is_generating_movie {
                eprintln!("[Studio] Already generating, ignoring duplicate trigger.");
                return;
            }
            ed.is_generating_movie = true;
            ed.movie_status = "AI Agents: memulai...".to_string();
            ed.ai_state = AiConversationState::Generating;
        }
        let pending_movie = { editor.borrow().pending_movie.clone() };
        let pending_progress = { editor.borrow().pending_progress.clone() };
        let plan_json = editor.borrow().pending_plan_json.clone();
        let episodes: Vec<serde_json::Value> = editor.borrow().story_episodes.iter().map(|ep| {
            serde_json::json!({ "part_number": ep.part_number, "title": ep.title, "summary": ep.summary })
        }).collect();

        let mut story_ctx: serde_json::Value =
            serde_json::from_str(&plan_json).unwrap_or_default();
        story_ctx["past_episodes"] = serde_json::json!(episodes);

        let input = serde_json::json!({ "story_context": story_ctx });
        let input_str = input.to_string();

        std::thread::spawn(move || {
            let script_path = Self::gemini_chat_path();
            eprintln!("[Studio] Generating episode via gemini_chat.py...");
            // Spawn child process for timeout control
            let mut child_result = std::process::Command::new(Self::python_path())
                .arg(&script_path)
                .arg("generate")
                .arg(&input_str)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
            // Spawn a reader thread to capture stderr progress in real-time
            if let Ok(ref mut child) = child_result {
                if let Some(stderr) = child.stderr.take() {
                    let progress_writer = pending_progress.clone();
                    std::thread::spawn(move || {
                        let reader = std::io::BufReader::new(stderr);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                let trimmed = line.trim().to_string();
                                if !trimmed.is_empty() {
                                    // Clean up for UI display
                                    let display = trimmed
                                        .trim_start_matches("[PROGRESS] ")
                                        .trim_start_matches("[DIRECTOR] ")
                                        .trim_start_matches("[Agent] ")
                                        .to_string();
                                    eprintln!("[Python] {}", trimmed);
                                    if let Ok(mut p) = progress_writer.lock() {
                                        *p = display;
                                    }
                                }
                            }
                        }
                    });
                }
            }
            let output = match child_result {
                Ok(mut child) => {
                    let timeout = std::time::Duration::from_secs(300);
                    let start = std::time::Instant::now();
                    loop {
                        if start.elapsed() > timeout {
                            eprintln!("[Studio] Timeout (300s) — killing Python subprocess");
                            let _ = child.kill();
                            let _ = child.wait();
                            // Write a fallback directly via pending_movie to unstick UI
                            let movie = animation::generate_cinematic_movie("cerita aksi cepat");
                            let parsed = animation::generate_stickman_script("aksi");
                            if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                            return;
                        }
                        match child.try_wait() {
                            Ok(Some(_)) => break,
                            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                            Err(e) => {
                                eprintln!("[Studio] Error waiting for child: {}", e);
                                let movie = animation::generate_cinematic_movie("cerita aksi cepat");
                                let parsed = animation::generate_stickman_script("aksi");
                                if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                                return;
                            }
                        }
                    }
                    child.wait_with_output()
                }
                Err(e) => {
                    eprintln!("[Studio] Failed to spawn gemini_chat.py: {}", e);
                    // Write fallback to unstick UI
                    let movie = animation::generate_cinematic_movie("cerita aksi cepat");
                    let parsed = animation::generate_stickman_script("aksi");
                    if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                    return;
                }
            };
            match output {
                Ok(out) if out.status.success() => {
                    let json_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    match serde_json::from_str::<serde_json::Value>(&json_str) {
                        Ok(val) if val.get("error").is_some() => {
                            eprintln!("[Studio] Gemini error: {}", val["error"]);
                            let movie = animation::generate_cinematic_movie("cerita aksi cepat");
                            let parsed = animation::generate_stickman_script("aksi");
                            if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                        }
                        _ => {
                            let movie_result = serde_json::from_str::<animation::CinematicMovie>(&json_str);
                            match movie_result {
                                Ok(movie) => {
                                    let parsed = animation::generate_stickman_script("aksi");
                                    eprintln!("[Studio] Episode generated: '{}' {:.0}s", movie.title, movie.total_duration);
                                    if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                                }
                                Err(e) => {
                                    eprintln!("[Studio] Parse error: {}. Using fallback.", e);
                                    let movie = animation::generate_cinematic_movie("cerita aksi cepat");
                                    let parsed = animation::generate_stickman_script("aksi");
                                    if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                                }
                            }
                        }
                    }
                }
                Ok(out) => {
                    eprintln!("[Studio] gemini_chat.py generate failed (exit {}).", out.status);
                    let movie = animation::generate_cinematic_movie("cerita aksi cepat");
                    let parsed = animation::generate_stickman_script("aksi");
                    if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                }
                Err(e) => {
                    eprintln!("[Studio] Failed to read output: {}", e);
                    let movie = animation::generate_cinematic_movie("cerita aksi cepat");
                    let parsed = animation::generate_stickman_script("aksi");
                    if let Ok(mut g) = pending_movie.lock() { *g = Some((movie, parsed)); }
                }
            }
        });
    }

    fn python_path() -> String {
        // Use .venv/bin/python if available, else system python3
        if std::path::Path::new(".venv/bin/python").exists() {
            ".venv/bin/python".to_string()
        } else if std::path::Path::new("./.venv/bin/python").exists() {
            "./.venv/bin/python".to_string()
        } else if std::path::Path::new("../.venv/bin/python").exists() {
            "../.venv/bin/python".to_string()
        } else {
            "python3".to_string()
        }
    }

    fn gemini_chat_path() -> std::path::PathBuf {
        // Try relative to exe directory first (works in production bundles)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let p = exe_dir.join("gemini_chat.py");
                if p.exists() { return p; }
                let p = exe_dir.join("apps/desktop/gemini_chat.py");
                if p.exists() { return p; }
            }
        }
        // Fallback to cwd-relative paths (development)
        let candidates = [
            std::path::PathBuf::from("apps/desktop/gemini_chat.py"),
            std::path::PathBuf::from("gemini_chat.py"),
        ];
        for p in &candidates {
            if p.exists() { return p.clone(); }
        }
        candidates[0].clone()
    }
}

impl Focusable for Sidebar {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for Sidebar {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let text = if self.active_tab == SidebarTab::Studio { &self.studio_prompt } else { &self.prompt_input };
        Some(text.chars().skip(range.start).take(range.end - range.start).collect())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let text = if self.active_tab == SidebarTab::Studio { &self.studio_prompt } else { &self.prompt_input };
        let offset = self.cursor_offset.min(text.len());
        Some(UTF16Selection {
            range: offset..offset,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        _range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab == SidebarTab::Studio {
            if text == "\n" || text == "\r" {
                // Enter = send the message
                let msg = self.studio_prompt.trim().to_string();
                if !msg.is_empty() {
                    Sidebar::send_chat_message(&self.editor, msg);
                    self.studio_prompt.clear();
                    self.cursor_offset = 0;
                    cx.notify();
                }
                return;
            }
            self.studio_prompt.push_str(text);
            self.cursor_offset = self.studio_prompt.len();
            cx.notify();
            return;
        }
        if text == "\n" || text == "\r" {
            Sidebar::start_generation(&self.gen_state);
            return;
        }
        self.prompt_input.push_str(text);
        self.cursor_offset = self.prompt_input.len();
        {
            let mut s = self.gen_state.lock().unwrap();
            s.prompt = self.prompt_input.clone();
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _range: Option<Range<usize>>,
        _new_text: &str,
        _new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        None
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let text = if self.active_tab == SidebarTab::Studio { &self.studio_prompt } else { &self.prompt_input };
        Some(self.cursor_offset.min(text.len()))
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let editor = self.editor.borrow();
        let media_count = editor.media_pool.assets.len();
        drop(editor);

        // ── Poll pending chat reply (written by background AI thread) ─────────
        let pending_chat_arc = { self.editor.borrow().pending_chat_reply.clone() };
        if let Ok(mut guard) = pending_chat_arc.try_lock() {
            if let Some(json_str) = guard.take() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    use crate::state::editor::{ChatSender, AiConversationState};
                    if let Some(reply) = val["ai_reply"].as_str() {
                        self.editor.borrow_mut().push_chat(ChatSender::AI, reply.to_string());
                    }
                    if let Some(ns) = val["next_state"].as_str() {
                        self.editor.borrow_mut().ai_state = AiConversationState::from_str(ns);
                    }
                    // Store updated story context as JSON
                    if let Some(ctx) = val.get("story_context") {
                        self.editor.borrow_mut().pending_plan_json = ctx.to_string();
                    }
                    // Update quick replies
                    if let Some(qr) = val["quick_replies"].as_array() {
                        let replies: Vec<String> = qr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        self.editor.borrow_mut().chat_quick_replies = replies;
                    }
                    cx.notify();
                }
            }
        }

        // Auto-start AI chat when user first opens Studio tab
        if self.active_tab == SidebarTab::Studio {
            Sidebar::start_ai_chat(&self.editor);
        }

        // Poll pending auto-continue from Preview (cross-render trigger)
        {
            let should_trigger = self.editor.borrow().pending_auto_continue;
            if should_trigger {
                self.editor.borrow_mut().pending_auto_continue = false;
                // Direct call on main thread (trigger_generate spawns its own bg thread)
                Sidebar::trigger_generate(&self.editor);
            }
        }

        div()
            .w(px(280.0))
            .h_full()
            .bg(Theme::bg_surface())
            .border_r_1()
            .border_color(Theme::border_subtle())
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(40.0))
                    .bg(Theme::bg_base())
                    .border_b_1()
                    .border_color(Theme::border_subtle())
                    .flex()
                    .items_center()
                    .px(px(8.0))
                    .gap(px(2.0))
                    .child(
                        tab_btn("Media", self.active_tab == SidebarTab::Media)
                            .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                this.active_tab = SidebarTab::Media;
                            })),
                    )
                    .child(
                        tab_btn("AI Gen", self.active_tab == SidebarTab::VideoGen)
                            .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                this.active_tab = SidebarTab::VideoGen;
                            })),
                    )
                    .child(
                        tab_btn("Studio", self.active_tab == SidebarTab::Studio)
                            .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                this.active_tab = SidebarTab::Studio;
                            })),
                    )
                    .child(
                        tab_btn("FX", self.active_tab == SidebarTab::Effects)
                            .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                this.active_tab = SidebarTab::Effects;
                            })),
                    )
                    .child(
                        tab_btn("Props", self.active_tab == SidebarTab::Properties)
                            .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                this.active_tab = SidebarTab::Properties;
                            })),
                    ),
            )
            .child(
                div()
                    .id("sidebar-content-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .p(px(12.0))
                    .child(match self.active_tab {
                        SidebarTab::Media => {
                            let editor = self.editor.borrow();
                            let panel = render_media_panel(&editor, media_count, cx);
                            drop(editor);
                            panel
                        }
                        SidebarTab::VideoGen => {
                            let gen_state = self.gen_state.clone();
                            let prompt_input = &mut self.prompt_input;
                            let fh = &self.focus_handle;
                            render_video_gen_panel(&gen_state, prompt_input, fh, cx)
                        }
                        SidebarTab::Studio => {
                            render_studio_chat_panel(&self.editor, &mut self.studio_prompt, &self.focus_handle, cx)
                        }
                        SidebarTab::Effects => render_effects_panel(),
                        SidebarTab::Properties => render_properties_panel(),
                    }),
            )
    }
}

fn tab_btn(label: &'static str, is_active: bool) -> gpui::Stateful<gpui::Div> {
    div()
        .id(label)
        .h(px(28.0))
        .px(px(8.0))
        .rounded(px(4.0))
        .flex()
        .items_center()
        .cursor_pointer()
        .text_size(px(11.0))
        .when(is_active, |d| {
            d.bg(Theme::bg_elevated())
                .text_color(Theme::text_primary())
        })
        .when(!is_active, |d| {
            d.text_color(Theme::text_secondary())
                .hover(|s| s.bg(Theme::bg_hover()))
        })
        .child(SharedString::from(label))
}

fn render_media_panel(
    editor: &EditorState,
    asset_count: usize,
    cx: &mut Context<'_, Sidebar>,
) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(Theme::text_primary())
                        .child(SharedString::from("Media Assets")),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(Theme::text_disabled())
                        .child(SharedString::from(format!("{} files", asset_count))),
                ),
        )
        .children(
            if asset_count == 0 {
                vec![div()
                    .w_full()
                    .h(px(100.0))
                    .bg(Theme::bg_elevated())
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(Theme::border_subtle())
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.0))
                    .text_color(Theme::text_disabled())
                    .child(SharedString::from("Drop media or use Import"))
                    .into_any_element()]
            } else {
                editor.media_pool.assets.iter().map(|asset| {
                    let icon = match asset.media_type {
                        media::MediaType::Video => "🎬",
                        media::MediaType::Image => "🖼",
                        media::MediaType::Audio => "🎵",
                    };
                    let name = SharedString::from(asset.name.clone());
                    let asset_id = SharedString::from(asset.id.clone());
                    let asset_name = asset.name.clone();
                    let asset_duration = asset.duration;
                    let media_type = asset.media_type;

                    div()
                        .id(asset_id)
                        .h(px(32.0))
                        .px(px(8.0))
                        .rounded(px(4.0))
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .bg(Theme::bg_elevated())
                        .hover(|s| s.bg(Theme::bg_hover()))
                        .cursor_pointer()
                        .child(div().text_size(px(12.0)).child(SharedString::from(icon)))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(Theme::text_primary())
                                        .child(name),
                                ),
                        )
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                            let mut editor = this.editor.borrow_mut();
                            let Some(scene) = editor.current_scene_mut() else { return };
                            let main_id = scene.tracks.main.id().to_string();
                            let element = create_element_from_asset(
                                &asset_name, asset_duration, media_type,
                            );
                            scene.insert_element(&main_id, element);
                            editor.mark_modified();
                        }))
                        .into_any_element()
                }).collect()
            },
        )
}

fn create_element_from_asset(
    name: &str,
    duration: Option<f64>,
    media_type: media::MediaType,
) -> timeline::TimelineElement {
    use timeline::*;
    let el_id = uuid::Uuid::new_v4().to_string();
    let dur = duration
        .and_then(time::MediaTime::from_seconds_f64)
        .unwrap_or(time::MediaTime::ZERO);

    let base = BaseTimelineElement {
        id: el_id,
        name: name.to_string(),
        duration: dur,
        start_time: time::MediaTime::ZERO,
        trim_start: time::MediaTime::ZERO,
        trim_end: dur,
        source_duration: Some(dur),
        animations: None,
        params: serde_json::Value::Object(Default::default()),
        effects: None,
        masks: None,
    };

    match media_type {
        media::MediaType::Video => TimelineElement::Video(VideoElementData {
            base,
            media_id: String::new(),
            is_source_audio_enabled: Some(true),
            hidden: Some(false),
            retime: None,
        }),
        media::MediaType::Image => TimelineElement::Image(ImageElementData {
            base,
            media_id: String::new(),
            hidden: Some(false),
        }),
        media::MediaType::Audio => TimelineElement::Audio(AudioElementData::Upload {
            base,
            media_id: String::new(),
            retime: None,
        }),
    }
}

fn render_video_gen_panel(
    state: &Arc<Mutex<VideoGenState>>,
    prompt_input: &mut String,
    focus_handle: &FocusHandle,
    cx: &mut Context<'_, Sidebar>,
) -> gpui::Div {
    let gen_state = state.lock().unwrap();
    let is_generating = gen_state.is_generating;
    let status = gen_state.status.clone();
    let progress = gen_state.progress;
    let output_path = gen_state.output_path.clone();
    let current_style = gen_state.style.clone();
    let current_platform = gen_state.platform.clone();
    let current_language = gen_state.language.clone();
    let current_duration = gen_state.duration;
    drop(gen_state);
    let state_arc = state.clone();

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .text_size(px(11.0))
                .text_color(Theme::text_primary())
                .child(SharedString::from("AI Video Generator")),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(Theme::text_disabled())
                .child(SharedString::from("Generate videos from text prompts")),
        )
        .child(field_label("Prompt"))
                        .child(
                            div()
                                .id("prompt-input")
                                .h(px(72.0))
                                .w_full()
                                .bg(Theme::bg_elevated())
                                .rounded(px(4.0))
                                .border_1()
                                .border_color(Theme::border_subtle())
                                .p(px(8.0))
                                .text_size(px(11.0))
                                .text_color(Theme::text_primary())
                                .cursor_text()
                                .track_focus(focus_handle)
                                .child(if prompt_input.is_empty() {
                                    SharedString::from("Type your video idea...")
                                } else {
                                    SharedString::from(prompt_input.clone())
                                })
                                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _w, _cx| {
                                    if event.keystroke.key == "backspace" {
                                        this.prompt_input.pop();
                                        let mut s = this.gen_state.lock().unwrap();
                                        s.prompt = this.prompt_input.clone();
                                    } else if event.keystroke.key == "return" || event.keystroke.key == "enter" {
                                        Sidebar::start_generation(&this.gen_state);
                                    } else if let Some(ch) = &event.keystroke.key_char {
                                        if ch.len() == 1 {
                                            this.prompt_input.push_str(ch);
                                            let mut s = this.gen_state.lock().unwrap();
                                            s.prompt = this.prompt_input.clone();
                                        }
                                    }
                                })),
                        )
        .child(
            div()
                .flex()
                .gap(px(8.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .flex_1()
                        .child(field_label("Duration"))
                        .child(
                            div()
                                .id("duration-display")
                                .h(px(28.0))
                                .bg(Theme::bg_elevated())
                                .rounded(px(4.0))
                                .flex()
                                .items_center()
                                .px(px(8.0))
                                .text_size(px(11.0))
                                .text_color(Theme::text_primary())
                                .cursor_pointer()
                                .child(SharedString::from(format!("{}s", current_duration)))
                                .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                                    let mut s = this.gen_state.lock().unwrap();
                                    let idx = DURATIONS.iter().position(|d| *d == s.duration);
                                    let next = idx.map(|i| (i + 1) % DURATIONS.len()).unwrap_or(0);
                                    s.duration = DURATIONS[next];
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .flex_1()
                        .child(field_label("Language"))
                        .child(
                            div()
                                .id("language-toggle")
                                .h(px(28.0))
                                .bg(Theme::bg_elevated())
                                .rounded(px(4.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .px(px(8.0))
                                .text_size(px(11.0))
                                .text_color(if current_language == "id" { Theme::accent() } else { Theme::text_primary() })
                                .cursor_pointer()
                                .child(SharedString::from(if current_language == "id" { "🇮🇩 ID" } else { "🇬🇧 EN" }))
                                .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                                    let mut s = this.gen_state.lock().unwrap();
                                    s.language = if s.language == "id" { "en".to_string() } else { "id".to_string() };
                                })),
                        ),
                ),
        )
        .child(field_label("Style"))
        .child(
            div()
                .flex()
                .gap(px(4.0))
                .flex_wrap()
                .children(STYLES.iter().map(|s| {
                    let style = *s;
                    let is_active = style == current_style;
                    div()
                        .id(SharedString::from(format!("style-{}", style)))
                        .h(px(26.0))
                        .px(px(8.0))
                        .rounded(px(4.0))
                        .flex()
                        .items_center()
                        .text_size(px(10.0))
                        .when(is_active, |d| {
                            d.bg(Theme::accent()).text_color(gpui::white())
                        })
                        .when(!is_active, |d| {
                            d.bg(Theme::bg_elevated())
                                .text_color(Theme::text_secondary())
                                .hover(|s| s.bg(Theme::bg_hover()).text_color(Theme::text_primary()))
                        })
                        .cursor_pointer()
                        .child(SharedString::from(style.to_string()))
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                            let mut s = this.gen_state.lock().unwrap();
                            s.style = style.to_string();
                        }))
                })),
        )
        .child(field_label("Platform"))
        .child(
            div()
                .flex()
                .gap(px(4.0))
                .flex_wrap()
                .children(PLATFORMS.iter().map(|p| {
                    let platform = *p;
                    let is_active = platform == current_platform;
                    div()
                        .id(SharedString::from(format!("platform-{}", platform)))
                        .h(px(26.0))
                        .px(px(8.0))
                        .rounded(px(4.0))
                        .flex()
                        .items_center()
                        .text_size(px(10.0))
                        .when(is_active, |d| {
                            d.bg(Theme::accent()).text_color(gpui::white())
                        })
                        .when(!is_active, |d| {
                            d.bg(Theme::bg_elevated())
                                .text_color(Theme::text_secondary())
                                .hover(|s| s.bg(Theme::bg_hover()).text_color(Theme::text_primary()))
                        })
                        .cursor_pointer()
                        .child(SharedString::from(platform.to_string()))
                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                            let mut s = this.gen_state.lock().unwrap();
                            s.platform = platform.to_string();
                        }))
                })),
        )
        .child(
            div()
                .h(px(4.0))
                .w_full()
                .bg(Theme::bg_elevated())
                .rounded(px(2.0))
                .overflow_hidden()
                .when(progress > 0.0 || is_generating, |d| {
                    let pct = progress.clamp(0.0, 1.0);
                    d.child(
                        div()
                            .h_full()
                            .w(px(280.0 * pct as f32))
                            .bg(Theme::accent())
                            .rounded(px(2.0)),
                    )
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(if status.contains("Error") || status.contains("Failed") {
                            Theme::error_color()
                        } else if status == "Completed!" {
                            Theme::success()
                        } else {
                            Theme::text_secondary()
                        })
                        .child(SharedString::from(status)),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(4.0))
                        .child(generate_btn(is_generating, &state_arc, cx))
                        .when(output_path.is_some(), |d| {
                            d.child(open_video_btn(output_path.as_deref().unwrap_or(""), cx))
                        }),
                ),
        )
}

fn generate_btn(
    is_generating: bool,
    state: &Arc<Mutex<VideoGenState>>,
    cx: &mut Context<'_, Sidebar>,
) -> gpui::Stateful<gpui::Div> {
    let state_clone = state.clone();
    div()
        .id("generate-btn")
        .h(px(28.0))
        .px(px(12.0))
        .bg(if is_generating { Theme::text_disabled() } else { Theme::accent() })
        .rounded(px(4.0))
        .flex()
        .items_center()
        .text_size(px(11.0))
        .text_color(gpui::white())
        .cursor_pointer()
        .when(!is_generating, |d| d.hover(|s| s.bg(Theme::accent_pressed())))
        .child(SharedString::from(if is_generating { "⏳..." } else { "Generate" }))
        .on_click(cx.listener(move |_this, _: &ClickEvent, _w, _cx| {
            Sidebar::start_generation(&state_clone);
        }))
}

fn open_video_btn(
    path: &str,
    cx: &mut Context<'_, Sidebar>,
) -> gpui::Stateful<gpui::Div> {
    let path_owned = path.to_string();
    div()
        .id("open-video-btn")
        .h(px(28.0))
        .px(px(12.0))
        .bg(Theme::success())
        .rounded(px(4.0))
        .flex()
        .items_center()
        .text_size(px(11.0))
        .text_color(gpui::white())
        .cursor_pointer()
        .hover(|s| s.bg(Theme::success_hover()))
        .child(SharedString::from("▶ Open"))
        .on_click(cx.listener(move |_this, _: &ClickEvent, _w, _cx| {
            Sidebar::open_video(&path_owned);
        }))
}

fn render_studio_chat_panel(
    editor: &Rc<RefCell<EditorState>>,
    chat_input: &mut String,
    focus_handle: &FocusHandle,
    cx: &mut Context<'_, Sidebar>,
) -> gpui::Div {
    use crate::state::editor::{ChatSender, AiConversationState, StoryEpisode};

    let chat_history = editor.borrow().chat_history.clone();
    let quick_replies = editor.borrow().chat_quick_replies.clone();
    let ai_state = editor.borrow().ai_state.clone();
    let is_generating = editor.borrow().is_generating_movie;
    let movie_status = editor.borrow().movie_status.clone();
    let episodes = editor.borrow().story_episodes.clone();
    let episode_count = episodes.len();
    let current_movie = editor.borrow().cinematic_movie.clone();

    // ── Header ────────────────────────────────────────────────────────────────
    let header = div()
        .flex()
        .items_center()
        .justify_between()
        .mb(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().text_size(px(14.0)).child(SharedString::from("🎬")))
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(Theme::text_primary())
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(SharedString::from("AI Story Director")),
                ),
        )
        .child(
            // Reset button
            div()
                .id("studio-reset-all")
                .h(px(22.0))
                .px(px(8.0))
                .bg(Theme::bg_elevated())
                .rounded(px(4.0))
                .flex()
                .items_center()
                .text_size(px(9.0))
                .text_color(Theme::text_secondary())
                .cursor_pointer()
                .hover(|s| s.bg(Theme::bg_hover()))
                .child(SharedString::from("↺ Reset"))
                .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                    this.studio_prompt.clear();
                    let pending_arc = this.editor.borrow().pending_movie.clone();
                    if let Ok(mut g) = pending_arc.lock() { *g = None; }
                    let pending_chat = this.editor.borrow().pending_chat_reply.clone();
                    if let Ok(mut g) = pending_chat.lock() { *g = None; }
                    let mut ed = this.editor.borrow_mut();
                    ed.animator.borrow_mut().set_timeline(vec![]);
                    ed.animator.borrow_mut().override_duration = None;
                    ed.cinematic_movie = None;
                    ed.studio_script = None;
                    ed.is_generating_movie = false;
                    ed.movie_status = String::new();
                    ed.chat_history.clear();
                    ed.chat_quick_replies.clear();
                    ed.story_episodes.clear();
                    ed.story_title = None;
                    ed.current_part = 0;
                    ed.ai_state = AiConversationState::Idle;
                    ed.pending_plan_json = String::new();
                })),
        );

    // ── Chat message bubbles ──────────────────────────────────────────────────
    let msg_views: Vec<_> = chat_history.iter().enumerate().map(|(i, msg)| {
        let is_user = msg.sender == ChatSender::User;
        let text = msg.text.clone();
        // Truncate very long messages for display
        let display_text = if text.len() > 300 {
            format!("{}...", &text[..300])
        } else {
            text.clone()
        };

        if is_user {
            // User bubble: right-aligned, accent color
            div()
                .flex()
                .justify_end()
                .w_full()
                .mb(px(4.0))
                .child(
                    div()
                        .max_w(px(200.0))
                        .bg(Theme::accent())
                        .rounded(px(8.0))
                        .p(px(7.0))
                        .text_size(px(10.0))
                        .text_color(gpui::white())
                        .child(SharedString::from(display_text))
                )
        } else {
            // AI bubble: left-aligned, dark bg with avatar
            div()
                .flex()
                .items_start()
                .gap(px(5.0))
                .w_full()
                .mb(px(4.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .flex_shrink_0()
                        .child(SharedString::from("🤖"))
                )
                .child(
                    div()
                        .max_w(px(200.0))
                        .bg(Theme::bg_elevated())
                        .rounded(px(8.0))
                        .p(px(7.0))
                        .text_size(px(10.0))
                        .text_color(Theme::text_primary())
                        .child(SharedString::from(display_text))
                )
        }
    }).collect();

    let chat_area = div()
        .id("studio-chat-area")
        .flex()
        .flex_col()
        .gap(px(2.0))
        .flex_1()
        .min_h(px(160.0))
        .overflow_y_scroll()
        .p(px(6.0))
        .bg(gpui::rgba(0x0d1117ff))
        .rounded(px(6.0))
        .border_1()
        .border_color(Theme::border_subtle())
        .children(msg_views)
        .when(chat_history.is_empty(), |d| {
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .h_full()
                    .gap(px(6.0))
                    .child(div().text_size(px(24.0)).child(SharedString::from("🎬")))
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(Theme::text_disabled())
                            .child(SharedString::from("AI Director sedang memuat..."))
                    )
            )
        });

    // ── Quick reply chips ─────────────────────────────────────────────────────
    let quick_reply_chips: Vec<_> = quick_replies.iter().map(|reply| {
        let r = reply.clone();
        let r2 = reply.clone();
        div()
            .id(SharedString::from(format!("qr-{}", r)))
            .h(px(24.0))
            .px(px(8.0))
            .bg(gpui::rgba(0x1e293bff))
            .border_1()
            .border_color(Theme::accent())
            .rounded(px(12.0))
            .flex()
            .items_center()
            .cursor_pointer()
            .hover(|s| s.bg(gpui::rgba(0x2d3748ff)))
            .text_size(px(9.0))
            .text_color(Theme::accent())
            .child(SharedString::from(r2))
            .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                use crate::state::editor::AiConversationState;
                let msg = r.clone();
                // Check if this is an auto-continue generate button
                if msg.contains("Auto-Generate") {
                    // Parse the number of episodes from the button text
                    let max_eps: u32 = if msg.contains("5") { 5 } else { 3 };
                    {
                        let mut ed = this.editor.borrow_mut();
                        ed.auto_continue = true;
                        ed.auto_continue_max_episodes = max_eps;
                    }
                    Sidebar::trigger_generate(&this.editor);
                } else if msg.contains("Generate") {
                    Sidebar::trigger_generate(&this.editor);
                } else {
                    Sidebar::send_chat_message(&this.editor, msg);
                }
                this.editor.borrow_mut().chat_quick_replies.clear();
            }))
    }).collect();

    let quick_area = if !quick_replies.is_empty() {
        div()
            .flex()
            .flex_wrap()
            .gap(px(4.0))
            .mt(px(4.0))
            .children(quick_reply_chips)
    } else {
        div()
    };

    // ── Chat input row ────────────────────────────────────────────────────────
    let input_row = div()
        .flex()
        .gap(px(4.0))
        .mt(px(6.0))
        .child(
            div()
                .id("studio-chat-input")
                .flex_1()
                .h(px(32.0))
                .bg(Theme::bg_elevated())
                .border_1()
                .border_color(Theme::border_subtle())
                .rounded(px(6.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .text_size(px(10.0))
                .text_color(if chat_input.is_empty() { Theme::text_disabled() } else { Theme::text_primary() })
                .cursor_text()
                .track_focus(focus_handle)
                .child(SharedString::from(if chat_input.is_empty() {
                    "Ketik pesan...".to_string()
                } else {
                    chat_input.clone()
                }))
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _w, _cx| {
                    if event.keystroke.key == "backspace" {
                        this.studio_prompt.pop();
                    } else if event.keystroke.key == "return" || event.keystroke.key == "enter" {
                        let msg = this.studio_prompt.trim().to_string();
                        if !msg.is_empty() {
                            use crate::state::editor::AiConversationState;
                            if msg.to_lowercase().contains("auto-generate") {
                                let max_eps: u32 = if msg.contains("5") { 5 } else { 3 };
                                {
                                    let mut ed = this.editor.borrow_mut();
                                    ed.auto_continue = true;
                                    ed.auto_continue_max_episodes = max_eps;
                                }
                                Sidebar::trigger_generate(&this.editor);
                            } else if msg.to_lowercase().contains("generate") {
                                Sidebar::trigger_generate(&this.editor);
                            } else {
                                Sidebar::send_chat_message(&this.editor, msg);
                            }
                            this.studio_prompt.clear();
                        }
                    } else if let Some(ch) = &event.keystroke.key_char {
                        if ch.len() == 1 { this.studio_prompt.push_str(ch); }
                    }
                })),
        )
        .child(
            // Send button
            div()
                .id("studio-send-btn")
                .w(px(32.0))
                .h(px(32.0))
                .bg(Theme::accent())
                .rounded(px(6.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(Theme::accent_pressed()))
                .text_size(px(14.0))
                .child(SharedString::from("➤"))
                .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                    let msg = this.studio_prompt.trim().to_string();
                    if !msg.is_empty() {
                        use crate::state::editor::AiConversationState;
                        if msg.to_lowercase().contains("auto-generate") {
                            let max_eps: u32 = if msg.contains("5") { 5 } else { 3 };
                            {
                                let mut ed = this.editor.borrow_mut();
                                ed.auto_continue = true;
                                ed.auto_continue_max_episodes = max_eps;
                            }
                            Sidebar::trigger_generate(&this.editor);
                        } else if msg.to_lowercase().contains("generate") {
                            Sidebar::trigger_generate(&this.editor);
                        } else {
                            Sidebar::send_chat_message(&this.editor, msg);
                        }
                        this.studio_prompt.clear();
                    }
                })),
        );

    // ── Generate episode button (shown when ready) ────────────────────────────
    let gen_btn = if ai_state == AiConversationState::ReadyToGenerate {
        let part_num = episode_count + 1;
        div()
            .id("studio-generate-btn")
            .mt(px(6.0))
            .h(px(34.0))
            .w_full()
            .bg(gpui::rgba(0x6366f1ff))
            .rounded(px(6.0))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(gpui::rgba(0x4f46e5ff)))
            .gap(px(6.0))
            .child(div().text_size(px(14.0)).child(SharedString::from("🎬")))
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(gpui::white())
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(SharedString::from(format!("Generate Part {} Sekarang!", part_num))),
            )
            .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                Sidebar::trigger_generate(&this.editor);
                this.editor.borrow_mut().chat_quick_replies.clear();
            }))
    } else {
        div()
            .id("studio-generate-btn-empty")
            .invisible()
    };

    // ── Generating status bar ─────────────────────────────────────────────────
    let status_bar = if is_generating {
        div()
            .id("studio-status-bar")
            .mt(px(6.0))
            .h(px(28.0))
            .w_full()
            .bg(gpui::rgba(0x1e1b4bff))
            .border_1()
            .border_color(gpui::rgba(0x6366f1ff))
            .rounded(px(6.0))
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().text_size(px(10.0)).child(SharedString::from("⏳")))
            .child(
                div()
                    .text_size(px(9.0))
                    .text_color(gpui::rgba(0xa5b4fcff))
                    .child(SharedString::from(movie_status.clone()))
            )
    } else {
        div()
            .id("studio-status-bar-empty")
            .invisible()
    };

    // ── Episode list (completed parts) ────────────────────────────────────────
    let episode_views: Vec<_> = episodes.iter().map(|ep| {
        let pn = ep.part_number;
        let title = ep.title.clone();
        let summary_short = if ep.summary.len() > 60 {
            format!("{}...", &ep.summary[..60])
        } else {
            ep.summary.clone()
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(4.0))
            .h(px(36.0))
            .px(px(6.0))
            .bg(Theme::bg_elevated())
            .rounded(px(4.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(Theme::text_primary())
                            .child(SharedString::from(format!("Part {}: {}", pn, title)))
                    )
                    .child(
                        div()
                            .text_size(px(8.0))
                            .text_color(Theme::text_disabled())
                            .child(SharedString::from(summary_short))
                    )
            )
            .child(
                div()
                    .id(SharedString::from(format!("ep-play-{}", pn)))
                    .w(px(24.0))
                    .h(px(24.0))
                    .bg(Theme::accent())
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(Theme::accent_pressed()))
                    .text_size(px(10.0))
                    .text_color(gpui::white())
                    .child(SharedString::from("▶"))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                        let mut ed = this.editor.borrow_mut();
                        if (pn as usize) <= ed.story_episodes.len() {
                            let movie = ed.story_episodes[(pn - 1) as usize].movie.clone();
                            ed.current_part = pn;
                            ed.cinematic_movie = Some(movie.clone());
                            ed.animator.borrow_mut().override_duration = Some(movie.total_duration);
                            ed.animator.borrow_mut().seek(0.0);
                            ed.animator.borrow_mut().play();
                        }
                    }))
            )
    }).collect();

    let episodes_section = if !episodes.is_empty() {
        div()
            .mt(px(8.0))
            .child(
                div()
                    .text_size(px(9.0))
                    .text_color(Theme::text_disabled())
                    .mb(px(4.0))
                    .child(SharedString::from(format!("📼 EPISODE SELESAI ({})", episodes.len())))
            )
            .children(episode_views)
    } else {
        div()
    };

    // ── Compose full panel ────────────────────────────────────────────────────
    // Layout: header (fixed) + chat (flex-1 scrollable) + quick chips + status + generate btn + input (fixed)
    div()
        .flex()
        .flex_col()
        .h_full()
        .gap(px(4.0))
        .child(header)
        .child(chat_area)          // takes all remaining vertical space
        .child(quick_area)         // chip row (auto height)
        .child(gen_btn)            // visible only when ReadyToGenerate
        .child(status_bar)         // visible only while generating
        .child(input_row)          // always at bottom
        .child(episodes_section)   // completed episodes list
}




fn render_effects_panel() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(11.0))
                .text_color(Theme::text_primary())
                .child(SharedString::from("Effects")),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(Theme::text_disabled())
                .child(SharedString::from("Click an effect to apply (preview)")),
        )
}

fn render_properties_panel() -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(11.0))
                .text_color(Theme::text_primary())
                .child(SharedString::from("Element Properties")),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(Theme::text_disabled())
                .child(SharedString::from(
                    "Select an element on the timeline to edit its properties.",
                )),
        )
}

fn field_label(text: &str) -> gpui::Div {
    div()
        .text_size(px(10.0))
        .text_color(Theme::text_disabled())
        .child(SharedString::from(text.to_string()))
}

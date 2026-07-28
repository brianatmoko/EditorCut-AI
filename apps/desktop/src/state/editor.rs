use std::cell::RefCell;
use std::rc::Rc;

use animation::StickmanAnimator;
use commands::CommandStack;
use media::MediaPool;
use project::Project;

// ── Chat / Story Director Types ───────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum ChatSender {
    User,
    AI,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub sender: ChatSender,
    pub text: String,
}

/// Summary of a completed story episode, passed to AI for continuity.
#[derive(Clone)]
pub struct StoryEpisode {
    pub part_number: u32,
    pub title: String,
    pub summary: String,
    pub movie: std::sync::Arc<animation::CinematicMovie>,
}


/// State machine for the AI conversation in the Studio chat panel.
#[derive(Clone, PartialEq, Debug)]
pub enum AiConversationState {
    /// No conversation started yet.
    Idle,
    /// AI asked "Ceritakan ide cerita kamu!"
    WaitingForStoryIdea,
    /// AI asked "Berapa menit per episode?"
    WaitingForDuration,
    /// AI proposed a setting/character plan, waiting for approval.
    WaitingForSettingApproval,
    /// Plan confirmed, user needs to press Generate.
    ReadyToGenerate,
    /// Background thread is generating the movie.
    Generating,
    /// Episode finished, asking if user wants Part 2.
    EpisodeDone,
}

impl AiConversationState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::WaitingForStoryIdea => "waiting_for_story_idea",
            Self::WaitingForDuration => "waiting_for_duration",
            Self::WaitingForSettingApproval => "waiting_for_setting_approval",
            Self::ReadyToGenerate => "ready_to_generate",
            Self::Generating => "generating",
            Self::EpisodeDone => "episode_done",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "waiting_for_story_idea" => Self::WaitingForStoryIdea,
            "waiting_for_duration" => Self::WaitingForDuration,
            "waiting_for_setting_approval" => Self::WaitingForSettingApproval,
            "ready_to_generate" => Self::ReadyToGenerate,
            "generating" => Self::Generating,
            "episode_done" => Self::EpisodeDone,
            _ => Self::Idle,
        }
    }
}

// ── Editor State ──────────────────────────────────────────────────────────────

pub struct EditorState {
    pub project: Project,
    pub media_pool: MediaPool,
    pub command_stack: CommandStack,
    pub animator: Rc<RefCell<StickmanAnimator>>,
    pub selected_element_ids: Vec<String>,
    pub selected_track_id: Option<String>,
    pub project_file_path: Option<String>,
    pub is_modified: bool,
    /// Scene environment theme: "city", "forest", "room", "cyberpunk", "space",
    /// "school", "desert", "ocean", "arctic", "volcano", "studio"
    pub scene_theme: String,
    /// Character visual type: "stickman", "robot", "michelle", "soldier"
    pub character_type: String,
    /// Active Cinematic Movie script
    pub cinematic_movie: Option<std::sync::Arc<animation::CinematicMovie>>,
    pub is_generating_movie: bool,
    pub movie_status: String,
    /// Thread-safe mailbox: background thread writes progress updates here.
    pub pending_progress: std::sync::Arc<std::sync::Mutex<String>>,
    /// Thread-safe mailbox: background thread writes result here, Preview::render picks it up.
    pub pending_movie: std::sync::Arc<std::sync::Mutex<Option<(animation::CinematicMovie, animation::ParsedStickmanScript)>>>,
    pub studio_script: Option<animation::ParsedStickmanScript>,

    // ── AI Story Director ──────────────────────────────────────────────────
    /// Thread-safe mailbox: background thread writes chat reply JSON here, Sidebar::render picks it up.
    pub pending_chat_reply: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    /// Full chat history for the Studio conversation panel.
    pub chat_history: Vec<ChatMessage>,
    /// Quick-reply button suggestions from the AI.
    pub chat_quick_replies: Vec<String>,
    /// Completed episodes in this series session.
    pub story_episodes: Vec<StoryEpisode>,
    /// Overall series title (set after first generation).
    pub story_title: Option<String>,
    /// Which episode part is currently loaded in the preview (1-indexed).
    pub current_part: u32,
    /// The AI conversation state machine.
    pub ai_state: AiConversationState,
    /// JSON blob of the current AI-proposed story plan (passed to generate).
    pub pending_plan_json: String,
    /// Auto-continue mode: AI generates full multi-episode series without user input.
    pub auto_continue: bool,
    /// Max episodes in auto-continue mode (safety cap).
    pub auto_continue_max_episodes: u32,
    /// Cross-render flag: Preview sets this, Sidebar picks it up & calls trigger_generate.
    pub pending_auto_continue: bool,
}

impl EditorState {
    pub fn new() -> Self {
        let animator = Rc::new(RefCell::new(StickmanAnimator::new()));
        let project = Project::default();

        Self {
            project,
            media_pool: MediaPool::default(),
            command_stack: CommandStack::new(),
            animator,
            selected_element_ids: Vec::new(),
            selected_track_id: None,
            project_file_path: None,
            is_modified: false,
            scene_theme: "city".to_string(),
            character_type: "police_1".to_string(),
            cinematic_movie: None,
            is_generating_movie: false,
            movie_status: String::new(),
            pending_progress: std::sync::Arc::new(std::sync::Mutex::new(String::new())),
            pending_movie: std::sync::Arc::new(std::sync::Mutex::new(None)),
            studio_script: None,

            pending_chat_reply: std::sync::Arc::new(std::sync::Mutex::new(None)),
            chat_history: Vec::new(),
            chat_quick_replies: Vec::new(),
            story_episodes: Vec::new(),
            story_title: None,
            current_part: 0,
            ai_state: AiConversationState::Idle,
            pending_plan_json: String::new(),
            auto_continue: false,
            auto_continue_max_episodes: 3,
            pending_auto_continue: false,
        }
    }

    /// Push a message to the chat history.
    pub fn push_chat(&mut self, sender: ChatSender, text: String) {
        self.chat_history.push(ChatMessage { sender, text });
    }

    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    pub fn mark_saved(&mut self) {
        self.is_modified = false;
        self.command_stack.mark_saved();
    }

    pub fn current_scene(&self) -> Option<&timeline::TimelineScene> {
        self.project
            .scenes
            .iter()
            .find(|s| s.id == self.project.current_scene_id)
    }

    pub fn current_scene_mut(&mut self) -> Option<&mut timeline::TimelineScene> {
        self.project
            .scenes
            .iter_mut()
            .find(|s| s.id == self.project.current_scene_id)
    }
}

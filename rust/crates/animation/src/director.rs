use crate::cinematic::{CameraShot, CinematicAct, CinematicMovie, DialogueLine, ShotType, StageEntity};

// Simple deterministic PRNG (xorshift) for varied story generation
struct FastRng(u64);

impl FastRng {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn f32(&mut self) -> f32 {
        // Take top 24 bits (mantissa precision of f32 is 24 bits)
        let bits = (self.next_u64() >> (64 - 24)) as f32;
        bits / ((1u64 << 24) as f32)
    }

    fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.f32() * (hi - lo)
    }

    fn pick<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        &slice[self.next_u64() as usize % slice.len()]
    }
}

const ACTIONS: &[&str] = &[
    "idle", "walk", "slow_walk", "run", "panic_run", "crouch_walk",
    "punch", "kick", "jab", "cross", "uppercut", "haymaker",
    "shoot", "shoot_pistol", "shoot_rifle", "dodge", "block",
    "grab", "tackle", "clothesline", "leg_sweep", "headbutt",
    "talk", "point", "wave", "surrender", "scream",
];

const THEMES: &[&str] = &[
    "city", "city_night", "cyberpunk", "warehouse", "alley",
    "rooftop", "highway", "forest", "beach", "desert",
    "snow", "room", "school", "temple", "castle", "cave",
];

const SKINS: &[&str] = &[
    "police_1", "police_2", "police_3",
    "terrorist_1", "terrorist_2", "terrorist_3",
    "chibi_summer", "chibi_autumn", "chibi_winter",
];

const DIALOGUES_ACTION: &[(&str, &str)] = &[
    ("Jangan bergerak atau aku tembak!", "shout"),
    ("Menyerahlah! Kau dikepung!", "shout"),
    ("Ayo lawan aku kalau berani!", "shout"),
    ("Kau tidak akan lolos kali ini!", "shout"),
    ("Sudah cukup! Berhenti!", "shout"),
    ("Ini belum berakhir!", "shout"),
    ("Sialan, kau memang cepat!", "shout"),
    ("Awas! Di belakangmu!", "shout"),
    ("Tangan di atas kepalamu!", "shout"),
    ("Serahkan dirimu sekarang!", "shout"),
];

const DIALOGUES_SETUP: &[&str] = &[
    "Apa yang kau lakukan di sini?",
    "Sudah lama kita tidak bertemu.",
    "Kukira kau sudah pergi dari kota ini.",
    "Apa urusanmu denganku?",
    "Kau datang tepat waktu.",
    "Jangan harap aku akan mundur.",
    "Ini antara kau dan aku.",
    "Kau membuat kesalahan besar datang ke sini.",
    "Aku sudah menunggumu.",
    "Jadi ini akhirnya pertemuan kita.",
];

// Generate a varied, non-template story with randomized elements
fn generate_random_story(prompt: &str, target_duration: f64) -> CinematicMovie {
    let mut rng = FastRng::new();
    let n_acts = rng.next_u64() as usize % 3 + 3; // 3-5 acts
    let act_duration = target_duration / n_acts as f64;

    // Detect characters from prompt
    let hero_skin = if prompt.to_lowercase().contains("polisi") || rng.f32() < 0.4 { "police_1" } else { "chibi_summer" };
    let villain_skin = if prompt.to_lowercase().contains("teroris") || rng.f32() < 0.4 { "terrorist_1" } else { "terrorist_2" };

    let mut acts = Vec::new();
    // Track where each character ended in the previous act for continuity.
    // Key = character_skin_id, Value = (end_x, end_y)
    let mut prev_positions: std::collections::HashMap<String, (f32, f32)> = std::collections::HashMap::new();
    // Track which skins we've seen so each act can keep the same character
    let skin_to_idx: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let _ = &skin_to_idx; // used for future character consistency tracking

    for i in 0..n_acts {
        let act_num = (i + 1) as u32;
        let start = i as f64 * act_duration;

        // Choose number of entities for this act (2-4)
        let n_entities = rng.next_u64() as usize % 3 + 2;
        let mut entities = Vec::new();

        // Theme changes slightly per act to show progression
        let theme_idx = (i + rng.next_u64() as usize) % THEMES.len();
        let theme = THEMES[theme_idx];

        // Build entities
        for e in 0..n_entities {
            let is_hero = e == 0;
            let skin_id = if is_hero { hero_skin } else {
                if rng.f32() < 0.5 { villain_skin } else { SKINS[rng.next_u64() as usize % SKINS.len()] }
            };

            // Position continuity: if this skin appeared in a previous act,
            // start from where it ended last time. Otherwise pick a new position.
            let side = if is_hero { -1.0 } else { 1.0 };
            let pos_x = if let Some(&(px, _py)) = prev_positions.get(skin_id) {
                // Continue from previous position, with slight variation
                let variant = rng.range_f32(-0.15, 0.15);
                (px + variant).clamp(-1.5, 1.5)
            } else {
                let p = side * rng.range_f32(0.2, 1.2) + (i as f32 * rng.range_f32(-0.1, 0.1));
                p.clamp(-1.5, 1.5)
            };
            let pos_z = if is_hero { rng.range_f32(1.0, 1.5) } else { rng.range_f32(1.5, 2.5) };

            let has_movement = rng.f32() < 0.5;
            let (action, end_x) = if is_hero {
                match i {
                    0 => ("walk".to_string(), if has_movement { Some((pos_x + rng.range_f32(0.3, 0.8)).clamp(-1.5, 1.5)) } else { None }),
                    _ if i == n_acts - 1 => {
                        let a = if rng.f32() < 0.5 { "talk" } else { "idle" };
                        (a.to_string(), None)
                    }
                    _ => ((*rng.pick(&[
                        "run", "shoot_pistol", "punch", "dodge", "tackle"
                    ])).to_string(),
                        if has_movement { Some((pos_x + rng.range_f32(-0.5, 0.5)).clamp(-1.5, 1.5)) } else { None })
                }
            } else {
                match i {
                    0 => ("idle".to_string(), None),
                    _ => ((*rng.pick(&["run", "shoot_rifle", "punch", "kick", "hurt", "block"])).to_string(),
                        if has_movement { Some((pos_x + rng.range_f32(-0.5, 0.5)).clamp(-1.5, 1.5)) } else { None })
                }
            };

            // Record where this character ends for the next act
            let final_x = end_x.unwrap_or(pos_x);
            prev_positions.insert(skin_id.to_string(), (final_x, 0.0));

            let facing_left = if is_hero { pos_x > 0.0 } else { pos_x < 0.0 };

            entities.push(StageEntity {
                id: format!("char_{}_{}", act_num, e),
                character_skin_id: skin_id.to_string(),
                name: if is_hero { format!("Pahlawan {}", act_num) } else { format!("Musuh {}", act_num) },
                pos_x,
                pos_y: 0.0,
                pos_z,
                action,
                facing_left,
                end_x,
                end_y: None,
                target_id: if !is_hero { Some(format!("char_{}_0", act_num)) } else { None },
                action_variant: None,
            });
        }

        // Add dialogue in some acts (70% chance)
        let mut dialogues = Vec::new();
        if rng.f32() < 0.7 {
            let n_lines = rng.next_u64() as usize % 3 + 1; // 1-3 lines
            for d in 0..n_lines {
                let speaker = if rng.f32() < 0.5 { &entities[0] } else { &entities[entities.len().min(1)] };
                let (text, emotion) = if i == 0 {
                    (DIALOGUES_SETUP[rng.next_u64() as usize % DIALOGUES_SETUP.len()].to_string(), "normal".to_string())
                } else {
                    let (t, e) = DIALOGUES_ACTION[rng.next_u64() as usize % DIALOGUES_ACTION.len()];
                    (t.to_string(), e.to_string())
                };
                let start_offset = d as f64 * act_duration / (n_lines as f64 + 1.0);
                dialogues.push(DialogueLine {
                    entity_id: speaker.id.clone(),
                    text,
                    start_time: start + start_offset,
                    duration: rng.range_f32(1.5, 3.5) as f64,
                    emotion,
                });
            }
        }

        // Choose shot type based on act progression
        let shot_type = if n_entities >= 3 { ShotType::GroupShot } else {
            match i {
                0 => ShotType::Wide,
                _ if i == n_acts - 1 => ShotType::Medium,
                _ => rng.pick(&[ShotType::Medium, ShotType::CloseUp, ShotType::ActionFollow, ShotType::OverShoulder]).clone(),
            }
        };

        // Tension rises through acts, then falls
        let intensity = if i == 0 { 0.2 } else if i == n_acts - 1 { 0.4 } else { 0.5 + (i as f32 / n_acts as f32) * 0.4 };

        acts.push(CinematicAct {
            act_number: act_num as u32,
            title: format!("Babak {}: {}", act_num, match i {
                0 => "Pertemuan",
                1 => "Ketegangan",
                2 => "Konfrontasi",
                3 => "Klimaks",
                _ => "Resolusi",
            }),
            description: format!("Babak {} dari cerita berdasarkan: {}", act_num, &prompt[..prompt.len().min(60)]),
            theme: theme.to_string(),
            emotional_tone: if i == 0 { "establish".to_string() } else if i == n_acts - 1 { "resolution".to_string() } else { "rising_action".to_string() },
            intensity,
            start_time: start,
            duration: act_duration,
            entities: entities.clone(),
            camera: CameraShot {
                shot_type,
                target_entity_id: Some(format!("char_{}_0", act_num)),
                pan_x: rng.range_f32(-0.2, 0.2),
                pan_y: 0.0,
                zoom: rng.range_f32(0.8, 1.5),
                shake: if i > 0 && i < n_acts - 1 { rng.range_f32(0.0, 0.15) } else { 0.0 },
                movement: crate::cinematic::CameraMovement::None,
                transition: if i == 0 { crate::cinematic::CameraTransition::FadeFromBlack } else { crate::cinematic::CameraTransition::Cut },
                movement_intensity: 0.0,
                tilt_angle: if i > 0 && i < n_acts - 1 && intensity > 0.5 { rng.range_f32(6.0, 14.0) } else if i == n_acts - 2 { rng.range_f32(4.0, 10.0) } else { 0.0 },
                depth_of_field: 0.0,
                secondary_entity_id: if entities.len() > 1 { Some(entities[1].id.clone()) } else { None },
                rule_of_thirds: if rng.f32() < 0.5 { -1 } else { 1 },
            },
            dialogues,
            transition: if i == n_acts - 1 { Some("fade".to_string()) } else { None },
        });
    }

    let total = acts.last().map(|a| a.start_time + a.duration).unwrap_or(target_duration);
    CinematicMovie {
        title: format!("{} — Episode {}", &prompt[..prompt.len().min(40)], rng.next_u64() % 100),
        summary: format!("Cerita {} babak berdasarkan: {}", n_acts, &prompt[..prompt.len().min(80)]),
        total_duration: total,
        acts,
    }
}

fn extract_duration(prompt: &str) -> f64 {
    let lower = prompt.to_lowercase();
    for pattern in &["detik", "second", "sekon"] {
        if let Some(pos) = lower.find(pattern) {
            let before = &lower[..pos].trim();
            if let Some(last_space) = before.rfind(|c: char| !c.is_ascii_digit() && c != '.') {
                if let Ok(n) = before[last_space + 1..].parse::<f64>() {
                    return n.max(10.0).min(300.0);
                }
            } else if let Ok(n) = before.parse::<f64>() {
                return n.max(10.0).min(300.0);
            }
        }
    }
    for pattern in &["menit", "minute", "mnt"] {
        if let Some(pos) = lower.find(pattern) {
            let before = &lower[..pos].trim();
            if let Some(last_space) = before.rfind(|c: char| !c.is_ascii_digit() && c != '.') {
                if let Ok(n) = before[last_space + 1..].parse::<f64>() {
                    return (n * 60.0).max(10.0).min(300.0);
                }
            } else if let Ok(n) = before.parse::<f64>() {
                return (n * 60.0).max(10.0).min(300.0);
            }
        }
    }
    30.0
}

pub fn generate_cinematic_movie(prompt: &str) -> CinematicMovie {
    eprintln!("[Director] Analisis prompt: {:?}", &prompt[..prompt.len().min(80)]);

    let python_path = if std::path::Path::new("./.venv/bin/python").exists() {
        "./.venv/bin/python"
    } else if std::path::Path::new(".venv/bin/python").exists() {
        ".venv/bin/python"
    } else {
        "python3"
    };

    let script_path = find_director_script().unwrap_or_else(|| std::path::PathBuf::from("apps/desktop/gemini_director.py"));

    eprintln!("[Director] 🤖 Menghubungi Google AI Studio (Gemini-2.5-Flash)...");
    let child_result = std::process::Command::new(python_path)
        .arg(&script_path)
        .arg(prompt)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    match child_result {
        Ok(mut child) => {
            let timeout = std::time::Duration::from_secs(300);
            let start = std::time::Instant::now();
            loop {
                if start.elapsed() > timeout {
                    eprintln!("[Director] ⚠️ Timeout (300s) — killing subprocess");
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Err(e) => {
                        eprintln!("[Director] ⚠️ Error waiting: {:?}", e);
                        break;
                    }
                }
            }
            let out = child.wait_with_output();
            match out {
                Ok(out) if out.status.success() => {
                    let stdout_str = String::from_utf8_lossy(&out.stdout);
                    match serde_json::from_str::<CinematicMovie>(&stdout_str) {
                        Ok(mut movie) => {
                            let mut current_start = 0.0;
                            for act in &mut movie.acts {
                                act.start_time = current_start;
                                current_start += act.duration;
                            }
                            movie.total_duration = current_start;
                            eprintln!("[Director] 🌟 Sukses! AI menghasilkan film: '{}' ({} Babak, total {:.1}s)", 
                                movie.title, movie.acts.len(), movie.total_duration);
                            return movie;
                        }
                        Err(e) => {
                            eprintln!("[Director] ⚠️ Gagal memparse JSON AI: {:?}. Menggunakan fallback...", e);
                        }
                    }
                }
                Ok(out) => {
                    let stderr_str = String::from_utf8_lossy(&out.stderr);
                    eprintln!("[Director] ⚠️ Subproses AI gagal (exit {}): {}. Menggunakan fallback...", 
                        out.status, stderr_str);
                }
                Err(e) => {
                    eprintln!("[Director] ⚠️ Gagal membaca output: {:?}. Menggunakan fallback...", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[Director] ⚠️ Gagal memanggil subproses AI: {:?}. Menggunakan fallback...", e);
        }
    }

    let target_duration = extract_duration(prompt);
    eprintln!("[Director] ✅ Menggunakan cerita procedural acak berdasarkan prompt");
    generate_random_story(prompt, target_duration)
}

fn find_director_script() -> Option<std::path::PathBuf> {
    // Try relative to exe directory first
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let p = exe_dir.join("gemini_director.py");
            if p.exists() { return Some(p); }
            let p = exe_dir.join("apps/desktop/gemini_director.py");
            if p.exists() { return Some(p); }
        }
    }
    // Fallback to cwd-relative paths
    let candidates = [
        std::path::PathBuf::from("apps/desktop/gemini_director.py"),
        std::path::PathBuf::from("gemini_director.py"),
    ];
    for p in &candidates {
        if p.exists() { return Some(p.clone()); }
    }
    None
}

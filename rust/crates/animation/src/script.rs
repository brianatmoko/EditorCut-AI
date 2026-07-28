use serde::{Deserialize, Serialize};

use crate::animator::AnimationClip;

struct ActionDef {
    name: &'static str,
    duration: f64,
    keywords: &'static [&'static str],
}

const ACTIONS: &[ActionDef] = &[
    ActionDef { name: "walk",       duration: 3.0, keywords: &["jalan", "berjalan", "walk", "stroll", "melangkah", "langkah", "steps"] },
    ActionDef { name: "slow_walk",  duration: 3.0, keywords: &["slow_walk", "jalan santai", "jalan lambat", "slow walk"] },
    ActionDef { name: "run",        duration: 3.0, keywords: &["lari", "berlari", "run", "sprint", "cepat", "fast"] },
    ActionDef { name: "sprint",     duration: 2.0, keywords: &["sprint", "lari kencang", "all out"] },
    ActionDef { name: "jump",       duration: 1.2, keywords: &["lompat", "melompat", "jump", "hop", "loncat", "tinggi", "high"] },
    ActionDef { name: "dodge",      duration: 0.8, keywords: &["dodge", "menghindar", "hindar", "sidestep", "menghindari"] },
    ActionDef { name: "stumble",    duration: 1.4, keywords: &["stumble", "tersandung", "trip", "hampir jatuh", "terantuk"] },
    ActionDef { name: "roll",       duration: 1.0, keywords: &["roll", "guling", "berguling", "combat roll"] },
    ActionDef { name: "crawl",      duration: 3.0, keywords: &["crawl", "merangkak", "rayap"] },
    ActionDef { name: "wave",       duration: 2.0, keywords: &["lambaikan", "wave", "halo", "hello", "sapa", "bye", "pamit"] },
    ActionDef { name: "dance",      duration: 4.0, keywords: &["dance", "menari", "tari", "dancing", "tarian", "joget"] },
    ActionDef { name: "punch",      duration: 0.6, keywords: &["pukul", "tinju", "punch", "fight", "bertarung", "tarung", "hantam", "memukul"] },
    ActionDef { name: "shoot",      duration: 0.8, keywords: &["tembak", "shoot", "menembak", "fire", "meletus", "dor"] },
    ActionDef { name: "aim",        duration: 1.5, keywords: &["aim", "bidik", "membidik", "target", "sasaran"] },
    ActionDef { name: "think",      duration: 3.0, keywords: &["pikir", "think", "berpikir", "fikir", "contemplate", "mikir"] },
    ActionDef { name: "fall",       duration: 1.6, keywords: &["jatuh", "fall", "terjatuh", "slip", "gelincir", "terpeleset"] },
    ActionDef { name: "panic_run",  duration: 3.0, keywords: &["panic", "panik", "lari panik", "panic run", "ketakutan"] },
    ActionDef { name: "stealth_walk", duration: 3.0, keywords: &["stealth", "mengendap", "sneak", "nyelinap", "diam-diam"] },
    ActionDef { name: "sad_walk",   duration: 3.0, keywords: &["sad walk", "jalan sedih", "lesu", "lunglai"] },
    ActionDef { name: "happy_hop",  duration: 2.0, keywords: &["happy", "gembira", "hop", "senang", "ria", "skipping"] },
    // KICKS
    ActionDef { name: "front_kick", duration: 0.7, keywords: &["front kick", "tendang depan", "front_kick", "depan"] },
    ActionDef { name: "roundhouse", duration: 0.9, keywords: &["roundhouse", "tendang putar", "roundhouse kick", "tendang kepala"] },
    ActionDef { name: "side_kick",  duration: 0.8, keywords: &["side kick", "tendang samping", "side_kick"] },
    ActionDef { name: "axe_kick",   duration: 1.0, keywords: &["axe kick", "tendang kapak", "axe_kick", "tendang atas"] },
    ActionDef { name: "flying_kick", duration: 1.2, keywords: &["flying kick", "tendang terbang", "lompat tendang"] },
    ActionDef { name: "crescent_kick", duration: 0.8, keywords: &["crescent kick", "tendang sabit", "sabit"] },
    ActionDef { name: "knee_strike",   duration: 0.5, keywords: &["knee strike", "lutut", "pukul lutut"] },
    ActionDef { name: "leg_sweep",     duration: 0.7, keywords: &["leg sweep", "sapu kaki", "sweep"] },
    ActionDef { name: "kick_leg",      duration: 0.6, keywords: &["kick leg", "tendang kaki", "sapu"] },
    ActionDef { name: "kick_body",     duration: 0.7, keywords: &["kick body", "tendang badan"] },
    // PUNCHES & STRIKES
    ActionDef { name: "jab",           duration: 0.4, keywords: &["jab", "pukul cepat"] },
    ActionDef { name: "cross",         duration: 0.6, keywords: &["cross", "pukul lurus"] },
    ActionDef { name: "hook",          duration: 0.6, keywords: &["hook", "pukul kait"] },
    ActionDef { name: "uppercut",      duration: 0.7, keywords: &["uppercut", "pukul dagu"] },
    ActionDef { name: "haymaker",      duration: 0.9, keywords: &["haymaker", "pukul ayun", "pukul keras"] },
    ActionDef { name: "body_blow",     duration: 0.6, keywords: &["body blow", "pukul badan", "pukul perut"] },
    ActionDef { name: "elbow_strike",  duration: 0.5, keywords: &["elbow", "siku", "pukul siku"] },
    ActionDef { name: "backfist",      duration: 0.6, keywords: &["backfist", "pukul balik"] },
    ActionDef { name: "palm_strike",   duration: 0.6, keywords: &["palm strike", "tampar", "telapak"] },
    ActionDef { name: "hammer_fist",   duration: 0.8, keywords: &["hammer fist", "pukul palu"] },
    // GRABS & THROWS
    ActionDef { name: "headlock",    duration: 2.0, keywords: &["headlock", "kunci kepala", "cekik"] },
    ActionDef { name: "body_slam",   duration: 1.2, keywords: &["body slam", "banting", "banting tubuh"] },
    ActionDef { name: "suplex",      duration: 1.4, keywords: &["suplex", "banting belakang"] },
    ActionDef { name: "hip_throw",   duration: 1.0, keywords: &["hip throw", "banting pinggul"] },
    ActionDef { name: "choke_hold",  duration: 2.0, keywords: &["choke", "jerat", "mencekik"] },
    ActionDef { name: "throw_push",  duration: 0.7, keywords: &["throw push", "dorong", "lempar"] },
    ActionDef { name: "clothesline", duration: 0.8, keywords: &["clothesline", "garong"] },
    ActionDef { name: "slide_tackle", duration: 1.0, keywords: &["slide tackle", "tackle kaki", "sliding"] },
    ActionDef { name: "grab",        duration: 0.6, keywords: &["grab", "tangkap", "pegang"] },
    ActionDef { name: "tackle",      duration: 1.0, keywords: &["tackle", "tekel", "tackel"] },
    // DEFENSE
    ActionDef { name: "block",       duration: 1.0, keywords: &["block", "tahan", "blok"] },
    ActionDef { name: "duck",        duration: 0.6, keywords: &["duck", "jongkok hindar"] },
    ActionDef { name: "parry",       duration: 0.5, keywords: &["parry", "tangkis"] },
    ActionDef { name: "weave",       duration: 1.0, keywords: &["weave", "hindar badan", "mengelak"] },
    ActionDef { name: "step_back",   duration: 0.5, keywords: &["step back", "mundur", "langkah mundur"] },
    // WEAPONS
    ActionDef { name: "draw_weapon", duration: 1.0, keywords: &["draw weapon", "cabut senjata", "tarik"] },
    ActionDef { name: "holster",     duration: 1.0, keywords: &["holster", "simpan senjata", "sarung"] },
    ActionDef { name: "melee_swing", duration: 0.9, keywords: &["melee swing", "ayun", "ayun senjata"] },
    ActionDef { name: "melee_stab",  duration: 0.6, keywords: &["stab", "tusuk", "tikam", "melee stab"] },
    ActionDef { name: "throw_weapon", duration: 0.8, keywords: &["throw weapon", "lempar senjata"] },
    // VEHICLES
    ActionDef { name: "enter_car",   duration: 2.0, keywords: &["enter car", "masuk mobil", "naik mobil"] },
    ActionDef { name: "exit_car",    duration: 2.0, keywords: &["exit car", "keluar mobil", "turun mobil"] },
    ActionDef { name: "drive",       duration: 4.0, keywords: &["drive", "mengemudi", "setir", "nyetir"] },
    ActionDef { name: "ride_motorcycle", duration: 3.0, keywords: &["ride motorcycle", "naik motor", "bonceng motor"] },
    ActionDef { name: "dismount_motorcycle", duration: 1.5, keywords: &["dismount motorcycle", "turun motor"] },
    ActionDef { name: "enter_helicopter", duration: 2.5, keywords: &["enter helicopter", "naik helikopter"] },
    ActionDef { name: "exit_helicopter",  duration: 2.0, keywords: &["exit helicopter", "turun helikopter"] },
    // ACROBATIC
    ActionDef { name: "salto",       duration: 1.2, keywords: &["salto", "somersault", "jungkir balik", "salto depan"] },
    ActionDef { name: "salto_backward", duration: 1.2, keywords: &["salto backward", "salto belakang"] },
    ActionDef { name: "back_handspring", duration: 1.2, keywords: &["back handspring", "handspring"] },
    ActionDef { name: "front_handspring", duration: 1.2, keywords: &["front handspring"] },
    ActionDef { name: "cartwheel",   duration: 1.0, keywords: &["cartwheel", "baling"] },
    ActionDef { name: "wall_flip",   duration: 1.0, keywords: &["wall flip", "flip tembok"] },
    ActionDef { name: "dive",        duration: 1.2, keywords: &["dive", "terjang"] },
    // ENVIRONMENT
    ActionDef { name: "climb",       duration: 3.0, keywords: &["climb", "panjat", "memanjat"] },
    ActionDef { name: "open_door",   duration: 1.0, keywords: &["open door", "buka pintu"] },
    ActionDef { name: "close_door",  duration: 1.0, keywords: &["close door", "tutup pintu"] },
    ActionDef { name: "crawl_through", duration: 2.0, keywords: &["crawl through", "merangkak"] },
    ActionDef { name: "jump_over",   duration: 1.0, keywords: &["jump over", "lompati", "lompat rintangan"] },
    // GROUND
    ActionDef { name: "prone_crawl", duration: 3.0, keywords: &["prone crawl", "tiarap", "merayap"] },
    ActionDef { name: "crouch_walk", duration: 2.0, keywords: &["crouch walk", "jongkok jalan", "jongkok"] },
    ActionDef { name: "ground_recover", duration: 1.5, keywords: &["ground recover", "bangun", "bangkit"] },
    ActionDef { name: "limp",        duration: 3.0, keywords: &["limp", "pincang", "jalan pincang"] },
    ActionDef { name: "stagger",     duration: 2.0, keywords: &["stagger", "terhuyung", "goyah"] },
    ActionDef { name: "slip",        duration: 1.0, keywords: &["slip", "tergelincir", "licin"] },
    // EMOTIONAL EXTENDED
    ActionDef { name: "surrender",     duration: 2.0, keywords: &["surrender", "menyerah", "angkat tangan"] },
    ActionDef { name: "triumph",       duration: 2.0, keywords: &["triumph", "jaya", "kemenangan"] },
    ActionDef { name: "bow",           duration: 1.5, keywords: &["bow", "membungkuk", "hormat"] },
    ActionDef { name: "celebrate",     duration: 2.0, keywords: &["celebrate", "sukacita", "hore"] },
    ActionDef { name: "victory",       duration: 2.0, keywords: &["victory", "menang"] },
    ActionDef { name: "exhausted",     duration: 3.0, keywords: &["exhausted", "lelah", "capek", "kelelahan"] },
    ActionDef { name: "taunt",         duration: 2.0, keywords: &["taunt", "ejek", "provokasi"] },
];

const MOODS: &[(&str, &[&str], f64)] = &[
    ("happy", &["senang", "happy", "gembira", "seru", "excited", "ceria", "joy"], 1.15),
    ("sad",   &["sedih", "sad", "lunglai", "down", "mellow"], 0.8),
    ("angry", &["marah", "angry", "kesal", "emosi", "furious", "mad"], 1.2),
];

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParsedStickmanScript {
    pub clips: Vec<AnimationClip>,
    pub detected_actions: Vec<String>,
    pub mood: String,
    pub direction: Option<String>,
    pub duration: f64,
    pub summary: String,
}

fn add_clip(
    clips: &mut Vec<AnimationClip>,
    t: &mut f64,
    action: &str,
    duration: f64,
    direction: Option<String>,
    speed: Option<f64>,
) {
    clips.push(AnimationClip {
        action: action.to_string(),
        start_time: *t,
        duration,
        direction,
        speed,
    });
    *t += duration;
}

fn extract_duration(text: &str) -> Option<f64> {
    // Look for "N detik", "N seconds", "N s", "Ns"
    let lower = text.to_lowercase();
    let patterns = [" detik", " seconds", " second", " sec", "s "];
    for pat in &patterns {
        if let Some(pos) = lower.find(pat) {
            // Grab word before pattern
            let before = &lower[..pos].trim_end().to_string();
            let word = before.split_whitespace().last().unwrap_or("");
            if let Ok(n) = word.parse::<f64>() {
                if n > 0.0 && n < 300.0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn extract_direction(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    if lower.contains("kiri") || lower.contains(" left") {
        Some("left".to_string())
    } else if lower.contains("kanan") || lower.contains(" right") {
        Some("right".to_string())
    } else {
        None
    }
}

fn build_summary(actions: &[String], mood: &str, direction: &Option<String>, duration: f64) -> String {
    let dir_str = match direction.as_deref() {
        Some("left") => " ke kiri",
        Some("right") => " ke kanan",
        _ => "",
    };
    let mood_str = if mood != "neutral" { format!(" (mood: {})", mood) } else { String::new() };
    if actions.is_empty() {
        format!("Diam selama {:.1}s{}{}", duration, dir_str, mood_str)
    } else {
        format!("{}{} selama {:.1}s{}", actions.join(", "), dir_str, duration, mood_str)
    }
}

pub fn generate_stickman_script(text: &str) -> ParsedStickmanScript {
    let lower = text.to_lowercase();

    // Detect mood
    let mood = MOODS
        .iter()
        .find(|(_, kws, _)| kws.iter().any(|kw| lower.contains(kw)))
        .map(|(name, _, _)| name.to_string())
        .unwrap_or_else(|| "neutral".to_string());

    let speed_mul = MOODS
        .iter()
        .find(|(name, _, _)| *name == mood.as_str())
        .map(|(_, _, mul)| *mul)
        .unwrap_or(1.0);

    // Detect direction
    let direction = extract_direction(&lower);

    // Detect actions in order of appearance
    let mut detected: Vec<&ActionDef> = Vec::new();
    for action in ACTIONS {
        if action.keywords.iter().any(|kw| lower.contains(kw)) {
            detected.push(action);
        }
    }

    let mut clips: Vec<AnimationClip> = Vec::new();
    let mut t = 0.0_f64;

    if detected.is_empty() {
        add_clip(&mut clips, &mut t, "idle", 3.0, None, None);
    } else {
        for action in &detected {
            let dur = action.duration * if mood == "sad" { 0.8 } else { 1.0 };
            add_clip(&mut clips, &mut t, action.name, dur, direction.clone(), Some(speed_mul));
        }

        if mood == "happy" && !detected.iter().any(|d| d.name == "wave") {
            add_clip(&mut clips, &mut t, "wave", 1.5, direction.clone(), Some(speed_mul));
        }
        add_clip(&mut clips, &mut t, "idle", 1.0, None, None);
    }

    let mut total = clips.iter().map(|c| c.duration).sum::<f64>();

    if let Some(requested) = extract_duration(text) {
        if requested < total {
            let mut acc = 0.0;
            let mut trimmed: Vec<AnimationClip> = Vec::new();
            for c in &clips {
                if acc >= requested { break; }
                let avail = requested - acc;
                if c.duration <= avail {
                    trimmed.push(c.clone());
                    acc += c.duration;
                } else {
                    let mut capped = c.clone();
                    capped.duration = avail;
                    trimmed.push(capped);
                    break;
                }
            }
            clips = trimmed;
        } else if requested > total {
            if let Some(last) = clips.last_mut() {
                last.duration += requested - total;
            } else {
                add_clip(&mut clips, &mut t, "idle", requested, None, None);
            }
        }
        total = clips.iter().map(|c| c.duration).sum::<f64>();
    }

    let detected_names: Vec<String> = detected.iter().map(|d| d.name.to_string()).collect();
    let summary = build_summary(&detected_names, &mood, &direction, total);

    ParsedStickmanScript {
        clips,
        detected_actions: detected_names,
        mood,
        direction,
        duration: total,
        summary,
    }
}

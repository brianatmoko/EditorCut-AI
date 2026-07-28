use serde::{Deserialize, Serialize};

use crate::pose::StickmanPose;
use crate::poses::{AnimationName, get_pose};

const BLEND_DURATION: f64 = 0.25;

fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AnimationClip {
    pub action: String,
    pub start_time: f64,
    pub duration: f64,
    pub direction: Option<String>,
    pub speed: Option<f64>,
}

pub struct StickmanAnimator {
    timeline: Vec<AnimationClip>,
    current_time: f64,
    playing: bool,
    started_at: f64,
    /// When set, overrides timeline-based total_duration (e.g. for 660s cinematic)
    pub override_duration: Option<f64>,
    /// Previous frame's pos_x to calculate smooth velocity
    prev_pose_x: f64,
    /// Accumulated smooth position (filters out jitter from pose pos_x oscillations)
    smooth_pos_x: f64,
}

impl StickmanAnimator {
    pub fn new() -> Self {
        Self {
            timeline: Vec::new(),
            current_time: 0.0,
            playing: false,
            started_at: 0.0,
            override_duration: None,
            prev_pose_x: 0.0,
            smooth_pos_x: 0.0,
        }
    }

    pub fn timeline(&self) -> &Vec<AnimationClip> {
        &self.timeline
    }

    pub fn set_timeline(&mut self, clips: Vec<AnimationClip>) {
        self.timeline = clips;
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.started_at = now_seconds() - self.current_time;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn seek(&mut self, time: f64) {
        self.current_time = time;
        self.prev_pose_x = 0.0;
        self.smooth_pos_x = 0.0;
        if self.playing {
            self.started_at = now_seconds() - time;
        }
    }

    pub fn update(&mut self) -> StickmanPose {
        if self.playing {
            let elapsed = now_seconds() - self.started_at;
            let total = self.total_duration();
            self.current_time = if total > 0.0 {
                elapsed % total
            } else {
                0.0
            };
        }
        
        let mut pose = self.current_pose();
        
        // Smooth position tracking: filter out jitter from rapid pos_x changes
        // Calculate movement delta from current pose's pos_x vs previous frame
        let pos_delta = pose.pos_x - self.prev_pose_x;
        
        // Only accumulate if delta is reasonable (avoid reset jumps)
        // If delta is huge (e.g., timeline jump or reset), snap to new position
        if pos_delta.abs() < 2.0 {
            self.smooth_pos_x += pos_delta;
        } else {
            self.smooth_pos_x = pose.pos_x;
        }
        
        self.prev_pose_x = pose.pos_x;
        pose.pos_x = self.smooth_pos_x;
        
        pose
    }

    pub fn current_pose(&self) -> StickmanPose {
        let t = self.current_time;
        if self.timeline.is_empty() {
            return StickmanPose::default();
        }

        // Find the active clip at time t
        let active = self.timeline.iter().find(|c| {
            t >= c.start_time && t < c.start_time + c.duration
        });

        let active = match active {
            Some(a) => a,
            None => {
                // Past end — return last clip's end pose
                let last = self.timeline.last().unwrap();
                let anim = parse_action(&last.action);
                let mut pose = get_pose(anim, 1.0, last.speed.unwrap_or(1.0));
                if last.direction.as_deref() == Some("left") {
                    pose.pos_x = -pose.pos_x;
                    pose.facing_left = true;
                }
                return pose;
            }
        };

        let anim = parse_action(&active.action);
        let progress = (t - active.start_time) / active.duration;
        let speed = active.speed.unwrap_or(1.0);
        let mut pose = get_pose(anim, progress, speed);

        // Blend with next clip if within BLEND_DURATION of next start
        let next = self.timeline.iter().find(|c| c.start_time > active.start_time);
        if let Some(next_clip) = next {
            let time_to_next = next_clip.start_time - t;
            if time_to_next < BLEND_DURATION {
                let next_anim = parse_action(&next_clip.action);
                let next_pose = get_pose(next_anim, 0.0, next_clip.speed.unwrap_or(1.0));
                let blend_out = time_to_next / BLEND_DURATION;
                pose = StickmanPose::lerp(&pose, &next_pose, ease_in_out(1.0 - blend_out));
            }
        }

        if active.direction.as_deref() == Some("left") {
            pose.pos_x = -pose.pos_x;
            pose.facing_left = true;
        } else {
            pose.facing_left = false;
        }

        pose
    }

    pub fn total_duration(&self) -> f64 {
        if let Some(d) = self.override_duration {
            return d;
        }
        self.timeline.last().map(|c| c.start_time + c.duration).unwrap_or(0.0)
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn time(&self) -> f64 {
        self.current_time
    }

    pub fn active_action(&self) -> String {
        let t = self.current_time;
        if let Some(c) = self.timeline.iter().find(|c| t >= c.start_time && t < c.start_time + c.duration) {
            c.action.clone()
        } else if let Some(last) = self.timeline.last() {
            last.action.clone()
        } else {
            "idle".to_string()
        }
    }
}

impl Default for StickmanAnimator {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_action(name: &str) -> AnimationName {
    match name.to_lowercase().as_str() {
        "idle" | "diam" => AnimationName::Idle,
        "walk" | "jalan" | "berjalan" => AnimationName::Walk,
        "run" | "lari" | "berlari" => AnimationName::Run,
        "sprint" => AnimationName::Sprint,
        "jump" | "lompat" | "melompat" => AnimationName::Jump,
        "wave" | "lambai" => AnimationName::Wave,
        "dance" | "menari" => AnimationName::Dance,
        "punch" | "pukul" | "tinju" => AnimationName::Punch,
        "fall" | "jatuh" | "terjatuh" => AnimationName::Fall,
        "think" | "pikir" => AnimationName::Think,
        "slow_walk" | "slowwalk" | "jalan_lambat" => AnimationName::SlowWalk,
        "crawl" | "merangkak" => AnimationName::Crawl,
        "panic_run" | "panicrun" | "lari_panik" => AnimationName::PanicRun,
        "stealth_walk" | "stealthwalk" | "mengendap" => AnimationName::StealthWalk,
        "sad_walk" | "sadwalk" | "jalan_sedih" => AnimationName::SadWalk,
        "happy_hop" | "happyhop" | "gembira" => AnimationName::HappyHop,
        "dodge" | "menghindar" => AnimationName::Dodge,
        "stumble" | "tersandung" => AnimationName::Stumble,
        "roll" | "guling" => AnimationName::Roll,
        "shoot" | "tembak" | "menembak" => AnimationName::Shoot,
        "aim" | "membidik" | "bidik" => AnimationName::Aim,
        // NEW KICKS
        "roundhouse" | "roundhouse_kick" | "tendang_putar" | "tendang kepala" => AnimationName::RoundhouseKick,
        "front_kick" | "tendang_depan" => AnimationName::FrontKick,
        "side_kick" | "tendang_samping" => AnimationName::SideKick,
        "axe_kick" | "tendang_kapak" | "tendang_atas_bawah" => AnimationName::AxeKick,
        "kick_head" | "tendang_kepala" => AnimationName::KickHead,
        "kick_body" | "tendang_badan" => AnimationName::KickBody,
        "kick_leg" | "tendang_kaki" | "sapu" => AnimationName::KickLeg,
        "flying_kick" | "tendang_terbang" | "lompat_tendang" => AnimationName::FlyingKick,
        "crescent_kick" | "tendang_sabit" => AnimationName::CrescentKick,
        "knee_strike" | "lutut" | "pukul_lutut" => AnimationName::KneeStrike,
        "double_kick" | "tendang_ganda" | "dua_tendangan" => AnimationName::DoubleKick,
        // NEW PUNCHES
        "haymaker" | "pukul_ayun" | "pukul_keras" => AnimationName::Haymaker,
        "body_blow" | "pukul_badan" | "pukul_perut" => AnimationName::BodyBlow,
        "elbow_strike" | "pukul_siku" | "siku" => AnimationName::ElbowStrike,
        "backfist" | "pukul_belakang" | "pukul_balik" => AnimationName::Backfist,
        "palm_strike" | "tampar" | "pukul_telapak" => AnimationName::PalmStrike,
        "hammer_fist" | "pukul_palu" | "pukul_turun" => AnimationName::HammerFist,
        // GRABS & THROWS
        "grab" | "tangkapan" | "menangkap" | "pegang" => AnimationName::Grab,
        "headlock" | "cekik" | "kunci_kepala" => AnimationName::GrabHeadlock,
        "body_slam" | "banting" | "banting_tubuh" => AnimationName::BodySlam,
        "suplex" | "banting_belakang" => AnimationName::Suplex,
        "hip_throw" | "banting_pinggul" => AnimationName::HipThrow,
        "choke_hold" | "jerat" | "mencekik" => AnimationName::ChokeHold,
        "throw_push" | "dorong" | "lempar_dorong" => AnimationName::ThrowPush,
        "clothesline" | "garong" | "sabet" => AnimationName::Clothesline,
        "leg_sweep" | "sapu_kaki" | "sweep" => AnimationName::LegSweep,
        // DEFENSE
        "block_high" | "tahan_atas" | "blok_atas" => AnimationName::BlockHigh,
        "block_mid" | "tahan_tengah" | "blok_tengah" => AnimationName::BlockMid,
        "block_low" | "tahan_bawah" | "blok_bawah" => AnimationName::BlockLow,
        "parry" | "tangkis" | "menangkis" => AnimationName::Parry,
        "weave" | "hindar_badan" | "mengelak" => AnimationName::Weave,
        "step_back" | "mundur" | "langkah_mundur" => AnimationName::StepBack,
        "cross_block" | "blok_silang" | "tahan_silang" => AnimationName::CrossBlock,
        // WEAPONS
        "draw_weapon" | "cabut_senjata" | "tarik_senjata" => AnimationName::DrawWeapon,
        "holster" | "simpan_senjata" | "sarung" => AnimationName::Holster,
        "melee_swing" | "ayun" | "ayun_senjata" => AnimationName::MeleeSwing,
        "melee_stab" | "tusuk" | "tikam" => AnimationName::MeleeStab,
        "throw_weapon" | "lempar_senjata" => AnimationName::ThrowWeapon,
        "weapon_block" | "blok_senjata" | "tahan_senjata" => AnimationName::WeaponBlock,
        // VEHICLES
        "enter_car" | "masuk_mobil" | "naik_mobil" => AnimationName::EnterCarDriver,
        "exit_car" | "keluar_mobil" | "turun_mobil" => AnimationName::ExitCarDriver,
        "drive" | "mengemudi" | "setir" | "nyetir" => AnimationName::Drive,
        "ride_motorcycle" | "naik_motor" | "bonceng_motor" => AnimationName::RideMotorcycle,
        "dismount_motorcycle" | "turun_motor" => AnimationName::DismountMotorcycle,
        "enter_helicopter" | "naik_helikopter" => AnimationName::EnterHelicopter,
        "exit_helicopter" | "turun_helikopter" => AnimationName::ExitHelicopter,
        // ACROBATIC EXTENDED
        "salto" | "salto_forward" | "salto_depan" => AnimationName::SaltoForward,
        "salto_backward" | "salto_belakang" => AnimationName::SaltoBackward,
        "aerial_cartwheel" | "baling_udara" => AnimationName::AerialCartwheel,
        "back_handspring" | "handspring_belakang" => AnimationName::BackHandspring,
        "front_handspring" | "handspring_depan" => AnimationName::FrontHandspring,
        "dive_roll" | "guling_terbang" | "terjang_guling" => AnimationName::DiveRoll,
        "wall_flip" | "flip_tembok" => AnimationName::WallFlip,
        // ENVIRONMENT EXTENDED
        "climb_wall" | "panjat_tembok" => AnimationName::ClimbWall,
        "pull_up" | "tarik_naik" | "pull_up" => AnimationName::PullUp,
        "hang_drop" | "jatuhkan_diri" => AnimationName::HangDrop,
        "open_door" | "buka_pintu" => AnimationName::OpenDoor,
        "close_door" | "tutup_pintu" => AnimationName::CloseDoor,
        "crawl_through" | "merangkak_masuk" | "sembunyi" => AnimationName::CrawlThrough,
        "jump_over" | "lompat_rintangan" | "lompati" => AnimationName::JumpOverObstacle,
        "drop_height" | "lompat_turun" | "turun_tinggi" => AnimationName::DropFromHeight,
        // GROUND & PRONE
        "prone_crawl" | "tiarap" | "merayap" => AnimationName::ProneCrawl,
        "crouch_walk" | "jongkok_jalan" | "jalan_jongkok" => AnimationName::CrouchWalk,
        "ground_recover" | "bangun" | "bangkit" => AnimationName::GroundRecover,
        "limp" | "pincang" | "jalan_pincang" => AnimationName::Limp,
        "stagger" | "stagger_back" | "terhuyung" | "goyah" => AnimationName::StaggerBack,
        "trip_fall" | "tersandung_jatuh" | "kesandung" => AnimationName::TripAndFall,
        "slip" | "tergelincir" | "licin" => AnimationName::Slip,
        // EMOTIONAL EXTENDED
        "celebrate_big" | "sukacita" | "hore" | "yes" => AnimationName::CelebrateArmsUp,
        "fist_pump" | "tinju_ke_atas" | "pump" => AnimationName::CelebrateFistPump,
        "despair_deep" | "putus_asa" | "terpuruk" => AnimationName::DespairDeep,
        "surrender" | "menyerah" | "angkat_tangan" => AnimationName::Surrender,
        "triumph" | "jaya" | "kemenangan" => AnimationName::Triumph,
        "exhausted" | "lelah" | "kelelahan" | "capek" => AnimationName::Exhausted,
        "confident" | "percaya_diri" | "pede" | "cool" => AnimationName::Confident,
        "taunt_provoke" | "ejek" | "mengolok" | "provokasi" => AnimationName::TauntProvoke,
        "bow" | "membungkuk" | "hormat" => AnimationName::Bow,
        "cover_face" | "tutup_muka" | "malu" => AnimationName::CoverFace,
        "cower" | "meringkuk" | "takut" => AnimationName::Cower,
        // MOVEMENT VARIANTS
        "strafe_left" | "geser_kiri" => AnimationName::StrafeLeft,
        "strafe_right" | "geser_kanan" => AnimationName::StrafeRight,
        _ => AnimationName::Idle,
    }
}

#[cfg(feature = "wasm")]
fn now_seconds() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() / 1000.0)
        .unwrap_or(0.0)
}

#[cfg(not(feature = "wasm"))]
fn now_seconds() -> f64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

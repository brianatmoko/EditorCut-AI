use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// ATTACK RANGE SYSTEM — Jarak Serangan & Tangkapan
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RangeType {
    Melee,
    Ranged,
    Throw,
    Grab,
    AreaOfEffect,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttackRange {
    pub min: f64,
    pub optimal: f64,
    pub max: f64,
    pub range_type: RangeType,
    pub damage: f32,
    pub knockback: f32,
    pub startup_frames: f32,
    pub active_frames: f32,
    pub recovery_frames: f32,
    pub hitbox_width: f64,
    pub hitbox_height: f64,
    pub hitbox_offset_x: f64,
    pub hitbox_offset_y: f64,
}

impl AttackRange {
    pub fn is_in_range(&self, distance: f64) -> bool {
        distance >= self.min && distance <= self.max
    }

    pub fn is_in_optimal(&self, distance: f64) -> bool {
        (distance - self.optimal).abs() < 0.15
    }
}

macro_rules! attack_def {
    ($name:ident, $min:expr, $opt:expr, $max:expr, $type:expr, $dmg:expr, $kb:expr, $startup:expr, $active:expr, $recovery:expr, $w:expr, $h:expr, $ox:expr, $oy:expr) => {
        pub fn $name() -> AttackRange {
            AttackRange {
                min: $min, optimal: $opt, max: $max,
                range_type: $type, damage: $dmg, knockback: $kb,
                startup_frames: $startup, active_frames: $active, recovery_frames: $recovery,
                hitbox_width: $w, hitbox_height: $h, hitbox_offset_x: $ox, hitbox_offset_y: $oy,
            }
        }
    };
}

impl AttackRange {
    attack_def!(jab_range, 0.0, 0.4, 0.7, RangeType::Melee, 5.0, 2.0, 0.08, 0.05, 0.10, 0.3, 0.15, 0.4, -0.1);
    attack_def!(cross_range, 0.1, 0.6, 1.0, RangeType::Melee, 8.0, 4.0, 0.12, 0.06, 0.14, 0.35, 0.15, 0.55, -0.1);
    attack_def!(hook_range, 0.05, 0.35, 0.6, RangeType::Melee, 7.0, 5.0, 0.14, 0.05, 0.12, 0.3, 0.2, 0.3, 0.0);
    attack_def!(uppercut_range, 0.0, 0.3, 0.55, RangeType::Melee, 9.0, 6.0, 0.15, 0.06, 0.16, 0.25, 0.3, 0.25, -0.2);
    attack_def!(haymaker_range, 0.1, 0.5, 0.9, RangeType::Melee, 12.0, 8.0, 0.18, 0.08, 0.18, 0.4, 0.15, 0.5, -0.05);
    attack_def!(body_blow_range, 0.0, 0.3, 0.6, RangeType::Melee, 6.0, 3.0, 0.09, 0.05, 0.11, 0.25, 0.25, 0.3, 0.0);
    attack_def!(elbow_range, 0.0, 0.15, 0.3, RangeType::Melee, 6.0, 3.0, 0.06, 0.04, 0.08, 0.15, 0.15, 0.1, 0.0);
    attack_def!(backfist_range, 0.05, 0.35, 0.65, RangeType::Melee, 5.0, 3.0, 0.10, 0.05, 0.12, 0.3, 0.15, 0.35, -0.05);
    attack_def!(palm_strike_range, 0.0, 0.3, 0.55, RangeType::Melee, 6.0, 7.0, 0.10, 0.05, 0.12, 0.25, 0.2, 0.3, -0.1);
    attack_def!(hammer_fist_range, 0.05, 0.3, 0.5, RangeType::Melee, 10.0, 5.0, 0.16, 0.06, 0.14, 0.2, 0.15, 0.25, -0.2);

    attack_def!(front_kick_range, 0.1, 0.5, 0.9, RangeType::Melee, 7.0, 5.0, 0.10, 0.07, 0.14, 0.35, 0.3, 0.4, -0.25);
    attack_def!(roundhouse_range, 0.15, 0.6, 1.0, RangeType::Melee, 10.0, 7.0, 0.16, 0.08, 0.18, 0.4, 0.35, 0.55, -0.2);
    attack_def!(side_kick_range, 0.15, 0.55, 0.95, RangeType::Melee, 9.0, 8.0, 0.14, 0.07, 0.16, 0.35, 0.3, 0.5, -0.15);
    attack_def!(axe_kick_range, 0.1, 0.45, 0.8, RangeType::Melee, 12.0, 6.0, 0.18, 0.06, 0.16, 0.2, 0.4, 0.3, -0.4);
    attack_def!(kick_head_range, 0.15, 0.55, 0.95, RangeType::Melee, 11.0, 6.0, 0.16, 0.07, 0.16, 0.3, 0.4, 0.45, -0.45);
    attack_def!(kick_body_range, 0.1, 0.45, 0.85, RangeType::Melee, 8.0, 5.0, 0.12, 0.06, 0.14, 0.3, 0.3, 0.4, -0.2);
    attack_def!(kick_leg_range, 0.05, 0.3, 0.6, RangeType::Melee, 5.0, 4.0, 0.08, 0.06, 0.12, 0.25, 0.2, 0.25, 0.0);
    attack_def!(flying_kick_range, 0.3, 0.8, 1.4, RangeType::Melee, 14.0, 9.0, 0.22, 0.10, 0.20, 0.4, 0.35, 0.7, -0.2);
    attack_def!(crescent_kick_range, 0.1, 0.5, 0.9, RangeType::Melee, 7.0, 4.0, 0.14, 0.07, 0.14, 0.35, 0.25, 0.45, -0.15);
    attack_def!(knee_strike_range, 0.0, 0.2, 0.35, RangeType::Melee, 8.0, 5.0, 0.06, 0.04, 0.08, 0.15, 0.2, 0.15, -0.15);
    attack_def!(double_kick_range, 0.1, 0.5, 0.85, RangeType::Melee, 12.0, 6.0, 0.20, 0.12, 0.18, 0.3, 0.3, 0.45, -0.2);

    attack_def!(leg_sweep_range, 0.0, 0.25, 0.5, RangeType::Melee, 4.0, 5.0, 0.08, 0.06, 0.10, 0.3, 0.1, 0.2, 0.1);
    attack_def!(slide_tackle_range, 0.2, 0.6, 1.2, RangeType::Melee, 6.0, 8.0, 0.10, 0.12, 0.14, 0.35, 0.15, 0.55, -0.1);
    attack_def!(clothesline_range, 0.1, 0.5, 0.9, RangeType::Melee, 8.0, 7.0, 0.12, 0.06, 0.14, 0.4, 0.2, 0.45, -0.1);
    attack_def!(tackle_range, 0.2, 0.6, 1.1, RangeType::Melee, 10.0, 9.0, 0.14, 0.10, 0.18, 0.35, 0.25, 0.5, -0.15);
    attack_def!(headlock_range, 0.0, 0.2, 0.4, RangeType::Grab, 3.0, 2.0, 0.10, 0.30, 0.20, 0.2, 0.25, 0.15, -0.1);
    attack_def!(body_slam_range, 0.0, 0.25, 0.5, RangeType::Grab, 15.0, 10.0, 0.20, 0.15, 0.25, 0.25, 0.3, 0.2, -0.2);
    attack_def!(suplex_range, 0.0, 0.25, 0.5, RangeType::Grab, 18.0, 12.0, 0.25, 0.15, 0.30, 0.25, 0.35, 0.2, -0.25);
    attack_def!(hip_throw_range, 0.0, 0.2, 0.45, RangeType::Grab, 12.0, 8.0, 0.15, 0.12, 0.20, 0.2, 0.3, 0.15, -0.15);
    attack_def!(choke_hold_range, 0.0, 0.15, 0.3, RangeType::Grab, 4.0, 1.0, 0.12, 0.40, 0.18, 0.15, 0.25, 0.1, -0.1);
    attack_def!(grab_range, 0.0, 0.3, 0.6, RangeType::Grab, 0.0, 0.0, 0.08, 0.10, 0.12, 0.25, 0.2, 0.3, -0.1);
    attack_def!(throw_push_range, 0.0, 0.3, 0.6, RangeType::Grab, 5.0, 8.0, 0.10, 0.08, 0.14, 0.3, 0.2, 0.3, -0.1);

    attack_def!(shoot_pistol_range, 0.5, 3.0, 8.0, RangeType::Ranged, 8.0, 3.0, 0.06, 0.04, 0.10, 0.1, 0.1, 0.6, -0.15);
    attack_def!(shoot_rifle_range, 0.8, 5.0, 15.0, RangeType::Ranged, 12.0, 4.0, 0.10, 0.06, 0.14, 0.1, 0.1, 0.7, -0.15);
    attack_def!(throw_weapon_range, 0.3, 1.0, 3.0, RangeType::Throw, 10.0, 5.0, 0.12, 0.06, 0.14, 0.15, 0.15, 0.5, -0.1);
    attack_def!(melee_swing_range, 0.2, 0.7, 1.2, RangeType::Melee, 14.0, 7.0, 0.16, 0.08, 0.18, 0.5, 0.2, 0.6, -0.1);
    attack_def!(melee_stab_range, 0.05, 0.35, 0.65, RangeType::Melee, 10.0, 3.0, 0.08, 0.05, 0.12, 0.3, 0.15, 0.3, -0.1);
    attack_def!(weapon_block_range, 0.0, 0.3, 0.5, RangeType::Melee, 0.0, 0.0, 0.06, 0.15, 0.08, 0.3, 0.25, 0.25, -0.1);
}

pub fn get_attack_range(name: &str) -> Option<AttackRange> {
    match name {
        "jab" => Some(AttackRange::jab_range()),
        "cross" => Some(AttackRange::cross_range()),
        "hook" => Some(AttackRange::hook_range()),
        "uppercut" => Some(AttackRange::uppercut_range()),
        "haymaker" => Some(AttackRange::haymaker_range()),
        "body_blow" => Some(AttackRange::body_blow_range()),
        "elbow_strike" => Some(AttackRange::elbow_range()),
        "backfist" => Some(AttackRange::backfist_range()),
        "palm_strike" => Some(AttackRange::palm_strike_range()),
        "hammer_fist" => Some(AttackRange::hammer_fist_range()),
        "front_kick" => Some(AttackRange::front_kick_range()),
        "roundhouse" => Some(AttackRange::roundhouse_range()),
        "side_kick" => Some(AttackRange::side_kick_range()),
        "axe_kick" => Some(AttackRange::axe_kick_range()),
        "kick_head" => Some(AttackRange::kick_head_range()),
        "kick_body" => Some(AttackRange::kick_body_range()),
        "kick_leg" => Some(AttackRange::kick_leg_range()),
        "flying_kick" => Some(AttackRange::flying_kick_range()),
        "crescent_kick" => Some(AttackRange::crescent_kick_range()),
        "knee_strike" => Some(AttackRange::knee_strike_range()),
        "double_kick" => Some(AttackRange::double_kick_range()),
        "leg_sweep" => Some(AttackRange::leg_sweep_range()),
        "slide_tackle" => Some(AttackRange::slide_tackle_range()),
        "clothesline" => Some(AttackRange::clothesline_range()),
        "tackle" => Some(AttackRange::tackle_range()),
        "headlock" => Some(AttackRange::headlock_range()),
        "body_slam" => Some(AttackRange::body_slam_range()),
        "suplex" => Some(AttackRange::suplex_range()),
        "hip_throw" => Some(AttackRange::hip_throw_range()),
        "choke_hold" => Some(AttackRange::choke_hold_range()),
        "grab" => Some(AttackRange::grab_range()),
        "throw_push" => Some(AttackRange::throw_push_range()),
        "shoot" | "shoot_pistol" => Some(AttackRange::shoot_pistol_range()),
        "shoot_rifle" => Some(AttackRange::shoot_rifle_range()),
        "throw_weapon" => Some(AttackRange::throw_weapon_range()),
        "melee_swing" => Some(AttackRange::melee_swing_range()),
        "melee_stab" => Some(AttackRange::melee_stab_range()),
        "weapon_block" => Some(AttackRange::weapon_block_range()),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════
// EXPANDED SHOT TYPE — 18+ Sinematik Shot Variants
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ShotType {
    ExtremeWide,
    Wide,
    FullShot,
    Medium,
    MediumCloseUp,
    CloseUp,
    ExtremeCloseUp,
    OverShoulder,
    TwoShot,
    GroupShot,
    DutchAngle,
    LowAngle,
    HighAngle,
    PointOfView,
    InsertShot,
    Cutaway,
    ActionFollow,
    Establishing,
    ReactionShot,
}

impl ShotType {
    /// Zoom factor hint: < 1.0 = zoom out (wider), > 1.0 = zoom in (tighter)
    pub fn zoom_hint(&self) -> f32 {
        match self {
            ShotType::ExtremeWide | ShotType::Establishing => 0.5,
            ShotType::Wide => 0.65,
            ShotType::FullShot => 0.8,
            ShotType::Medium | ShotType::TwoShot | ShotType::GroupShot => 1.0,
            ShotType::MediumCloseUp => 1.15,
            ShotType::CloseUp | ShotType::OverShoulder | ShotType::ReactionShot => 1.3,
            ShotType::ExtremeCloseUp | ShotType::InsertShot => 1.6,
            ShotType::DutchAngle | ShotType::LowAngle | ShotType::HighAngle => 1.1,
            ShotType::PointOfView => 0.9,
            ShotType::Cutaway => 1.0,
            ShotType::ActionFollow => 0.85,
        }
    }

    /// Offset untuk menggeser pusat kamera RELATIVE terhadap target.
    /// Nilai positif = kamera bergeser ke KANAN target → target muncul di KIRI layar.
    /// Nilai negatif = kamera bergeser ke KIRI target → target muncul di KANAN layar.
    pub fn framing_offset_x(&self) -> f32 {
        match self {
            // ActionFollow: target di 1/3 kiri layar → geser kamera 0.45 unit ke kanan target
            ShotType::ActionFollow => 0.45,
            // OverShoulder: pembicara di kiri layar, lawan bicara di kanan
            ShotType::OverShoulder => 0.20,
            // sisanya: target di tengah
            _ => 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// CAMERA MOVEMENT — 14 Gerakan Kamera Sinematik
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum CameraMovement {
    #[default]
    None,
    Pan,
    Tilt,
    DollyIn,
    DollyOut,
    Truck,
    Pedestal,
    ZoomIn,
    ZoomOut,
    RackFocus,
    WhipPan,
    Follow,
    Orbit,
    Crane,
    Boom,
}

// ═══════════════════════════════════════════════════════════════
// CAMERA TRANSITION — 10 Jenis Transisi
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum CameraTransition {
    #[default]
    Cut,
    FadeToBlack,
    FadeFromBlack,
    Dissolve,
    Wipe,
    SmashCut,
    IrisIn,
    IrisOut,
    CrossFade,
    Push,
}

// ═══════════════════════════════════════════════════════════════
// CAMERA SHOT — Frame Kamera dengan Movement & Transition
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CameraShot {
    pub shot_type: ShotType,
    pub target_entity_id: Option<String>,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
    pub shake: f32,
    #[serde(default)]
    pub movement: CameraMovement,
    #[serde(default)]
    pub transition: CameraTransition,
    /// Intensity of the current movement (0.0–1.0), e.g. dolly speed, orbit radius
    #[serde(default)]
    pub movement_intensity: f32,
    /// Horizontal tilt angle in degrees (for DutchAngle)
    #[serde(default)]
    pub tilt_angle: f32,
    /// Depth of field hint: 0.0 = deep focus, 1.0 = shallow focus
    #[serde(default)]
    pub depth_of_field: f32,
    /// Secondary entity for TwoShot / OverShoulder framing
    #[serde(default)]
    pub secondary_entity_id: Option<String>,
    /// Rule-of-thirds offset: which third the target sits in (-1=left, 0=center, 1=right)
    #[serde(default)]
    pub rule_of_thirds: i32,
}

impl Default for CameraShot {
    fn default() -> Self {
        Self {
            shot_type: ShotType::Wide,
            target_entity_id: None,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            shake: 0.0,
            movement: CameraMovement::None,
            transition: CameraTransition::Cut,
            movement_intensity: 0.0,
            tilt_angle: 0.0,
            depth_of_field: 0.0,
            secondary_entity_id: None,
            rule_of_thirds: 0,
        }
    }
}

impl CameraShot {
    pub fn lerp(&self, other: &CameraShot, t: f32) -> CameraShot {
        CameraShot {
            shot_type: if t < 0.5 { self.shot_type.clone() } else { other.shot_type.clone() },
            target_entity_id: if t < 0.5 { self.target_entity_id.clone() } else { other.target_entity_id.clone() },
            pan_x: self.pan_x + (other.pan_x - self.pan_x) * t,
            pan_y: self.pan_y + (other.pan_y - self.pan_y) * t,
            zoom: (self.zoom + (other.zoom - self.zoom) * t).clamp(0.5, 3.0),
            shake: self.shake + (other.shake - self.shake) * t,
            movement: CameraMovement::None,
            transition: self.transition.clone(),
            movement_intensity: self.movement_intensity + (other.movement_intensity - self.movement_intensity) * t,
            tilt_angle: self.tilt_angle + (other.tilt_angle - self.tilt_angle) * t,
            depth_of_field: self.depth_of_field + (other.depth_of_field - self.depth_of_field) * t,
            secondary_entity_id: if t < 0.5 { self.secondary_entity_id.clone() } else { other.secondary_entity_id.clone() },
            rule_of_thirds: if t < 0.5 { self.rule_of_thirds } else { other.rule_of_thirds },
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// SMART CAMERA DIRECTOR — Sutradara Kamera Otomatis
// ═══════════════════════════════════════════════════════════════

/// An action beat marks a noteworthy moment in the scene.
#[derive(Clone, Debug)]
pub struct ActionBeat {
    pub time: f64,
    pub entity_id: String,
    pub action: String,
    pub is_impact: bool,
    pub is_epic: bool,
    pub priority: u32,
}

impl ActionBeat {
    pub fn classify(action: &str) -> (bool, bool) {
        let a = action.to_lowercase();
        let is_impact = matches!(
            a.as_str(),
            "punch" | "kick" | "hit" | "jab" | "cross" | "hook" | "uppercut"
                | "haymaker" | "body_blow" | "elbow_strike" | "backfist" | "palm_strike"
                | "hammer_fist" | "front_kick" | "roundhouse" | "side_kick" | "axe_kick"
                | "kick_head" | "kick_body" | "kick_leg" | "knee_strike"
                | "slash" | "stab" | "shoot" | "shoot_pistol" | "shoot_rifle"
                | "headbutt" | "clothesline" | "leg_sweep" | "weapon_block"
        );
        let is_epic = matches!(
            a.as_str(),
            "flying_kick" | "double_kick" | "suplex" | "body_slam" | "hip_throw"
                | "slide_tackle" | "tackle" | "haymaker" | "body_slam"
                | "powerbomb" | "diving_attack" | "crescent_kick" | "axe_kick"
                | "hammer_fist" | "melee_swing" | "death_blow"
                | "explosion" | "crash" | "shatter"
        );
        (is_impact, is_epic)
    }
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SmartCameraDirector {
    /// Minimum duration (seconds) to hold a shot before re-evaluating
    pub min_shot_hold: f64,
    /// Whether to use dynamic shot selection (false = use act camera as-is)
    pub enabled: bool,
    /// Timestamp when the current dynamic shot was selected (updated by select_shot)
    #[serde(default)]
    pub last_shot_time: f64,
    /// The last selected shot type (for min_shot_hold logic)
    #[serde(default)]
    pub last_shot: Option<CameraShot>,
}

impl Default for SmartCameraDirector {
    fn default() -> Self {
        Self {
            min_shot_hold: 2.5,
            enabled: true,
            last_shot_time: -1.0,
            last_shot: None,
        }
    }
}

impl SmartCameraDirector {
    /// Select the best CameraShot for the current moment in an act.
    ///
    /// Priority order (highest first):
    /// 1. Epic moment → dramatic shot (LowAngle, DutchAngle, ExtremeCloseUp)
    /// 2. Active dialogue → shot of speaker (CloseUp / MediumCloseUp / OverShoulder)
    /// 3. Combat impact → impact shot (CloseUp impact detail + shake)
    /// 4. Action/tracking → ActionFollow
    /// 5. Establishing / group → Wide / ExtremeWide
    /// 6. Default → Medium if 1–2 entities, Wide if 3+
    pub fn select_shot(
        &mut self,
        act: &CinematicAct,
        current_time: f64,
        _last_shot: Option<&CameraShot>,
        beats: &[ActionBeat],
    ) -> CameraShot {
        if !self.enabled {
            return act.camera.clone();
        }

        // ── Hold the current shot for min_shot_hold seconds before re-evaluating ───
        if self.last_shot.is_some() && self.last_shot_time >= 0.0 && (current_time - self.last_shot_time).abs() < self.min_shot_hold {
            return self.last_shot.clone().unwrap();
        }

        let entity_count = act.entities.len();
        let active_beats: Vec<&ActionBeat> = beats.iter()
            .filter(|b| (current_time - b.time).abs() < 0.5)
            .collect();

        let new_shot = self.evaluate_shot(act, current_time, &active_beats, entity_count);
        // Hysteresis: don't reset hold timer if the new shot is structurally identical.
        // Also compare zoom within a tolerance to prevent rapid zoom oscillation.
        let same_as_last = self.last_shot.as_ref().map_or(false, |last| {
            let zoom_close = (last.zoom - new_shot.zoom).abs() < 0.2;
            let pan_close = (last.pan_x - new_shot.pan_x).abs() < 0.1;
            last.shot_type == new_shot.shot_type
                && last.target_entity_id == new_shot.target_entity_id
                && zoom_close
                && pan_close
        });
        if !same_as_last {
            self.last_shot_time = current_time;
            self.last_shot = Some(new_shot.clone());
        } else {
            // Smoothly interpolate zoom/pan toward the new shot (avoid freezing)
            if let Some(ref mut last) = self.last_shot {
                let lerp_rate = 0.05; // gentle convergence per frame
                last.zoom = last.zoom + (new_shot.zoom - last.zoom) * lerp_rate;
                last.pan_x = last.pan_x + (new_shot.pan_x - last.pan_x) * lerp_rate;
                last.pan_y = last.pan_y + (new_shot.pan_y - last.pan_y) * lerp_rate;
                last.shake = last.shake + (new_shot.shake - last.shake) * lerp_rate;
            }
        }
        self.last_shot.clone().unwrap_or(new_shot)
    }

    fn evaluate_shot(
        &self,
        act: &CinematicAct,
        current_time: f64,
        active_beats: &[&ActionBeat],
        entity_count: usize,
    ) -> CameraShot {
        // ── Priority 1: Epic moment ────────────────────────────
        if let Some(epic) = active_beats.iter().find(|b| b.is_epic) {
            // Stable shot: pick one based on entity id hash so it doesn't oscillate
            let st = match epic.entity_id.chars().map(|c| c as u32).sum::<u32>() % 3 {
                0 => ShotType::LowAngle,
                1 => ShotType::DutchAngle,
                _ => ShotType::ExtremeCloseUp,
            };
            let zh = st.zoom_hint() * 1.1;
            let ta = if st == ShotType::DutchAngle { 15.0 } else { 0.0 };

            return CameraShot {
                shot_type: st,
                target_entity_id: Some(epic.entity_id.clone()),
                pan_x: 0.0,
                pan_y: 0.0,
                zoom: zh,
                shake: 6.0,
                movement: CameraMovement::None,
                transition: CameraTransition::Cut,
                movement_intensity: 0.0,
                tilt_angle: ta,
                depth_of_field: 0.8,
                secondary_entity_id: None,
                rule_of_thirds: 0,
            };
        }

        // ── Priority 2: Active dialogue ────────────────────────
        if let Some(dlg) = act.dialogues.iter()
            .find(|d| current_time >= d.start_time && current_time <= d.start_time + d.duration)
        {
            let speaker = act.entities.iter().find(|e| e.id == dlg.entity_id);
            let entity_x = speaker.map(|e| e.pos_x).unwrap_or(0.0);

            let (shot_type, secondary, ofs) = if let Some(px) = act.entities.iter()
                .filter(|e| e.id != dlg.entity_id && (e.pos_x - entity_x).abs() < 3.0)
                .map(|e| e.pos_x).next()
            {
                if entity_x < px {
                    (ShotType::OverShoulder, Some(dlg.entity_id.clone()), 0.20)
                } else {
                    (ShotType::OverShoulder, Some(dlg.entity_id.clone()), -0.20)
                }
            } else {
                match dlg.emotion.as_str() {
                    "shout" => (ShotType::CloseUp, None, 0.0),
                    "whisper" => (ShotType::ExtremeCloseUp, None, 0.0),
                    _ => (ShotType::MediumCloseUp, None, 0.0),
                }
            };

            let zh = shot_type.zoom_hint();
            return CameraShot {
                shot_type,
                target_entity_id: Some(dlg.entity_id.clone()),
                pan_x: ofs,
                pan_y: 0.0,
                zoom: zh,
                shake: 0.0,
                movement: CameraMovement::None,
                transition: CameraTransition::Cut,
                movement_intensity: 0.0,
                tilt_angle: 0.0,
                depth_of_field: 0.3,
                secondary_entity_id: secondary,
                rule_of_thirds: 0,
            };
        }

        // ── Priority 3: Combat impact ──────────────────────────
        if let Some(impact) = active_beats.iter().find(|b| b.is_impact) {
            return CameraShot {
                shot_type: ShotType::CloseUp,
                target_entity_id: Some(impact.entity_id.clone()),
                pan_x: 0.0,
                pan_y: 0.0,
                zoom: ShotType::CloseUp.zoom_hint(),
                shake: 4.0,
                movement: CameraMovement::None,
                transition: CameraTransition::Cut,
                movement_intensity: 0.0,
                tilt_angle: 0.0,
                depth_of_field: 0.5,
                secondary_entity_id: None,
                rule_of_thirds: 0,
            };
        }

        // ── Priority 4: Action tracking ────────────────────────
        let any_action = act.entities.iter().any(|e| {
            let a = e.action.to_lowercase();
            matches!(a.as_str(),
                "run" | "running" | "lari" | "berlari" | "sprint" | "chase"
                | "dodge" | "roll" | "jump" | "melompat" | "vault" | "climb"
                | "slide" | "crawl" | "swim" | "fly"
            )
        });

        if any_action && entity_count <= 3 {
            let runner = act.entities.iter()
                .find(|e| matches!(e.action.to_lowercase().as_str(),
                    "run" | "running" | "lari" | "berlari" | "sprint" | "chase"
                ))
                .or_else(|| act.entities.first());

            if let Some(r) = runner {
                return CameraShot {
                    shot_type: ShotType::ActionFollow,
                    target_entity_id: Some(r.id.clone()),
                    pan_x: 0.0,
                    pan_y: 0.0,
                    zoom: ShotType::ActionFollow.zoom_hint(),
                    shake: 2.0,
                    movement: CameraMovement::Follow,
                    transition: CameraTransition::Cut,
                    movement_intensity: 0.4,
                    tilt_angle: 0.0,
                    depth_of_field: 0.2,
                    secondary_entity_id: None,
                    rule_of_thirds: -1,
                };
            }
        }

        // ── Priority 5: Wide / Establishing for large groups ───
        if entity_count >= 4 {
            return CameraShot {
                shot_type: ShotType::Wide,
                target_entity_id: None,
                pan_x: 0.0,
                pan_y: 0.0,
                zoom: ShotType::Wide.zoom_hint(),
                shake: 0.0,
                movement: CameraMovement::Pan,
                transition: CameraTransition::Cut,
                movement_intensity: 0.1,
                tilt_angle: 0.0,
                depth_of_field: 0.0,
                secondary_entity_id: None,
                rule_of_thirds: 0,
            };
        }

        // ── Priority 6: Default framing ────────────────────────
        match entity_count {
            0 => act.camera.clone(),
            1 => {
                CameraShot {
                    shot_type: ShotType::Medium,
                    target_entity_id: Some(act.entities[0].id.clone()),
                    pan_x: 0.0,
                    pan_y: 0.0,
                    zoom: ShotType::Medium.zoom_hint(),
                    shake: 0.0,
                    movement: CameraMovement::None,
                    transition: CameraTransition::Cut,
                    movement_intensity: 0.0,
                    tilt_angle: 0.0,
                    depth_of_field: 0.0,
                    secondary_entity_id: None,
                    rule_of_thirds: 0,
                }
            }
            2 => {
                CameraShot {
                    shot_type: ShotType::TwoShot,
                    target_entity_id: Some(act.entities[0].id.clone()),
                    pan_x: 0.0,
                    pan_y: 0.0,
                    zoom: ShotType::Medium.zoom_hint(),
                    shake: 0.0,
                    movement: CameraMovement::None,
                    transition: CameraTransition::Cut,
                    movement_intensity: 0.0,
                    tilt_angle: 0.0,
                    depth_of_field: 0.0,
                    secondary_entity_id: Some(act.entities[1].id.clone()),
                    rule_of_thirds: 0,
                }
            }
            _ => {
                CameraShot {
                    shot_type: ShotType::GroupShot,
                    target_entity_id: None,
                    pan_x: 0.0,
                    pan_y: 0.0,
                    zoom: ShotType::GroupShot.zoom_hint(),
                    shake: 0.0,
                    movement: CameraMovement::Pan,
                    transition: CameraTransition::Cut,
                    movement_intensity: 0.1,
                    tilt_angle: 0.0,
                    depth_of_field: 0.0,
                    secondary_entity_id: None,
                    rule_of_thirds: 0,
                }
            }
        }
    }

    /// Extract action beats from an act for the smart camera to use.
    pub fn extract_beats(act: &CinematicAct) -> Vec<ActionBeat> {
        let mut beats = Vec::new();

        let n_entities = act.entities.len().max(1);
        for (i, entity) in act.entities.iter().enumerate() {
            let (is_impact, is_epic) = ActionBeat::classify(&entity.action);
            if is_impact || is_epic {
                let priority = if is_epic { 3 } else { 2 };
                // Place beat at when movement completes, not evenly distributed.
                // Estimate when entity reaches target: use distance / speed formula
                // matching executor.rs (base_speed ~0.6 for actions, ~1.2 for run).
                let target_x = entity.end_x.unwrap_or(entity.pos_x);
                let distance = (target_x - entity.pos_x).abs();
                let base_speed = if entity.action.contains("run") || entity.action.contains("sprint") {
                    1.2_f32
                } else if entity.action.contains("walk") {
                    0.4_f32
                } else {
                    0.6_f32
                };
                let movement_time = if distance > 0.01 {
                    (distance / base_speed) as f64
                } else {
                    0.0
                };
                // Beat fires when movement completes, or at 60% of act if no movement
                let beat_time = if movement_time > 0.0 && movement_time < act.duration {
                    act.start_time + movement_time
                } else {
                    // No movement: distribute beats across the act, not all at start
                    act.start_time + act.duration * (i as f64 + 1.0) / (n_entities as f64 + 1.0)
                };
                beats.push(ActionBeat {
                    time: beat_time,
                    entity_id: entity.id.clone(),
                    action: entity.action.clone(),
                    is_impact,
                    is_epic,
                    priority,
                });
            }
        }

        // Also add mid-act beats from end_x / end_y movement (position changes are notable)
        for entity in &act.entities {
            if entity.end_x.is_some() || entity.end_y.is_some() {
                beats.push(ActionBeat {
                    time: act.start_time + act.duration * 0.5,
                    entity_id: entity.id.clone(),
                    action: "move".to_string(),
                    is_impact: false,
                    is_epic: false,
                    priority: 1,
                });
            }
        }

        beats.sort_by(|a, b| b.priority.cmp(&a.priority));
        beats
    }
}

// ═══════════════════════════════════════════════════════════════
// CORE TYPES — StageEntity, DialogueLine, CinematicAct, CinematicMovie
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StageEntity {
    pub id: String,
    pub character_skin_id: String,
    pub name: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub action: String,
    pub facing_left: bool,
    #[serde(default)]
    pub end_x: Option<f32>,
    #[serde(default)]
    pub end_y: Option<f32>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub action_variant: Option<String>,
}

impl Default for StageEntity {
    fn default() -> Self {
        Self {
            id: String::new(),
            character_skin_id: String::new(),
            name: String::new(),
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 1.0,
            action: "idle".to_string(),
            facing_left: false,
            end_x: None,
            end_y: None,
            target_id: None,
            action_variant: None,
        }
    }
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DialogueLine {
    pub entity_id: String,
    pub text: String,
    pub start_time: f64,
    pub duration: f64,
    #[serde(default)]
    pub emotion: String,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CinematicAct {
    pub act_number: u32,
    pub title: String,
    /// Detailed directing notes: character motivation, dramatic purpose, staging details
    pub description: String,
    /// Background/environment theme: "city", "cyberpunk", "forest", "room", "alley", etc.
    pub theme: String,
    /// Emotional arc type for this beat: "establish", "rising_action", "tension", "climax",
    /// "falling_action", "resolution", "hope", "despair", "betrayal", "revelation", "sacrifice"
    #[serde(default)]
    pub emotional_tone: String,
    /// Action intensity 0.0–1.0: 0.0=calm dialogue, 0.3=walking, 0.6=combat, 1.0=climax/explosion
    #[serde(default)]
    pub intensity: f32,
    pub start_time: f64,
    pub duration: f64,
    pub entities: Vec<StageEntity>,
    pub camera: CameraShot,
    #[serde(default)]
    pub dialogues: Vec<DialogueLine>,
    /// Legacy act-to-act transition type: "cut", "fade", "smash_cut", "wipe"
    #[serde(default)]
    pub transition: Option<String>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CinematicMovie {
    pub title: String,
    pub summary: String,
    pub total_duration: f64,
    pub acts: Vec<CinematicAct>,
}

use crate::shapes::{
    limb, limb_nc, circle_part, circle_part_joint, ellipse,
    PartShape, ColorRole, CharacterDef,
};

// ─── Character factory ───────────────────────────────────────────────────────
pub fn get_character(name: &str) -> CharacterDef {
    match name {
        "anonymous_back"  => mannequin_back(),
        "anonymous_left"  => mannequin_side(false),
        "anonymous_right" => mannequin_side(true),
        "casual"          => casual_front(),
        "casual_back"     => casual_back(),
        "casual_left"     => casual_side(false),
        "casual_right"    => casual_side(true),
        "robot"           => robot_front(),
        "robot_back"      => robot_back(),
        "robot_left"      => robot_side(false),
        "robot_right"     => robot_side(true),
        _                 => casual_front(),
    }
}

// ─── PROPORTIONS ─────────────────────────────────────────────────────────────
//
// Bone lengths (world units) from render.rs:
//   HEAD_R   = 0.100  → bone_length = 0.200 (diameter)
//   NECK     = 0.040
//   TORSO    = 0.260
//   SHOULDER_W = 0.120 (clavicle half-span, bone length is SHOULDER_W)
//   HIP_W    = 0.080  (hip half-span, bone length is HIP_W)
//   UPPER_ARM = 0.150
//   FOREARM  = 0.130
//   HAND     = 0.060
//   UPPER_LEG = 0.220
//   LOWER_LEG = 0.200
//   FOOT     = 0.090
//
// w = desired_half_width_world / bone_length_world
//
// Desired half-widths (anatomically calibrated for a slender mannequin):
//   Torso (at hips)  : 0.070 / 0.260 = 0.27
//   Torso (at shoulders): 0.100 / 0.260 = 0.38
//   Neck             : 0.020 / 0.040 = 0.50
//   Upper arm        : 0.030 / 0.150 = 0.20
//   Forearm          : 0.025 / 0.130 = 0.19
//   Hand             : 0.028 / 0.060 = 0.47
//   Upper leg (thigh): 0.045 / 0.220 = 0.20
//   Lower leg (calf) : 0.032 / 0.200 = 0.16
//   Foot ellipse     : rx=1.5, ry=0.45 (in bone-length fractions)

const W_TORSO_BOT: f32 = 0.27;   // hips end (wider)
const W_TORSO_TOP: f32 = 0.38;   // shoulder end (wider)
const W_NECK:      f32 = 0.50;
const W_UA:        f32 = 0.20;   // upper arm
const W_FA:        f32 = 0.19;   // forearm
const W_HAND:      f32 = 0.47;
const W_UL:        f32 = 0.20;   // upper leg
const W_LL:        f32 = 0.16;   // lower leg

// ─── FRONT VIEW ─────────────────────────────────────────────────────────────
fn mannequin_front() -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();

    // ─ BACK side: right arm + right leg (drawn behind torso) ─────────────
    // Arms extend from shoulder point. Start=-0.20 overlaps into the shoulder
    // joint area so no gap appears between arm and torso.
    p.push(limb("ua_r",    "upper_arm_r", -0.15, 1.0, W_UA, W_FA,   ColorRole::White, -30));
    p.push(limb("fa_r",    "forearm_r",    0.0,  1.0, W_FA, W_FA,   ColorRole::White, -29));
    p.push(circle_part("hand_r", "hand_r", 0.5,  0.0, W_HAND, ColorRole::White, -28));

    p.push(limb("ul_r",    "upper_leg_r",  0.0,  1.0, W_UL, W_LL,   ColorRole::White, -20));
    p.push(circle_part_joint("knee_r", "upper_leg_r", 1.0, 0.0, W_LL, ColorRole::White, -19));
    p.push(limb("ll_r",    "lower_leg_r",  0.0,  1.0, W_LL, W_LL*0.88, ColorRole::White, -18));
    p.push(ellipse("foot_r", "foot_r", 1.0, 0.0, 1.6, 0.50, ColorRole::White, -17));

    // ─ TORSO (center, draws over back limbs) ──────────────────────────────
    // Torso goes from hips (y1=0) up to shoulders (y2=0.26)
    // start=-0.05 extends down slightly to cover hip joint area
    // end=1.05 extends up slightly to cover shoulder joint area
    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT, W_TORSO_TOP, ColorRole::White, 0));

    // Hip joint fill caps (hides clavicle/hip connector bones)
    p.push(circle_part("hip_fill",    "torso", 0.0, 0.0, W_TORSO_BOT, ColorRole::White, 1));
    p.push(circle_part("shoulder_fill","torso", 1.0, 0.0, W_TORSO_TOP, ColorRole::White, 2));

    // Neck: extends from torso top up to head base
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK, W_NECK * 0.85, ColorRole::White, 3));

    // ─ HEAD: perfect oval ─────────────────────────────────────────────────
    // Head bone: x1=neck_top, length=0.20 (= 2*HEAD_R)
    // offset_x=0.5 centers the ellipse on the bone midpoint (= head center)
    // rx=0.60 → half-width = 0.60 * 0.20 = 0.12  (≈ 60% of HEAD_R each side)
    // ry=0.50 → half-height = 0.50 * 0.20 = 0.10 = HEAD_R (hemisphere)
    p.push(ellipse("head", "head", 0.5, 0.0, 0.62, 0.52, ColorRole::White, 5));

    // ─ FRONT side: left leg + left arm (drawn over torso) ────────────────
    p.push(limb("ul_l",    "upper_leg_l",  0.0,  1.0, W_UL, W_LL,   ColorRole::White, 10));
    p.push(circle_part_joint("knee_l", "upper_leg_l", 1.0, 0.0, W_LL, ColorRole::White, 11));
    p.push(limb("ll_l",    "lower_leg_l",  0.0,  1.0, W_LL, W_LL*0.88, ColorRole::White, 12));
    p.push(ellipse("foot_l", "foot_l", 1.0, 0.0, 1.6, 0.50, ColorRole::White, 13));

    p.push(limb("ua_l",    "upper_arm_l", -0.15, 1.0, W_UA, W_FA,   ColorRole::White, 20));
    p.push(limb("fa_l",    "forearm_l",    0.0,  1.0, W_FA, W_FA,   ColorRole::White, 21));
    p.push(circle_part("hand_l", "hand_l", 0.5,  0.0, W_HAND, ColorRole::White, 22));

    CharacterDef { name: "anonymous_front".to_string(), parts: p }
}

// ─── BACK VIEW ───────────────────────────────────────────────────────────────
fn mannequin_back() -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();

    // Back view: L/R appearance is mirrored (left arm appears on screen right)
    p.push(limb("ua_l",    "upper_arm_l", -0.15, 1.0, W_UA, W_FA,   ColorRole::White, -30));
    p.push(limb("fa_l",    "forearm_l",    0.0,  1.0, W_FA, W_FA,   ColorRole::White, -29));
    p.push(circle_part("hand_l", "hand_l", 0.5,  0.0, W_HAND, ColorRole::White, -28));

    p.push(limb("ul_l",    "upper_leg_l",  0.0,  1.0, W_UL, W_LL,   ColorRole::White, -20));
    p.push(circle_part_joint("knee_l", "upper_leg_l", 1.0, 0.0, W_LL, ColorRole::White, -19));
    p.push(limb("ll_l",    "lower_leg_l",  0.0,  1.0, W_LL, W_LL*0.88, ColorRole::White, -18));
    p.push(ellipse("foot_l", "foot_l", 1.0, 0.0, 1.6, 0.50, ColorRole::White, -17));

    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT, W_TORSO_TOP, ColorRole::White, 0));
    p.push(circle_part("hip_fill",     "torso", 0.0, 0.0, W_TORSO_BOT, ColorRole::White, 1));
    p.push(circle_part("shoulder_fill","torso", 1.0, 0.0, W_TORSO_TOP, ColorRole::White, 2));
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK, W_NECK * 0.85, ColorRole::White, 3));
    // Back of head (no face)
    p.push(ellipse("head", "head", 0.5, 0.0, 0.62, 0.52, ColorRole::White, 5));

    p.push(limb("ul_r",    "upper_leg_r",  0.0,  1.0, W_UL, W_LL,   ColorRole::White, 10));
    p.push(circle_part_joint("knee_r", "upper_leg_r", 1.0, 0.0, W_LL, ColorRole::White, 11));
    p.push(limb("ll_r",    "lower_leg_r",  0.0,  1.0, W_LL, W_LL*0.88, ColorRole::White, 12));
    p.push(ellipse("foot_r", "foot_r", 1.0, 0.0, 1.6, 0.50, ColorRole::White, 13));

    p.push(limb("ua_r",    "upper_arm_r", -0.15, 1.0, W_UA, W_FA,   ColorRole::White, 20));
    p.push(limb("fa_r",    "forearm_r",    0.0,  1.0, W_FA, W_FA,   ColorRole::White, 21));
    p.push(circle_part("hand_r", "hand_r", 0.5,  0.0, W_HAND, ColorRole::White, 22));

    CharacterDef { name: "anonymous_back".to_string(), parts: p }
}

// ─── SIDE VIEW ───────────────────────────────────────────────────────────────
// From the side, the character is visually narrower.
// Back arm/leg = "r" bones (behind). Front arm/leg = "l" bones (in front).
fn mannequin_side(facing_right: bool) -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();

    let ts = 0.50_f32; // torso width scale for side view (thinner)
    let ls = 0.85_f32; // limb width scale for side view

    // Back leg
    p.push(limb("ul_back","upper_leg_r", 0.0, 1.0, W_UL*ls, W_LL*ls, ColorRole::White, -20));
    p.push(circle_part_joint("knee_back","upper_leg_r",1.0,0.0,W_LL*ls,ColorRole::White,-19));
    p.push(limb("ll_back","lower_leg_r", 0.0, 1.0, W_LL*ls, W_LL*ls*0.88, ColorRole::White, -18));
    p.push(ellipse("foot_back","foot_r", 1.0, 0.0, 1.4, 0.45, ColorRole::White, -17));

    // Back arm
    p.push(limb("ua_back","upper_arm_r",-0.15,1.0,W_UA*ls,W_FA*ls, ColorRole::White, -10));
    p.push(limb("fa_back","forearm_r",   0.0, 1.0,W_FA*ls,W_FA*ls, ColorRole::White,  -9));

    // Torso (narrower from the side)
    p.push(limb("torso","torso",-0.05,1.05,W_TORSO_BOT*ts,W_TORSO_TOP*ts,ColorRole::White,0));
    p.push(circle_part("hip_fill",     "torso", 0.0, 0.0, W_TORSO_BOT*ts, ColorRole::White, 1));
    p.push(circle_part("shoulder_fill","torso", 1.0, 0.0, W_TORSO_TOP*ts, ColorRole::White, 2));
    p.push(limb("neck","neck",-0.15,1.15,W_NECK*ts*1.2,W_NECK*ts,ColorRole::White, 3));

    // Head (slightly oval from the side — slightly less wide)
    p.push(ellipse("head","head",0.5,0.0,0.55,0.52,ColorRole::White,5));

    // Front leg
    p.push(limb("ul_front","upper_leg_l", 0.0, 1.0, W_UL*ls, W_LL*ls, ColorRole::White, 10));
    p.push(circle_part_joint("knee_front","upper_leg_l",1.0,0.0,W_LL*ls,ColorRole::White,11));
    p.push(limb("ll_front","lower_leg_l", 0.0, 1.0, W_LL*ls, W_LL*ls*0.88, ColorRole::White, 12));
    p.push(ellipse("foot_front","foot_l", 1.0, 0.0, 1.4, 0.45, ColorRole::White, 13));

    // Front arm
    p.push(limb("ua_front","upper_arm_l",-0.15,1.0,W_UA*ls,W_FA*ls, ColorRole::White, 20));
    p.push(limb("fa_front","forearm_l",   0.0, 1.0,W_FA*ls,W_FA*ls, ColorRole::White, 21));

    let name = if facing_right { "anonymous_right" } else { "anonymous_left" };
    CharacterDef { name: name.to_string(), parts: p }
}

// ─── CASUAL CHARACTER: Front view with clothes and natural skin tones ───────
fn casual_front() -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();

    // ─ BACK side (drawn behind torso) ─────────────────────────────────────
    p.push(limb("ua_r", "upper_arm_r", -0.15, 1.0, W_UA, W_FA, ColorRole::ClothMain, -30));
    p.push(limb("fa_r", "forearm_r", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothMain, -29));
    p.push(circle_part("hand_r", "hand_r", 0.5, 0.0, W_HAND, ColorRole::Skin, -28));

    p.push(limb("ul_r", "upper_leg_r", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, -20));
    p.push(circle_part_joint("knee_r", "upper_leg_r", 1.0, 0.0, W_LL, ColorRole::Skin, -19));
    p.push(limb("ll_r", "lower_leg_r", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::Skin, -18));
    p.push(ellipse("foot_r", "foot_r", 1.0, 0.0, 1.6, 0.50, ColorRole::Shoe, -17));

    // ─ TORSO (center, draws over back limbs) ────────────────────────────
    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT, W_TORSO_TOP, ColorRole::ClothMain, 0));
    p.push(circle_part("hip_fill", "torso", 0.0, 0.0, W_TORSO_BOT, ColorRole::ClothMain, 1));
    p.push(circle_part("shoulder_fill", "torso", 1.0, 0.0, W_TORSO_TOP, ColorRole::ClothMain, 2));

    // ─ NECK & HEAD ────────────────────────────────────────────────────────
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK, W_NECK * 0.85, ColorRole::Skin, 3));
    p.push(ellipse("head", "head", 0.5, 0.0, 0.62, 0.52, ColorRole::Skin, 5));
    // Hair back
    p.push(circle_part("hair_back", "head", 0.5, -0.25, 0.65, ColorRole::Hair, 4));

    // ─ FRONT side (left limbs, drawn over torso) ──────────────────────────
    p.push(limb("ul_l", "upper_leg_l", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, 10));
    p.push(circle_part_joint("knee_l", "upper_leg_l", 1.0, 0.0, W_LL, ColorRole::Skin, 11));
    p.push(limb("ll_l", "lower_leg_l", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::Skin, 12));
    p.push(ellipse("foot_l", "foot_l", 1.0, 0.0, 1.6, 0.50, ColorRole::Shoe, 13));

    p.push(limb("ua_l", "upper_arm_l", -0.15, 1.0, W_UA, W_FA, ColorRole::Skin, 20));
    p.push(limb("fa_l", "forearm_l", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothAccent, 21));
    p.push(circle_part("hand_l", "hand_l", 0.5, 0.0, W_HAND, ColorRole::Skin, 22));

    CharacterDef { name: "casual".to_string(), parts: p }
}

// ─── CASUAL CHARACTER: Back view ─────────────────────────────────────────────
fn casual_back() -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();

    p.push(limb("ua_l", "upper_arm_l", -0.15, 1.0, W_UA, W_FA, ColorRole::ClothMain, -30));
    p.push(limb("fa_l", "forearm_l", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothMain, -29));
    p.push(circle_part("hand_l", "hand_l", 0.5, 0.0, W_HAND, ColorRole::Skin, -28));

    p.push(limb("ul_l", "upper_leg_l", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, -20));
    p.push(circle_part_joint("knee_l", "upper_leg_l", 1.0, 0.0, W_LL, ColorRole::Skin, -19));
    p.push(limb("ll_l", "lower_leg_l", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::Skin, -18));
    p.push(ellipse("foot_l", "foot_l", 1.0, 0.0, 1.6, 0.50, ColorRole::Shoe, -17));

    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT, W_TORSO_TOP, ColorRole::ClothMain, 0));
    p.push(circle_part("hip_fill", "torso", 0.0, 0.0, W_TORSO_BOT, ColorRole::ClothMain, 1));
    p.push(circle_part("shoulder_fill", "torso", 1.0, 0.0, W_TORSO_TOP, ColorRole::ClothMain, 2));
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK, W_NECK * 0.85, ColorRole::Skin, 3));
    p.push(ellipse("head", "head", 0.5, 0.0, 0.62, 0.52, ColorRole::Skin, 5));
    p.push(circle_part("hair_back", "head", 0.5, -0.25, 0.65, ColorRole::Hair, 4));

    p.push(limb("ul_r", "upper_leg_r", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, 10));
    p.push(circle_part_joint("knee_r", "upper_leg_r", 1.0, 0.0, W_LL, ColorRole::Skin, 11));
    p.push(limb("ll_r", "lower_leg_r", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::Skin, 12));
    p.push(ellipse("foot_r", "foot_r", 1.0, 0.0, 1.6, 0.50, ColorRole::Shoe, 13));

    p.push(limb("ua_r", "upper_arm_r", -0.15, 1.0, W_UA, W_FA, ColorRole::Skin, 20));
    p.push(limb("fa_r", "forearm_r", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothAccent, 21));
    p.push(circle_part("hand_r", "hand_r", 0.5, 0.0, W_HAND, ColorRole::Skin, 22));

    CharacterDef { name: "casual_back".to_string(), parts: p }
}

// ─── CASUAL CHARACTER: Side view ──────────────────────────────────────────────
fn casual_side(facing_right: bool) -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();
    let ts = 0.50_f32;
    let ls = 0.85_f32;

    p.push(limb("ul_back", "upper_leg_r", 0.0, 1.0, W_UL * ls, W_LL * ls, ColorRole::ClothDark, -20));
    p.push(circle_part_joint("knee_back", "upper_leg_r", 1.0, 0.0, W_LL * ls, ColorRole::Skin, -19));
    p.push(limb("ll_back", "lower_leg_r", 0.0, 1.0, W_LL * ls, W_LL * ls * 0.88, ColorRole::Skin, -18));
    p.push(ellipse("foot_back", "foot_r", 1.0, 0.0, 1.4, 0.45, ColorRole::Shoe, -17));

    p.push(limb("ua_back", "upper_arm_r", -0.15, 1.0, W_UA * ls, W_FA * ls, ColorRole::Skin, -10));
    p.push(limb("fa_back", "forearm_r", 0.0, 1.0, W_FA * ls, W_FA * ls, ColorRole::ClothAccent, -9));

    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT * ts, W_TORSO_TOP * ts, ColorRole::ClothMain, 0));
    p.push(circle_part("hip_fill", "torso", 0.0, 0.0, W_TORSO_BOT * ts, ColorRole::ClothMain, 1));
    p.push(circle_part("shoulder_fill", "torso", 1.0, 0.0, W_TORSO_TOP * ts, ColorRole::ClothMain, 2));
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK * ts * 1.2, W_NECK * ts, ColorRole::Skin, 3));
    p.push(ellipse("head", "head", 0.5, 0.0, 0.55, 0.52, ColorRole::Skin, 5));
    p.push(circle_part("hair_back", "head", 0.5, -0.25, 0.60, ColorRole::Hair, 4));

    p.push(limb("ul_front", "upper_leg_l", 0.0, 1.0, W_UL * ls, W_LL * ls, ColorRole::ClothDark, 10));
    p.push(circle_part_joint("knee_front", "upper_leg_l", 1.0, 0.0, W_LL * ls, ColorRole::Skin, 11));
    p.push(limb("ll_front", "lower_leg_l", 0.0, 1.0, W_LL * ls, W_LL * ls * 0.88, ColorRole::Skin, 12));
    p.push(ellipse("foot_front", "foot_l", 1.0, 0.0, 1.4, 0.45, ColorRole::Shoe, 13));

    p.push(limb("ua_front", "upper_arm_l", -0.15, 1.0, W_UA * ls, W_FA * ls, ColorRole::Skin, 20));
    p.push(limb("fa_front", "forearm_l", 0.0, 1.0, W_FA * ls, W_FA * ls, ColorRole::ClothAccent, 21));

    let name = if facing_right { "casual_right" } else { "casual_left" };
    CharacterDef { name: name.to_string(), parts: p }
}

// ─── ROBOT CHARACTER: Front view ──────────────────────────────────────────────
fn robot_front() -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();

    // ─ BACK side ───────────────────────────────────────────────────────────
    p.push(limb("ua_r", "upper_arm_r", -0.15, 1.0, W_UA, W_FA, ColorRole::ClothDark, -30));
    p.push(limb("fa_r", "forearm_r", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothDark, -29));
    p.push(circle_part("hand_r", "hand_r", 0.5, 0.0, W_HAND, ColorRole::ClothMain, -28));

    p.push(limb("ul_r", "upper_leg_r", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, -20));
    p.push(circle_part_joint("knee_r", "upper_leg_r", 1.0, 0.0, W_LL, ColorRole::ClothAccent, -19));
    p.push(limb("ll_r", "lower_leg_r", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::ClothDark, -18));
    p.push(ellipse("foot_r", "foot_r", 1.0, 0.0, 1.6, 0.50, ColorRole::ClothDark, -17));

    // ─ TORSO ────────────────────────────────────────────────────────────────
    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT, W_TORSO_TOP, ColorRole::ClothMain, 0));
    p.push(circle_part("hip_fill", "torso", 0.0, 0.0, W_TORSO_BOT, ColorRole::ClothMain, 1));
    p.push(circle_part("shoulder_fill", "torso", 1.0, 0.0, W_TORSO_TOP, ColorRole::ClothMain, 2));

    // ─ NECK & HEAD ────────────────────────────────────────────────────────
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK, W_NECK * 0.85, ColorRole::ClothMain, 3));
    p.push(ellipse("head", "head", 0.5, 0.0, 0.62, 0.52, ColorRole::ClothMain, 5));
    // Head visor
    p.push(circle_part("visor", "head", 0.0, -0.08, 0.35, ColorRole::ClothAccent, 6));

    // ─ FRONT side ──────────────────────────────────────────────────────────
    p.push(limb("ul_l", "upper_leg_l", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, 10));
    p.push(circle_part_joint("knee_l", "upper_leg_l", 1.0, 0.0, W_LL, ColorRole::ClothAccent, 11));
    p.push(limb("ll_l", "lower_leg_l", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::ClothDark, 12));
    p.push(ellipse("foot_l", "foot_l", 1.0, 0.0, 1.6, 0.50, ColorRole::ClothDark, 13));

    p.push(limb("ua_l", "upper_arm_l", -0.15, 1.0, W_UA, W_FA, ColorRole::ClothMain, 20));
    p.push(limb("fa_l", "forearm_l", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothMain, 21));
    p.push(circle_part("hand_l", "hand_l", 0.5, 0.0, W_HAND, ColorRole::ClothDark, 22));

    CharacterDef { name: "robot".to_string(), parts: p }
}

// ─── ROBOT CHARACTER: Back view ───────────────────────────────────────────────
fn robot_back() -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();

    p.push(limb("ua_l", "upper_arm_l", -0.15, 1.0, W_UA, W_FA, ColorRole::ClothDark, -30));
    p.push(limb("fa_l", "forearm_l", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothDark, -29));
    p.push(circle_part("hand_l", "hand_l", 0.5, 0.0, W_HAND, ColorRole::ClothMain, -28));

    p.push(limb("ul_l", "upper_leg_l", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, -20));
    p.push(circle_part_joint("knee_l", "upper_leg_l", 1.0, 0.0, W_LL, ColorRole::ClothAccent, -19));
    p.push(limb("ll_l", "lower_leg_l", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::ClothDark, -18));
    p.push(ellipse("foot_l", "foot_l", 1.0, 0.0, 1.6, 0.50, ColorRole::ClothDark, -17));

    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT, W_TORSO_TOP, ColorRole::ClothMain, 0));
    p.push(circle_part("hip_fill", "torso", 0.0, 0.0, W_TORSO_BOT, ColorRole::ClothMain, 1));
    p.push(circle_part("shoulder_fill", "torso", 1.0, 0.0, W_TORSO_TOP, ColorRole::ClothMain, 2));
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK, W_NECK * 0.85, ColorRole::ClothMain, 3));
    p.push(ellipse("head", "head", 0.5, 0.0, 0.62, 0.52, ColorRole::ClothMain, 5));
    p.push(circle_part("visor", "head", 0.0, -0.08, 0.35, ColorRole::ClothAccent, 6));

    p.push(limb("ul_r", "upper_leg_r", 0.0, 1.0, W_UL, W_LL, ColorRole::ClothDark, 10));
    p.push(circle_part_joint("knee_r", "upper_leg_r", 1.0, 0.0, W_LL, ColorRole::ClothAccent, 11));
    p.push(limb("ll_r", "lower_leg_r", 0.0, 1.0, W_LL, W_LL * 0.88, ColorRole::ClothDark, 12));
    p.push(ellipse("foot_r", "foot_r", 1.0, 0.0, 1.6, 0.50, ColorRole::ClothDark, 13));

    p.push(limb("ua_r", "upper_arm_r", -0.15, 1.0, W_UA, W_FA, ColorRole::ClothMain, 20));
    p.push(limb("fa_r", "forearm_r", 0.0, 1.0, W_FA, W_FA, ColorRole::ClothMain, 21));
    p.push(circle_part("hand_r", "hand_r", 0.5, 0.0, W_HAND, ColorRole::ClothDark, 22));

    CharacterDef { name: "robot_back".to_string(), parts: p }
}

// ─── ROBOT CHARACTER: Side view ───────────────────────────────────────────────
fn robot_side(facing_right: bool) -> CharacterDef {
    let mut p: Vec<PartShape> = Vec::new();
    let ts = 0.50_f32;
    let ls = 0.85_f32;

    p.push(limb("ul_back", "upper_leg_r", 0.0, 1.0, W_UL * ls, W_LL * ls, ColorRole::ClothDark, -20));
    p.push(circle_part_joint("knee_back", "upper_leg_r", 1.0, 0.0, W_LL * ls, ColorRole::ClothAccent, -19));
    p.push(limb("ll_back", "lower_leg_r", 0.0, 1.0, W_LL * ls, W_LL * ls * 0.88, ColorRole::ClothDark, -18));
    p.push(ellipse("foot_back", "foot_r", 1.0, 0.0, 1.4, 0.45, ColorRole::ClothDark, -17));

    p.push(limb("ua_back", "upper_arm_r", -0.15, 1.0, W_UA * ls, W_FA * ls, ColorRole::ClothMain, -10));
    p.push(limb("fa_back", "forearm_r", 0.0, 1.0, W_FA * ls, W_FA * ls, ColorRole::ClothMain, -9));

    p.push(limb("torso", "torso", -0.05, 1.05, W_TORSO_BOT * ts, W_TORSO_TOP * ts, ColorRole::ClothMain, 0));
    p.push(circle_part("hip_fill", "torso", 0.0, 0.0, W_TORSO_BOT * ts, ColorRole::ClothMain, 1));
    p.push(circle_part("shoulder_fill", "torso", 1.0, 0.0, W_TORSO_TOP * ts, ColorRole::ClothMain, 2));
    p.push(limb("neck", "neck", -0.15, 1.15, W_NECK * ts * 1.2, W_NECK * ts, ColorRole::ClothMain, 3));
    p.push(ellipse("head", "head", 0.5, 0.0, 0.55, 0.52, ColorRole::ClothMain, 5));
    p.push(circle_part("visor", "head", 0.0, -0.08, 0.30, ColorRole::ClothAccent, 6));

    p.push(limb("ul_front", "upper_leg_l", 0.0, 1.0, W_UL * ls, W_LL * ls, ColorRole::ClothDark, 10));
    p.push(circle_part_joint("knee_front", "upper_leg_l", 1.0, 0.0, W_LL * ls, ColorRole::ClothAccent, 11));
    p.push(limb("ll_front", "lower_leg_l", 0.0, 1.0, W_LL * ls, W_LL * ls * 0.88, ColorRole::ClothDark, 12));
    p.push(ellipse("foot_front", "foot_l", 1.0, 0.0, 1.4, 0.45, ColorRole::ClothDark, 13));

    p.push(limb("ua_front", "upper_arm_l", -0.15, 1.0, W_UA * ls, W_FA * ls, ColorRole::ClothMain, 20));
    p.push(limb("fa_front", "forearm_l", 0.0, 1.0, W_FA * ls, W_FA * ls, ColorRole::ClothMain, 21));

    let name = if facing_right { "robot_right" } else { "robot_left" };
    CharacterDef { name: name.to_string(), parts: p }
}

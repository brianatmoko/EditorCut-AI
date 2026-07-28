use crate::pose::StickmanPose;

pub struct LineSegment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

pub struct BoneTransform {
    pub label: &'static str,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub angle: f64,
    pub length: f64,
    pub width: f64,
}

pub struct StickmanRenderData {
    pub segments: Vec<LineSegment>,
    pub bones: Vec<BoneTransform>,
    pub head_cx: f64,
    pub head_cy: f64,
    pub head_r: f64,
    pub facing_left: bool,
    pub eyebrow: f64,
    pub mouth: f64,
    pub eye_blink: f64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub action_name: String,
    pub scene_theme: String,
    pub character_type: String,
    pub is_side_view: bool,
    /// 0 = front, PI = back, +PI/2 = right side, -PI/2 = left side
    pub facing_angle: f64,
}

// ─── Skeleton proportions (world units) ────────────────────────────────────
// Total height from ground (y=0) to top of head = ~1.0
//   Legs:  upper_leg=0.22 + lower_leg=0.20 + foot=0.08 = 0.50
//   Torso: 0.26
//   Neck:  0.04
//   Head:  0.20 (diameter, radius=0.10)
//   Total ≈ 1.00

const HEAD_R:     f64 = 0.100;
const NECK:       f64 = 0.040;
const TORSO:      f64 = 0.260;
const SHOULDER_W: f64 = 0.120;  // half-width of shoulders from spine
const HIP_W:      f64 = 0.080;  // half-width of hips from spine
const UPPER_ARM:  f64 = 0.150;
const FOREARM:    f64 = 0.130;
const HAND:       f64 = 0.060;
const UPPER_LEG:  f64 = 0.220;
const LOWER_LEG:  f64 = 0.200;
const FOOT:       f64 = 0.090;

fn seg(x1: f64, y1: f64, x2: f64, y2: f64) -> LineSegment {
    LineSegment { x1, y1, x2, y2 }
}

pub fn stickman_to_segments(
    pose: &StickmanPose,
    scene_theme: &str,
    character_type: &str,
    action_name: &str,
) -> StickmanRenderData {
    let body_tilt = pose.body_tilt;
    let body_y    = pose.body_y;
    let pos_x     = pose.pos_x;

    // Hips center (world root, at ground level y=0)
    let hips_cx = pos_x;
    let hips_cy = 0.0 + body_y;

    // ── Spine / Torso ─────────────────────────────────────────────────────
    // Angle +PI/2 is straight UP in Y-up world space.
    let spine_angle  = std::f64::consts::FRAC_PI_2 + body_tilt + pose.spine_lower;
    let torso_top_x  = hips_cx  + TORSO * spine_angle.cos();
    let torso_top_y  = hips_cy  + TORSO * spine_angle.sin();
    let torso_seg    = seg(hips_cx, hips_cy, torso_top_x, torso_top_y);

    // ── Neck ──────────────────────────────────────────────────────────────
    let neck_angle  = spine_angle + pose.neck;
    let neck_top_x  = torso_top_x + NECK * neck_angle.cos();
    let neck_top_y  = torso_top_y + NECK * neck_angle.sin();
    let neck_seg    = seg(torso_top_x, torso_top_y, neck_top_x, neck_top_y);

    // ── Head ──────────────────────────────────────────────────────────────
    // Head bone: starts at neck_top, extends one diameter UP along head_angle
    let head_angle  = neck_angle + pose.head_turn;
    let head_top_x  = neck_top_x + (2.0 * HEAD_R) * head_angle.cos();
    let head_top_y  = neck_top_y + (2.0 * HEAD_R) * head_angle.sin();
    let head_cx     = neck_top_x + HEAD_R * head_angle.cos();
    let head_cy     = neck_top_y + HEAD_R * head_angle.sin();

    // ── Clavicles (Shoulder positions) ────────────────────────────────────
    // Perpendicular to spine:
    //   spine points UP (+PI/2), so shoulders are at +PI/2 ± PI/2
    //   shoulder_l is at spine_angle - PI/2  (front view: left = negative X)
    //   shoulder_r is at spine_angle + PI/2  (front view: right = positive X)
    let perp_l = spine_angle - std::f64::consts::FRAC_PI_2;
    let perp_r = spine_angle + std::f64::consts::FRAC_PI_2;

    let shoulder_l_x = torso_top_x + SHOULDER_W * perp_l.cos();
    let shoulder_l_y = torso_top_y + SHOULDER_W * perp_l.sin();

    let shoulder_r_x = torso_top_x + SHOULDER_W * perp_r.cos();
    let shoulder_r_y = torso_top_y + SHOULDER_W * perp_r.sin();

    let clavicle_l_seg = seg(torso_top_x, torso_top_y, shoulder_l_x, shoulder_l_y);
    let clavicle_r_seg = seg(torso_top_x, torso_top_y, shoulder_r_x, shoulder_r_y);

    // ── Arms ──────────────────────────────────────────────────────────────
    // Arms hang DOWN from shoulder: base angle = -PI/2 (downward)
    let ua_l_angle  = -std::f64::consts::FRAC_PI_2 + body_tilt + pose.shoulder_l;
    let elbow_l_x   = shoulder_l_x + UPPER_ARM * ua_l_angle.cos();
    let elbow_l_y   = shoulder_l_y + UPPER_ARM * ua_l_angle.sin();
    let upper_arm_l_seg = seg(shoulder_l_x, shoulder_l_y, elbow_l_x, elbow_l_y);

    let fa_l_angle  = ua_l_angle + pose.elbow_l;
    let wrist_l_x   = elbow_l_x + FOREARM * fa_l_angle.cos();
    let wrist_l_y   = elbow_l_y + FOREARM * fa_l_angle.sin();
    let forearm_l_seg = seg(elbow_l_x, elbow_l_y, wrist_l_x, wrist_l_y);

    let hand_l_angle = fa_l_angle + pose.wrist_l;
    let hand_l_x    = wrist_l_x + HAND * hand_l_angle.cos();
    let hand_l_y    = wrist_l_y + HAND * hand_l_angle.sin();
    let hand_l_seg  = seg(wrist_l_x, wrist_l_y, hand_l_x, hand_l_y);

    let ua_r_angle  = -std::f64::consts::FRAC_PI_2 + body_tilt + pose.shoulder_r;
    let elbow_r_x   = shoulder_r_x + UPPER_ARM * ua_r_angle.cos();
    let elbow_r_y   = shoulder_r_y + UPPER_ARM * ua_r_angle.sin();
    let upper_arm_r_seg = seg(shoulder_r_x, shoulder_r_y, elbow_r_x, elbow_r_y);

    let fa_r_angle  = ua_r_angle + pose.elbow_r;
    let wrist_r_x   = elbow_r_x + FOREARM * fa_r_angle.cos();
    let wrist_r_y   = elbow_r_y + FOREARM * fa_r_angle.sin();
    let forearm_r_seg = seg(elbow_r_x, elbow_r_y, wrist_r_x, wrist_r_y);

    let hand_r_angle = fa_r_angle + pose.wrist_r;
    let hand_r_x    = wrist_r_x + HAND * hand_r_angle.cos();
    let hand_r_y    = wrist_r_y + HAND * hand_r_angle.sin();
    let hand_r_seg  = seg(wrist_r_x, wrist_r_y, hand_r_x, hand_r_y);

    // ── Hips ──────────────────────────────────────────────────────────────
    let hip_l_x = hips_cx + HIP_W * perp_l.cos();
    let hip_l_y = hips_cy + HIP_W * perp_l.sin();
    let hip_r_x = hips_cx + HIP_W * perp_r.cos();
    let hip_r_y = hips_cy + HIP_W * perp_r.sin();

    let hip_l_seg = seg(hips_cx, hips_cy, hip_l_x, hip_l_y);
    let hip_r_seg = seg(hips_cx, hips_cy, hip_r_x, hip_r_y);

    // ── Legs ──────────────────────────────────────────────────────────────
    // Legs hang DOWN from hip: base angle = -PI/2
    let ul_l_angle  = -std::f64::consts::FRAC_PI_2 + body_tilt + pose.hip_l;
    let knee_l_x    = hip_l_x + UPPER_LEG * ul_l_angle.cos();
    let knee_l_y    = hip_l_y + UPPER_LEG * ul_l_angle.sin();
    let upper_leg_l_seg = seg(hip_l_x, hip_l_y, knee_l_x, knee_l_y);

    let ll_l_angle  = ul_l_angle + pose.knee_l;
    let ankle_l_x   = knee_l_x + LOWER_LEG * ll_l_angle.cos();
    let ankle_l_y   = knee_l_y + LOWER_LEG * ll_l_angle.sin();
    let lower_leg_l_seg = seg(knee_l_x, knee_l_y, ankle_l_x, ankle_l_y);

    // Foot: rotates 90° from lower leg direction (points left for left foot, right for right foot)
    let foot_l_angle = ll_l_angle - std::f64::consts::FRAC_PI_2 + pose.ankle_l;
    let foot_l_x     = ankle_l_x + FOOT * foot_l_angle.cos();
    let foot_l_y     = ankle_l_y + FOOT * foot_l_angle.sin();
    let foot_l_seg   = seg(ankle_l_x, ankle_l_y, foot_l_x, foot_l_y);

    let ul_r_angle  = -std::f64::consts::FRAC_PI_2 + body_tilt + pose.hip_r;
    let knee_r_x    = hip_r_x + UPPER_LEG * ul_r_angle.cos();
    let knee_r_y    = hip_r_y + UPPER_LEG * ul_r_angle.sin();
    let upper_leg_r_seg = seg(hip_r_x, hip_r_y, knee_r_x, knee_r_y);

    let ll_r_angle  = ul_r_angle + pose.knee_r;
    let ankle_r_x   = knee_r_x + LOWER_LEG * ll_r_angle.cos();
    let ankle_r_y   = knee_r_y + LOWER_LEG * ll_r_angle.sin();
    let lower_leg_r_seg = seg(knee_r_x, knee_r_y, ankle_r_x, ankle_r_y);

    let foot_r_angle = ll_r_angle + std::f64::consts::FRAC_PI_2 + pose.ankle_r;
    let foot_r_x     = ankle_r_x + FOOT * foot_r_angle.cos();
    let foot_r_y     = ankle_r_y + FOOT * foot_r_angle.sin();
    let foot_r_seg   = seg(ankle_r_x, ankle_r_y, foot_r_x, foot_r_y);

    // ── Segment & Bone list ───────────────────────────────────────────────
    let s: Vec<LineSegment> = vec![
        torso_seg,
        neck_seg,
        clavicle_l_seg, clavicle_r_seg,
        upper_arm_l_seg, forearm_l_seg, hand_l_seg,
        upper_arm_r_seg, forearm_r_seg, hand_r_seg,
        hip_l_seg, hip_r_seg,
        upper_leg_l_seg, lower_leg_l_seg, foot_l_seg,
        upper_leg_r_seg, lower_leg_r_seg, foot_r_seg,
    ];

    let bone_labels: &[&'static str] = &[
        "torso", "neck",
        "clavicle_l", "clavicle_r",
        "upper_arm_l", "forearm_l", "hand_l",
        "upper_arm_r", "forearm_r", "hand_r",
        "hip_l", "hip_r",
        "upper_leg_l", "lower_leg_l", "foot_l",
        "upper_leg_r", "lower_leg_r", "foot_r",
    ];

    // Anatomical widths (half-width of each bone, in world units)
    let bone_widths: &[f64] = &[
        0.120, 0.030,          // torso, neck
        0.025, 0.025,          // clavicle_l, clavicle_r (thin collarbone)
        0.040, 0.035, 0.030,   // upper_arm_l, forearm_l, hand_l
        0.040, 0.035, 0.030,   // upper_arm_r, forearm_r, hand_r
        0.025, 0.025,          // hip_l, hip_r (connector, thin)
        0.055, 0.045, 0.035,   // upper_leg_l, lower_leg_l, foot_l
        0.055, 0.045, 0.035,   // upper_leg_r, lower_leg_r, foot_r
    ];

    let mut bones: Vec<BoneTransform> = Vec::with_capacity(s.len() + 1);

    // Head bone: x1=neck_top (base), extends up along head_angle by 2*HEAD_R
    bones.push(BoneTransform {
        label:  "head",
        x1:     neck_top_x,
        y1:     neck_top_y,
        x2:     head_top_x,
        y2:     head_top_y,
        angle:  head_angle,
        length: 2.0 * HEAD_R,
        width:  HEAD_R,
    });

    for (i, seg) in s.iter().enumerate() {
        if i >= bone_labels.len() { break; }
        let dx     = seg.x2 - seg.x1;
        let dy     = seg.y2 - seg.y1;
        let angle  = dy.atan2(dx);
        let length = (dx * dx + dy * dy).sqrt().max(1e-6);
        bones.push(BoneTransform {
            label:  bone_labels[i],
            x1:     seg.x1,
            y1:     seg.y1,
            x2:     seg.x2,
            y2:     seg.y2,
            angle,
            length,
            width: bone_widths[i],
        });
    }

    StickmanRenderData {
        segments:       s,
        bones,
        head_cx,
        head_cy,
        head_r:         HEAD_R,
        facing_left:    pose.facing_left,
        eyebrow:        pose.eyebrow,
        mouth:          pose.mouth,
        eye_blink:      pose.eye_blink,
        pos_x:          pose.pos_x,
        pos_y:          pose.pos_y + pose.body_y,
        action_name:    action_name.to_string(),
        scene_theme:    scene_theme.to_string(),
        character_type: character_type.to_string(),
        is_side_view:   false,
        facing_angle:   0.0,
    }
}

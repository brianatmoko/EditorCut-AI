//! Inverse Kinematics (IK) Solver 2D untuk Skeletal Rig
//! Digunakan untuk Foot Grounding, Hand Reaching, dan Gerakan Merangkak Organik.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance(&self, other: &Vec2) -> f64 {
        Vec2::new(self.x - other.x, self.y - other.y).length()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoBoneIKResult {
    pub angle_1: f64, // Angle sendi pangkal (misal: Shoulder atau Hip) dalam radian
    pub angle_2: f64, // Angle sendi tengah (misal: Elbow atau Knee) dalam radian
    pub reachable: bool,
}

/// Dynamic 2-Bone Analytical IK Solver
/// `root`: Koordinat pangkal sendi (misal: Hips atau Shoulder)
/// `target`: Koordinat tujuan yang ingin dicapai oleh end-effector (misal: Ankle atau Wrist)
/// `l1`: Panjang tulang pertama (misal: Upper Leg / Upper Arm)
/// `l2`: Panjang tulang kedua (misal: Lower Leg / Forearm)
/// `bend_positive`: Arah tekukan sendi (true = knee forward/elbow down, false = sebaliknya)
pub fn solve_2bone_ik(
    root: Vec2,
    target: Vec2,
    l1: f64,
    l2: f64,
    bend_positive: bool,
) -> TwoBoneIKResult {
    let dx = target.x - root.x;
    let dy = target.y - root.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist <= 1e-6 {
        return TwoBoneIKResult {
            angle_1: 0.0,
            angle_2: 0.0,
            reachable: false,
        };
    }

    // Clamp distance agar tidak melebihi jangkauan maksimal tulang (l1 + l2)
    let max_dist = (l1 + l2) * 0.9999;
    let min_dist = (l1 - l2).abs() * 1.0001;
    let clamped_dist = dist.clamp(min_dist, max_dist);
    let reachable = dist <= (l1 + l2);

    // Law of Cosines
    // cos(angle_2) = (dist^2 - l1^2 - l2^2) / (2 * l1 * l2)
    let cos_angle_2 = (clamped_dist * clamped_dist - l1 * l1 - l2 * l2) / (2.0 * l1 * l2);
    let cos_angle_2 = cos_angle_2.clamp(-1.0, 1.0);

    // Sendi tengah (Elbow / Knee)
    let angle_2_magnitude = cos_angle_2.acos();
    let angle_2 = if bend_positive {
        angle_2_magnitude
    } else {
        -angle_2_magnitude
    };

    // Angle ke target
    let alpha = dy.atan2(dx);

    // Angle offset internal tulang 1
    // cos(beta) = (l1^2 + dist^2 - l2^2) / (2 * l1 * dist)
    let cos_beta = (l1 * l1 + clamped_dist * clamped_dist - l2 * l2) / (2.0 * l1 * clamped_dist);
    let cos_beta = cos_beta.clamp(-1.0, 1.0);
    let beta = cos_beta.acos();

    let angle_1 = if bend_positive {
        alpha - beta
    } else {
        alpha + beta
    };

    TwoBoneIKResult {
        angle_1,
        angle_2,
        reachable,
    }
}

/// Menghitung penyesuaian posisi kaki agar menapak di tanah (Y = ground_y)
pub fn solve_foot_grounding(
    hip_pos: Vec2,
    ground_y: f64,
    upper_leg_len: f64,
    lower_leg_len: f64,
    facing_right: bool,
) -> (f64, f64) {
    let target_foot = Vec2::new(hip_pos.x, ground_y);
    let ik = solve_2bone_ik(
        hip_pos,
        target_foot,
        upper_leg_len,
        lower_leg_len,
        facing_right,
    );
    (ik.angle_1, ik.angle_2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2bone_ik_reachability() {
        let root = Vec2::new(0.0, 0.0);
        let target = Vec2::new(0.3, 0.0);
        let result = solve_2bone_ik(root, target, 0.2, 0.2, true);
        assert!(result.reachable);
        assert!(result.angle_2 > 0.0);
    }

    #[test]
    fn test_2bone_ik_overreach() {
        let root = Vec2::new(0.0, 0.0);
        let target = Vec2::new(1.0, 0.0); // Terlalu jauh untuk l1=0.2, l2=0.2
        let result = solve_2bone_ik(root, target, 0.2, 0.2, true);
        assert!(!result.reachable);
    }
}

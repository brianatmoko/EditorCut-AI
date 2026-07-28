use crate::pose::StickmanPose;

const DEG: f64 = std::f64::consts::PI / 180.0;

fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

fn clamp01(x: f64) -> f64 { x.clamp(0.0, 1.0) }

// ═══════════════════════════════════════════════════════════════
// 1. LOCOMOTION — Pergerakan Dasar
// ═══════════════════════════════════════════════════════════════

pub fn pose_idle(t: f64) -> StickmanPose {
    let breath = (t * 1.5).sin() * 0.015;
    StickmanPose {
        body_y: breath,
        mouth: (t * 0.8).sin().abs() * 0.3,
        eye_blink: (0.0f64).max((t * 3.0 - 2.0).sin()) * 0.15,
        ..StickmanPose::neutral()
    }
}

pub fn pose_idle_shift(t: f64) -> StickmanPose {
    let shift = (t * 0.5).sin() * 0.02;
    StickmanPose {
        body_y: (t * 1.5).sin() * 0.015,
        pos_x: shift,
        pelvis: (t * 0.5).sin() * 2.0 * DEG,
        hip_l: shift * 30.0 * DEG,
        hip_r: -shift * 30.0 * DEG,
        mouth: (t * 0.8).sin().abs() * 0.3,
        eye_blink: (0.0f64).max((t * 3.0 - 2.0).sin()) * 0.15,
        ..StickmanPose::neutral()
    }
}

pub fn pose_walk(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.0 * std::f64::consts::PI;
    let leg_swing = cycle.sin() * 40.0 * DEG;
    let arm_swing = (cycle + std::f64::consts::PI).sin() * 35.0 * DEG;
    let bounce = (cycle * 2.0).sin().abs() * 0.04;
    let spine_twist = cycle.sin() * 3.0 * DEG;
    StickmanPose {
        body_y: bounce, shoulder_l: arm_swing, shoulder_r: -arm_swing,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG,
        hip_l: leg_swing, hip_r: -leg_swing,
        knee_l: (0.0f64).max(-cycle.sin()) * 35.0 * DEG,
        knee_r: (0.0f64).max(cycle.sin()) * 35.0 * DEG,
        body_tilt: -3.0 * DEG, pos_x: t * 0.8, neck: -2.0 * DEG,
        spine_upper: spine_twist, spine_lower: spine_twist * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_walk_backward(t: f64, speed: f64) -> StickmanPose {
    let mut p = pose_walk(t, speed);
    p.body_tilt = 5.0 * DEG;
    p.neck = 5.0 * DEG;
    p.pos_x = -t * 0.6;
    p.head_turn = 20.0 * DEG;
    p.eyebrow = 0.3;
    p
}

pub fn pose_walk_side(t: f64, speed: f64) -> StickmanPose {
    // Sideways walk: one shoulder leads
    let cycle = t * speed * 1.8 * std::f64::consts::PI;
    let step = cycle.sin() * 25.0 * DEG;
    StickmanPose {
        body_y: (cycle * 2.0).sin().abs() * 0.03,
        shoulder_l: 20.0 * DEG, shoulder_r: -40.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 15.0 * DEG,
        hip_l: step, hip_r: -step,
        knee_l: (0.0f64).max(-cycle.sin()) * 25.0 * DEG,
        knee_r: (0.0f64).max(cycle.sin()) * 25.0 * DEG,
        body_tilt: -10.0 * DEG, pos_x: t * 0.5,
        head_turn: -30.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_run(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.5 * std::f64::consts::PI;
    let leg_swing = cycle.sin() * 60.0 * DEG;
    let arm_swing = (cycle + std::f64::consts::PI).sin() * 55.0 * DEG;
    let bounce = (cycle * 2.0).sin().abs() * 0.07;
    let tilt = -8.0 * DEG;
    StickmanPose {
        body_y: bounce, body_tilt: tilt,
        shoulder_l: arm_swing, shoulder_r: -arm_swing,
        elbow_l: 40.0 * DEG, elbow_r: 40.0 * DEG,
        hip_l: leg_swing, hip_r: -leg_swing,
        knee_l: (0.0f64).max(-cycle.sin()) * 65.0 * DEG,
        knee_r: (0.0f64).max(cycle.sin()) * 65.0 * DEG,
        pos_x: t * 1.6, neck: tilt * 0.5,
        spine_upper: tilt * 0.6, spine_lower: tilt * 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_sprint(t: f64, speed: f64) -> StickmanPose {
    let mut p = pose_run(t, speed * 1.5);
    p.body_tilt = -15.0 * DEG;
    p.pos_x = t * 2.5;
    p.neck = -10.0 * DEG;
    p.elbow_l = 60.0 * DEG;
    p.elbow_r = 60.0 * DEG;
    p.knee_l = p.knee_l * 1.3;
    p.knee_r = p.knee_r * 1.3;
    p.eyebrow = 0.5;
    p.jaw_open = 0.4;
    p
}

pub fn pose_jog(t: f64, speed: f64) -> StickmanPose {
    let mut p = pose_run(t, speed * 0.7);
    p.body_tilt = -4.0 * DEG;
    p.pos_x = t * 1.0;
    p.elbow_l = 25.0 * DEG;
    p.elbow_r = 25.0 * DEG;
    p.mouth = 0.3;
    p
}

pub fn pose_slow_walk(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.2 * std::f64::consts::PI;
    let leg_swing = cycle.sin() * 22.0 * DEG;
    let arm_swing = (cycle + std::f64::consts::PI).sin() * 18.0 * DEG;
    StickmanPose {
        body_y: (cycle * 2.0).sin().abs() * 0.02,
        shoulder_l: arm_swing, shoulder_r: -arm_swing,
        elbow_l: 10.0 * DEG, elbow_r: 10.0 * DEG,
        hip_l: leg_swing, hip_r: -leg_swing,
        knee_l: (0.0f64).max(-cycle.sin()) * 20.0 * DEG,
        knee_r: (0.0f64).max(cycle.sin()) * 20.0 * DEG,
        body_tilt: -1.5 * DEG, pos_x: t * 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_crawl(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.0 * std::f64::consts::PI;
    let arm_swing = cycle.sin() * 45.0 * DEG;
    let leg_swing = (cycle + std::f64::consts::PI).sin() * 50.0 * DEG;
    StickmanPose {
        body_y: -0.18, body_tilt: 75.0 * DEG, spine_upper: -15.0 * DEG,
        neck: -55.0 * DEG, shoulder_l: arm_swing - 20.0 * DEG,
        shoulder_r: -arm_swing - 20.0 * DEG,
        elbow_l: 85.0 * DEG, elbow_r: 85.0 * DEG,
        hip_l: leg_swing, hip_r: -leg_swing,
        knee_l: 75.0 * DEG, knee_r: 75.0 * DEG,
        pos_x: t * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_crawl_high(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.8 * std::f64::consts::PI;
    let arm = cycle.sin() * 35.0 * DEG;
    let leg = (cycle + std::f64::consts::PI).sin() * 40.0 * DEG;
    StickmanPose {
        body_y: -0.10, body_tilt: 45.0 * DEG,
        neck: -35.0 * DEG,
        shoulder_l: arm - 10.0 * DEG, shoulder_r: -arm - 10.0 * DEG,
        elbow_l: 60.0 * DEG, elbow_r: 60.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: 50.0 * DEG, knee_r: 50.0 * DEG,
        pos_x: t * 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_panic_run(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 3.5 * std::f64::consts::PI;
    let leg_swing = cycle.sin() * 70.0 * DEG;
    let arm_flail = (cycle * 2.0).sin() * 65.0 * DEG;
    let bounce = (cycle * 2.0).sin().abs() * 0.09;
    StickmanPose {
        body_y: bounce, body_tilt: -12.0 * DEG,
        shoulder_l: arm_flail - 40.0 * DEG, shoulder_r: -arm_flail - 40.0 * DEG,
        elbow_l: 90.0 * DEG, elbow_r: 90.0 * DEG,
        hip_l: leg_swing, hip_r: -leg_swing,
        knee_l: (0.0f64).max(-cycle.sin()) * 80.0 * DEG,
        knee_r: (0.0f64).max(cycle.sin()) * 80.0 * DEG,
        head_turn: (t * 20.0).sin() * 10.0 * DEG,
        pos_x: t * 2.2, eyebrow: 0.9, eye_squint: -0.4, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_stealth_walk(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.5 * std::f64::consts::PI;
    let leg_step = cycle.sin() * 30.0 * DEG;
    StickmanPose {
        body_y: -0.06, body_tilt: 20.0 * DEG,
        shoulder_l: 30.0 * DEG, shoulder_r: 30.0 * DEG,
        elbow_l: 80.0 * DEG, elbow_r: 80.0 * DEG,
        hip_l: leg_step, hip_r: -leg_step,
        knee_l: 40.0 * DEG, knee_r: 40.0 * DEG,
        neck: -10.0 * DEG, pos_x: t * 0.35,
        eyebrow: 0.5, eye_squint: 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_sad_walk(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.0 * std::f64::consts::PI;
    let leg_swing = cycle.sin() * 18.0 * DEG;
    StickmanPose {
        body_y: -0.03, body_tilt: 10.0 * DEG,
        spine_upper: 15.0 * DEG, neck: 25.0 * DEG,
        shoulder_l: -15.0 * DEG, shoulder_r: -15.0 * DEG,
        elbow_l: 10.0 * DEG, elbow_r: 10.0 * DEG,
        hip_l: leg_swing, hip_r: -leg_swing,
        knee_l: (0.0f64).max(-cycle.sin()) * 15.0 * DEG,
        knee_r: (0.0f64).max(cycle.sin()) * 15.0 * DEG,
        pos_x: t * 0.3, eyebrow: -0.7, mouth: -0.8,
        ..StickmanPose::neutral()
    }
}

pub fn pose_happy_hop(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.5 * std::f64::consts::PI;
    let hop = (cycle).sin().abs() * 0.08;
    let arm_waving = (cycle).cos() * 40.0 * DEG;
    StickmanPose {
        body_y: hop, body_tilt: -4.0 * DEG,
        shoulder_l: arm_waving - 30.0 * DEG, shoulder_r: -arm_waving - 30.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: cycle.sin() * 25.0 * DEG, hip_r: -cycle.sin() * 25.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        pos_x: t * 1.1, eyebrow: 0.6, mouth: 0.9,
        ..StickmanPose::neutral()
    }
}

pub fn pose_tip_toe(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.3 * std::f64::consts::PI;
    let step = cycle.sin() * 20.0 * DEG;
    StickmanPose {
        body_y: 0.06, body_tilt: 5.0 * DEG,
        shoulder_l: 15.0 * DEG, shoulder_r: -15.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: step, hip_r: -step,
        knee_l: 60.0 * DEG, knee_r: 60.0 * DEG,
        ankle_l: -30.0 * DEG, ankle_r: -30.0 * DEG,
        neck: -15.0 * DEG, pos_x: t * 0.25,
        eyebrow: 0.6, mouth: 0.2,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 2. TRANSITIONS — Perpindahan Antar Pose
// ═══════════════════════════════════════════════════════════════

pub fn pose_stand_to_crouch(t: f64) -> StickmanPose {
    let s = ease_in_out(clamp01(t / 0.5));
    StickmanPose {
        body_y: -0.10 * s, body_tilt: 25.0 * DEG * s,
        shoulder_l: 10.0 * DEG * s, shoulder_r: 10.0 * DEG * s,
        elbow_l: 40.0 * DEG * s, elbow_r: 40.0 * DEG * s,
        knee_l: 50.0 * DEG * s, knee_r: 50.0 * DEG * s,
        neck: -5.0 * DEG * s,
        ..StickmanPose::neutral()
    }
}

pub fn pose_stand_to_sit(t: f64) -> StickmanPose {
    let s = ease_in_out(clamp01(t / 0.6));
    StickmanPose {
        body_y: -0.25 * s, body_tilt: 15.0 * DEG * s,
        pelvis: 30.0 * DEG * s,
        hip_l: -30.0 * DEG * s, hip_r: 30.0 * DEG * s,
        knee_l: 90.0 * DEG * s, knee_r: 90.0 * DEG * s,
        ankle_l: -20.0 * DEG * s, ankle_r: -20.0 * DEG * s,
        shoulder_l: -5.0 * DEG, shoulder_r: 5.0 * DEG,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        neck: 5.0 * DEG * s,
        ..StickmanPose::neutral()
    }
}

pub fn pose_stand_to_kneel(t: f64) -> StickmanPose {
    let s = ease_in_out(clamp01(t / 0.5));
    StickmanPose {
        body_y: -0.18 * s, body_tilt: 10.0 * DEG * s,
        shoulder_l: -5.0 * DEG, shoulder_r: 5.0 * DEG,
        hip_l: -20.0 * DEG * s, hip_r: -20.0 * DEG * s,
        knee_l: 80.0 * DEG * s, knee_r: 80.0 * DEG * s,
        ankle_r: 30.0 * DEG * s,
        ..StickmanPose::neutral()
    }
}

pub fn pose_stand_to_lie(t: f64) -> StickmanPose {
    let s = ease_in_out(clamp01(t / 0.8));
    StickmanPose {
        body_y: -0.01 * s, body_tilt: 90.0 * DEG * s,
        pos_y: -0.15 * s,
        shoulder_l: 30.0 * DEG * s, shoulder_r: -30.0 * DEG * s,
        elbow_l: 20.0 * DEG * s, elbow_r: 20.0 * DEG * s,
        hip_l: -10.0 * DEG * s, hip_r: 10.0 * DEG * s,
        knee_l: 5.0 * DEG * s, knee_r: 5.0 * DEG * s,
        neck: -40.0 * DEG * s,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 3. INTERACTIONS — Interaksi dengan Objek
// ═══════════════════════════════════════════════════════════════

pub fn pose_reach_up(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: 0.05, body_tilt: -5.0 * DEG,
        shoulder_r: -150.0 * DEG, elbow_r: -30.0 * DEG, wrist_r: -10.0 * DEG,
        finger_r: 30.0 * DEG,
        shoulder_l: -30.0 * DEG, elbow_l: 20.0 * DEG,
        neck: -15.0 * DEG, head_turn: 10.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_reach_down(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.08, body_tilt: 45.0 * DEG,
        spine_upper: 20.0 * DEG, spine_lower: 10.0 * DEG,
        shoulder_r: 30.0 * DEG, elbow_r: -60.0 * DEG, wrist_r: -20.0 * DEG,
        finger_r: 40.0 * DEG,
        shoulder_l: -10.0 * DEG, elbow_l: 10.0 * DEG,
        neck: 30.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_reach_forward(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: 15.0 * DEG, pos_x: 0.05,
        shoulder_r: -60.0 * DEG, elbow_r: -10.0 * DEG, wrist_r: -5.0 * DEG,
        finger_r: 20.0 * DEG,
        shoulder_l: -20.0 * DEG, elbow_l: 15.0 * DEG,
        neck: 10.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_grab(t: f64) -> StickmanPose {
    // Snatch/grab motion
    let phase = clamp01(t / 0.3);
    let s = ease_in_out(phase);
    StickmanPose {
        body_tilt: 20.0 * DEG * (1.0 - s) + 5.0 * DEG * s,
        shoulder_r: -80.0 * DEG * (1.0 - s) - 30.0 * DEG * s,
        elbow_r: -20.0 * DEG * (1.0 - s) + 30.0 * DEG * s,
        wrist_r: -30.0 * DEG * (1.0 - s),
        finger_r: 60.0 * DEG * (1.0 - s), // open → close
        shoulder_l: -10.0 * DEG, elbow_l: 10.0 * DEG,
        neck: 15.0 * DEG * (1.0 - s),
        ..StickmanPose::neutral()
    }
}

pub fn pose_pick_up(t: f64) -> StickmanPose {
    // Bend down and pick up from ground
    let s = ease_in_out(clamp01(t / 0.4));
    let recover = ease_in_out(clamp01((t - 0.4) / 0.3));
    let active = if t < 0.4 { 1.0 - s } else { recover };
    StickmanPose {
        body_y: -0.12 * active, body_tilt: 60.0 * DEG * active,
        spine_upper: 30.0 * DEG * active, spine_lower: 15.0 * DEG * active,
        shoulder_r: 20.0 * DEG * active, elbow_r: -80.0 * DEG * active,
        wrist_r: -30.0 * DEG * active, finger_r: 50.0 * DEG * (1.0 - active),
        shoulder_l: 10.0 * DEG * active, elbow_l: -40.0 * DEG * active,
        neck: 45.0 * DEG * active,
        knee_l: 30.0 * DEG * active, knee_r: 30.0 * DEG * active,
        ..StickmanPose::neutral()
    }
}

pub fn pose_carry(t: f64) -> StickmanPose {
    // Carrying something in arms
    StickmanPose {
        body_tilt: -8.0 * DEG,
        shoulder_l: -40.0 * DEG, shoulder_r: 40.0 * DEG,
        elbow_l: 80.0 * DEG, elbow_r: -80.0 * DEG,
        wrist_l: 20.0 * DEG, wrist_r: -20.0 * DEG,
        finger_l: 40.0 * DEG, finger_r: 40.0 * DEG,
        neck: -3.0 * DEG,
        squash_y: 0.95, stretch_x: 0.98,
        ..StickmanPose::neutral()
    }
}

pub fn pose_carry_one_hand(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -10.0 * DEG,
        shoulder_r: -70.0 * DEG, elbow_r: 30.0 * DEG, wrist_r: 10.0 * DEG,
        finger_r: 50.0 * DEG,
        shoulder_l: 5.0 * DEG, elbow_l: 10.0 * DEG,
        neck: -2.0 * DEG, head_turn: 15.0 * DEG,
        squash_y: 0.97,
        ..StickmanPose::neutral()
    }
}

pub fn pose_push(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: 25.0 * DEG, pos_x: 0.03,
        shoulder_l: -40.0 * DEG, shoulder_r: 40.0 * DEG,
        elbow_l: -20.0 * DEG, elbow_r: 20.0 * DEG,
        wrist_l: 10.0 * DEG, wrist_r: -10.0 * DEG,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: -10.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_pull(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -15.0 * DEG, pos_x: -0.03,
        shoulder_l: 50.0 * DEG, shoulder_r: -50.0 * DEG,
        elbow_l: -60.0 * DEG, elbow_r: 60.0 * DEG,
        wrist_l: -10.0 * DEG, wrist_r: 10.0 * DEG,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: 10.0 * DEG,
        squash_y: 0.95,
        ..StickmanPose::neutral()
    }
}

pub fn pose_lift(t: f64) -> StickmanPose {
    let phase = clamp01(t / 0.6);
    let s = ease_in_out(phase);
    StickmanPose {
        body_y: -0.10 * (1.0 - s), body_tilt: 30.0 * DEG * (1.0 - s) - 10.0 * DEG * s,
        shoulder_l: -60.0 * DEG * (1.0 - s) - 80.0 * DEG * s,
        shoulder_r: 60.0 * DEG * (1.0 - s) + 80.0 * DEG * s,
        elbow_l: 90.0 * DEG * (1.0 - s), elbow_r: -90.0 * DEG * (1.0 - s),
        wrist_l: 20.0 * DEG, wrist_r: -20.0 * DEG,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        knee_l: 60.0 * DEG * (1.0 - s), knee_r: 60.0 * DEG * (1.0 - s),
        neck: 5.0 * DEG * (1.0 - s),
        ..StickmanPose::neutral()
    }
}

pub fn pose_throw(t: f64) -> StickmanPose {
    // Wind-up 0-0.3, release 0.3-0.5, follow-through 0.5-1.0
    let (tilt, sh, el, wr, hip, knee) = if t < 0.3 {
        let s = ease_in_out(t / 0.3);
        (-15.0 * DEG * s, 60.0 * DEG * s, -90.0 * DEG * s, -20.0 * DEG * s, 15.0 * DEG * s, 20.0 * DEG * s)
    } else if t < 0.5 {
        let s = (t - 0.3) / 0.2;
        (-15.0 * DEG - 30.0 * DEG * s, 60.0 * DEG - 120.0 * DEG * s, -90.0 * DEG + 120.0 * DEG * s, -20.0 * DEG + 30.0 * DEG * s, 15.0 * DEG - 25.0 * DEG * s, 20.0 * DEG + 10.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.5) / 0.5);
        (-45.0 * DEG * (1.0 - s), -60.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: sh, elbow_r: el, wrist_r: wr,
        finger_r: 50.0 * DEG,
        shoulder_l: 10.0 * DEG, elbow_l: 20.0 * DEG,
        hip_l: hip, hip_r: -hip,
        knee_l: knee, knee_r: knee * 0.5,
        neck: -tilt * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_catch(t: f64) -> StickmanPose {
    let s = ease_in_out(clamp01(t / 0.3));
    StickmanPose {
        body_tilt: -5.0 * DEG * (1.0 - s),
        shoulder_r: -60.0 * DEG * (1.0 - s) - 30.0 * DEG * s,
        elbow_r: 10.0 * DEG * (1.0 - s) + 30.0 * DEG * s,
        wrist_r: -20.0 * DEG * (1.0 - s),
        finger_r: 60.0 * DEG * (1.0 - s),
        shoulder_l: 20.0 * DEG, elbow_l: 15.0 * DEG,
        neck: -3.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_place(t: f64) -> StickmanPose {
    let s = ease_in_out(clamp01(t / 0.4));
    StickmanPose {
        body_tilt: 25.0 * DEG * s, body_y: -0.05 * s,
        shoulder_r: 20.0 * DEG * s, elbow_r: -50.0 * DEG * s,
        wrist_r: -10.0 * DEG * s, finger_r: 30.0 * DEG * (1.0 - s),
        shoulder_l: 10.0 * DEG * s, elbow_l: -20.0 * DEG * s,
        neck: 15.0 * DEG * s,
        knee_l: 15.0 * DEG * s, knee_r: 15.0 * DEG * s,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 4. ENVIRONMENT — Interaksi dengan Lingkungan
// ═══════════════════════════════════════════════════════════════

pub fn pose_sit_chair(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.22, body_tilt: 5.0 * DEG,
        pelvis: 25.0 * DEG,
        hip_l: -25.0 * DEG, hip_r: 25.0 * DEG,
        knee_l: 85.0 * DEG, knee_r: 85.0 * DEG,
        ankle_l: -15.0 * DEG, ankle_r: -15.0 * DEG,
        shoulder_l: -5.0 * DEG, shoulder_r: 5.0 * DEG,
        elbow_l: 25.0 * DEG, elbow_r: 25.0 * DEG,
        wrist_l: 5.0 * DEG, wrist_r: -5.0 * DEG,
        finger_l: 10.0 * DEG, finger_r: 10.0 * DEG,
        neck: 3.0 * DEG,
        spine_upper: -5.0 * DEG, spine_lower: 3.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_sit_ground(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.30, body_tilt: 10.0 * DEG,
        pelvis: 30.0 * DEG,
        hip_l: -40.0 * DEG, hip_r: 40.0 * DEG,
        knee_l: 60.0 * DEG, knee_r: -60.0 * DEG,
        ankle_l: 20.0 * DEG, ankle_r: -20.0 * DEG,
        shoulder_l: 5.0 * DEG, shoulder_r: -5.0 * DEG,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        neck: 8.0 * DEG,
        spine_upper: 10.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_sit_knees_up(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.28, body_tilt: -5.0 * DEG,
        pelvis: 40.0 * DEG,
        hip_l: -50.0 * DEG, hip_r: 50.0 * DEG,
        knee_l: 110.0 * DEG, knee_r: 110.0 * DEG,
        shoulder_l: 10.0 * DEG, shoulder_r: -10.0 * DEG,
        elbow_l: 40.0 * DEG, elbow_r: 40.0 * DEG,
        neck: -10.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_sit_stool(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.15, body_tilt: 3.0 * DEG,
        pelvis: 15.0 * DEG,
        hip_l: -15.0 * DEG, hip_r: 15.0 * DEG,
        knee_l: 95.0 * DEG, knee_r: 95.0 * DEG,
        ankle_l: -25.0 * DEG, ankle_r: -25.0 * DEG,
        shoulder_l: -3.0 * DEG, shoulder_r: 3.0 * DEG,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG,
        neck: 2.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_lie_back(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: 0.02, body_tilt: 90.0 * DEG, pos_y: -0.15,
        shoulder_l: 20.0 * DEG, shoulder_r: -20.0 * DEG,
        elbow_l: 90.0 * DEG, elbow_r: 90.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: 30.0 * DEG, knee_r: 30.0 * DEG,
        neck: -60.0 * DEG,
        head_turn: 5.0 * DEG,
        mouth: 0.2,
        ..StickmanPose::neutral()
    }
}

pub fn pose_lie_side(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: 0.01, body_tilt: 90.0 * DEG, pos_y: -0.15,
        shoulder_l: 10.0 * DEG, shoulder_r: 30.0 * DEG,
        elbow_l: 60.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: -10.0 * DEG, hip_r: 20.0 * DEG,
        knee_l: 40.0 * DEG, knee_r: 20.0 * DEG,
        neck: -50.0 * DEG, head_turn: 20.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_lie_stomach(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.01, body_tilt: -90.0 * DEG, pos_y: -0.15,
        shoulder_l: -30.0 * DEG, shoulder_r: 30.0 * DEG,
        elbow_l: 90.0 * DEG, elbow_r: -90.0 * DEG,
        wrist_l: -10.0 * DEG, wrist_r: 10.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        neck: 40.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_lean_wall(t: f64) -> StickmanPose {
    let shift = (t * 0.3).sin() * 0.005;
    StickmanPose {
        body_y: -0.02, body_tilt: 15.0 * DEG,
        shoulder_l: 10.0 * DEG, shoulder_r: -30.0 * DEG,
        elbow_l: 20.0 * DEG, elbow_r: -60.0 * DEG,
        wrist_r: 10.0 * DEG,
        hip_l: -10.0 * DEG, hip_r: 10.0 * DEG,
        knee_l: 5.0 * DEG, knee_r: 5.0 * DEG,
        ankle_l: -5.0 * DEG,
        neck: -5.0 * DEG, head_turn: -15.0 * DEG,
        pos_x: shift,
        ..StickmanPose::neutral()
    }
}

pub fn pose_kneel(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.18, body_tilt: 5.0 * DEG,
        shoulder_l: -5.0 * DEG, shoulder_r: 5.0 * DEG,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG,
        hip_l: -15.0 * DEG, hip_r: -15.0 * DEG,
        knee_l: 80.0 * DEG, knee_r: 80.0 * DEG,
        ankle_r: 30.0 * DEG,
        neck: -2.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_kneel_one(t: f64) -> StickmanPose {
    // One knee down
    StickmanPose {
        body_y: -0.12, body_tilt: 3.0 * DEG,
        hip_l: -10.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 90.0 * DEG, knee_r: 30.0 * DEG,
        ankle_l: 20.0 * DEG,
        shoulder_l: -3.0 * DEG, shoulder_r: 3.0 * DEG,
        elbow_l: 10.0 * DEG, elbow_r: 10.0 * DEG,
        neck: -1.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_crouch(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.10, body_tilt: 25.0 * DEG,
        shoulder_l: 10.0 * DEG, shoulder_r: 10.0 * DEG,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 55.0 * DEG, knee_r: 55.0 * DEG,
        neck: -5.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_squat(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.20, body_tilt: -5.0 * DEG,
        pelvis: 30.0 * DEG,
        hip_l: -20.0 * DEG, hip_r: 20.0 * DEG,
        knee_l: 120.0 * DEG, knee_r: 120.0 * DEG,
        ankle_l: -30.0 * DEG, ankle_r: -30.0 * DEG,
        shoulder_l: 5.0 * DEG, shoulder_r: -5.0 * DEG,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        neck: 5.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_climb(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.0 * std::f64::consts::PI;
    let pull = (cycle).sin().abs() * 30.0 * DEG;
    let step = (cycle + std::f64::consts::PI).sin() * 25.0 * DEG;
    StickmanPose {
        body_y: 0.05 + (cycle * 2.0).sin().abs() * 0.04,
        body_tilt: -10.0 * DEG,
        shoulder_l: -120.0 * DEG * (1.0 - pull / 30.0 * DEG),
        shoulder_r: 120.0 * DEG * (pull / 30.0 * DEG),
        elbow_l: 40.0 * DEG, elbow_r: 40.0 * DEG,
        hip_l: step, hip_r: -step,
        knee_l: 90.0 * DEG, knee_r: 90.0 * DEG,
        neck: -15.0 * DEG,
        pos_y: t * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_hang(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: 0.25, squash_y: 0.90, stretch_x: 1.05,
        shoulder_l: -170.0 * DEG, shoulder_r: 170.0 * DEG,
        elbow_l: -10.0 * DEG, elbow_r: 10.0 * DEG,
        wrist_l: -10.0 * DEG, wrist_r: 10.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 0.0, hip_r: 0.0,
        knee_l: 5.0 * DEG, knee_r: 5.0 * DEG,
        neck: -5.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_hang_one_arm(t: f64) -> StickmanPose {
    let sway = (t * 1.5).sin() * 5.0 * DEG;
    StickmanPose {
        body_y: 0.25, squash_y: 0.90,
        body_tilt: sway * 0.5,
        shoulder_r: -170.0 * DEG, elbow_r: -5.0 * DEG,
        wrist_r: -5.0 * DEG, finger_r: 60.0 * DEG,
        shoulder_l: -20.0 * DEG, elbow_l: 30.0 * DEG,
        hip_l: sway, hip_r: -sway,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: 5.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_vault(t: f64) -> StickmanPose {
    // Vault over obstacle: 0-0.3 approach, 0.3-0.7 over, 0.7-1.0 land
    let phase = t;
    let (tilt, body, arm, leg) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-10.0 * DEG * s, 0.0, -60.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.7 {
        let s = (phase - 0.3) / 0.4;
        (-20.0 * DEG - 30.0 * DEG * s, 0.15 * s, -60.0 * DEG + 100.0 * DEG * s, 40.0 * DEG + 60.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.7) / 0.3);
        (-50.0 * DEG * (1.0 - s), 0.15 * (1.0 - s), 40.0 * DEG * (1.0 - s), 100.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.8,
        shoulder_l: arm - 30.0 * DEG, shoulder_r: -arm + 30.0 * DEG,
        elbow_l: 80.0 * DEG, elbow_r: 80.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: 60.0 * DEG, knee_r: 60.0 * DEG,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_swing(t: f64) -> StickmanPose {
    let phase = t * 2.0 * std::f64::consts::PI;
    let swing = phase.sin() * 30.0 * DEG;
    StickmanPose {
        body_y: 0.25, body_tilt: swing,
        stretch_x: 1.05,
        shoulder_l: -170.0 * DEG, shoulder_r: 170.0 * DEG,
        elbow_l: -5.0 * DEG, elbow_r: 5.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: swing * 0.3, hip_r: -swing * 0.3,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: swing * 0.2,
        ..StickmanPose::neutral()
    }
}

pub fn pose_dive(t: f64) -> StickmanPose {
    // Dive forward: 0-0.3 jump, 0.3-0.7 airborne arms out, 0.7-1.0 impact
    let phase = t;
    let (body, tilt, arm, leg) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-0.05 * s, 15.0 * DEG * s, -20.0 * DEG * s, -10.0 * DEG * s)
    } else if phase < 0.7 {
        let s = (phase - 0.3) / 0.4;
        (0.15 * (1.0 - (2.0 * s - 1.0).powi(2)), 15.0 * DEG + 30.0 * DEG * s, -40.0 * DEG - 40.0 * DEG * s, 10.0 * DEG + 20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.7) / 0.3);
        (-0.08 * s, 45.0 * DEG * (1.0 - s), -80.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_y: body, body_tilt: tilt, pos_x: t * 0.5,
        shoulder_l: arm, shoulder_r: -arm,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: 15.0 * DEG * (phase / 0.7).min(1.0),
        knee_r: 15.0 * DEG * (phase / 0.7).min(1.0),
        neck: -tilt * 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_slide(t: f64) -> StickmanPose {
    // Slide on knees
    let phase = clamp01(t / 0.5);
    let s = ease_in_out(phase);
    StickmanPose {
        body_y: -0.15 * s, body_tilt: 15.0 * DEG * s,
        pos_x: t * 0.4,
        shoulder_l: 20.0 * DEG * s, shoulder_r: -20.0 * DEG * s,
        elbow_l: 50.0 * DEG * s, elbow_r: 50.0 * DEG * s,
        hip_l: -10.0 * DEG * s, hip_r: 10.0 * DEG * s,
        knee_l: 90.0 * DEG * s, knee_r: 90.0 * DEG * s,
        ankle_l: 20.0 * DEG * s, ankle_r: 20.0 * DEG * s,
        neck: -5.0 * DEG * s,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 5. COMBAT — Pertarungan & Tembak-menembak
// ═══════════════════════════════════════════════════════════════

pub fn pose_jab(t: f64) -> StickmanPose {
    let (shoulder, elbow, tilt) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (-10.0 * DEG * s, 40.0 * DEG * s, -3.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (-10.0 * DEG - 30.0 * DEG * s, 40.0 * DEG - 50.0 * DEG * s, -3.0 * DEG + 8.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (-40.0 * DEG * (1.0 - s) - 10.0 * DEG * s, -10.0 * DEG * (1.0 - s) + 5.0 * DEG * s, 5.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        shoulder_r: shoulder, elbow_r: elbow, body_tilt: tilt,
        shoulder_l: 15.0 * DEG, elbow_l: 20.0 * DEG,
        neck: -tilt * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_cross(t: f64) -> StickmanPose {
    let (shoulder, elbow, tilt, hip) = if t < 0.25 {
        let s = ease_in_out(t / 0.25);
        (-20.0 * DEG * s, 50.0 * DEG * s, -5.0 * DEG * s, 10.0 * DEG * s)
    } else if t < 0.5 {
        let s = (t - 0.25) / 0.25;
        (-20.0 * DEG - 50.0 * DEG * s, 50.0 * DEG - 70.0 * DEG * s, -5.0 * DEG + 15.0 * DEG * s, 10.0 * DEG - 20.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.5) / 0.5);
        (-70.0 * DEG * (1.0 - s), -20.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        shoulder_r: shoulder, elbow_r: elbow, body_tilt: tilt,
        hip_l: hip, hip_r: -hip * 0.5,
        shoulder_l: 20.0 * DEG, elbow_l: 25.0 * DEG,
        neck: -tilt * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_uppercut(t: f64) -> StickmanPose {
    let (shoulder, elbow, tilt, knee) = if t < 0.3 {
        let s = ease_in_out(t / 0.3);
        (20.0 * DEG * s, 90.0 * DEG * s, -8.0 * DEG * s, 20.0 * DEG * s)
    } else if t < 0.55 {
        let s = (t - 0.3) / 0.25;
        (20.0 * DEG - 70.0 * DEG * s, 90.0 * DEG - 100.0 * DEG * s, -8.0 * DEG - 10.0 * DEG * s, 20.0 * DEG - 10.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.55) / 0.45);
        (-50.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s), -18.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        shoulder_r: shoulder, elbow_r: elbow, body_tilt: tilt,
        wrist_r: -10.0 * DEG,
        hip_l: 15.0 * DEG, hip_r: -15.0 * DEG,
        knee_l: knee, knee_r: knee * 0.7,
        shoulder_l: 10.0 * DEG, elbow_l: 15.0 * DEG,
        neck: -tilt * 0.4, jaw_open: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_kick(t: f64) -> StickmanPose {
    let (hip, knee, body, arm) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (-15.0 * DEG * s, 60.0 * DEG * s, -5.0 * DEG * s, -10.0 * DEG * s)
    } else if t < 0.45 {
        let s = (t - 0.2) / 0.25;
        (-15.0 * DEG + 55.0 * DEG * s, 60.0 * DEG - 50.0 * DEG * s, -5.0 * DEG + 15.0 * DEG * s, -10.0 * DEG + 30.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.45) / 0.55);
        (40.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), 20.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: body,
        hip_r: hip, knee_r: knee, ankle_r: -20.0 * DEG,
        toe_r: 30.0 * DEG,
        hip_l: -hip * 0.5, knee_l: 15.0 * DEG,
        shoulder_l: -20.0 * DEG - arm, shoulder_r: 20.0 * DEG + arm,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG,
        neck: -body * 0.5, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_block(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -10.0 * DEG, body_y: -0.02,
        shoulder_l: -40.0 * DEG, shoulder_r: 40.0 * DEG,
        elbow_l: 110.0 * DEG, elbow_r: 110.0 * DEG,
        wrist_l: 20.0 * DEG, wrist_r: -20.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: 5.0 * DEG, eye_squint: -0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_duck(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.12, body_tilt: 30.0 * DEG,
        shoulder_l: 15.0 * DEG, shoulder_r: 15.0 * DEG,
        elbow_l: 60.0 * DEG, elbow_r: 60.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 60.0 * DEG, knee_r: 60.0 * DEG,
        neck: 15.0 * DEG, head_turn: 10.0 * DEG,
        eye_squint: -0.5, eyebrow: 0.8,
        ..StickmanPose::neutral()
    }
}

pub fn pose_shoot(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, shoulder, elbow, head) = if phase < 0.15 {
        let s = ease_in_out(phase / 0.15);
        (-3.0 * DEG * s, -50.0 * DEG * s, 60.0 * DEG * s, -5.0 * DEG * s)
    } else if phase < 0.25 {
        let s = (phase - 0.15) / 0.1;
        (-3.0 * DEG - 10.0 * DEG * s, -50.0 * DEG + 15.0 * DEG * s, 60.0 * DEG - 20.0 * DEG * s, -5.0 * DEG - 10.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.25) / 0.75);
        (-13.0 * DEG * (1.0 - s), -35.0 * DEG * (1.0 - s) - 50.0 * DEG * s, 40.0 * DEG * (1.0 - s) + 60.0 * DEG * s, -15.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow, wrist_r: -30.0 * DEG,
        shoulder_l: 5.0 * DEG, elbow_l: 10.0 * DEG,
        neck: head, eye_squint: -0.8, eyebrow: 0.5, jaw_open: 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_aim(t: f64) -> StickmanPose {
    let mut p = pose_shoot(t * 0.5);
    p.mouth = 0.2;
    p.eye_blink = 0.0;
    p
}

pub fn pose_shoot_rifle(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, sh_l, sh_r, el_l, el_r) = if phase < 0.15 {
        let s = ease_in_out(phase / 0.15);
        (-5.0 * DEG * s, -40.0 * DEG * s, 40.0 * DEG * s, 50.0 * DEG * s, 50.0 * DEG * s)
    } else if phase < 0.3 {
        let s = (phase - 0.15) / 0.15;
        (-5.0 * DEG - 8.0 * DEG * s, -40.0 * DEG + 10.0 * DEG * s, 40.0 * DEG - 10.0 * DEG * s, 50.0 * DEG - 20.0 * DEG * s, 50.0 * DEG - 20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.3) / 0.7);
        (-13.0 * DEG * (1.0 - s), -30.0 * DEG * (1.0 - s) - 30.0 * DEG * s, 30.0 * DEG * (1.0 - s) + 30.0 * DEG * s, 30.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_l: sh_l, shoulder_r: sh_r,
        elbow_l: el_l, elbow_r: el_r,
        wrist_l: -20.0 * DEG, wrist_r: 20.0 * DEG,
        neck: -10.0 * DEG, eye_squint: -0.9, eyebrow: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_shoot_from_cover(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.10, body_tilt: 30.0 * DEG,
        shoulder_l: 20.0 * DEG, shoulder_r: -60.0 * DEG,
        elbow_l: 40.0 * DEG, elbow_r: 60.0 * DEG,
        wrist_r: -30.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 55.0 * DEG, knee_r: 55.0 * DEG,
        neck: -5.0 * DEG, eye_squint: -0.8, eyebrow: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_reload(t: f64) -> StickmanPose {
    let phase = t;
    let (sh, el, head) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-70.0 * DEG * s, 10.0 * DEG * s, -5.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (-70.0 * DEG + 40.0 * DEG * s, 10.0 * DEG + 60.0 * DEG * s, -5.0 * DEG - 15.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (-30.0 * DEG * (1.0 - s) - 50.0 * DEG * s, 70.0 * DEG * (1.0 - s), -20.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        shoulder_r: sh, elbow_r: el,
        shoulder_l: 10.0 * DEG, elbow_l: 20.0 * DEG,
        wrist_r: 10.0 * DEG,
        neck: head,
        ..StickmanPose::neutral()
    }
}

pub fn pose_tackle(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, arm, leg) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-10.0 * DEG * s, -30.0 * DEG * s, 10.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.3) / 0.7);
        (-10.0 * DEG - 35.0 * DEG * s, -30.0 * DEG - 30.0 * DEG * s, 10.0 * DEG + 20.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, body_y: -(phase * 0.08).min(0.06), pos_x: t * 0.6,
        shoulder_l: arm - 10.0 * DEG, shoulder_r: -arm + 10.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: 40.0 * DEG, knee_r: 40.0 * DEG,
        neck: -tilt * 0.5, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_stun(t: f64) -> StickmanPose {
    let wobble = (t * 8.0).sin() * 8.0 * DEG;
    StickmanPose {
        body_tilt: 15.0 * DEG + wobble,
        body_y: -0.03,
        shoulder_l: 15.0 * DEG, shoulder_r: -15.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: wobble * 0.5, hip_r: -wobble * 0.5,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: 10.0 * DEG + wobble * 0.5,
        head_turn: wobble * 0.5,
        eye_blink: 0.5,
        eyebrow: -0.3,
        mouth: -0.5,
        jaw_open: 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_takedown(t: f64) -> StickmanPose {
    // Grab and throw opponent
    let phase = t;
    let (tilt, arm_l, arm_r) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-10.0 * DEG * s, -40.0 * DEG * s, 40.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (-10.0 * DEG - 20.0 * DEG * s, -40.0 * DEG - 20.0 * DEG * s, 40.0 * DEG + 20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (-30.0 * DEG * (1.0 - s), -60.0 * DEG * (1.0 - s), 60.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: -(phase * 0.05).min(0.04),
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 15.0 * DEG, hip_r: -15.0 * DEG,
        knee_l: 30.0 * DEG, knee_r: 30.0 * DEG,
        neck: -tilt * 0.5, jaw_open: 0.8,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 6. EXPRESSIVE — Gestur Ekspresif & Emosi
// ═══════════════════════════════════════════════════════════════

pub fn pose_wave(t: f64) -> StickmanPose {
    let wave = ((t * 4.0).sin() * 0.5 + 0.5) * 35.0 * DEG;
    StickmanPose {
        shoulder_r: -90.0 * DEG, elbow_r: -wave - 20.0 * DEG, wrist_r: wave * 0.5,
        shoulder_l: -5.0 * DEG, elbow_l: 5.0 * DEG,
        mouth: 0.8, neck: -5.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_wave_both(t: f64) -> StickmanPose {
    let w = ((t * 3.5).sin() * 0.5 + 0.5) * 30.0 * DEG;
    StickmanPose {
        shoulder_l: 90.0 * DEG, shoulder_r: -90.0 * DEG,
        elbow_l: -w - 10.0 * DEG, elbow_r: -w - 10.0 * DEG,
        wrist_l: w * 0.3, wrist_r: w * 0.3,
        mouth: 0.9, neck: -8.0 * DEG, body_y: (t * 2.0).sin().abs() * 0.02,
        ..StickmanPose::neutral()
    }
}

pub fn pose_point(t: f64) -> StickmanPose {
    StickmanPose {
        shoulder_r: -30.0 * DEG, elbow_r: -30.0 * DEG, wrist_r: -10.0 * DEG,
        finger_r: 60.0 * DEG,
        shoulder_l: 10.0 * DEG, elbow_l: 15.0 * DEG,
        body_tilt: -5.0 * DEG, neck: -5.0 * DEG, head_turn: 20.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_cheer(t: f64) -> StickmanPose {
    let j = (t * 3.0).sin().abs() * 0.04;
    StickmanPose {
        body_y: j,
        shoulder_l: -130.0 * DEG, shoulder_r: 130.0 * DEG,
        elbow_l: -10.0 * DEG, elbow_r: 10.0 * DEG,
        wrist_l: 20.0 * DEG, wrist_r: -20.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        neck: -10.0 * DEG, mouth: 1.0,
        eyebrow: 0.8,
        ..StickmanPose::neutral()
    }
}

pub fn pose_victory(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: 0.03, body_tilt: -5.0 * DEG,
        shoulder_l: -140.0 * DEG, shoulder_r: 140.0 * DEG,
        elbow_l: -30.0 * DEG, elbow_r: -30.0 * DEG,
        wrist_l: 10.0 * DEG, wrist_r: -10.0 * DEG,
        finger_l: 70.0 * DEG, finger_r: 70.0 * DEG,
        neck: -12.0 * DEG, mouth: 1.0, eyebrow: 1.0,
        ..StickmanPose::neutral()
    }
}

pub fn pose_despair(t: f64) -> StickmanPose {
    let shake = (t * 6.0).sin() * 3.0 * DEG;
    StickmanPose {
        body_y: -0.03, body_tilt: 15.0 * DEG + shake,
        spine_upper: 20.0 * DEG, neck: 30.0 * DEG,
        shoulder_l: 60.0 * DEG, shoulder_r: -60.0 * DEG,
        elbow_l: -90.0 * DEG, elbow_r: 90.0 * DEG,
        wrist_l: -10.0 * DEG, wrist_r: 10.0 * DEG,
        finger_l: 40.0 * DEG, finger_r: 40.0 * DEG,
        hip_l: shake * 0.5, hip_r: -shake * 0.5,
        eyebrow: -0.9, mouth: -0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_facepalm(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: 5.0 * DEG, neck: 10.0 * DEG, head_turn: -5.0 * DEG,
        shoulder_r: 40.0 * DEG, elbow_r: -100.0 * DEG, wrist_r: 20.0 * DEG,
        finger_r: 40.0 * DEG,
        shoulder_l: -5.0 * DEG, elbow_l: 5.0 * DEG,
        eyebrow: -0.8, mouth: -0.5, eye_squint: -0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_shrug(t: f64) -> StickmanPose {
    let s = (t * 1.0).sin() * 0.5 + 0.5;
    StickmanPose {
        body_y: -0.01,
        shoulder_l: 30.0 * DEG * s, shoulder_r: -30.0 * DEG * s,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        wrist_l: 20.0 * DEG, wrist_r: -20.0 * DEG,
        finger_l: 30.0 * DEG, finger_r: 30.0 * DEG,
        neck: -5.0 * DEG * s, mouth: -0.3, eyebrow: 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_salute(t: f64) -> StickmanPose {
    StickmanPose {
        shoulder_r: -120.0 * DEG, elbow_r: -10.0 * DEG, wrist_r: 10.0 * DEG,
        finger_r: 50.0 * DEG,
        shoulder_l: -5.0 * DEG, elbow_l: 5.0 * DEG,
        neck: -8.0 * DEG, body_tilt: -3.0 * DEG,
        eyebrow: 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_taunt(t: f64) -> StickmanPose {
    let beckon = ((t * 2.0).sin() * 0.5 + 0.5) * 30.0 * DEG;
    StickmanPose {
        body_tilt: 5.0 * DEG,
        shoulder_r: -60.0 * DEG, elbow_r: -10.0 * DEG,
        wrist_r: -30.0 * DEG + beckon,
        finger_r: 50.0 * DEG,
        shoulder_l: 15.0 * DEG, elbow_l: 20.0 * DEG,
        neck: 5.0 * DEG, head_turn: 25.0 * DEG,
        eyebrow: 0.7, mouth: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_nod(t: f64) -> StickmanPose {
    let n = (t * 3.0).sin() * 10.0 * DEG;
    StickmanPose {
        neck: n, body_tilt: n * 0.3,
        mouth: 0.2,
        ..StickmanPose::neutral()
    }
}

pub fn pose_shake_head(t: f64) -> StickmanPose {
    let s = (t * 4.0).sin() * 15.0 * DEG;
    StickmanPose {
        head_turn: s, neck: -3.0 * DEG,
        eyebrow: -0.3, mouth: -0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_celebrate(t: f64) -> StickmanPose {
    let jump = (t * 3.0).sin().abs() * 0.08;
    let arms = (t * 4.0).sin() * 15.0 * DEG;
    StickmanPose {
        body_y: jump,
        shoulder_l: -100.0 * DEG + arms, shoulder_r: 100.0 * DEG - arms,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        wrist_l: 30.0 * DEG, wrist_r: -30.0 * DEG,
        finger_l: 70.0 * DEG, finger_r: 70.0 * DEG,
        hip_l: arms * 0.5, hip_r: -arms * 0.5,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: -15.0 * DEG, mouth: 1.0, eyebrow: 1.0, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 7. INJURY — Cedera, Jatuh, & Kondisi Luka
// ═══════════════════════════════════════════════════════════════

pub fn pose_hurt(t: f64) -> StickmanPose {
    let recoil = (t * 15.0).sin().exp() * 5.0 * DEG;
    let recover = (-t * 2.0).exp();
    StickmanPose {
        body_tilt: 10.0 * DEG * recover + recoil,
        body_y: -0.02 * recover,
        shoulder_l: 20.0 * DEG, shoulder_r: -40.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 60.0 * DEG,
        wrist_r: -10.0 * DEG,
        hip_l: recoil * 0.5, hip_r: -recoil * 0.5,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: 10.0 * DEG * recover,
        eye_blink: 0.7, eyebrow: -0.5, mouth: -0.6, jaw_open: 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_hurt_heavy(t: f64) -> StickmanPose {
    let wobble = (t * 6.0).sin() * 10.0 * DEG;
    let sink = (-t * 1.5).exp();
    StickmanPose {
        body_tilt: 25.0 * DEG * sink + wobble,
        body_y: -0.08 * sink,
        shoulder_l: 30.0 * DEG, shoulder_r: -30.0 * DEG,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        hip_l: wobble * 0.5, hip_r: -wobble * 0.5,
        knee_l: 30.0 * DEG, knee_r: 30.0 * DEG,
        neck: 20.0 * DEG * sink + wobble * 0.5,
        head_turn: wobble,
        eye_blink: 0.9, eyebrow: -0.8, mouth: -0.9, jaw_open: 0.5,
        eye_squint: -0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_knock_down(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, arm, leg) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-20.0 * DEG * s, 0.0, 20.0 * DEG * s, 10.0 * DEG * s)
    } else if phase < 0.7 {
        let s = (phase - 0.3) / 0.4;
        (-20.0 * DEG - 70.0 * DEG * s, -0.02 * s, 20.0 * DEG + 40.0 * DEG * s, 10.0 * DEG + 30.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.7) / 0.3);
        (-90.0 * DEG * (1.0 - s), -0.02 * (1.0 - s) - 0.10 * s, 60.0 * DEG * (1.0 - s) + 80.0 * DEG * s, 40.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_y: -0.10 * phase.min(1.0),
        shoulder_l: arm, shoulder_r: -arm,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: -tilt * 0.4,
        eye_blink: 0.8, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_get_up(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body) = if phase < 0.4 {
        let s = ease_in_out(phase / 0.4);
        (90.0 * DEG * (1.0 - s), -0.12 + 0.06 * s)
    } else if phase < 0.7 {
        let s = (phase - 0.4) / 0.3;
        (0.0, -0.06 + 0.04 * s)
    } else {
        let s = ease_in_out((phase - 0.7) / 0.3);
        (0.0, -0.02 + 0.02 * s)
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_y: -0.10 * (1.0 - phase.min(0.7) / 0.7),
        shoulder_l: 30.0 * DEG * (1.0 - phase.min(1.0)),
        shoulder_r: -30.0 * DEG * (1.0 - phase.min(1.0)),
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        hip_l: 20.0 * DEG * (1.0 - phase.min(1.0)),
        hip_r: -20.0 * DEG * (1.0 - phase.min(1.0)),
        knee_l: 60.0 * DEG, knee_r: 60.0 * DEG,
        neck: tilt * 0.3,
        eyebrow: -0.3, mouth: -0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_drag_limp(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.0 * std::f64::consts::PI;
    let drag = cycle.sin() * 15.0 * DEG;
    StickmanPose {
        body_tilt: 20.0 * DEG, body_y: -0.05,
        spine_upper: 15.0 * DEG, neck: 20.0 * DEG,
        shoulder_l: -20.0 * DEG, shoulder_r: 20.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: drag, hip_r: drag * 0.5,
        knee_l: -10.0 * DEG, knee_r: 30.0 * DEG,
        ankle_r: -15.0 * DEG,
        pos_x: t * 0.2,
        eyebrow: -0.5, mouth: -0.7, eye_squint: -0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_dead(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: 90.0 * DEG, pos_y: -0.15,
        shoulder_l: 40.0 * DEG, shoulder_r: -40.0 * DEG,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        hip_l: -15.0 * DEG, hip_r: 15.0 * DEG,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: -50.0 * DEG, head_turn: 15.0 * DEG,
        eye_blink: 1.0, mouth: -0.5,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 8. ACROBATIC — Gerakan Akrobatik
// ═══════════════════════════════════════════════════════════════

pub fn pose_cartwheel(t: f64) -> StickmanPose {
    let spin = t * 360.0 * DEG;
    let height = (t * std::f64::consts::PI).sin().abs() * 0.20;
    StickmanPose {
        body_tilt: spin, body_y: height,
        shoulder_l: -90.0 * DEG, shoulder_r: 90.0 * DEG,
        elbow_l: -10.0 * DEG, elbow_r: 10.0 * DEG,
        hip_l: -30.0 * DEG, hip_r: 30.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: -20.0 * DEG,
        pos_x: t * 0.8,
        ..StickmanPose::neutral()
    }
}

pub fn pose_flip(t: f64) -> StickmanPose {
    let spin = t * 360.0 * DEG;
    let tuck = 1.0 - (t * 2.0 - 1.0).powi(2);
    StickmanPose {
        body_tilt: spin, body_y: 0.25 * (t * std::f64::consts::PI).sin(),
        shoulder_l: -30.0 * DEG * tuck, shoulder_r: 30.0 * DEG * tuck,
        elbow_l: 100.0 * DEG * tuck, elbow_r: 100.0 * DEG * tuck,
        hip_l: -20.0 * DEG * tuck, hip_r: 20.0 * DEG * tuck,
        knee_l: 100.0 * DEG * tuck, knee_r: 100.0 * DEG * tuck,
        neck: -20.0 * DEG * tuck,
        ..StickmanPose::neutral()
    }
}

pub fn pose_handstand(t: f64) -> StickmanPose {
    let wobble = (t * 2.0).sin() * 5.0 * DEG;
    StickmanPose {
        body_tilt: 180.0 * DEG + wobble, body_y: 0.30,
        shoulder_l: 170.0 * DEG, shoulder_r: -170.0 * DEG,
        elbow_l: -5.0 * DEG, elbow_r: 5.0 * DEG,
        wrist_l: -10.0 * DEG, wrist_r: 10.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: wobble, hip_r: -wobble,
        knee_l: -5.0 * DEG, knee_r: -5.0 * DEG,
        neck: 5.0 * DEG + wobble * 0.5,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 9. KICKS — Variasi Tendangan Lengkap
// ═══════════════════════════════════════════════════════════════

pub fn pose_roundhouse_kick(t: f64) -> StickmanPose {
    let (tilt, hip, knee, arm, toe) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (-3.0 * DEG * s, -10.0 * DEG * s, 50.0 * DEG * s, 10.0 * DEG * s, 0.0)
    } else if t < 0.5 {
        let s = (t - 0.2) / 0.3;
        (-3.0 * DEG - 20.0 * DEG * s, -10.0 * DEG + 70.0 * DEG * s, 50.0 * DEG - 40.0 * DEG * s, 10.0 * DEG + 30.0 * DEG * s, s * 30.0 * DEG)
    } else {
        let s = ease_in_out((t - 0.5) / 0.5);
        (-23.0 * DEG * (1.0 - s), 60.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), 40.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, hip_r: hip, knee_r: knee, ankle_r: -15.0 * DEG, toe_r: toe,
        hip_l: -hip * 0.4, knee_l: 15.0 * DEG,
        shoulder_l: -30.0 * DEG - arm, shoulder_r: 10.0 * DEG + arm,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG, neck: -tilt * 0.4,
        spine_upper: tilt * 0.3, head_turn: -15.0 * DEG, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_front_kick(t: f64) -> StickmanPose {
    let (tilt, hip, knee, arm) = if t < 0.15 {
        let s = ease_in_out(t / 0.15);
        (-3.0 * DEG * s, -10.0 * DEG * s, 60.0 * DEG * s, 5.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.15) / 0.25;
        (-3.0 * DEG - 12.0 * DEG * s, -10.0 * DEG + 50.0 * DEG * s, 60.0 * DEG - 50.0 * DEG * s, 5.0 * DEG + 25.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (-15.0 * DEG * (1.0 - s), 40.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, hip_r: hip, knee_r: knee, ankle_r: -10.0 * DEG, toe_r: 20.0 * DEG,
        hip_l: -hip * 0.4, knee_l: 10.0 * DEG,
        shoulder_l: -15.0 * DEG - arm, shoulder_r: 15.0 * DEG + arm,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG, neck: -tilt * 0.4, jaw_open: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_side_kick(t: f64) -> StickmanPose {
    let (tilt, hip, knee, body_twist) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (5.0 * DEG * s, -15.0 * DEG * s, 55.0 * DEG * s, -10.0 * DEG * s)
    } else if t < 0.45 {
        let s = (t - 0.2) / 0.25;
        (20.0 * DEG * s, -15.0 * DEG + 60.0 * DEG * s, 55.0 * DEG - 45.0 * DEG * s, -10.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.45) / 0.55);
        (20.0 * DEG * (1.0 - s), 45.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, hip_r: hip, knee_r: knee, ankle_r: -20.0 * DEG, toe_r: 25.0 * DEG,
        hip_l: -hip * 0.5, knee_l: 15.0 * DEG,
        body_angle: 0.5 + body_twist * 0.3, pelvis: body_twist,
        shoulder_l: -40.0 * DEG, shoulder_r: 40.0 * DEG,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG, neck: -tilt * 0.3, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_axe_kick(t: f64) -> StickmanPose {
    let (tilt, hip, knee, arm_l, arm_r) = if t < 0.25 {
        let s = ease_in_out(t / 0.25);
        (10.0 * DEG * s, -30.0 * DEG * s, 80.0 * DEG * s, -5.0 * DEG * s, 5.0 * DEG * s)
    } else if t < 0.55 {
        let s = (t - 0.25) / 0.3;
        (10.0 * DEG - 30.0 * DEG * s, -30.0 * DEG + 80.0 * DEG * s, 80.0 * DEG - 90.0 * DEG * s, -5.0 * DEG - 40.0 * DEG * s, 5.0 * DEG + 40.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.55) / 0.45);
        (-20.0 * DEG * (1.0 - s), 50.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s), -45.0 * DEG * (1.0 - s), 45.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, hip_r: hip, knee_r: knee, ankle_r: -30.0 * DEG,
        hip_l: -hip * 0.3, knee_l: 5.0 * DEG,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 10.0 * DEG, elbow_r: 10.0 * DEG, neck: -tilt * 0.3, jaw_open: 0.8,
        ..StickmanPose::neutral()
    }
}

pub fn pose_kick_head(t: f64) -> StickmanPose {
    let high = pose_roundhouse_kick(t);
    StickmanPose { body_y: high.body_y + 0.08, toe_r: high.toe_r + 15.0 * DEG, knee_r: high.knee_r.max(20.0 * DEG), jaw_open: 0.9, ..high }
}

pub fn pose_kick_body(t: f64) -> StickmanPose {
    let mut p = pose_front_kick(t);
    p.body_tilt += 5.0 * DEG;
    p.knee_r = p.knee_r * 0.7;
    p.toe_r = 10.0 * DEG;
    p
}

pub fn pose_kick_leg(t: f64) -> StickmanPose {
    let (tilt, hip, knee) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (10.0 * DEG * s, -20.0 * DEG * s, 40.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (10.0 * DEG + 10.0 * DEG * s, -20.0 * DEG + 40.0 * DEG * s, 40.0 * DEG - 30.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (20.0 * DEG * (1.0 - s), 20.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: -0.04 * t.min(1.0),
        hip_r: hip, knee_r: knee, ankle_r: -5.0 * DEG,
        hip_l: -hip * 0.3, knee_l: 15.0 * DEG,
        shoulder_l: -20.0 * DEG, shoulder_r: 20.0 * DEG,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG, neck: -5.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_flying_kick(t: f64) -> StickmanPose {
    let phase = t;
    let (body, tilt, hip, knee, arm_l, arm_r) = if phase < 0.2 {
        let s = ease_in_out(phase / 0.2);
        (-0.05 * s, -5.0 * DEG * s, -20.0 * DEG * s, 70.0 * DEG * s, -10.0 * DEG * s, 10.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.2) / 0.3;
        (0.20 * s, -5.0 * DEG - 20.0 * DEG * s, -20.0 * DEG + 70.0 * DEG * s, 70.0 * DEG - 50.0 * DEG * s, -10.0 * DEG - 30.0 * DEG * s, 10.0 * DEG + 30.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.5) / 0.5);
        (0.20 * (1.0 - 2.0 * s), -25.0 * DEG * (1.0 - s), 50.0 * DEG * (1.0 - s), 20.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s), 40.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_y: body, body_tilt: tilt, pos_x: t * 0.8,
        hip_r: hip, knee_r: knee, ankle_r: -15.0 * DEG, toe_r: 20.0 * DEG,
        hip_l: -hip * 0.4, knee_l: 20.0 * DEG,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG, neck: -tilt * 0.3, jaw_open: 0.9,
        ..StickmanPose::neutral()
    }
}

pub fn pose_crescent_kick(t: f64) -> StickmanPose {
    let (tilt, hip, knee, arm) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (-5.0 * DEG * s, -10.0 * DEG * s, 60.0 * DEG * s, 5.0 * DEG * s)
    } else if t < 0.5 {
        let s = (t - 0.2) / 0.3;
        (-5.0 * DEG - 15.0 * DEG * s, -10.0 * DEG + 60.0 * DEG * s, 60.0 * DEG - 30.0 * DEG * s, 5.0 * DEG + 20.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.5) / 0.5);
        (-20.0 * DEG * (1.0 - s) + 5.0 * DEG * s, 50.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s), 25.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt,         pelvis: (t * 5.0).clamp(-1.0, 1.0).asin() * 20.0 * DEG,
        hip_r: hip, knee_r: knee, ankle_r: -20.0 * DEG, toe_r: 25.0 * DEG,
        hip_l: -hip * 0.4, knee_l: 10.0 * DEG,
        shoulder_l: -20.0 * DEG - arm, shoulder_r: 20.0 * DEG + arm,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG, neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_double_kick(t: f64) -> StickmanPose {
    let first = (t * 2.0).min(1.0);
    let second = ((t - 0.5) * 2.0).clamp(0.0, 1.0);
    let active = if t < 0.5 { first } else { second };
    let p = pose_front_kick(active);
    StickmanPose { pos_x: t * 0.4, jaw_open: 0.8, ..p }
}

pub fn pose_knee_strike(t: f64) -> StickmanPose {
    let (tilt, hip, knee, arm) = if t < 0.15 {
        let s = ease_in_out(t / 0.15);
        (-8.0 * DEG * s, 10.0 * DEG * s, 40.0 * DEG * s, -5.0 * DEG * s)
    } else if t < 0.35 {
        let s = (t - 0.15) / 0.2;
        (-8.0 * DEG - 15.0 * DEG * s, 10.0 * DEG + 40.0 * DEG * s, 40.0 * DEG + 50.0 * DEG * s, -5.0 * DEG + 40.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.35) / 0.65);
        (-23.0 * DEG * (1.0 - s), 50.0 * DEG * (1.0 - s), 90.0 * DEG * (1.0 - s), 35.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: -0.03 * t.min(1.0),
        hip_r: hip, knee_r: knee, ankle_r: -30.0 * DEG,
        hip_l: -hip * 0.3, knee_l: 20.0 * DEG,
        shoulder_r: -50.0 * DEG - arm, elbow_r: 20.0 * DEG,
        shoulder_l: 20.0 * DEG * (1.0 - t.min(1.0)),
        neck: -tilt * 0.3, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 10. PUNCHES & STRIKES — Variasi Pukulan Lengkap
// ═══════════════════════════════════════════════════════════════

pub fn pose_haymaker(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow, hip) = if t < 0.3 {
        let s = ease_in_out(t / 0.3);
        (-5.0 * DEG * s, 60.0 * DEG * s, 90.0 * DEG * s, 15.0 * DEG * s)
    } else if t < 0.55 {
        let s = (t - 0.3) / 0.25;
        (-5.0 * DEG - 20.0 * DEG * s, 60.0 * DEG - 120.0 * DEG * s, 90.0 * DEG - 90.0 * DEG * s, 15.0 * DEG - 25.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.55) / 0.45);
        (-25.0 * DEG * (1.0 - s), -60.0 * DEG * (1.0 - s), 0.0, -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow, wrist_r: -10.0 * DEG,
        hip_l: hip, hip_r: -hip * 0.5,
        shoulder_l: 25.0 * DEG, elbow_l: 30.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 10.0 * DEG,
        neck: -tilt * 0.4, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_body_blow(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (10.0 * DEG * s, 10.0 * DEG * s, 60.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (10.0 * DEG + 10.0 * DEG * s, 10.0 * DEG - 50.0 * DEG * s, 60.0 * DEG - 60.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (20.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s), 0.0)
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow,
        hip_l: 10.0 * DEG * t.min(1.0), hip_r: -5.0 * DEG * t.min(1.0),
        shoulder_l: 10.0 * DEG, elbow_l: 15.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_elbow_strike(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow) = if t < 0.12 {
        let s = ease_in_out(t / 0.12);
        (5.0 * DEG * s, 20.0 * DEG * s, 100.0 * DEG * s)
    } else if t < 0.3 {
        let s = (t - 0.12) / 0.18;
        (5.0 * DEG + 15.0 * DEG * s, 20.0 * DEG - 60.0 * DEG * s, 100.0 * DEG - 30.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.3) / 0.7);
        (20.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s), 70.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow,
        hip_l: 8.0 * DEG, hip_r: -4.0 * DEG,
        shoulder_l: 5.0 * DEG, elbow_l: 10.0 * DEG,
        neck: -tilt * 0.3, jaw_open: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_backfist(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (-5.0 * DEG * s, 60.0 * DEG * s, 60.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (-5.0 * DEG - 10.0 * DEG * s, 60.0 * DEG - 100.0 * DEG * s, 60.0 * DEG - 70.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (-15.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow, wrist_r: 20.0 * DEG,
        finger_r: 50.0 * DEG,
        shoulder_l: 10.0 * DEG, elbow_l: 15.0 * DEG,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_palm_strike(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (5.0 * DEG * s, -10.0 * DEG * s, 50.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (5.0 * DEG + 10.0 * DEG * s, -10.0 * DEG - 50.0 * DEG * s, 50.0 * DEG - 60.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (15.0 * DEG * (1.0 - s), -60.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow, wrist_r: -5.0 * DEG,
        finger_r: 0.0, finger_shape: 0.1,
        shoulder_l: 10.0 * DEG, elbow_l: 15.0 * DEG,
        hip_l: 8.0 * DEG, hip_r: -4.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_hammer_fist(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow) = if t < 0.25 {
        let s = ease_in_out(t / 0.25);
        (10.0 * DEG * s, 50.0 * DEG * s, 100.0 * DEG * s)
    } else if t < 0.5 {
        let s = (t - 0.25) / 0.25;
        (10.0 * DEG + 20.0 * DEG * s, 50.0 * DEG - 90.0 * DEG * s, 100.0 * DEG - 110.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.5) / 0.5);
        (30.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow,
        wrist_r: -20.0 * DEG, finger_r: 50.0 * DEG,
        hip_r: 10.0 * DEG * t.min(1.0),
        shoulder_l: 30.0 * DEG, elbow_l: 30.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 15.0 * DEG,
        neck: -tilt * 0.3, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 11. GRABS & THROWS — Tangkapan & Bantingan
// ═══════════════════════════════════════════════════════════════

pub fn pose_grab_headlock(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -10.0 * DEG, body_y: -0.02,
        shoulder_l: -80.0 * DEG, shoulder_r: 60.0 * DEG,
        elbow_l: -30.0 * DEG, elbow_r: 30.0 * DEG,
        wrist_l: 10.0 * DEG, wrist_r: -10.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: 5.0 * DEG, head_turn: -10.0 * DEG,
        eyebrow: 0.7, mouth: 0.8, jaw_open: 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_body_slam(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, arm_l, arm_r) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-10.0 * DEG * s, -0.05 * s, -40.0 * DEG * s, 40.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (-20.0 * DEG * s, -0.08 * s, -60.0 * DEG * s, 60.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (-20.0 * DEG - 10.0 * DEG * s, -0.08 + 0.06 * s, -60.0 * DEG + 30.0 * DEG * s, 60.0 * DEG - 30.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.4,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 60.0 * DEG, elbow_r: 60.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 20.0 * DEG, hip_r: -20.0 * DEG,
        knee_l: 40.0 * DEG, knee_r: 40.0 * DEG,
        neck: -tilt * 0.3, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_suplex(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, arm_l, arm_r, knee) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (5.0 * DEG * s, -0.03 * s, -30.0 * DEG * s, 30.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (5.0 * DEG + 30.0 * DEG * s, -0.03 - 0.10 * s, -50.0 * DEG * s, 50.0 * DEG * s, 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (35.0 * DEG - 55.0 * DEG * s, -0.13 + 0.10 * s, -50.0 * DEG + 70.0 * DEG * s, 50.0 * DEG - 70.0 * DEG * s, 40.0 * DEG + 30.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, body_y: body,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 20.0 * DEG, hip_r: -20.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.3, jaw_open: 0.9,
        ..StickmanPose::neutral()
    }
}

pub fn pose_hip_throw(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, pelvis_t, arm_l, arm_r, hip_t, knee) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (5.0 * DEG * s, 10.0 * DEG * s, -40.0 * DEG * s, 40.0 * DEG * s, 10.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (5.0 * DEG + 25.0 * DEG * s, 10.0 * DEG + 30.0 * DEG * s, -60.0 * DEG * s, 60.0 * DEG * s, 10.0 * DEG + 20.0 * DEG * s, 20.0 * DEG + 30.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (30.0 * DEG - 40.0 * DEG * s, 40.0 * DEG - 30.0 * DEG * s, -60.0 * DEG + 80.0 * DEG * s, 60.0 * DEG - 80.0 * DEG * s, 30.0 * DEG - 20.0 * DEG * s, 50.0 * DEG - 30.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, pelvis: pelvis_t,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: hip_t, hip_r: -hip_t,
        knee_l: knee, knee_r: knee * 0.7,
        neck: -tilt * 0.3, jaw_open: 0.8,
        ..StickmanPose::neutral()
    }
}

pub fn pose_choke_hold(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -8.0 * DEG,
        shoulder_l: -60.0 * DEG, shoulder_r: 60.0 * DEG,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        wrist_l: -20.0 * DEG, wrist_r: 20.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: 5.0 * DEG, eyebrow: 0.9, mouth: 1.0, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_throw_push(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow, hip, knee) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (-5.0 * DEG * s, -10.0 * DEG * s, 40.0 * DEG * s, -5.0 * DEG * s, 10.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (-5.0 * DEG - 20.0 * DEG * s, -10.0 * DEG - 40.0 * DEG * s, 40.0 * DEG - 50.0 * DEG * s, -5.0 * DEG + 15.0 * DEG * s, 10.0 * DEG + 15.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (-25.0 * DEG * (1.0 - s), -50.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), 25.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, pos_x: t * 0.3,
        shoulder_r: shoulder, elbow_r: elbow, wrist_r: -10.0 * DEG,
        finger_r: 60.0 * DEG,
        shoulder_l: -20.0 * DEG, elbow_l: 20.0 * DEG,
        finger_l: 60.0 * DEG,
        hip_l: hip, hip_r: -hip,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.4, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_clothesline(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, arm_r, hip_r, knee_r) = if phase < 0.2 {
        let s = ease_in_out(phase / 0.2);
        (-5.0 * DEG * s, 10.0 * DEG * s, -5.0 * DEG * s, 10.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.2) / 0.8);
        (-5.0 * DEG - 15.0 * DEG * s, 10.0 * DEG + 50.0 * DEG * s, -5.0 * DEG - 15.0 * DEG * s, 10.0 * DEG + 5.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, pos_x: t * 0.5,
        shoulder_r: -80.0 * DEG + arm_r, elbow_r: 10.0 * DEG,
        wrist_r: -10.0 * DEG, finger_r: 40.0 * DEG,
        shoulder_l: -20.0 * DEG, elbow_l: 20.0 * DEG,
        hip_r: hip_r, knee_r: knee_r,
        hip_l: 10.0 * DEG, knee_l: 15.0 * DEG,
        neck: -tilt * 0.3, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_leg_sweep(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, hip, knee) = if phase < 0.2 {
        let s = ease_in_out(phase / 0.2);
        (15.0 * DEG * s, -0.03 * s, -10.0 * DEG * s, 40.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.2) / 0.3;
        (15.0 * DEG + 20.0 * DEG * s, -0.03 * s, -10.0 * DEG + 50.0 * DEG * s, 40.0 * DEG - 30.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.5) / 0.5);
        (35.0 * DEG * (1.0 - s), -0.03 * (1.0 - s), 40.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.3,
        hip_r: hip, knee_r: knee, ankle_r: -30.0 * DEG,
        hip_l: -hip * 0.3, knee_l: 30.0 * DEG,
        shoulder_l: 20.0 * DEG, shoulder_r: -20.0 * DEG,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 12. DEFENSE — Pertahanan
// ═══════════════════════════════════════════════════════════════

pub fn pose_block_high(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -5.0 * DEG, body_y: -0.01,
        shoulder_l: -60.0 * DEG, shoulder_r: 60.0 * DEG,
        elbow_l: 130.0 * DEG, elbow_r: 130.0 * DEG,
        wrist_l: 30.0 * DEG, wrist_r: -30.0 * DEG,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        hip_l: -3.0 * DEG, hip_r: 3.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: 10.0 * DEG, eye_squint: -0.7, eyebrow: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_block_mid(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -8.0 * DEG, body_y: -0.02,
        shoulder_l: -30.0 * DEG, shoulder_r: 30.0 * DEG,
        elbow_l: 90.0 * DEG, elbow_r: 90.0 * DEG,
        wrist_l: 10.0 * DEG, wrist_r: -10.0 * DEG,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: 5.0 * DEG, eye_squint: -0.6, eyebrow: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_block_low(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: 15.0 * DEG, body_y: -0.06,
        shoulder_l: 10.0 * DEG, shoulder_r: -10.0 * DEG,
        elbow_l: -60.0 * DEG, elbow_r: 60.0 * DEG,
        wrist_l: 0.0, wrist_r: 0.0,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: 40.0 * DEG, knee_r: 40.0 * DEG,
        neck: 15.0 * DEG, eye_squint: -0.5, eyebrow: 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_parry(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, shoulder, elbow, wrist) = if phase < 0.15 {
        let s = ease_in_out(phase / 0.15);
        (5.0 * DEG * s, 20.0 * DEG * s, 70.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.35 {
        let s = (phase - 0.15) / 0.2;
        (5.0 * DEG + 10.0 * DEG * s, 20.0 * DEG - 50.0 * DEG * s, 70.0 * DEG - 40.0 * DEG * s, 20.0 * DEG - 30.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.35) / 0.65);
        (15.0 * DEG * (1.0 - s), -30.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow, wrist_r: wrist,
        finger_r: 60.0 * DEG,
        shoulder_l: -10.0 * DEG, elbow_l: 10.0 * DEG,
        neck: -tilt * 0.3, eye_squint: -0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_weave(t: f64) -> StickmanPose {
    let sway = (t * 4.0).sin() * 10.0 * DEG;
    StickmanPose {
        body_tilt: sway, pos_x: (t * 4.0).sin() * 0.08,
        shoulder_l: 10.0 * DEG, shoulder_r: -10.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: sway * 0.5, hip_r: -sway * 0.5,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: sway * 0.5, eye_squint: -0.6, eyebrow: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_step_back(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, pos, arm_l, arm_r) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (10.0 * DEG * s, -0.02 * s, 10.0 * DEG * s, -10.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (10.0 * DEG * (1.0 - s), -0.02 - 0.04 * s, 10.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (0.0, -0.06 * (1.0 - s), 0.0, 0.0)
    };
    StickmanPose {
        body_tilt: tilt, pos_x: -t * 0.3 + pos,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: tilt * 0.3, eye_squint: -0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_cross_block(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -5.0 * DEG, body_y: -0.03,
        shoulder_l: -50.0 * DEG, shoulder_r: 50.0 * DEG,
        elbow_l: 100.0 * DEG, elbow_r: 100.0 * DEG,
        wrist_l: -20.0 * DEG, wrist_r: 20.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: 10.0 * DEG, head_turn: -10.0 * DEG,
        eye_squint: -0.8, eyebrow: 0.7,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 13. WEAPONS — Senjata
// ═══════════════════════════════════════════════════════════════

pub fn pose_draw_weapon(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, shoulder_r, elbow_r, head) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (5.0 * DEG * s, 40.0 * DEG * s, 90.0 * DEG * s, -5.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (5.0 * DEG + 5.0 * DEG * s, 40.0 * DEG - 80.0 * DEG * s, 90.0 * DEG - 70.0 * DEG * s, -5.0 * DEG - 5.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (10.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s), 20.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder_r, elbow_r: elbow_r, wrist_r: -30.0 * DEG,
        finger_r: 50.0 * DEG,
        shoulder_l: 5.0 * DEG, elbow_l: 10.0 * DEG,
        neck: head, eye_squint: -0.5, eyebrow: 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_holster(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, shoulder_r, elbow_r, wrist_r) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-3.0 * DEG * s, -40.0 * DEG * s, 20.0 * DEG * s, -20.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (-3.0 * DEG * (1.0 - s), -40.0 * DEG + 60.0 * DEG * s, 20.0 * DEG + 50.0 * DEG * s, -20.0 * DEG * (1.0 - s))
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (0.0, 20.0 * DEG * (1.0 - s), 70.0 * DEG * (1.0 - s), -20.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder_r, elbow_r: elbow_r, wrist_r: wrist_r,
        finger_r: 20.0 * DEG,
        shoulder_l: 10.0 * DEG, elbow_l: 10.0 * DEG,
        neck: -3.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_melee_swing(t: f64) -> StickmanPose {
    let (tilt, shoulder_r, elbow_r, hip, knee) = if t < 0.25 {
        let s = ease_in_out(t / 0.25);
        (-5.0 * DEG * s, 70.0 * DEG * s, 100.0 * DEG * s, 15.0 * DEG * s, 15.0 * DEG * s)
    } else if t < 0.55 {
        let s = (t - 0.25) / 0.3;
        (-5.0 * DEG - 20.0 * DEG * s, 70.0 * DEG - 140.0 * DEG * s, 100.0 * DEG - 90.0 * DEG * s, 15.0 * DEG - 25.0 * DEG * s, 15.0 * DEG - 5.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.55) / 0.45);
        (-25.0 * DEG * (1.0 - s), -70.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder_r, elbow_r: elbow_r,
        wrist_r: -30.0 * DEG, finger_r: 60.0 * DEG,
        shoulder_l: 30.0 * DEG, elbow_l: 15.0 * DEG,
        hip_l: hip, hip_r: -hip,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.3, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_melee_stab(t: f64) -> StickmanPose {
    let (tilt, shoulder_r, elbow_r) = if t < 0.2 {
        let s = ease_in_out(t / 0.2);
        (-5.0 * DEG * s, 30.0 * DEG * s, 90.0 * DEG * s)
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (-5.0 * DEG - 15.0 * DEG * s, 30.0 * DEG - 70.0 * DEG * s, 90.0 * DEG - 100.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.4) / 0.6);
        (-20.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, pos_x: t * 0.3,
        shoulder_r: shoulder_r, elbow_r: elbow_r, wrist_r: -20.0 * DEG,
        finger_r: 60.0 * DEG,
        shoulder_l: 10.0 * DEG, elbow_l: 15.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: -tilt * 0.3, eye_squint: -0.7, eyebrow: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_throw_weapon(t: f64) -> StickmanPose {
    let (tilt, shoulder, elbow, wrist, finger) = if t < 0.3 {
        let s = ease_in_out(t / 0.3);
        (-15.0 * DEG * s, 50.0 * DEG * s, -80.0 * DEG * s, -20.0 * DEG * s, 20.0 * DEG * s)
    } else if t < 0.5 {
        let s = (t - 0.3) / 0.2;
        (-15.0 * DEG - 20.0 * DEG * s, 50.0 * DEG - 110.0 * DEG * s, -80.0 * DEG + 100.0 * DEG * s, -20.0 * DEG + 30.0 * DEG * s, 20.0 * DEG + 30.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.5) / 0.5);
        (-35.0 * DEG * (1.0 - s), -60.0 * DEG * (1.0 - s), 20.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s), 50.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, shoulder_r: shoulder, elbow_r: elbow, wrist_r: wrist,
        finger_r: finger,
        shoulder_l: 10.0 * DEG, elbow_l: 20.0 * DEG,
        hip_l: 15.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 10.0 * DEG,
        neck: -tilt * 0.4, jaw_open: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_weapon_block(t: f64) -> StickmanPose {
    StickmanPose {
        body_tilt: -10.0 * DEG,
        shoulder_l: -70.0 * DEG, shoulder_r: 70.0 * DEG,
        elbow_l: 80.0 * DEG, elbow_r: 80.0 * DEG,
        wrist_l: -20.0 * DEG, wrist_r: 20.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: 20.0 * DEG, knee_r: 20.0 * DEG,
        neck: 8.0 * DEG, eye_squint: -0.8, eyebrow: 0.6,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 14. VEHICLES — Kendaraan (Masuk, Keluar, Mengemudi)
// ═══════════════════════════════════════════════════════════════

pub fn pose_enter_car_driver(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, hip, knee, arm_l, arm_r) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (10.0 * DEG * s, -0.05 * s, -10.0 * DEG * s, 30.0 * DEG * s, -20.0 * DEG * s, -60.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (10.0 * DEG + 10.0 * DEG * s, -0.05 * s, -10.0 * DEG + 30.0 * DEG * s, 30.0 * DEG + 40.0 * DEG * s, -20.0 * DEG + 40.0 * DEG * s, -60.0 * DEG + 30.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (20.0 * DEG * (1.0 - s), -0.10 * (1.0 - s), 20.0 * DEG * (1.0 - s) - 15.0 * DEG * s, 70.0 * DEG * (1.0 - s) + 15.0 * DEG * s, 20.0 * DEG * (1.0 - s) - 10.0 * DEG * s, -30.0 * DEG * (1.0 - s) + 30.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.3,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 20.0 * DEG, elbow_r: 50.0 * DEG,
        wrist_l: 5.0 * DEG, wrist_r: 10.0 * DEG,
        finger_l: 30.0 * DEG, finger_r: 40.0 * DEG,
        hip_l: hip, hip_r: -hip * 0.5,
        knee_l: knee, knee_r: knee * 0.6,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_exit_car_driver(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, hip, knee, arm_l, arm_r) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-5.0 * DEG * s, -0.08 * (1.0 - s), 5.0 * DEG * s, 30.0 * DEG * s, -30.0 * DEG * s, 10.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (-5.0 * DEG - 10.0 * DEG * s, -0.04 * s, 5.0 * DEG + 20.0 * DEG * s, 30.0 * DEG + 30.0 * DEG * s, -30.0 * DEG - 20.0 * DEG * s, 10.0 * DEG + 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (-15.0 * DEG * (1.0 - s), -0.04 * (1.0 - s), 25.0 * DEG * (1.0 - s), 60.0 * DEG * (1.0 - s), -50.0 * DEG * (1.0 - s), 50.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.4,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 30.0 * DEG, elbow_r: 20.0 * DEG,
        wrist_l: -10.0 * DEG, wrist_r: -5.0 * DEG,
        hip_l: hip, hip_r: -hip * 0.5,
        knee_l: knee, knee_r: knee * 0.7,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_enter_car_passenger(t: f64) -> StickmanPose {
    let mut p = pose_enter_car_driver(t);
    p.shoulder_l = -p.shoulder_l;
    p.shoulder_r = -p.shoulder_r;
    p.head_turn = -10.0 * DEG;
    p
}

pub fn pose_exit_car_passenger(t: f64) -> StickmanPose {
    let mut p = pose_exit_car_driver(t);
    p.shoulder_l = -p.shoulder_l;
    p.shoulder_r = -p.shoulder_r;
    p.head_turn = 10.0 * DEG;
    p
}

pub fn pose_drive(t: f64) -> StickmanPose {
    let steer = (t * 2.0).sin() * 5.0 * DEG;
    let bounce = (t * 1.5).sin().abs() * 0.01;
    StickmanPose {
        body_y: -0.15 + bounce, body_tilt: 3.0 * DEG + steer * 0.3,
        pelvis: 15.0 * DEG,
        shoulder_l: -30.0 * DEG, shoulder_r: 30.0 * DEG,
        elbow_l: 60.0 * DEG, elbow_r: -60.0 * DEG,
        wrist_l: 10.0 * DEG + steer, wrist_r: -10.0 * DEG - steer,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        hip_l: -15.0 * DEG, hip_r: 15.0 * DEG,
        knee_l: 90.0 * DEG, knee_r: 90.0 * DEG,
        ankle_l: -25.0 * DEG, ankle_r: -25.0 * DEG,
        neck: 3.0 * DEG, head_turn: steer * 2.0,
        ..StickmanPose::neutral()
    }
}

pub fn pose_ride_motorcycle(t: f64) -> StickmanPose {
    let lean = (t * 1.2).sin() * 3.0 * DEG;
    StickmanPose {
        body_y: -0.12, body_tilt: -15.0 * DEG + lean,
        spine_upper: -10.0 * DEG, spine_lower: -5.0 * DEG,
        shoulder_l: -50.0 * DEG, shoulder_r: 50.0 * DEG,
        elbow_l: 40.0 * DEG, elbow_r: -40.0 * DEG,
        wrist_l: 20.0 * DEG, wrist_r: -20.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 100.0 * DEG, knee_r: 100.0 * DEG,
        ankle_l: -30.0 * DEG, ankle_r: -30.0 * DEG,
        neck: -10.0 * DEG + lean * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_dismount_motorcycle(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, knee, arm_l, arm_r) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (15.0 * DEG * s, 90.0 * DEG * (1.0 - s) + 30.0 * DEG * s, -30.0 * DEG * s, 30.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (15.0 * DEG * (1.0 - s), 30.0 * DEG * (1.0 - s) + 40.0 * DEG * s, -30.0 * DEG - 20.0 * DEG * s, 30.0 * DEG + 20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (0.0, 40.0 * DEG * (1.0 - s), -50.0 * DEG * (1.0 - s), 50.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: -(phase * 0.06).min(0.06), pos_x: t * 0.3,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: knee, knee_r: knee * 0.8,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_enter_helicopter(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, knee) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (30.0 * DEG * s, -0.08 * s, 50.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (30.0 * DEG * (1.0 - s) + 15.0 * DEG * s, -0.08 * (1.0 - s) - 0.06 * s, 50.0 * DEG * (1.0 - s) + 30.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (15.0 * DEG * (1.0 - s), -0.14 * (1.0 - s), 30.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.2,
        shoulder_l: 10.0 * DEG, shoulder_r: -10.0 * DEG,
        elbow_l: 40.0 * DEG, elbow_r: 40.0 * DEG,
        hip_l: -10.0 * DEG, hip_r: 10.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_exit_helicopter(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, knee) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (15.0 * DEG * s, -0.10 * (1.0 - s), 30.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (15.0 * DEG + 15.0 * DEG * s, -0.04 + 0.04 * s, 30.0 * DEG + 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (30.0 * DEG * (1.0 - s), 0.0, 70.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.3,
        shoulder_l: -10.0 * DEG, shoulder_r: 10.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.4,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 15. ACROBATIC EXTENDED — Akrobatik Lanjutan
// ═══════════════════════════════════════════════════════════════

pub fn pose_salto_forward(t: f64) -> StickmanPose {
    let spin = t * 360.0 * DEG;
    let height = (t * std::f64::consts::PI).sin() * 0.25;
    let tuck = 1.0 - (t * 2.0 - 1.0).powi(2);
    StickmanPose {
        body_tilt: spin, body_y: height,
        shoulder_l: -30.0 * DEG * tuck, shoulder_r: 30.0 * DEG * tuck,
        elbow_l: 110.0 * DEG * tuck, elbow_r: 110.0 * DEG * tuck,
        hip_l: -25.0 * DEG * tuck, hip_r: 25.0 * DEG * tuck,
        knee_l: 110.0 * DEG * tuck, knee_r: 110.0 * DEG * tuck,
        neck: -25.0 * DEG * tuck, pos_x: t * 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_salto_backward(t: f64) -> StickmanPose {
    let spin = -t * 360.0 * DEG;
    let height = (t * std::f64::consts::PI).sin() * 0.25;
    let tuck = 1.0 - (t * 2.0 - 1.0).powi(2);
    StickmanPose {
        body_tilt: spin, body_y: height,
        shoulder_l: 30.0 * DEG * tuck, shoulder_r: -30.0 * DEG * tuck,
        elbow_l: 110.0 * DEG * tuck, elbow_r: 110.0 * DEG * tuck,
        hip_l: 25.0 * DEG * tuck, hip_r: -25.0 * DEG * tuck,
        knee_l: 110.0 * DEG * tuck, knee_r: 110.0 * DEG * tuck,
        neck: 25.0 * DEG * tuck, pos_x: -t * 0.2,
        ..StickmanPose::neutral()
    }
}

pub fn pose_aerial_cartwheel(t: f64) -> StickmanPose {
    let spin = t * 360.0 * DEG;
    let height = (t * std::f64::consts::PI).sin() * 0.22;
    StickmanPose {
        body_tilt: spin, body_y: height,
        shoulder_l: -100.0 * DEG, shoulder_r: 100.0 * DEG,
        elbow_l: -20.0 * DEG, elbow_r: 20.0 * DEG,
        hip_l: -35.0 * DEG, hip_r: 35.0 * DEG,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: -15.0 * DEG, pos_x: t * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_back_handspring(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, arm_l, arm_r, knee) = if phase < 0.25 {
        let s = ease_in_out(phase / 0.25);
        (-10.0 * DEG * s, -0.03 * s, -20.0 * DEG * s, 20.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.25) / 0.25;
        (-10.0 * DEG - 50.0 * DEG * s, -0.03 + 0.15 * s, -20.0 * DEG + 60.0 * DEG * s, 20.0 * DEG - 60.0 * DEG * s, 20.0 * DEG + 40.0 * DEG * s)
    } else if phase < 0.75 {
        let s = (phase - 0.5) / 0.25;
        (-60.0 * DEG + 120.0 * DEG * s, 0.12 * (1.0 - s), 40.0 * DEG - 80.0 * DEG * s, -40.0 * DEG + 80.0 * DEG * s, 60.0 * DEG - 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.75) / 0.25);
        (60.0 * DEG * (1.0 - s), 0.0, -40.0 * DEG * (1.0 - s), 40.0 * DEG * (1.0 - s), 20.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.5,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: -10.0 * DEG, elbow_r: -10.0 * DEG,
        wrist_l: -20.0 * DEG, wrist_r: 20.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 15.0 * DEG, hip_r: -15.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_front_handspring(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, arm_l, arm_r, knee) = if phase < 0.2 {
        let s = ease_in_out(phase / 0.2);
        (15.0 * DEG * s, 0.0, 30.0 * DEG * s, -30.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.2) / 0.3;
        (15.0 * DEG + 75.0 * DEG * s, 0.15 * s, 30.0 * DEG + 60.0 * DEG * s, -30.0 * DEG - 60.0 * DEG * s, 20.0 * DEG + 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.5) / 0.5);
        (90.0 * DEG * (1.0 - s), 0.15 * (1.0 - s), 90.0 * DEG * (1.0 - s), -90.0 * DEG * (1.0 - s), 60.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.6,
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 0.0, elbow_r: 0.0,
        wrist_l: -20.0 * DEG, wrist_r: 20.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_dive_roll(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, hip, knee) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (30.0 * DEG * s, 0.0, 10.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (30.0 * DEG + 60.0 * DEG * s, 0.12 * s, 10.0 * DEG + 20.0 * DEG * s, 20.0 * DEG + 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (90.0 * DEG * (1.0 - s), 0.12 * (1.0 - s) + 0.08 * s, 30.0 * DEG * (1.0 - s), 60.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.7, pos_y: -0.08 * phase.min(1.0),
        shoulder_l: -40.0 * DEG, shoulder_r: 40.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: hip, hip_r: -hip,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_wall_flip(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, hip, knee) = if phase < 0.2 {
        let s = ease_in_out(phase / 0.2);
        (-15.0 * DEG * s, 0.0, -10.0 * DEG * s, 20.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.2) / 0.3;
        (-15.0 * DEG - 30.0 * DEG * s, 0.20 * s, -10.0 * DEG + 40.0 * DEG * s, 20.0 * DEG + 30.0 * DEG * s)
    } else if phase < 0.8 {
        let s = (phase - 0.5) / 0.3;
        (-45.0 * DEG + 135.0 * DEG * s, 0.20 * (1.0 - s), 30.0 * DEG - 50.0 * DEG * s, 50.0 * DEG - 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.8) / 0.2);
        (90.0 * DEG * (1.0 - s), 0.0, -20.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.4,
        shoulder_l: -50.0 * DEG, shoulder_r: 50.0 * DEG,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        hip_l: hip, hip_r: -hip,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.3, jaw_open: 0.8,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 16. ENVIRONMENT EXTENDED — Interaksi Lanjutan
// ═══════════════════════════════════════════════════════════════

pub fn pose_climb_wall(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.0 * std::f64::consts::PI;
    let pull = (cycle).sin().abs() * 40.0 * DEG;
    let step = (cycle + std::f64::consts::PI).sin() * 35.0 * DEG;
    StickmanPose {
        body_y: 0.10 + (cycle * 2.0).sin().abs() * 0.05, pos_y: t * 0.5,
        body_tilt: -5.0 * DEG,
        shoulder_l: -150.0 * DEG * (1.0 - pull / 40.0 * DEG),
        shoulder_r: 150.0 * DEG * (pull / 40.0 * DEG),
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: step, hip_r: -step,
        knee_l: 120.0 * DEG, knee_r: 120.0 * DEG,
        ankle_l: -20.0 * DEG, ankle_r: -20.0 * DEG,
        neck: -10.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_pull_up(t: f64) -> StickmanPose {
    let phase = t;
    let (body, shoulder, elbow) = if phase < 0.4 {
        let s = ease_in_out(phase / 0.4);
        (0.20 - 0.12 * s, -170.0 * DEG * (1.0 - s * 0.2), 10.0 * DEG * (1.0 - s * 0.3))
    } else if phase < 0.7 {
        let s = (phase - 0.4) / 0.3;
        (0.08 + 0.05 * s, -136.0 * DEG + 20.0 * DEG * s, 7.0 * DEG + 10.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.7) / 0.3);
        (0.13 + 0.07 * s, -116.0 * DEG - 34.0 * DEG * s, 17.0 * DEG - 17.0 * DEG * s)
    };
    StickmanPose {
        body_y: body, squash_y: 0.92 + 0.08 * (1.0 - phase.min(1.0)),
        shoulder_l: -shoulder, shoulder_r: shoulder,
        elbow_l: elbow, elbow_r: elbow,
        wrist_l: -10.0 * DEG, wrist_r: 10.0 * DEG,
        finger_l: 60.0 * DEG, finger_r: 60.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: -8.0 * DEG, jaw_open: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_hang_drop(t: f64) -> StickmanPose {
    let phase = t;
    let (body, tilt) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (0.25 - 0.10 * s, -5.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (0.15 + 0.10 * s, -5.0 * DEG + 15.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (0.25 * (1.0 - s), 10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_y: body, body_tilt: tilt, pos_y: -t * 0.15,
        shoulder_l: -170.0 * DEG, shoulder_r: 170.0 * DEG,
        elbow_l: 0.0, elbow_r: 0.0,
        finger_l: 50.0 * DEG, finger_r: 50.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: -tilt * 0.3, jaw_open: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_open_door(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, shoulder_r, elbow_r, body_pos) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (-5.0 * DEG * s, -40.0 * DEG * s, 50.0 * DEG * s, 0.0)
    } else {
        let s = ease_in_out((phase - 0.3) / 0.7);
        (-5.0 * DEG - 5.0 * DEG * s, -40.0 * DEG - 40.0 * DEG * s, 50.0 * DEG - 30.0 * DEG * s, 0.1 * s)
    };
    StickmanPose {
        body_tilt: tilt, pos_x: body_pos,
        shoulder_r: shoulder_r, elbow_r: elbow_r, wrist_r: 10.0 * DEG,
        finger_r: 50.0 * DEG,
        shoulder_l: 5.0 * DEG, elbow_l: 10.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_close_door(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, shoulder_r, elbow_r, body_pos) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (5.0 * DEG * s, -20.0 * DEG * s, 30.0 * DEG * s, 0.05 * s)
    } else {
        let s = ease_in_out((phase - 0.3) / 0.7);
        (5.0 * DEG - 10.0 * DEG * s, -20.0 * DEG + 20.0 * DEG * s, 30.0 * DEG + 10.0 * DEG * s, 0.05 * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, pos_x: body_pos,
        shoulder_r: shoulder_r, elbow_r: elbow_r, wrist_r: -10.0 * DEG,
        finger_r: 50.0 * DEG,
        shoulder_l: 5.0 * DEG, elbow_l: 10.0 * DEG,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_crawl_through(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.5 * std::f64::consts::PI;
    let arm = cycle.sin() * 40.0 * DEG;
    let leg = (cycle + std::f64::consts::PI).sin() * 45.0 * DEG;
    StickmanPose {
        body_y: -0.22, body_tilt: 80.0 * DEG, spine_upper: -20.0 * DEG,
        neck: -60.0 * DEG,
        shoulder_l: arm - 30.0 * DEG, shoulder_r: -arm - 30.0 * DEG,
        elbow_l: 100.0 * DEG, elbow_r: 100.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: 85.0 * DEG, knee_r: 85.0 * DEG,
        pos_x: t * 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_jump_over_obstacle(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, hip, knee, arm) = if phase < 0.25 {
        let s = ease_in_out(phase / 0.25);
        (-10.0 * DEG * s, -0.04 * s, -10.0 * DEG * s, 40.0 * DEG * s, -15.0 * DEG * s)
    } else if phase < 0.55 {
        let s = (phase - 0.25) / 0.3;
        (-10.0 * DEG - 15.0 * DEG * s, 0.20 * s, -10.0 * DEG + 30.0 * DEG * s, 40.0 * DEG + 20.0 * DEG * s, -15.0 * DEG - 25.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.55) / 0.45);
        (-25.0 * DEG * (1.0 - s), 0.20 * (1.0 - s), 20.0 * DEG * (1.0 - s), 60.0 * DEG * (1.0 - s), -40.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_x: t * 0.6,
        shoulder_l: arm - 10.0 * DEG, shoulder_r: -arm + 10.0 * DEG,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: hip, hip_r: -hip,
        knee_l: knee, knee_r: knee * 0.7,
        neck: -tilt * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_drop_from_height(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, knee, arm) = if phase < 0.25 {
        let s = ease_in_out(phase / 0.25);
        (-10.0 * DEG * s, 0.15 * s, 80.0 * DEG * s, -10.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.25) / 0.25;
        (-10.0 * DEG + 20.0 * DEG * s, 0.15 - 0.10 * s, 80.0 * DEG - 50.0 * DEG * s, -10.0 * DEG + 20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.5) / 0.5);
        (10.0 * DEG * (1.0 - s), 0.05 * (1.0 - s), 30.0 * DEG * (1.0 - s), 10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_y: -t * 0.20,
        squash_y: 0.75 + 0.25 * (1.0 - phase.min(1.0)),
        stretch_x: 1.15 - 0.15 * (1.0 - phase.min(1.0)),
        shoulder_l: arm - 20.0 * DEG, shoulder_r: -arm + 20.0 * DEG,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.3, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 17. GROUND & PRONE — Posisi Tanah & Tiarap
// ═══════════════════════════════════════════════════════════════

pub fn pose_prone_crawl(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.8 * std::f64::consts::PI;
    let pull = (cycle * 0.5).sin().abs() * 30.0 * DEG;
    let kick = ((cycle + std::f64::consts::PI) * 0.5).sin().abs() * 35.0 * DEG;
    StickmanPose {
        body_y: -0.01, body_tilt: -90.0 * DEG, pos_y: -0.18,
        spine_upper: -20.0 * DEG, neck: 50.0 * DEG,
        shoulder_l: pull - 30.0 * DEG, shoulder_r: -pull - 30.0 * DEG,
        elbow_l: 100.0 * DEG * (pull / 30.0 * DEG),
        elbow_r: 100.0 * DEG * (pull / 30.0 * DEG),
        hip_l: 10.0 * DEG + kick, hip_r: -10.0 * DEG - kick,
        knee_l: 40.0 * DEG, knee_r: 40.0 * DEG,
        pos_x: t * 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_crouch_walk(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.5 * std::f64::consts::PI;
    let leg = cycle.sin() * 25.0 * DEG;
    StickmanPose {
        body_y: -0.15, body_tilt: 35.0 * DEG,
        shoulder_l: 15.0 * DEG, shoulder_r: 15.0 * DEG,
        elbow_l: 60.0 * DEG, elbow_r: 60.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: 70.0 * DEG, knee_r: 70.0 * DEG,
        neck: 10.0 * DEG, pos_x: t * 0.3,
        eye_squint: 0.3, eyebrow: 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_ground_sit(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: -0.28, body_tilt: 5.0 * DEG,
        pelvis: 35.0 * DEG,
        hip_l: -45.0 * DEG, hip_r: 45.0 * DEG,
        knee_l: 50.0 * DEG, knee_r: -50.0 * DEG,
        ankle_l: 15.0 * DEG, ankle_r: -15.0 * DEG,
        shoulder_l: 5.0 * DEG, shoulder_r: -5.0 * DEG,
        elbow_l: 15.0 * DEG, elbow_r: 15.0 * DEG,
        neck: 3.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_ground_recover(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, pos_y_val, knee) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (90.0 * DEG - 30.0 * DEG * s, -0.12 + 0.04 * s, -0.15 + 0.05 * s, 20.0 * DEG * s)
    } else if phase < 0.6 {
        let s = (phase - 0.3) / 0.3;
        (60.0 * DEG - 40.0 * DEG * s, -0.08 + 0.03 * s, -0.10 + 0.05 * s, 20.0 * DEG + 40.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (20.0 * DEG - 20.0 * DEG * s, -0.05 + 0.05 * s, -0.05 + 0.05 * s, 60.0 * DEG - 30.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_y: pos_y_val,
        shoulder_l: 30.0 * DEG * (1.0 - phase.min(0.5) / 0.5),
        shoulder_r: -30.0 * DEG * (1.0 - phase.min(0.5) / 0.5),
        elbow_l: 40.0 * DEG, elbow_r: 40.0 * DEG,
        hip_l: 10.0 * DEG, hip_r: -10.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: tilt * 0.2, eyebrow: -0.3, mouth: -0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_limp(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.5 * std::f64::consts::PI;
    let leg = cycle.sin() * 20.0 * DEG;
    let bounce = (cycle * 2.0).sin().abs() * 0.05;
    StickmanPose {
        body_y: -0.03 + bounce, body_tilt: 8.0 * DEG,
        shoulder_l: 10.0 * DEG, shoulder_r: -30.0 * DEG,
        elbow_l: 15.0 * DEG, elbow_r: 50.0 * DEG,
        hip_l: leg, hip_r: -leg * 0.5,
        knee_l: 25.0 * DEG, knee_r: 10.0 * DEG,
        ankle_r: -10.0 * DEG,
        neck: 10.0 * DEG, pos_x: t * 0.2,
        eyebrow: -0.5, mouth: -0.6, eye_squint: -0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_stagger_back(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 2.0 * std::f64::consts::PI;
    let wobble = cycle.sin() * 12.0 * DEG;
    StickmanPose {
        body_tilt: 15.0 * DEG + wobble,
        body_y: -0.02,
        shoulder_l: 20.0 * DEG + wobble, shoulder_r: -20.0 * DEG - wobble,
        elbow_l: 40.0 * DEG, elbow_r: 40.0 * DEG,
        hip_l: wobble * 0.5, hip_r: -wobble * 0.5,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        neck: 10.0 * DEG + wobble * 0.3,
        head_turn: wobble * 0.5, pos_x: -t * 0.2,
        eye_blink: 0.4, mouth: -0.5, jaw_open: 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_trip_and_fall(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, body, arm_l, arm_r, knee) = if phase < 0.2 {
        let s = ease_in_out(phase / 0.2);
        (15.0 * DEG * s, 0.0, -10.0 * DEG * s, 20.0 * DEG * s, 5.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.2) / 0.3;
        (15.0 * DEG + 35.0 * DEG * s, -0.04 * s, -10.0 * DEG - 40.0 * DEG * s, 20.0 * DEG + 30.0 * DEG * s, 5.0 * DEG + 20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.5) / 0.5);
        (50.0 * DEG + 40.0 * DEG * s, -0.04 - 0.08 * s, -50.0 * DEG - 20.0 * DEG * s, 50.0 * DEG + 20.0 * DEG * s, 25.0 * DEG + 10.0 * DEG * s)
    };
    StickmanPose {
        body_tilt: tilt, body_y: body, pos_y: -0.05 * phase.min(1.0),
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG,
        hip_l: 15.0 * DEG, hip_r: -15.0 * DEG,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.3, jaw_open: 0.8, eye_blink: 0.9, eyebrow: 0.9,
        ..StickmanPose::neutral()
    }
}

pub fn pose_slip(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, arm_l, arm_r, knee, leg) = if phase < 0.2 {
        let s = ease_in_out(phase / 0.2);
        (15.0 * DEG * s, -40.0 * DEG * s, 40.0 * DEG * s, 10.0 * DEG * s, 10.0 * DEG * s)
    } else if phase < 0.5 {
        let s = (phase - 0.2) / 0.3;
        (15.0 * DEG + 30.0 * DEG * s, -40.0 * DEG - 20.0 * DEG * s, 40.0 * DEG + 20.0 * DEG * s, 10.0 * DEG + 30.0 * DEG * s, 10.0 * DEG - 20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.5) / 0.5);
        (45.0 * DEG * (1.0 - s), -60.0 * DEG * (1.0 - s), 60.0 * DEG * (1.0 - s), 40.0 * DEG * (1.0 - s), -10.0 * DEG * (1.0 - s))
    };
    StickmanPose {
        body_tilt: tilt, body_y: -(phase * 0.06).min(0.06), pos_y: -0.03 * phase.min(1.0),
        shoulder_l: arm_l, shoulder_r: arm_r,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        hip_l: leg, hip_r: -leg,
        knee_l: knee, knee_r: knee,
        neck: -tilt * 0.4, jaw_open: 0.9, eye_blink: 0.9, eyebrow: 0.8,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 18. EMOTIONAL EXTENDED — Ekspresi Lanjutan
// ═══════════════════════════════════════════════════════════════

pub fn pose_celebrate_arms_up(t: f64) -> StickmanPose {
    let jump = (t * 3.0).sin().abs() * 0.06;
    StickmanPose {
        body_y: jump, body_tilt: -8.0 * DEG,
        shoulder_l: -170.0 * DEG, shoulder_r: 170.0 * DEG,
        elbow_l: -20.0 * DEG, elbow_r: -20.0 * DEG,
        wrist_l: 30.0 * DEG, wrist_r: -30.0 * DEG,
        finger_l: 80.0 * DEG, finger_r: 80.0 * DEG,
        neck: -15.0 * DEG, mouth: 1.0, eyebrow: 1.0, jaw_open: 0.7,
        ..StickmanPose::neutral()
    }
}

pub fn pose_celebrate_fist_pump(t: f64) -> StickmanPose {
    let pump = ((t * 4.0).sin() * 0.5 + 0.5) * 40.0 * DEG;
    let jump = (t * 2.5).sin().abs() * 0.04;
    StickmanPose {
        body_y: jump, body_tilt: -5.0 * DEG,
        shoulder_r: -70.0 * DEG - pump, elbow_r: 30.0 * DEG,
        wrist_r: 10.0 * DEG, finger_r: 60.0 * DEG,
        shoulder_l: -20.0 * DEG, elbow_l: 20.0 * DEG,
        finger_l: 40.0 * DEG,
        neck: -10.0 * DEG, mouth: 1.0, eyebrow: 1.0,
        ..StickmanPose::neutral()
    }
}

pub fn pose_despair_deep(t: f64) -> StickmanPose {
    let tremble = (t * 5.0).sin() * 2.0 * DEG;
    StickmanPose {
        body_y: -0.05 + tremble * 0.1, body_tilt: 20.0 * DEG + tremble,
        spine_upper: 25.0 * DEG, neck: 35.0 * DEG,
        head_turn: -5.0 * DEG,
        shoulder_l: 50.0 * DEG, shoulder_r: -50.0 * DEG,
        elbow_l: -100.0 * DEG, elbow_r: 100.0 * DEG,
        wrist_l: -20.0 * DEG, wrist_r: 20.0 * DEG,
        finger_l: 30.0 * DEG, finger_r: 30.0 * DEG,
        hip_l: tremble, hip_r: -tremble,
        knee_l: 10.0 * DEG, knee_r: 10.0 * DEG,
        eyebrow: -1.0, mouth: -0.9, eye_squint: -0.7,
        eye_blink: 0.6,
        ..StickmanPose::neutral()
    }
}

pub fn pose_surrender(t: f64) -> StickmanPose {
    let shake = (t * 8.0).sin() * 3.0 * DEG;
    StickmanPose {
        body_tilt: 5.0 * DEG + shake,
        shoulder_l: -160.0 * DEG, shoulder_r: 160.0 * DEG,
        elbow_l: -30.0 * DEG, elbow_r: 30.0 * DEG,
        wrist_l: 10.0 * DEG, wrist_r: -10.0 * DEG,
        finger_l: 80.0 * DEG, finger_r: 80.0 * DEG,
        neck: 5.0 * DEG + shake * 0.5,
        eyebrow: 0.9, mouth: -0.5, eye_squint: -0.6, jaw_open: 0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_triumph(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: 0.04, body_tilt: -8.0 * DEG,
        shoulder_l: -160.0 * DEG, shoulder_r: 160.0 * DEG,
        elbow_l: -30.0 * DEG, elbow_r: -30.0 * DEG,
        wrist_l: 20.0 * DEG, wrist_r: -20.0 * DEG,
        finger_l: 70.0 * DEG, finger_r: 70.0 * DEG,
        hip_l: -5.0 * DEG, hip_r: 5.0 * DEG,
        neck: -15.0 * DEG, mouth: 1.0, eyebrow: 1.0,
        ..StickmanPose::neutral()
    }
}

pub fn pose_exhausted(t: f64) -> StickmanPose {
    let heave = (t * 2.0).sin() * 0.03;
    StickmanPose {
        body_y: -0.06 + heave, body_tilt: 25.0 * DEG,
        spine_upper: 20.0 * DEG, neck: 30.0 * DEG,
        shoulder_l: 20.0 * DEG, shoulder_r: -20.0 * DEG,
        elbow_l: 50.0 * DEG, elbow_r: 50.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        knee_l: 15.0 * DEG, knee_r: 15.0 * DEG,
        mouth: -0.8, eyebrow: -0.7, jaw_open: 0.5,
        eye_squint: -0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_confident(t: f64) -> StickmanPose {
    StickmanPose {
        body_y: 0.02, body_tilt: -5.0 * DEG,
        shoulder_l: -30.0 * DEG, shoulder_r: 30.0 * DEG,
        elbow_l: 10.0 * DEG, elbow_r: 10.0 * DEG,
        wrist_l: 5.0 * DEG, wrist_r: -5.0 * DEG,
        hip_l: 5.0 * DEG, hip_r: -5.0 * DEG,
        neck: -8.0 * DEG,
        eyebrow: 0.5, mouth: 0.4,
        ..StickmanPose::neutral()
    }
}

pub fn pose_taunt_provoke(t: f64) -> StickmanPose {
    let gesture = ((t * 2.5).sin() * 0.5 + 0.5) * 25.0 * DEG;
    StickmanPose {
        body_tilt: 8.0 * DEG,
        shoulder_r: -50.0 * DEG, elbow_r: -20.0 * DEG,
        wrist_r: -gesture, finger_r: 50.0 * DEG,
        shoulder_l: 50.0 * DEG, elbow_l: 30.0 * DEG,
        neck: 8.0 * DEG, head_turn: 30.0 * DEG,
        eyebrow: 0.8, mouth: 0.6, eye_squint: -0.3,
        ..StickmanPose::neutral()
    }
}

pub fn pose_bow(t: f64) -> StickmanPose {
    let s = ease_in_out(clamp01(t / 0.4));
    let recover = ease_in_out(clamp01((t - 0.4) / 0.4));
    let active = if t < 0.4 { s } else { 1.0 - recover };
    StickmanPose {
        body_tilt: 60.0 * DEG * active,
        spine_upper: 30.0 * DEG * active,
        neck: 40.0 * DEG * active,
        head_turn: -5.0 * DEG,
        shoulder_l: 10.0 * DEG * active, shoulder_r: -10.0 * DEG * active,
        elbow_l: 20.0 * DEG, elbow_r: 20.0 * DEG,
        hip_l: -5.0 * DEG * active, hip_r: 5.0 * DEG * active,
        knee_l: 10.0 * DEG * active, knee_r: 10.0 * DEG * active,
        ..StickmanPose::neutral()
    }
}

pub fn pose_cover_face(t: f64) -> StickmanPose {
    let s = (t * 4.0).sin().abs() * 0.5 + 0.5;
    StickmanPose {
        body_tilt: 10.0 * DEG,
        shoulder_l: 50.0 * DEG, shoulder_r: -50.0 * DEG,
        elbow_l: -120.0 * DEG, elbow_r: 120.0 * DEG,
        wrist_l: 30.0 * DEG, wrist_r: -30.0 * DEG,
        finger_l: 40.0 * DEG, finger_r: 40.0 * DEG,
        neck: 15.0 * DEG, head_turn: -5.0 * DEG,
        eyebrow: -0.9, mouth: -0.8, eye_squint: -0.5,
        eye_blink: s * 0.5,
        ..StickmanPose::neutral()
    }
}

pub fn pose_cower(t: f64) -> StickmanPose {
    let tremble = (t * 6.0).sin() * 3.0 * DEG;
    StickmanPose {
        body_y: -0.15, body_tilt: 30.0 * DEG + tremble,
        spine_upper: 25.0 * DEG, neck: 35.0 * DEG + tremble * 0.5,
        shoulder_l: 30.0 * DEG, shoulder_r: -30.0 * DEG,
        elbow_l: 70.0 * DEG, elbow_r: 70.0 * DEG,
        wrist_l: 10.0 * DEG, wrist_r: -10.0 * DEG,
        finger_l: 40.0 * DEG, finger_r: 40.0 * DEG,
        hip_l: tremble, hip_r: -tremble,
        knee_l: 60.0 * DEG, knee_r: 60.0 * DEG,
        eyebrow: 0.9, mouth: -0.7, eye_squint: -0.8,
        eye_blink: 0.5,
        ..StickmanPose::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════
// 19. MOVEMENT VARIANTS — Variasi Gerakan
// ═══════════════════════════════════════════════════════════════

pub fn pose_walk_backward_ext(t: f64, speed: f64) -> StickmanPose {
    let mut p = pose_walk(t, speed * 0.8);
    p.body_tilt = 8.0 * DEG;
    p.neck = 8.0 * DEG;
    p.pos_x = -t * 0.5;
    p.head_turn = 25.0 * DEG;
    p.eyebrow = 0.2;
    p
}

pub fn pose_strafe_left(t: f64, speed: f64) -> StickmanPose {
    let cycle = t * speed * 1.8 * std::f64::consts::PI;
    let step = cycle.sin() * 22.0 * DEG;
    StickmanPose {
        body_y: (cycle * 2.0).sin().abs() * 0.03,
        shoulder_l: 25.0 * DEG, shoulder_r: -35.0 * DEG,
        elbow_l: 25.0 * DEG, elbow_r: 15.0 * DEG,
        hip_l: step, hip_r: -step,
        knee_l: (0.0f64).max(-cycle.sin()) * 22.0 * DEG,
        knee_r: (0.0f64).max(cycle.sin()) * 22.0 * DEG,
        body_tilt: -8.0 * DEG, pos_x: 0.0,
        head_turn: -40.0 * DEG,
        ..StickmanPose::neutral()
    }
}

pub fn pose_strafe_right(t: f64, speed: f64) -> StickmanPose {
    let mut p = pose_strafe_left(t, speed);
    p.shoulder_l = -35.0 * DEG;
    p.shoulder_r = 25.0 * DEG;
    p.head_turn = 40.0 * DEG;
    p
}

// ═══════════════════════════════════════════════════════════════
// 9. LEGACY POSES — Dipertahankan untuk kompatibilitas
// ═══════════════════════════════════════════════════════════════

pub fn pose_jump(t: f64) -> StickmanPose {
    let phase = t;
    let (body_y, knee, hip, arm, tilt) = if phase < 0.4 {
        let s = ease_in_out(phase / 0.4);
        (-0.06 * s, 45.0 * DEG * s, -15.0 * DEG * s, -30.0 * DEG * s, -5.0 * DEG * s)
    } else if phase < 0.7 {
        let s = (phase - 0.4) / 0.3;
        let air_s = ease_in_out(s);
        (0.25 * (1.0 - (2.0 * s - 1.0).powi(2)), -30.0 * DEG * (1.0 - air_s), 10.0 * DEG, 60.0 * DEG * air_s, 8.0 * DEG)
    } else {
        let s = ease_in_out((phase - 0.7) / 0.3);
        (-0.04 * s, 35.0 * DEG * s, -10.0 * DEG * s, -20.0 * DEG * s, -4.0 * DEG * s)
    };
    StickmanPose { body_y, body_tilt: tilt, shoulder_l: arm, shoulder_r: arm, hip_l: hip, hip_r: hip, knee_l: knee, knee_r: knee, neck: -tilt * 0.5, ..StickmanPose::neutral() }
}

pub fn pose_dance(t: f64) -> StickmanPose {
    let cycle = t * 2.0 * std::f64::consts::PI * 1.5;
    let arm_l = cycle.sin() * 55.0 * DEG;
    let arm_r = (cycle + 1.0).sin() * 55.0 * DEG;
    let hip_sway = cycle.sin() * 12.0 * DEG;
    let bounce = (cycle * 2.0).sin().abs() * 0.03;
    StickmanPose { body_y: bounce, body_tilt: hip_sway * 0.4, shoulder_l: arm_l - 30.0 * DEG, shoulder_r: -(arm_r - 30.0 * DEG), elbow_l: 45.0 * DEG, elbow_r: 45.0 * DEG, hip_l: hip_sway, hip_r: -hip_sway, knee_l: (0.0f64).max(hip_sway) * 0.5, knee_r: (0.0f64).max(-hip_sway) * 0.5, mouth: 0.6, head_turn: cycle.sin() * 10.0 * DEG, neck: cycle.cos() * 5.0 * DEG, ..StickmanPose::neutral() }
}

pub fn pose_punch(t: f64) -> StickmanPose {
    let (shoulder, elbow, tilt) = if t < 0.3 {
        let s = ease_in_out(t / 0.3);
        (-30.0 * DEG * s, 70.0 * DEG * s, -5.0 * DEG * s)
    } else if t < 0.55 {
        let s = ease_in_out((t - 0.3) / 0.25);
        (-30.0 * DEG * (1.0 - s * 2.0).max(-1.0), 70.0 * DEG * (1.0 - s), 10.0 * DEG * s)
    } else {
        let s = ease_in_out((t - 0.55) / 0.45);
        (30.0 * DEG * (1.0 - s), 0.0, 10.0 * DEG * (1.0 - s))
    };
    StickmanPose { shoulder_r: shoulder, elbow_r: elbow, body_tilt: tilt, shoulder_l: 10.0 * DEG, elbow_l: 15.0 * DEG, neck: -tilt * 0.5, ..StickmanPose::neutral() }
}

pub fn pose_fall(t: f64) -> StickmanPose {
    let spin = t * 90.0 * DEG;
    let spread = ease_in_out(clamp01(t * 2.0));
    StickmanPose { body_tilt: spin, body_y: -t * t * 0.5, shoulder_l: -80.0 * DEG * spread, shoulder_r: 80.0 * DEG * spread, elbow_l: 30.0 * DEG, elbow_r: 30.0 * DEG, hip_l: -30.0 * DEG * spread, hip_r: 30.0 * DEG * spread, ..StickmanPose::neutral() }
}

pub fn pose_think(t: f64) -> StickmanPose {
    let nod = (t * 1.5).sin() * 3.0 * DEG;
    StickmanPose { shoulder_l: -10.0 * DEG, shoulder_r: 45.0 * DEG, elbow_r: -100.0 * DEG, body_tilt: nod, neck: nod, head_turn: -15.0 * DEG, wrist_r: -30.0 * DEG, mouth: -0.2, ..StickmanPose::neutral() }
}

pub fn pose_dodge(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, shift, arm) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (20.0 * DEG * s, 0.0, -30.0 * DEG * s)
    } else if phase < 0.6 {
        let s = ease_in_out((phase - 0.3) / 0.3);
        (20.0 * DEG * (1.0 - s), 0.25 * s, -30.0 * DEG * (1.0 - s) + 60.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (0.0, 0.25 * (1.0 - s), 60.0 * DEG * (1.0 - s))
    };
    StickmanPose { body_tilt: tilt, pos_x: shift, shoulder_l: arm - 40.0 * DEG, shoulder_r: 20.0 * DEG, elbow_l: 50.0 * DEG, elbow_r: 20.0 * DEG, hip_l: -shift * 0.5, hip_r: shift * 0.5, knee_l: 15.0 * DEG, knee_r: 15.0 * DEG, eye_squint: -0.5, eyebrow: 0.8, ..StickmanPose::neutral() }
}

pub fn pose_stumble(t: f64) -> StickmanPose {
    let phase = t;
    let (tilt, arm_l, arm_r, leg) = if phase < 0.3 {
        let s = ease_in_out(phase / 0.3);
        (15.0 * DEG * s, -20.0 * DEG * s, 30.0 * DEG * s, 10.0 * DEG * s)
    } else if phase < 0.6 {
        let s = ease_in_out((phase - 0.3) / 0.3);
        (35.0 * DEG * s, -60.0 * DEG * s, 80.0 * DEG * s, -20.0 * DEG * s)
    } else {
        let s = ease_in_out((phase - 0.6) / 0.4);
        (35.0 * DEG * (1.0 - s), -60.0 * DEG * (1.0 - s), 80.0 * DEG * (1.0 - s), 0.0)
    };
    StickmanPose { body_tilt: tilt, shoulder_l: arm_l - 20.0 * DEG, shoulder_r: arm_r + 10.0 * DEG, elbow_l: 70.0 * DEG, elbow_r: 30.0 * DEG, hip_l: leg, hip_r: -leg, knee_l: 30.0 * DEG + leg, knee_r: 30.0 * DEG - leg, body_y: -(phase * 0.08).min(0.06), head_turn: phase.sin() * 15.0 * DEG, jaw_open: 0.6, eyebrow: 0.9, ..StickmanPose::neutral() }
}

pub fn pose_roll(t: f64) -> StickmanPose {
    let spin = t * 360.0 * DEG;
    let tuck = 1.0 - (t * 2.0 - 1.0).powi(2);
    StickmanPose { body_y: -0.15 * tuck, body_tilt: spin, shoulder_l: -60.0 * DEG * tuck, shoulder_r: 60.0 * DEG * tuck, elbow_l: 120.0 * DEG * tuck, elbow_r: 120.0 * DEG * tuck, hip_l: -40.0 * DEG * tuck, hip_r: 40.0 * DEG * tuck, knee_l: 110.0 * DEG * tuck, knee_r: 110.0 * DEG * tuck, neck: -20.0 * DEG * tuck, pos_x: t * 0.6, ..StickmanPose::neutral() }
}

// ═══════════════════════════════════════════════════════════════
// ENUM & DISPATCH
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationName {
    // LOCOMOTION
    Idle, IdleShift, Walk, WalkBackward, WalkSide, Run, Sprint, Jog,
    SlowWalk, Crawl, CrawlHigh, PanicRun, StealthWalk, SadWalk, HappyHop, TipToe,
    // TRANSITIONS
    StandToCrouch, StandToSit, StandToKneel, StandToLie,
    // INTERACTIONS
    ReachUp, ReachDown, ReachForward, Grab, PickUp, Carry, CarryOneHand,
    Push, Pull, Lift, Throw, Catch, Place,
    // ENVIRONMENT
    SitChair, SitGround, SitKneesUp, SitStool,
    LieBack, LieSide, LieStomach, LeanWall,
    Kneel, KneelOne, Crouch, Squat,
    Climb, Hang, HangOneArm, Vault, Swing, Dive, Slide,
    // COMBAT
    Jab, Cross, Uppercut, Kick, Block, Duck,
    Shoot, Aim, ShootRifle, ShootFromCover, Reload,
    Tackle, Stun, Takedown,
    // EXPRESSIVE
    Wave, WaveBoth, Point, Cheer, Victory, Despair, Facepalm,
    Shrug, Salute, Taunt, Nod, ShakeHead, Celebrate,
    // INJURY
    Hurt, HurtHeavy, KnockDown, GetUp, DragLimp, Dead,
    // ACROBATIC
    Cartwheel, Flip, Handstand,
    // ACROBATIC EXTENDED
    SaltoForward, SaltoBackward, AerialCartwheel, BackHandspring, FrontHandspring, DiveRoll, WallFlip,
    // KICKS
    RoundhouseKick, FrontKick, SideKick, AxeKick, KickHead, KickBody, KickLeg,
    FlyingKick, CrescentKick, KneeStrike, DoubleKick,
    // PUNCHES & STRIKES
    Haymaker, BodyBlow, ElbowStrike, Backfist, PalmStrike, HammerFist,
    // GRABS & THROWS
    GrabHeadlock, BodySlam, Suplex, HipThrow, ChokeHold, ThrowPush, Clothesline, LegSweep,
    // DEFENSE
    BlockHigh, BlockMid, BlockLow, Parry, Weave, StepBack, CrossBlock,
    // WEAPONS
    DrawWeapon, Holster, MeleeSwing, MeleeStab, ThrowWeapon, WeaponBlock,
    // VEHICLES
    EnterCarDriver, ExitCarDriver, EnterCarPassenger, ExitCarPassenger,
    Drive, RideMotorcycle, DismountMotorcycle, EnterHelicopter, ExitHelicopter,
    // ENVIRONMENT EXTENDED
    ClimbWall, PullUp, HangDrop, OpenDoor, CloseDoor, CrawlThrough, JumpOverObstacle, DropFromHeight,
    // GROUND & PRONE
    ProneCrawl, CrouchWalk, GroundSit, GroundRecover, Limp, StaggerBack, TripAndFall, Slip,
    // EMOTIONAL EXTENDED
    CelebrateArmsUp, CelebrateFistPump, DespairDeep, Surrender, Triumph, Exhausted, Confident,
    TauntProvoke, Bow, CoverFace, Cower,
    // MOVEMENT VARIANTS
    WalkBackwardExt, StrafeLeft, StrafeRight,
    // LEGACY ALIASES
    Jump, Dance, Punch, Fall, Think,
    Dodge, Stumble, Roll,
}

pub fn get_pose(anim: AnimationName, t: f64, speed: f64) -> StickmanPose {
    match anim {
        // LOCOMOTION
        AnimationName::Idle          => pose_idle(t),
        AnimationName::IdleShift     => pose_idle_shift(t),
        AnimationName::Walk          => pose_walk(t, speed),
        AnimationName::WalkBackward  => pose_walk_backward(t, speed),
        AnimationName::WalkSide      => pose_walk_side(t, speed),
        AnimationName::Run           => pose_run(t, speed),
        AnimationName::Sprint        => pose_sprint(t, speed),
        AnimationName::Jog           => pose_jog(t, speed),
        AnimationName::SlowWalk      => pose_slow_walk(t, speed),
        AnimationName::Crawl         => pose_crawl(t, speed),
        AnimationName::CrawlHigh     => pose_crawl_high(t, speed),
        AnimationName::PanicRun      => pose_panic_run(t, speed),
        AnimationName::StealthWalk   => pose_stealth_walk(t, speed),
        AnimationName::SadWalk       => pose_sad_walk(t, speed),
        AnimationName::HappyHop      => pose_happy_hop(t, speed),
        AnimationName::TipToe        => pose_tip_toe(t, speed),
        // TRANSITIONS
        AnimationName::StandToCrouch => pose_stand_to_crouch(t),
        AnimationName::StandToSit    => pose_stand_to_sit(t),
        AnimationName::StandToKneel  => pose_stand_to_kneel(t),
        AnimationName::StandToLie    => pose_stand_to_lie(t),
        // INTERACTIONS
        AnimationName::ReachUp       => pose_reach_up(t),
        AnimationName::ReachDown     => pose_reach_down(t),
        AnimationName::ReachForward  => pose_reach_forward(t),
        AnimationName::Grab          => pose_grab(t),
        AnimationName::PickUp        => pose_pick_up(t),
        AnimationName::Carry         => pose_carry(t),
        AnimationName::CarryOneHand  => pose_carry_one_hand(t),
        AnimationName::Push          => pose_push(t),
        AnimationName::Pull          => pose_pull(t),
        AnimationName::Lift          => pose_lift(t),
        AnimationName::Throw         => pose_throw(t),
        AnimationName::Catch         => pose_catch(t),
        AnimationName::Place         => pose_place(t),
        // ENVIRONMENT
        AnimationName::SitChair      => pose_sit_chair(t),
        AnimationName::SitGround     => pose_sit_ground(t),
        AnimationName::SitKneesUp    => pose_sit_knees_up(t),
        AnimationName::SitStool      => pose_sit_stool(t),
        AnimationName::LieBack       => pose_lie_back(t),
        AnimationName::LieSide       => pose_lie_side(t),
        AnimationName::LieStomach    => pose_lie_stomach(t),
        AnimationName::LeanWall      => pose_lean_wall(t),
        AnimationName::Kneel         => pose_kneel(t),
        AnimationName::KneelOne      => pose_kneel_one(t),
        AnimationName::Crouch        => pose_crouch(t),
        AnimationName::Squat         => pose_squat(t),
        AnimationName::Climb         => pose_climb(t, speed),
        AnimationName::Hang          => pose_hang(t),
        AnimationName::HangOneArm    => pose_hang_one_arm(t),
        AnimationName::Vault         => pose_vault(t),
        AnimationName::Swing         => pose_swing(t),
        AnimationName::Dive          => pose_dive(t),
        AnimationName::Slide         => pose_slide(t),
        // COMBAT
        AnimationName::Jab           => pose_jab(t),
        AnimationName::Cross         => pose_cross(t),
        AnimationName::Uppercut      => pose_uppercut(t),
        AnimationName::Kick          => pose_kick(t),
        AnimationName::Block         => pose_block(t),
        AnimationName::Duck          => pose_duck(t),
        AnimationName::Shoot         => pose_shoot(t),
        AnimationName::Aim           => pose_aim(t),
        AnimationName::ShootRifle    => pose_shoot_rifle(t),
        AnimationName::ShootFromCover => pose_shoot_from_cover(t),
        AnimationName::Reload        => pose_reload(t),
        AnimationName::Tackle        => pose_tackle(t),
        AnimationName::Stun          => pose_stun(t),
        AnimationName::Takedown      => pose_takedown(t),
        // EXPRESSIVE
        AnimationName::Wave          => pose_wave(t),
        AnimationName::WaveBoth      => pose_wave_both(t),
        AnimationName::Point         => pose_point(t),
        AnimationName::Cheer         => pose_cheer(t),
        AnimationName::Victory       => pose_victory(t),
        AnimationName::Despair       => pose_despair(t),
        AnimationName::Facepalm      => pose_facepalm(t),
        AnimationName::Shrug         => pose_shrug(t),
        AnimationName::Salute        => pose_salute(t),
        AnimationName::Taunt         => pose_taunt(t),
        AnimationName::Nod           => pose_nod(t),
        AnimationName::ShakeHead     => pose_shake_head(t),
        AnimationName::Celebrate     => pose_celebrate(t),
        // INJURY
        AnimationName::Hurt          => pose_hurt(t),
        AnimationName::HurtHeavy     => pose_hurt_heavy(t),
        AnimationName::KnockDown     => pose_knock_down(t),
        AnimationName::GetUp         => pose_get_up(t),
        AnimationName::DragLimp      => pose_drag_limp(t, speed),
        AnimationName::Dead          => pose_dead(t),
        // ACROBATIC
        AnimationName::Cartwheel     => pose_cartwheel(t),
        AnimationName::Flip          => pose_flip(t),
        AnimationName::Handstand     => pose_handstand(t),
        // KICKS
        AnimationName::RoundhouseKick  => pose_roundhouse_kick(t),
        AnimationName::FrontKick       => pose_front_kick(t),
        AnimationName::SideKick        => pose_side_kick(t),
        AnimationName::AxeKick         => pose_axe_kick(t),
        AnimationName::KickHead        => pose_kick_head(t),
        AnimationName::KickBody        => pose_kick_body(t),
        AnimationName::KickLeg         => pose_kick_leg(t),
        AnimationName::FlyingKick      => pose_flying_kick(t),
        AnimationName::CrescentKick    => pose_crescent_kick(t),
        AnimationName::KneeStrike      => pose_knee_strike(t),
        AnimationName::DoubleKick      => pose_double_kick(t),
        // PUNCHES & STRIKES
        AnimationName::Haymaker        => pose_haymaker(t),
        AnimationName::BodyBlow        => pose_body_blow(t),
        AnimationName::ElbowStrike     => pose_elbow_strike(t),
        AnimationName::Backfist        => pose_backfist(t),
        AnimationName::PalmStrike      => pose_palm_strike(t),
        AnimationName::HammerFist      => pose_hammer_fist(t),
        // GRABS & THROWS
        AnimationName::GrabHeadlock    => pose_grab_headlock(t),
        AnimationName::BodySlam        => pose_body_slam(t),
        AnimationName::Suplex          => pose_suplex(t),
        AnimationName::HipThrow        => pose_hip_throw(t),
        AnimationName::ChokeHold       => pose_choke_hold(t),
        AnimationName::ThrowPush       => pose_throw_push(t),
        AnimationName::Clothesline     => pose_clothesline(t),
        AnimationName::LegSweep        => pose_leg_sweep(t),
        // DEFENSE
        AnimationName::BlockHigh       => pose_block_high(t),
        AnimationName::BlockMid        => pose_block_mid(t),
        AnimationName::BlockLow        => pose_block_low(t),
        AnimationName::Parry           => pose_parry(t),
        AnimationName::Weave           => pose_weave(t),
        AnimationName::StepBack        => pose_step_back(t),
        AnimationName::CrossBlock      => pose_cross_block(t),
        // WEAPONS
        AnimationName::DrawWeapon      => pose_draw_weapon(t),
        AnimationName::Holster         => pose_holster(t),
        AnimationName::MeleeSwing      => pose_melee_swing(t),
        AnimationName::MeleeStab       => pose_melee_stab(t),
        AnimationName::ThrowWeapon     => pose_throw_weapon(t),
        AnimationName::WeaponBlock     => pose_weapon_block(t),
        // VEHICLES
        AnimationName::EnterCarDriver       => pose_enter_car_driver(t),
        AnimationName::ExitCarDriver        => pose_exit_car_driver(t),
        AnimationName::EnterCarPassenger    => pose_enter_car_passenger(t),
        AnimationName::ExitCarPassenger     => pose_exit_car_passenger(t),
        AnimationName::Drive                => pose_drive(t),
        AnimationName::RideMotorcycle       => pose_ride_motorcycle(t),
        AnimationName::DismountMotorcycle   => pose_dismount_motorcycle(t),
        AnimationName::EnterHelicopter      => pose_enter_helicopter(t),
        AnimationName::ExitHelicopter       => pose_exit_helicopter(t),
        // ACROBATIC EXTENDED
        AnimationName::SaltoForward         => pose_salto_forward(t),
        AnimationName::SaltoBackward        => pose_salto_backward(t),
        AnimationName::AerialCartwheel      => pose_aerial_cartwheel(t),
        AnimationName::BackHandspring       => pose_back_handspring(t),
        AnimationName::FrontHandspring      => pose_front_handspring(t),
        AnimationName::DiveRoll             => pose_dive_roll(t),
        AnimationName::WallFlip             => pose_wall_flip(t),
        // ENVIRONMENT EXTENDED
        AnimationName::ClimbWall            => pose_climb_wall(t, speed),
        AnimationName::PullUp               => pose_pull_up(t),
        AnimationName::HangDrop             => pose_hang_drop(t),
        AnimationName::OpenDoor             => pose_open_door(t),
        AnimationName::CloseDoor            => pose_close_door(t),
        AnimationName::CrawlThrough         => pose_crawl_through(t, speed),
        AnimationName::JumpOverObstacle     => pose_jump_over_obstacle(t),
        AnimationName::DropFromHeight       => pose_drop_from_height(t),
        // GROUND & PRONE
        AnimationName::ProneCrawl           => pose_prone_crawl(t, speed),
        AnimationName::CrouchWalk           => pose_crouch_walk(t, speed),
        AnimationName::GroundSit            => pose_ground_sit(t),
        AnimationName::GroundRecover        => pose_ground_recover(t),
        AnimationName::Limp                 => pose_limp(t, speed),
        AnimationName::StaggerBack          => pose_stagger_back(t, speed),
        AnimationName::TripAndFall          => pose_trip_and_fall(t),
        AnimationName::Slip                 => pose_slip(t),
        // EMOTIONAL EXTENDED
        AnimationName::CelebrateArmsUp      => pose_celebrate_arms_up(t),
        AnimationName::CelebrateFistPump    => pose_celebrate_fist_pump(t),
        AnimationName::DespairDeep          => pose_despair_deep(t),
        AnimationName::Surrender            => pose_surrender(t),
        AnimationName::Triumph              => pose_triumph(t),
        AnimationName::Exhausted            => pose_exhausted(t),
        AnimationName::Confident            => pose_confident(t),
        AnimationName::TauntProvoke         => pose_taunt_provoke(t),
        AnimationName::Bow                  => pose_bow(t),
        AnimationName::CoverFace            => pose_cover_face(t),
        AnimationName::Cower                => pose_cower(t),
        // MOVEMENT VARIANTS
        AnimationName::WalkBackwardExt      => pose_walk_backward_ext(t, speed),
        AnimationName::StrafeLeft           => pose_strafe_left(t, speed),
        AnimationName::StrafeRight          => pose_strafe_right(t, speed),
        // LEGACY ALIASES
        AnimationName::Jump          => pose_jump(t),
        AnimationName::Dance         => pose_dance(t),
        AnimationName::Punch         => pose_punch(t),
        AnimationName::Fall          => pose_fall(t),
        AnimationName::Think         => pose_think(t),
        AnimationName::Dodge         => pose_dodge(t),
        AnimationName::Stumble       => pose_stumble(t),
        AnimationName::Roll          => pose_roll(t),
    }
}

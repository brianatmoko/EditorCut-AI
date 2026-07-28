//! Modul Parametric Emotion Engine
//! Memberikan overlay emosi (Panik, Sedih, Senang, Takut, Marah, Bingung) di atas pose locomotion dasar.

use crate::pose::StickmanPose;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmotionState {
    Neutral,
    Panic,
    Sadness,
    Happiness,
    Fear,
    Anger,
    Thinking,
}

/// Menerapkan modifikasi emosi pada `pose` berdasarkan `emotion` dan intensitas `intensity` (0.0 .. 1.0) dan waktu `t`.
pub fn apply_emotion_overlay(
    pose: &mut StickmanPose,
    emotion: EmotionState,
    intensity: f64,
    t: f64,
) {
    if intensity <= 0.001 {
        return;
    }

    let deg = |deg_val: f64| deg_val * std::f64::consts::PI / 180.0;

    match emotion {
        EmotionState::Panic => {
            // Panik: Tremble/gemetar cepat, bahu terangkat tinggi, mata terbelalak, napas cepat terengah
            let tremble_x = (t * 25.0).sin() * 0.015 * intensity;
            let tremble_y = (t * 30.0).cos() * 0.010 * intensity;
            let head_jerk = (t * 12.0).sin() * deg(8.0) * intensity;

            pose.pos_x += tremble_x;
            pose.body_y += tremble_y;
            pose.head_turn += head_jerk;
            pose.neck += (t * 18.0).sin() * deg(4.0) * intensity;

            // Bahu terangkat
            pose.clavicle_l += deg(15.0) * intensity;
            pose.clavicle_r += deg(15.0) * intensity;

            // Siku menempel melindung dada
            pose.elbow_l += deg(40.0) * intensity;
            pose.elbow_r += deg(40.0) * intensity;

            // Ekspresi wajah panik
            pose.eyebrow = 0.8 * intensity;
            pose.brow_inner = 1.0 * intensity;
            pose.eye_squint = -0.5 * intensity; // Terbelalak
            pose.jaw_open = 0.6 * intensity;
            pose.secondary_hair_sway += (t * 20.0).sin() * 0.05 * intensity;
        }

        EmotionState::Sadness => {
            // Sedih: Postur membungkuk, kepala tertunduk, pergerakan lambat, alis miring ke atas
            let droop = deg(-12.0) * intensity;
            pose.spine_upper += droop;
            pose.spine_lower += droop * 0.5;
            pose.neck += deg(15.0) * intensity; // Menunduk
            pose.body_tilt += deg(6.0) * intensity;

            pose.shoulder_l -= deg(10.0) * intensity;
            pose.shoulder_r -= deg(10.0) * intensity;

            // Wajah sedih
            pose.eyebrow = -0.6 * intensity;
            pose.brow_inner = 0.8 * intensity;
            pose.mouth = -0.7 * intensity; // Cemberut
            pose.eye_blink = (t * 0.5).sin().max(0.0) * 0.4 * intensity;
        }

        EmotionState::Happiness => {
            // Senang: Postur membusung tegak, gerakan mengayun tinggi, senyum lebar
            let bounce = (t * 4.0).sin().abs() * 0.025 * intensity;
            pose.body_y += bounce;
            pose.spine_upper -= deg(6.0) * intensity;

            pose.shoulder_l += (t * 4.0).sin() * deg(12.0) * intensity;
            pose.shoulder_r -= (t * 4.0).sin() * deg(12.0) * intensity;

            // Wajah senang
            pose.eyebrow = 0.5 * intensity;
            pose.mouth = 0.9 * intensity; // Senyum lebar
            pose.jaw_open = 0.2 * intensity;
            pose.eye_squint = 0.3 * intensity;
        }

        EmotionState::Fear => {
            // Takut: Menarik badan ke belakang, tangan membenteng muka
            let lean_back = deg(-10.0) * intensity;
            pose.body_tilt += lean_back;
            pose.neck -= lean_back * 0.8;

            pose.shoulder_l += deg(45.0) * intensity;
            pose.shoulder_r += deg(45.0) * intensity;
            pose.elbow_l += deg(80.0) * intensity;
            pose.elbow_r += deg(80.0) * intensity;

            pose.eyebrow = 0.9 * intensity;
            pose.eye_squint = -0.6 * intensity;
            pose.mouth = -0.3 * intensity;
        }

        EmotionState::Anger => {
            // Marah: Dada membusung maju, tangan mengepal, alis menunduk tajam
            let lean_forward = deg(12.0) * intensity;
            pose.body_tilt += lean_forward;
            pose.spine_upper += deg(8.0) * intensity;

            pose.shoulder_l -= deg(20.0) * intensity;
            pose.shoulder_r -= deg(20.0) * intensity;
            pose.finger_l = 1.0 * intensity; // Mengepal
            pose.finger_r = 1.0 * intensity;

            pose.eyebrow = -0.9 * intensity;
            pose.brow_inner = -0.9 * intensity;
            pose.mouth = -0.5 * intensity;
            pose.jaw_open = 0.3 * intensity;
        }

        EmotionState::Thinking => {
            // Bingung / Berpikir: Kepala miring, satu tangan menempel dagu
            let head_tilt = deg(15.0) * intensity;
            pose.head_turn += head_tilt;
            pose.neck += deg(5.0) * intensity;

            pose.shoulder_r += deg(60.0) * intensity;
            pose.elbow_r += deg(110.0) * intensity;
            pose.wrist_r += deg(40.0) * intensity;

            pose.eyebrow = 0.4 * intensity;
            pose.mouth = -0.2 * intensity;
            pose.pupil_x = 0.5 * intensity;
            pose.pupil_y = 0.5 * intensity;
        }

        EmotionState::Neutral => {}
    }
}

//! Modul Hierarchical Animation State Machine (HFSM) & Pose Cross-Fading
//! Mengatur transisi mulus (*smooth transition*) antar-pose animasi 2D.

use crate::emotions::{apply_emotion_overlay, EmotionState};
use crate::pose::StickmanPose;
use crate::poses::{get_pose, AnimationName};

#[derive(Debug, Clone)]
pub struct AnimationStateMachine {
    pub current_anim: AnimationName,
    pub target_anim: Option<AnimationName>,
    pub emotion: EmotionState,
    pub emotion_intensity: f64,
    pub transition_timer: f64,
    pub transition_duration: f64,
    pub anim_time: f64,
    pub speed: f64,
}

impl AnimationStateMachine {
    pub fn new(initial_anim: AnimationName) -> Self {
        Self {
            current_anim: initial_anim,
            target_anim: None,
            emotion: EmotionState::Neutral,
            emotion_intensity: 0.0,
            transition_timer: 0.0,
            transition_duration: 0.25, // 250ms smooth cross-fade default
            anim_time: 0.0,
            speed: 1.0,
        }
    }

    pub fn transition_to(&mut self, new_anim: AnimationName, duration: f64) {
        if self.current_anim == new_anim && self.target_anim.is_none() {
            return;
        }
        self.target_anim = Some(new_anim);
        self.transition_timer = 0.0;
        self.transition_duration = duration.max(0.01);
    }

    pub fn set_emotion(&mut self, emotion: EmotionState, intensity: f64) {
        self.emotion = emotion;
        self.emotion_intensity = intensity.clamp(0.0, 1.0);
    }

    /// Evaluasi pose pada delta time `dt`
    pub fn update(&mut self, dt: f64) -> StickmanPose {
        self.anim_time += dt;

        let mut current_pose = get_pose(self.current_anim, self.anim_time, self.speed);

        if let Some(target) = self.target_anim {
            self.transition_timer += dt;
            let blend = (self.transition_timer / self.transition_duration).clamp(0.0, 1.0);
            let target_pose = get_pose(target, self.anim_time, self.speed);

            current_pose = StickmanPose::lerp(&current_pose, &target_pose, blend);

            if blend >= 1.0 {
                self.current_anim = target;
                self.target_anim = None;
                self.transition_timer = 0.0;
            }
        }

        // Terapkan overlay emosi
        apply_emotion_overlay(
            &mut current_pose,
            self.emotion,
            self.emotion_intensity,
            self.anim_time,
        );

        current_pose
    }
}

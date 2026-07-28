#![allow(dead_code, unused_variables, unused_imports, unreachable_patterns, non_snake_case)]

mod animator;
mod character;
mod cinematic;
mod director;
mod emotions;
mod executor;
mod ik;
mod physics_secondary;
mod pose;
mod poses;
mod render;
mod script;
mod shapes;
mod state_machine;

pub use animator::{AnimationClip, StickmanAnimator};
pub use executor::{SceneExecutor, EntityExecutionState, PositionSnapshot, ExecutionLog};
pub use character::get_character;
pub use cinematic::{ActionBeat, AttackRange, CameraMovement, CameraShot, CameraTransition, CinematicAct, CinematicMovie, DialogueLine, RangeType, ShotType, SmartCameraDirector, StageEntity, get_attack_range};
pub use director::generate_cinematic_movie;
pub use emotions::{apply_emotion_overlay, EmotionState};
pub use ik::{solve_2bone_ik, solve_foot_grounding, TwoBoneIKResult, Vec2};
pub use physics_secondary::{SecondaryPhysicsEngine, SpringState};
pub use pose::StickmanPose;
pub use poses::AnimationName;
pub use render::{BoneTransform, LineSegment, StickmanRenderData, stickman_to_segments};
pub use script::{ParsedStickmanScript, generate_stickman_script};
pub use shapes::{CharacterDef, ColorRole, Palette, PartShape, ResolvedShape, ShapeKind, resolve_character_shapes};
pub use state_machine::AnimationStateMachine;

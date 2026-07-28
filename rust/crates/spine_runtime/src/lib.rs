//! SCML (BrashMonkey Spriter) runtime for OpenCut desktop preview.
//!
//! Parses SCML XML and computes world-space bone/object transforms for
//! any animation at any time (linear interpolation).
//!
//! All character animation is bone-based modular from EPS + SCML.
//! The old Spine skeleton system and PNG spritesheet frame sequences
//! have been removed.

#![allow(dead_code, non_snake_case)]

pub mod scml;

pub use scml::{ScmlData, ScmlEntity, ScmlAnimation, ScmlPose, ScmlPoseObject,
    ScmlTransform, ScmlBone, ScmlFile, ScmlFolder,
    parse_scml, evaluate_pose, resolve_bone_hierarchy,
    get_file_name, get_file_info, apply_character_map, CharacterMapEntry};

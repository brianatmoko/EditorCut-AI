use serde::{Deserialize, Serialize};
use time::MediaTime;

mod background;
mod canvas_size;
mod metadata;
mod settings;
mod view_state;

pub use background::*;
pub use canvas_size::*;
pub use metadata::*;
pub use settings::*;
pub use view_state::*;

use timeline::TimelineScene;

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub metadata: ProjectMetadata,
    pub scenes: Vec<TimelineScene>,
    pub current_scene_id: String,
    pub settings: ProjectSettings,
    pub version: u32,
    pub timeline_view_state: Option<TimelineViewState>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            metadata: ProjectMetadata {
                id: String::new(),
                name: "Untitled".to_string(),
                thumbnail: None,
                duration: MediaTime::ZERO,
                created_at: String::new(),
                updated_at: String::new(),
            },
            scenes: Vec::new(),
            current_scene_id: String::new(),
            settings: ProjectSettings {
                fps: time::FrameRate::FPS_30,
                canvas_size: CanvasSize {
                    width: 1920,
                    height: 1080,
                },
                canvas_size_mode: None,
                last_custom_canvas_size: None,
                original_canvas_size: None,
                background: Background::Color {
                    color: "#000000".to_string(),
                },
            },
            version: 1,
            timeline_view_state: None,
        }
    }
}

use serde::{Deserialize, Serialize};
use time::FrameRate;

use crate::{Background, CanvasSize};

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub fps: FrameRate,
    pub canvas_size: CanvasSize,
    pub canvas_size_mode: Option<String>,
    pub last_custom_canvas_size: Option<CanvasSize>,
    pub original_canvas_size: Option<CanvasSize>,
    pub background: Background,
}

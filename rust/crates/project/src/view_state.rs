use serde::{Deserialize, Serialize};
use time::MediaTime;

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimelineViewState {
    pub zoom_level: f64,
    pub scroll_left: f64,
    pub playhead_time: MediaTime,
}

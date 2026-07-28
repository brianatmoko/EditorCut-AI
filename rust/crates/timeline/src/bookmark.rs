use serde::{Deserialize, Serialize};
use time::MediaTime;

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub time: MediaTime,
    pub note: Option<String>,
    pub color: Option<String>,
    pub duration: Option<MediaTime>,
}

use serde::{Deserialize, Serialize};
use time::MediaTime;

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub id: String,
    pub name: String,
    pub thumbnail: Option<String>,
    pub duration: MediaTime,
    pub created_at: String,
    pub updated_at: String,
}

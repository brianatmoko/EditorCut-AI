use serde::{Deserialize, Serialize};

use crate::TimelineElement;

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TimelineTrack {
    #[serde(rename = "video")]
    Video(VideoTrack),
    #[serde(rename = "text")]
    Text(TextTrack),
    #[serde(rename = "audio")]
    Audio(AudioTrack),
    #[serde(rename = "graphic")]
    Graphic(GraphicTrack),
    #[serde(rename = "effect")]
    Effect(EffectTrack),
}

impl TimelineTrack {
    pub fn id(&self) -> &str {
        match self {
            TimelineTrack::Video(t) => &t.id,
            TimelineTrack::Text(t) => &t.id,
            TimelineTrack::Audio(t) => &t.id,
            TimelineTrack::Graphic(t) => &t.id,
            TimelineTrack::Effect(t) => &t.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            TimelineTrack::Video(t) => &t.name,
            TimelineTrack::Text(t) => &t.name,
            TimelineTrack::Audio(t) => &t.name,
            TimelineTrack::Graphic(t) => &t.name,
            TimelineTrack::Effect(t) => &t.name,
        }
    }

    pub fn elements(&self) -> &[TimelineElement] {
        match self {
            TimelineTrack::Video(t) => &t.elements,
            TimelineTrack::Text(t) => &t.elements,
            TimelineTrack::Audio(t) => &t.elements,
            TimelineTrack::Graphic(t) => &t.elements,
            TimelineTrack::Effect(t) => &t.elements,
        }
    }

    pub fn elements_mut(&mut self) -> &mut Vec<TimelineElement> {
        match self {
            TimelineTrack::Video(t) => &mut t.elements,
            TimelineTrack::Text(t) => &mut t.elements,
            TimelineTrack::Audio(t) => &mut t.elements,
            TimelineTrack::Graphic(t) => &mut t.elements,
            TimelineTrack::Effect(t) => &mut t.elements,
        }
    }

    pub fn track_type_str(&self) -> &str {
        match self {
            TimelineTrack::Video(_) => "video",
            TimelineTrack::Text(_) => "text",
            TimelineTrack::Audio(_) => "audio",
            TimelineTrack::Graphic(_) => "graphic",
            TimelineTrack::Effect(_) => "effect",
        }
    }

    pub fn is_overlay(&self) -> bool {
        matches!(
            self,
            TimelineTrack::Video(_)
                | TimelineTrack::Text(_)
                | TimelineTrack::Graphic(_)
                | TimelineTrack::Effect(_)
        )
    }
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoTrack {
    pub id: String,
    pub name: String,
    pub elements: Vec<TimelineElement>,
    pub muted: bool,
    pub hidden: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextTrack {
    pub id: String,
    pub name: String,
    pub elements: Vec<TimelineElement>,
    pub hidden: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub id: String,
    pub name: String,
    pub elements: Vec<TimelineElement>,
    pub muted: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphicTrack {
    pub id: String,
    pub name: String,
    pub elements: Vec<TimelineElement>,
    pub hidden: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EffectTrack {
    pub id: String,
    pub name: String,
    pub elements: Vec<TimelineElement>,
    pub hidden: bool,
}

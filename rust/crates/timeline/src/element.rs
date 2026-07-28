use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::MediaTime;

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetimeConfig {
    pub rate: f64,
    pub maintain_pitch: Option<bool>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BaseTimelineElement {
    pub id: String,
    pub name: String,
    pub duration: MediaTime,
    pub start_time: MediaTime,
    pub trim_start: MediaTime,
    pub trim_end: MediaTime,
    pub source_duration: Option<MediaTime>,
    pub animations: Option<Value>,
    pub params: Value,
    pub effects: Option<Vec<EffectRef>>,
    pub masks: Option<Vec<MaskRef>>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EffectRef {
    pub id: String,
    pub effect_type: String,
    pub params: Value,
    pub enabled: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MaskRef {
    pub id: String,
    pub mask_type: String,
    pub params: Value,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TimelineElement {
    #[serde(rename = "video")]
    Video(VideoElementData),
    #[serde(rename = "image")]
    Image(ImageElementData),
    #[serde(rename = "text")]
    Text(TextElementData),
    #[serde(rename = "sticker")]
    Sticker(StickerElementData),
    #[serde(rename = "graphic")]
    Graphic(GraphicElementData),
    #[serde(rename = "audio")]
    Audio(AudioElementData),
    #[serde(rename = "effect")]
    Effect(EffectElementData),
}

impl TimelineElement {
    pub fn base(&self) -> &BaseTimelineElement {
        match self {
            TimelineElement::Video(e) => &e.base,
            TimelineElement::Image(e) => &e.base,
            TimelineElement::Text(e) => &e.base,
            TimelineElement::Sticker(e) => &e.base,
            TimelineElement::Graphic(e) => &e.base,
            TimelineElement::Audio(e) => e.base(),
            TimelineElement::Effect(e) => &e.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseTimelineElement {
        match self {
            TimelineElement::Video(e) => &mut e.base,
            TimelineElement::Image(e) => &mut e.base,
            TimelineElement::Text(e) => &mut e.base,
            TimelineElement::Sticker(e) => &mut e.base,
            TimelineElement::Graphic(e) => &mut e.base,
            TimelineElement::Audio(e) => e.base_mut(),
            TimelineElement::Effect(e) => &mut e.base,
        }
    }

    pub fn element_type_str(&self) -> &str {
        match self {
            TimelineElement::Video(_) => "video",
            TimelineElement::Image(_) => "image",
            TimelineElement::Text(_) => "text",
            TimelineElement::Sticker(_) => "sticker",
            TimelineElement::Graphic(_) => "graphic",
            TimelineElement::Audio(_) => "audio",
            TimelineElement::Effect(_) => "effect",
        }
    }

    pub fn is_visual(&self) -> bool {
        matches!(
            self,
            TimelineElement::Video(_)
                | TimelineElement::Image(_)
                | TimelineElement::Text(_)
                | TimelineElement::Sticker(_)
                | TimelineElement::Graphic(_)
        )
    }

    pub fn is_mutable(&self) -> bool {
        matches!(
            self,
            TimelineElement::Video(_)
                | TimelineElement::Image(_)
                | TimelineElement::Graphic(_)
        )
    }

    pub fn is_retimable(&self) -> bool {
        matches!(self, TimelineElement::Video(_) | TimelineElement::Audio(_))
    }
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoElementData {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub media_id: String,
    pub is_source_audio_enabled: Option<bool>,
    pub hidden: Option<bool>,
    pub retime: Option<RetimeConfig>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageElementData {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub media_id: String,
    pub hidden: Option<bool>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextElementData {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub hidden: Option<bool>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StickerElementData {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub sticker_id: String,
    pub intrinsic_width: Option<f64>,
    pub intrinsic_height: Option<f64>,
    pub hidden: Option<bool>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphicElementData {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub definition_id: String,
    pub hidden: Option<bool>,
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "sourceType", rename_all = "camelCase")]
pub enum AudioElementData {
    #[serde(rename = "upload")]
    Upload {
        #[serde(flatten)]
        base: BaseTimelineElement,
        media_id: String,
        retime: Option<RetimeConfig>,
    },
    #[serde(rename = "library")]
    Library {
        #[serde(flatten)]
        base: BaseTimelineElement,
        source_url: String,
        retime: Option<RetimeConfig>,
    },
}

impl AudioElementData {
    pub fn base(&self) -> &BaseTimelineElement {
        match self {
            AudioElementData::Upload { base, .. } => base,
            AudioElementData::Library { base, .. } => base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseTimelineElement {
        match self {
            AudioElementData::Upload { base, .. } => base,
            AudioElementData::Library { base, .. } => base,
        }
    }
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EffectElementData {
    #[serde(flatten)]
    pub base: BaseTimelineElement,
    pub effect_type: String,
}

"""Core data models for the auto-editing system.

All workers import types from this module. No circular dependencies allowed.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Literal, Optional
from enum import Enum


# ─── Enums ────────────────────────────────────────────────────────

class AspectRatio(str, Enum):
    RATIO_16_9 = "16:9"
    RATIO_9_16 = "9:16"
    RATIO_1_1 = "1:1"
    RATIO_4_3 = "4:3"
    RATIO_21_9 = "21:9"

class EditingStyle(str, Enum):
    CINEMATIC = "cinematic"
    VLOG = "vlog"
    TUTORIAL = "tutorial"
    PRODUCT = "product"
    MUSIC = "music"
    PRESENTATION = "presentation"
    CUSTOM = "custom"

class Mood(str, Enum):
    PROFESSIONAL = "professional"
    FUN = "fun"
    SERIOUS = "serious"
    EMOTIONAL = "emotional"
    ENERGETIC = "energetic"
    CALM = "calm"

class Platform(str, Enum):
    TIKTOK = "tiktok"
    YOUTUBE = "youtube"
    INSTAGRAM = "instagram"
    REELS = "reels"
    SHORTS = "shorts"
    LINKEDIN = "linkedin"
    CUSTOM = "custom"

class EditingIntent(str, Enum):
    AUTO_EDIT = "auto_edit"
    ADD_VOICEOVER = "add_voiceover"
    ADD_SUBTITLE = "add_subtitle"
    TRIM = "trim"
    CHANGE_LAYOUT = "change_layout"
    ADD_EFFECTS = "add_effects"
    RENDER = "render"
    BATCH_RENDER = "batch_render"
    UNKNOWN = "unknown"

class SceneType(str, Enum):
    ESTABLISHING = "establishing"
    PRODUCT = "product"
    TALKING_HEAD = "talking_head"
    B_ROLL = "b_roll"
    TRANSITION = "transition"
    DETAIL = "detail"
    CLOSEUP = "closeup"
    WIDE = "wide"
    MONTAGE = "montage"

class TransitionType(str, Enum):
    CROSSFADE = "crossfade"
    DIP_TO_BLACK = "dip_to_black"
    HARD_CUT = "hard_cut"
    FADE_IN = "fade_in"
    FADE_OUT = "fade_out"
    SLIDE = "slide"
    ZOOM = "zoom"
    BLUR = "blur"

class Mode(str, Enum):
    OFFLINE = "offline"
    HYBRID = "hybrid"
    CLOUD = "cloud"

class ConfidenceSource(str, Enum):
    RULE_ENGINE = "rule_engine"
    LOCAL_LLM = "local_llm"
    API_LLM = "api_llm"


# ─── Position & Transform ─────────────────────────────────────────

@dataclass
class Position:
    """Position in normalized canvas space (0.0 - 1.0)."""
    x: float = 0.5
    y: float = 0.5
    z: int = 0

@dataclass
class Size:
    """Element size in normalized space."""
    width: float = 0.5
    height: float = 0.5
    unit: Literal["normalized", "pixel", "percent"] = "normalized"

@dataclass
class Timeline:
    """When element appears/disappears in seconds."""
    start: float = 0.0
    end: float = 10.0

    @property
    def duration(self) -> float:
        return self.end - self.start

@dataclass
class Transform:
    """Visual transformation."""
    rotation: float = 0.0
    scale: float = 1.0
    opacity: float = 1.0
    anchor: Literal["center", "top_left", "top_right", "bottom_left", "bottom_right"] = "center"

@dataclass
class Keyframe:
    """Single keyframe for animation."""
    time: float
    x: Optional[float] = None
    y: Optional[float] = None
    scale: Optional[float] = None
    opacity: Optional[float] = None
    rotation: Optional[float] = None

@dataclass
class Animation:
    """Element animation specification."""
    keyframes: list[Keyframe] = field(default_factory=list)
    easing: Literal["linear", "ease_in", "ease_out", "ease_in_out", "bounce"] = "ease_in_out"


# ─── Visual Styles ────────────────────────────────────────────────

@dataclass
class TextStyle:
    """Style properties for text elements."""
    text: str = ""
    font_family: str = "Inter"
    font_size: int = 48
    font_weight: int = 400
    color: str = "#FFFFFF"
    text_align: Literal["left", "center", "right"] = "center"
    line_height: float = 1.2
    letter_spacing: float = 0.0
    background_color: Optional[str] = None
    border_radius: int = 0
    shadow: Optional[dict] = None

@dataclass
class VideoStyle:
    """Style properties for video/image elements."""
    fit: Literal["cover", "contain", "fill", "none"] = "cover"
    crop: Optional[dict] = None
    flip_horizontal: bool = False
    flip_vertical: bool = False
    border_radius: int = 0
    border: Optional[dict] = None
    shadow: Optional[dict] = None

@dataclass
class ShapeStyle:
    """Style properties for shape elements."""
    background_color: str = "#000000"
    border_radius: int = 0
    border: Optional[dict] = None
    gradient: Optional[dict] = None


# ─── Elements ─────────────────────────────────────────────────────

@dataclass
class Effect:
    """Single video/audio effect."""
    type: str
    params: dict = field(default_factory=dict)
    intensity: float = 1.0

@dataclass
class CoordinateElement:
    """Single visual element positioned in 4D space (x,y,z,t)."""
    id: str
    type: Literal["video", "image", "text", "shape", "effect"]
    position: Position = field(default_factory=Position)
    size: Size = field(default_factory=Size)
    timeline: Timeline = field(default_factory=Timeline)
    transform: Transform = field(default_factory=Transform)
    animation: Optional[Animation] = None
    effects: list[Effect] = field(default_factory=list)
    text_style: Optional[TextStyle] = None
    video_style: Optional[VideoStyle] = None
    shape_style: Optional[ShapeStyle] = None


# ─── Audio ────────────────────────────────────────────────────────

@dataclass
class VoiceoverSegment:
    """Single segment of voiceover audio."""
    text: str
    start: float
    end: float
    audio_path: Optional[str] = None

@dataclass
class VoiceoverConfig:
    """Configuration for voiceover generation."""
    language: str = "id"
    voice: str = "default"
    speed: float = 1.0
    pitch: float = 1.0
    style: str = "narasi_tenang"
    script: Optional[str] = None
    segments: list[VoiceoverSegment] = field(default_factory=list)

@dataclass
class AudioConfig:
    """Background audio configuration."""
    music_style: Optional[str] = None
    music_path: Optional[str] = None
    music_volume: float = 0.3
    voiceover_volume: float = 1.0
    sound_effects: list[str] = field(default_factory=list)


# ─── Scene & Plan ─────────────────────────────────────────────────

@dataclass
class Scene:
    """Single scene in the storyboard."""
    id: int
    scene_type: SceneType = SceneType.B_ROLL
    duration: float = 5.0
    source: str = "auto_find"
    source_keywords: list[str] = field(default_factory=list)
    layout: Optional[CoordinateElement] = None
    voiceover_segment: Optional[VoiceoverSegment] = None
    transition_in: TransitionType = TransitionType.HARD_CUT
    transition_out: TransitionType = TransitionType.HARD_CUT
    color_grade: Optional[str] = None

@dataclass
class EffectsConfig:
    """Effects configuration for the entire edit."""
    color_grade_preset: Optional[str] = None
    auto_color_grade: bool = True
    transitions: bool = True
    text_overlays: bool = True
    subtitles: bool = True

@dataclass
class EditingPlan:
    """Complete editing plan — output from brief analysis."""
    intent: EditingIntent = EditingIntent.AUTO_EDIT
    duration: int = 30
    aspect_ratio: AspectRatio = AspectRatio.RATIO_16_9
    style: EditingStyle = EditingStyle.CINEMATIC
    mood: Mood = Mood.PROFESSIONAL
    target_platform: Platform = Platform.YOUTUBE
    voiceover: Optional[VoiceoverConfig] = None
    scenes: list[Scene] = field(default_factory=list)
    audio: AudioConfig = field(default_factory=AudioConfig)
    effects: EffectsConfig = field(default_factory=EffectsConfig)
    template_name: Optional[str] = None


# ─── Workflow & Results ───────────────────────────────────────────

@dataclass
class TokenUsage:
    """Token usage tracking."""
    local_llm: int = 0
    api_llm: int = 0
    total: int = 0

    def add_local(self, tokens: int) -> None:
        self.local_llm += tokens
        self.total += tokens

    def add_api(self, tokens: int) -> None:
        self.api_llm += tokens
        self.total += tokens

@dataclass
class EditError:
    """Single error that occurred during editing."""
    node_id: str
    error_type: str
    message: str
    recoverable: bool = True
    recovery_action: Optional[str] = None

@dataclass
class WorkflowResult:
    """Result of a workflow execution."""
    success: bool
    output_path: Optional[str] = None
    token_usage: TokenUsage = field(default_factory=TokenUsage)
    errors: list[EditError] = field(default_factory=list)
    quality_score: float = 1.0
    processing_time: float = 0.0


# ─── Configuration ────────────────────────────────────────────────

@dataclass
class LocalConfig:
    """Local AI model configuration."""
    llm_model: str = "moko/MOKO-AI-4B-Q3_K_M.gguf"
    tts_model: str = "cosyvoice/CosyVoice-300M.gguf"
    asr_model: str = "whisper/whisper-small.gguf"
    models_dir: str = "./models/"

@dataclass
class APIConfig:
    """External API configuration."""
    llm_provider: str = "openai"
    llm_model: str = "gpt-4o-mini"
    llm_max_tokens: int = 2000
    pexels_api_key: str = ""
    pixabay_api_key: str = ""

@dataclass
class ResourceConfig:
    """Hardware resource limits."""
    max_vram_gb: int = 4
    max_ram_gb: int = 8
    max_threads: int = 4

@dataclass
class BehaviorConfig:
    """Behavioral settings."""
    confidence_threshold: float = 0.7
    max_retries: int = 2
    cache_enabled: bool = True
    cache_ttl_minutes: int = 60

@dataclass
class AutoEditorConfig:
    """Root configuration dataclass."""
    mode: Mode = Mode.HYBRID
    local: LocalConfig = field(default_factory=LocalConfig)
    api: APIConfig = field(default_factory=APIConfig)
    resources: ResourceConfig = field(default_factory=ResourceConfig)
    behavior: BehaviorConfig = field(default_factory=BehaviorConfig)

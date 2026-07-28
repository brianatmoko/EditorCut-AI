# AGENT 1: FOUNDATION & CORE INFRASTRUCTURE

> **Peran:** Membangun fondasi yang akan dipakai Agent 2 dan Agent 3.
> **Lingkup kerja:** `Opencut/auto-editor/` + OpenCut clone
> **Kamu tidak perlu melakukan pekerjaan Agent 2 (worker) dan Agent 3 (integration).**
> Fokus hanya pada task di bawah — selesai 100% baru lanjut ke task berikutnya.

---

## PENTING — Aturan Main

```
1. JANGAN lompat ke task berikutnya sebelum task sebelumnya selesai 100%.
2. Setiap file yang kamu buat HARUS:
   - Type hints wajib di semua fungsi
   - Docstring minimalis (1 baris) menjelaskan WHAT, bukan HOW
   - Tidak ada komentar kode (kecuali TODO yang memang belum diimplementasi)
   - Error handling: graceful degradation (jangan raise Exception mentah)
3. Semua path RELATIVE ke `Opencut/` (root proyek ini).
4. Format kode: Python 3.12, gunakan `@dataclass` untuk data, `Protocol` untuk interface.
```

---

## Task 1.1 — Clone OpenCut Classic

### Instruksi

Clone repo OpenCut classic ke folder saat ini (`/home/brianatmokoo/Documents/Linux/Opencut`).

### Detail

```bash
# Hapus isi folder Opencut (kecuali docs/ dan file agent*.md)
# Clone repositori
git clone https://github.com/OpenCut-app/OpenCut.git .
# Atau jika repo classic terpisah, clone opencut-app/opencut-classic

# Install dependencies
bun install

# Verifikasi bisa jalan
bun run dev:web
```

### Verifikasi

- `localhost:5173` muncul dengan interface OpenCut
- Folder `apps/web/` terisi lengkap
- File `package.json` ada di root

### Jika Gagal

Jika repo tidak bisa di-clone (salah URL, dll), buat laporan error dan tawarkan
alternatif: clone dari fork terpercaya, atau setup manual dari source.

---

## Task 1.2 — Setup Struktur `auto-editor/`

### Instruksi

Buat folder proyek `auto-editor/` di root `Opencut/`. Setiap file dan folder
wajib ada — jangan dilewatkan.

### Struktur Lengkap

```
Opencut/
└── auto-editor/
    ├── __init__.py                    # Expose public API
    ├── models.py                      # Semua dataclass
    ├── main.py                        # CLI entry point
    │
    ├── orchestrator/                  # JANTUNG SISTEM — paling penting
    │   ├── __init__.py
    │   ├── intent_router.py           # Klasifikasi perintah
    │   ├── mandor_llm.py              # Bridge ke local LLM
    │   ├── workflow_engine.py         # Pipeline DAG executor
    │   └── template_db.py             # Layout template manager
    │
    ├── workers/                       # Skeleton — Agent 2 yang isi
    │   ├── __init__.py
    │   ├── scene_detector/
    │   │   ├── __init__.py
    │   │   └── detector.py           # Stub: return NotImplementedError
    │   ├── asset_finder/
    │   │   ├── __init__.py
    │   │   ├── crawler.py            # Stub
    │   │   ├── rag_search.py         # Stub
    │   │   └── downloader.py         # Stub
    │   ├── layout_engine/
    │   │   ├── __init__.py
    │   │   ├── coordinate.py         # Implementasi penuh (0 token math)
    │   │   ├── compositor.py         # Stub
    │   │   └── template.py           # Stub
    │   ├── audio_pipeline/
    │   │   ├── __init__.py
    │   │   ├── tts_engine.py         # Stub
    │   │   ├── asr_whisper.py        # Stub
    │   │   ├── alignment.py          # Stub
    │   │   └── mixer.py              # Stub
    │   ├── effects/
    │   │   ├── __init__.py
    │   │   ├── color_grade.py        # Stub
    │   │   ├── transition.py         # Stub
    │   │   └── text_overlay.py       # Stub
    │   └── renderer/
    │       ├── __init__.py
    │       ├── opencut_bridge.py     # Stub
    │       └── ffmpeg_pipeline.py    # Stub
    │
    ├── api/                           # Skeleton — Agent 3 yang isi
    │   ├── __init__.py
    │   ├── routes.py                 # Stub
    │   └── websocket.py              # Stub
    │
    ├── config/
    │   ├── __init__.py
    │   ├── settings.yaml             # Konfigurasi global
    │   ├── settings_loader.py        # YAML → dataclass
    │   └── templates/                # Folder template layout
    │       ├── cinematic.yaml        # Template cinematic
    │       ├── tiktok_product.yaml   # Template TikTok product
    │       └── slideshow.yaml        # Template slideshow
    │
    └── tests/
        ├── __init__.py
        ├── test_intent_router.py
        ├── test_workflow_engine.py
        ├── test_template_db.py
        ├── test_models.py
        └── test_coordinate.py
```

### Aturan Pembuatan Stub

Setiap stub worker minimal berisi:

```python
"""1 line description of what this worker does."""

from typing import Protocol

class SceneDetector(Protocol):
    """Interface for scene detection workers."""
    
    def detect(self, video_path: str) -> list[dict]:
        """Detect scenes. Returns list of {start, end, type}."""
        raise NotImplementedError("Agent 2 will implement this")

def create_scene_detector() -> SceneDetector:
    """Factory function — returns SceneDetector instance."""
    raise NotImplementedError("Agent 2 will implement this")
```

Gunakan `Protocol` dari `typing` untuk mendefinisikan interface setiap worker.
Semua stub WAJIB pakai Protocol agar Agent 2 tahu kontrak yang harus dipenuhi.

---

## Task 1.3 — Core Data Models

### Instruksi

Buat `auto-editor/models.py`. File ini berisi **semua data class** yang dipakai
lintas komponen. Setiap worker akan import dari sini.

### Implementasi Lengkap

```python
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
    OFFLINE = "offline"    # 0 token, pure lokal
    HYBRID = "hybrid"      # lokal + API quality boost
    CLOUD = "cloud"        # full API

class ConfidenceSource(str, Enum):
    RULE_ENGINE = "rule_engine"   # Selalu reliable
    LOCAL_LLM = "local_llm"       # Threshold 0.7
    API_LLM = "api_llm"          # Threshold 0.9


# ─── Position & Transform ─────────────────────────────────────────

@dataclass
class Position:
    """Position in normalized canvas space (0.0 - 1.0)."""
    x: float = 0.5          # 0=left, 0.5=center, 1.0=right
    y: float = 0.5          # 0=top, 0.5=center, 1.0=bottom
    z: int = 0              # layer stacking (0=background)

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
    rotation: float = 0.0       # degrees (-360 to 360)
    scale: float = 1.0          # 0.0 to 10.0
    opacity: float = 1.0        # 0.0 to 1.0
    anchor: Literal["center", "top_left", "top_right", "bottom_left", "bottom_right"] = "center"

@dataclass
class Keyframe:
    """Single keyframe for animation."""
    time: float                 # seconds (relative to element start)
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
    font_weight: int = 400       # 100-900
    color: str = "#FFFFFF"
    text_align: Literal["left", "center", "right"] = "center"
    line_height: float = 1.2
    letter_spacing: float = 0.0
    background_color: Optional[str] = None
    border_radius: int = 0
    shadow: Optional[dict] = None  # {offset: [x,y], blur: int, color: str}

@dataclass
class VideoStyle:
    """Style properties for video/image elements."""
    fit: Literal["cover", "contain", "fill", "none"] = "cover"
    crop: Optional[dict] = None    # {x, y, width, height}
    flip_horizontal: bool = False
    flip_vertical: bool = False
    border_radius: int = 0
    border: Optional[dict] = None  # {width: int, color: str}
    shadow: Optional[dict] = None

@dataclass
class ShapeStyle:
    """Style properties for shape elements."""
    background_color: str = "#000000"
    border_radius: int = 0
    border: Optional[dict] = None
    gradient: Optional[dict] = None  # {type, colors[], angle}


# ─── Elements ─────────────────────────────────────────────────────

@dataclass
class Effect:
    """Single video/audio effect."""
    type: str                       # "color_grade", "blur", "noise_reduction", etc
    params: dict = field(default_factory=dict)
    intensity: float = 1.0          # 0.0 - 1.0

@dataclass
class CoordinateElement:
    """Single visual element positioned in 4D space (x,y,z,t).
    
    This is the fundamental unit of the layout system.
    Every visual element in the final video is represented by this.
    """
    id: str
    type: Literal["video", "image", "text", "shape", "effect"]
    position: Position = field(default_factory=Position)
    size: Size = field(default_factory=Size)
    timeline: Timeline = field(default_factory=Timeline)
    transform: Transform = field(default_factory=Transform)
    animation: Optional[Animation] = None
    effects: list[Effect] = field(default_factory=list)
    
    # Style (hanya satu yang relevan berdasarkan type)
    text_style: Optional[TextStyle] = None
    video_style: Optional[VideoStyle] = None
    shape_style: Optional[ShapeStyle] = None


# ─── Audio ────────────────────────────────────────────────────────

@dataclass
class VoiceoverSegment:
    """Single segment of voiceover audio."""
    text: str
    start: float                  # start time in seconds
    end: float                    # end time in seconds
    audio_path: Optional[str] = None  # path to generated audio file

@dataclass
class VoiceoverConfig:
    """Configuration for voiceover generation."""
    language: str = "id"          # language code
    voice: str = "default"        # voice profile name
    speed: float = 1.0            # 0.5 - 2.0
    pitch: float = 1.0            # 0.5 - 2.0
    style: str = "narasi_tenang"  # speaking style
    script: Optional[str] = None  # pre-written script (if None, AI generates)
    segments: list[VoiceoverSegment] = field(default_factory=list)

@dataclass
class AudioConfig:
    """Background audio configuration."""
    music_style: Optional[str] = None   # "cinematic", "upbeat", "calm"
    music_path: Optional[str] = None    # explicit music file
    music_volume: float = 0.3           # 0.0 - 1.0
    voiceover_volume: float = 1.0       # 0.0 - 1.0
    sound_effects: list[str] = field(default_factory=list)  # paths to SFX


# ─── Scene & Plan ─────────────────────────────────────────────────

@dataclass
class Scene:
    """Single scene in the storyboard."""
    id: int
    scene_type: SceneType = SceneType.B_ROLL
    duration: float = 5.0               # seconds
    source: str = "auto_find"           # "auto_find" | "user_upload" | file path
    source_keywords: list[str] = field(default_factory=list)  # for asset finding
    layout: Optional[CoordinateElement] = None
    voiceover_segment: Optional[VoiceoverSegment] = None
    transition_in: TransitionType = TransitionType.HARD_CUT
    transition_out: TransitionType = TransitionType.HARD_CUT
    color_grade: Optional[str] = None   # preset name or None

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
    """Complete editing plan — output from brief analysis.
    
    This is the central data structure that flows through all pipeline stages.
    Agent 1 (Mandor LLM) produces this.
    Agent 2 (Workers) consume this to produce final video.
    """
    intent: EditingIntent = EditingIntent.AUTO_EDIT
    duration: int = 30                  # total duration in seconds
    aspect_ratio: AspectRatio = AspectRatio.RATIO_16_9
    style: EditingStyle = EditingStyle.CINEMATIC
    mood: Mood = Mood.PROFESSIONAL
    target_platform: Platform = Platform.YOUTUBE
    voiceover: Optional[VoiceoverConfig] = None
    scenes: list[Scene] = field(default_factory=list)
    audio: AudioConfig = field(default_factory=AudioConfig)
    effects: EffectsConfig = field(default_factory=EffectsConfig)
    template_name: Optional[str] = None # specific template to use


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
    node_id: str            # which workflow node
    error_type: str         # "asset_not_found" | "tts_failed" | "render_error" | etc
    message: str
    recoverable: bool = True
    recovery_action: Optional[str] = None  # "fallback" | "retry" | "skip"

@dataclass
class WorkflowResult:
    """Result of a workflow execution."""
    success: bool
    output_path: Optional[str] = None
    token_usage: TokenUsage = field(default_factory=TokenUsage)
    errors: list[EditError] = field(default_factory=list)
    quality_score: float = 1.0          # 0.0 - 1.0
    processing_time: float = 0.0        # seconds


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
```

### Aturan

1. Jangan tambahkan method atau logic bisnis di dataclass — ini murni data.
2. Satu-satunya pengecualian: properties sederhana (seperti `duration`).
3. Import models.py dari file mana pun TIDAK boleh menyebabkan circular import.
4. Setiap Enum punya member `CUSTOM` sebagai fallback.

---

## Task 1.4 — Intent Router

### Instruksi

Buat `auto-editor/orchestrator/intent_router.py`.

Router mengklasifikasikan perintah user menggunakan **rule-based pattern matching**.
Tidak perlu LLM — ini murni string matching yang cepat dan 0 token.

### Implementasi

```python
"""Classify user editing commands using rule-based pattern matching.

Rule-based = 0 token cost. Falls back to UNKNOWN intent when no pattern matches.
"""

from __future__ import annotations
import re
from typing import Optional
from ..models import EditingIntent, EditingPlan, EditingStyle, Platform


# Pattern definitions: list of (compiled_regex, intent, param_extractor)
# Urutan penting — pattern lebih spesifik diletakkan di atas
_INTENT_PATTERNS = [
    # AUTO_EDIT — perintah membuat video baru
    (re.compile(
        r'\b(buat|bikin|buatkan|hasilkan|create|make|generate)\s.*\b(video|konten|content)\b',
        re.IGNORECASE
    ), EditingIntent.AUTO_EDIT, _extract_auto_edit_params),
    
    # AUTO_EDIT — versi pendek: "video [durasi] [style]"
    (re.compile(
        r'\bvideo\b.*\b(\d+)\s*(detik|second|menit|minute|min)\b',
        re.IGNORECASE
    ), EditingIntent.AUTO_EDIT, _extract_auto_edit_params),
    
    # ADD_VOICEOVER
    (re.compile(
        r'\b(voiceover|narasi|dubbing|suara|audio|voice)\b',
        re.IGNORECASE
    ), EditingIntent.ADD_VOICEOVER, _extract_voiceover_params),
    
    # ADD_SUBTITLE
    (re.compile(
        r'\b(subtitle|teks|caption|takarir|terjemahan|srt)\b',
        re.IGNORECASE
    ), EditingIntent.ADD_SUBTITLE, _extract_subtitle_params),
    
    # TRIM
    (re.compile(
        r'\b(potong|trim|cut|hapus|remove|buang)\b',
        re.IGNORECASE
    ), EditingIntent.TRIM, _extract_trim_params),
    
    # CHANGE_LAYOUT
    (re.compile(
        r'\b(layout|tata letak|posisi|template|templat|susun|atur|arrange)\b',
        re.IGNORECASE
    ), EditingIntent.CHANGE_LAYOUT, _extract_layout_params),
    
    # ADD_EFFECTS
    (re.compile(
        r'\b(efek|filter|transisi|color|warna|grading|effect)\b',
        re.IGNORECASE
    ), EditingIntent.ADD_EFFECTS, _extract_effects_params),
    
    # RENDER
    (re.compile(
        r'\b(render|export|simpan|download|save|publikasi|publish)\b',
        re.IGNORECASE
    ), EditingIntent.RENDER, _extract_render_params),
    
    # BATCH_RENDER
    (re.compile(
        r'\b(batch|semua|all|massal|render\s+semua)\b',
        re.IGNORECASE
    ), EditingIntent.BATCH_RENDER, _extract_batch_params),
]


class IntentRouter:
    """Route user commands to the correct editing intent using pattern matching."""
    
    def classify(self, query: str) -> tuple[EditingIntent, dict]:
        """Classify query and extract parameters.
        
        Args:
            query: Raw user input string.
            
        Returns:
            Tuple of (intent, extracted_params).
            Params is empty dict if no pattern matches.
        """
        if not query or not query.strip():
            return EditingIntent.UNKNOWN, {}
        
        for pattern, intent, extractor in _INTENT_PATTERNS:
            match = pattern.search(query)
            if match:
                params = extractor(match, query)
                return intent, params
        
        return EditingIntent.UNKNOWN, {}
    
    def extract_duration(self, query: str) -> Optional[int]:
        """Extract duration in seconds from query."""
        patterns = [
            r'(\d+)\s*(detik|second|s\b)',
            r'(\d+)\s*(menit|minute|min|m\b)',
            r'durasi?\s*(\d+)',
        ]
        for pat in patterns:
            match = re.search(pat, query, re.IGNORECASE)
            if match:
                val = int(match.group(1))
                unit = match.group(2).lower() if len(match.groups()) > 1 else 'detik'
                if unit in ('menit', 'minute', 'min', 'm'):
                    val *= 60
                return val
        return None
    
    def extract_style(self, query: str) -> Optional[str]:
        """Extract editing style from query."""
        style_keywords = {
            'cinematic': ['cinematic', 'film', 'movie', 'sinematik'],
            'vlog': ['vlog', 'daily', 'harian'],
            'tutorial': ['tutorial', 'guide', 'panduan'],
            'product': ['product', 'produk', 'review', 'unboxing'],
            'music': ['music', 'musik', 'lyric', 'lirik'],
        }
        query_lower = query.lower()
        for style, keywords in style_keywords.items():
            if any(kw in query_lower for kw in keywords):
                return style
        return None
    
    def extract_platform(self, query: str) -> Optional[str]:
        """Extract target platform from query."""
        platform_keywords = {
            'tiktok': ['tiktok', 'tk'],
            'youtube': ['youtube', 'yt'],
            'instagram': ['instagram', 'ig', 'reels'],
            'shorts': ['shorts', 'short'],
        }
        query_lower = query.lower()
        for platform, keywords in platform_keywords.items():
            if any(kw in query_lower for kw in keywords):
                return platform
        return None
    
    def extract_aspect_ratio(self, query: str) -> Optional[str]:
        """Extract aspect ratio from query."""
        ratio_patterns = [
            (r'16\s*[:\/]\s*9', '16:9'),
            (r'9\s*[:\/]\s*16', '9:16'),
            (r'1\s*[:\/]\s*1', '1:1'),
            (r'4\s*[:\/]\s*3', '4:3'),
            (r'21\s*[:\/]\s*9', '21:9'),
            (r'\b(vertikal|vertical|potrait|tiktok)\b', '9:16'),
            (r'\b(horizontal|landscape|youtube|cinematic)\b', '16:9'),
            (r'\b(persegi|square|instagram)\b', '1:1'),
        ]
        query_lower = query.lower()
        for pat, ratio in ratio_patterns:
            if re.search(pat, query_lower):
                return ratio
        return None
    
    def create_plan(self, query: str) -> EditingPlan:
        """Full analysis: classify + extract all params → EditingPlan."""
        intent, _ = self.classify(query)
        
        plan = EditingPlan(
            intent=intent,
            duration=self.extract_duration(query) or 30,
            style=self._parse_style(self.extract_style(query)),
            target_platform=self._parse_platform(self.extract_platform(query)),
        )
        
        # Set aspect ratio based on platform
        ratio = self.extract_aspect_ratio(query)
        if ratio:
            plan.aspect_ratio = self._parse_aspect_ratio(ratio)
        elif plan.target_platform in (Platform.TIKTOK, Platform.REELS, Platform.SHORTS):
            plan.aspect_ratio = AspectRatio.RATIO_9_16
        elif plan.target_platform == Platform.INSTAGRAM:
            plan.aspect_ratio = AspectRatio.RATIO_1_1
        else:
            plan.aspect_ratio = AspectRatio.RATIO_16_9
        
        return plan
    
    def _parse_style(self, style: Optional[str]) -> EditingStyle:
        mapping = {
            'cinematic': EditingStyle.CINEMATIC,
            'vlog': EditingStyle.VLOG,
            'tutorial': EditingStyle.TUTORIAL,
            'product': EditingStyle.PRODUCT,
            'music': EditingStyle.MUSIC,
        }
        return mapping.get(style, EditingStyle.CUSTOM)
    
    def _parse_platform(self, platform: Optional[str]) -> Platform:
        mapping = {
            'tiktok': Platform.TIKTOK,
            'youtube': Platform.YOUTUBE,
            'instagram': Platform.INSTAGRAM,
            'shorts': Platform.SHORTS,
        }
        return mapping.get(platform, Platform.CUSTOM)
    
    def _parse_aspect_ratio(self, ratio: str) -> AspectRatio:
        mapping = {
            '16:9': AspectRatio.RATIO_16_9,
            '9:16': AspectRatio.RATIO_9_16,
            '1:1': AspectRatio.RATIO_1_1,
            '4:3': AspectRatio.RATIO_4_3,
            '21:9': AspectRatio.RATIO_21_9,
        }
        return mapping.get(ratio, AspectRatio.RATIO_16_9)


# ─── Parameter Extractors ─────────────────────────────────────────

def _extract_auto_edit_params(match: re.Match, query: str) -> dict:
    """Extract parameters for auto-edit command."""
    return {
        "has_duration": bool(re.search(r'\d+\s*(detik|second|menit)', query, re.IGNORECASE)),
        "has_style": bool(re.search(r'(cinematic|vlog|tutorial|product)', query, re.IGNORECASE)),
    }

def _extract_voiceover_params(match: re.Match, query: str) -> dict:
    """Extract parameters for voiceover command."""
    lang = "id"
    if re.search(r'\b(english|inggris|en\b)', query, re.IGNORECASE):
        lang = "en"
    return {"language": lang}

def _extract_subtitle_params(match: re.Match, query: str) -> dict:
    """Extract parameters for subtitle command."""
    lang = "id"
    if re.search(r'\b(english|inggris|en\b)', query, re.IGNORECASE):
        lang = "en"
    return {"language": lang}

def _extract_trim_params(match: re.Match, query: str) -> dict:
    """Extract parameters for trim command."""
    return {}

def _extract_layout_params(match: re.Match, query: str) -> dict:
    """Extract parameters for layout command."""
    return {}

def _extract_effects_params(match: re.Match, query: str) -> dict:
    """Extract parameters for effects command."""
    return {}

def _extract_render_params(match: re.Match, query: str) -> dict:
    """Extract parameters for render command."""
    return {}

def _extract_batch_params(match: re.Match, query: str) -> dict:
    """Extract parameters for batch render command."""
    return {}
```

### Verifikasi

```python
router = IntentRouter()

# Test cases yang HARUS lolos:
assert router.classify("buat video cinematic 30 detik")[0] == EditingIntent.AUTO_EDIT
assert router.classify("bikin video produk")[0] == EditingIntent.AUTO_EDIT
assert router.classify("tambah voiceover")[0] == EditingIntent.ADD_VOICEOVER
assert router.classify("buat subtitle")[0] == EditingIntent.ADD_SUBTITLE
assert router.classify("render semua")[0] == EditingIntent.BATCH_RENDER
assert router.classify("apa kabar")[0] == EditingIntent.UNKNOWN
assert router.extract_duration("30 detik") == 30
assert router.extract_duration("2 menit") == 120
```

---

## Task 1.5 — Mandor LLM Bridge

### Instruksi

Buat `auto-editor/orchestrator/mandor_llm.py`.

Ini adalah **interface ke local LLM** (MOKO-4B nantinya).
Sekarang buat **mock implementation** yang return data dummy tapi strukturnya benar.
Agent 3 nanti yang akan menghubungkan ini ke MOKO OS yang sesungguhnya.

### Implementasi

```python
"""Bridge to local LLM (Mandor AI).

This module provides the interface for communicating with the local LLM.
For now, uses a mock implementation. Agent 3 will integrate with real MOKO-4B.

Key responsibilities:
1. analyze_brief → structured EditingPlan from user query
2. generate_script → voiceover script text from plan
3. storyboard → scene breakdown from plan + assets
4. review → quality check on results
5. refine → fix instructions for identified issues
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, Callable
import json
import time

from ..models import (
    EditingPlan, EditingIntent, Scene, SceneType, TransitionType,
    VoiceoverConfig, VoiceoverSegment,
    TokenUsage, EditError, WorkflowResult,
    Mode, ConfidenceSource,
)


@dataclass
class Decision:
    """Single decision from the LLM with confidence scoring."""
    content: dict
    confidence: float                # 0.0 - 1.0
    source: ConfidenceSource
    token_cost: int = 0
    reasoning: Optional[str] = None  # chain-of-thought (if available)

    def is_reliable(self) -> bool:
        thresholds = {
            ConfidenceSource.RULE_ENGINE: 0.0,   # always reliable
            ConfidenceSource.LOCAL_LLM: 0.7,
            ConfidenceSource.API_LLM: 0.9,
        }
        return self.confidence >= thresholds.get(self.source, 0.7)


class MandorLLM:
    """Interface to local LLM for editing decisions.
    
    Current: mock implementation returning structured dummy data.
    Future: bridges to MOKO-4B via moko_bridge (Agent 3).
    """
    
    def __init__(self, mode: Mode = Mode.HYBRID):
        self.mode = mode
        self.token_usage = TokenUsage()
        self._confidence = 0.75  # simulated confidence
    
    def analyze_brief(self, query: str, context: Optional[dict] = None) -> Decision:
        """Analyze user query → structured EditingPlan.
        
        Args:
            query: Raw user input / editing brief.
            context: Optional additional context (previous edits, etc).
            
        Returns:
            Decision containing EditingPlan-compatible dict.
        """
        # TODO: Agent 3 — replace with real MOKO-4B call
        # For now: return structured mock data
        plan = {
            "intent": "auto_edit",
            "duration": 30,
            "style": "cinematic",
            "mood": "professional",
            "target_platform": "youtube",
            "voiceover": {
                "language": "id",
                "voice": "default",
                "speed": 1.0,
                "pitch": 1.0,
                "style": "narasi_tenang"
            },
            "scenes": [
                {
                    "id": 1,
                    "scene_type": "establishing",
                    "duration": 8.0,
                    "source": "auto_find",
                    "source_keywords": ["establishing", "wide", context.get("topic", "general")],
                    "transition_in": "hard_cut",
                    "transition_out": "crossfade"
                },
                {
                    "id": 2,
                    "scene_type": "product",
                    "duration": 12.0,
                    "source": "auto_find",
                    "source_keywords": [context.get("topic", "product"), "detail"],
                    "transition_in": "crossfade",
                    "transition_out": "crossfade"
                },
                {
                    "id": 3,
                    "scene_type": "b_roll",
                    "duration": 10.0,
                    "source": "auto_find",
                    "source_keywords": ["action", context.get("topic", "process")],
                    "transition_in": "crossfade",
                    "transition_out": "fade_out"
                }
            ]
        }
        
        cost = 500  # simulated token cost
        self.token_usage.add_local(cost)
        
        return Decision(
            content=plan,
            confidence=self._confidence,
            source=ConfidenceSource.LOCAL_LLM,
            token_cost=cost,
            reasoning="Extracted editing parameters from user query: "
                      f"intent=auto_edit, duration=30s, style=cinematic"
        )
    
    def generate_script(self, plan: EditingPlan, topic: str = "") -> Decision:
        """Generate voiceover script from editing plan.
        
        Args:
            plan: The editing plan to generate script for.
            topic: Optional topic to guide script content.
            
        Returns:
            Decision containing script text and per-scene segments.
        """
        # TODO: Agent 3 — replace with real MOKO-4B call
        segments = []
        for scene in plan.scenes:
            segments.append({
                "scene_id": scene.id,
                "text": f"[Script for scene {scene.id}: {topic or 'content'}]",
                "start": sum(s.duration for s in plan.scenes[:scene.id - 1]),
                "end": sum(s.duration for s in plan.scenes[:scene.id]),
            })
        
        full_script = " ".join(s["text"] for s in segments)
        
        cost = 800
        self.token_usage.add_local(cost)
        
        return Decision(
            content={
                "full_script": full_script,
                "segments": segments,
                "language": plan.voiceover.language if plan.voiceover else "id"
            },
            confidence=self._confidence,
            source=ConfidenceSource.LOCAL_LLM,
            token_cost=cost
        )
    
    def storyboard(self, plan: EditingPlan, assets: list[dict]) -> Decision:
        """Generate detailed scene layout from plan + available assets.
        
        Args:
            plan: Editing plan with scene definitions.
            assets: List of available assets found by AssetFinder.
            
        Returns:
            Decision containing enhanced scenes with asset assignments.
        """
        # TODO: Agent 3 — replace with real MOKO-4B call
        scenes = []
        for i, scene in enumerate(plan.scenes):
            # Match asset to scene
            matching_assets = [
                a for a in assets
                if any(kw in a.get("keywords", []) for kw in scene.source_keywords)
            ]
            best_asset = matching_assets[0] if matching_assets else None
            
            scenes.append({
                "scene_id": scene.id,
                "asset_url": best_asset.get("url") if best_asset else None,
                "asset_confidence": 0.85 if best_asset else 0.0,
                "suggested_duration": scene.duration,
                "notes": f"Scene {i+1}: {scene.scene_type.value}"
            })
        
        cost = 600
        self.token_usage.add_local(cost)
        
        return Decision(
            content={"scenes": scenes},
            confidence=self._confidence,
            source=ConfidenceSource.LOCAL_LLM,
            token_cost=cost
        )
    
    def review(self, result: WorkflowResult) -> Decision:
        """Quality review of editing result.
        
        Args:
            result: The workflow result to review.
            
        Returns:
            Decision with quality assessment and issues.
        """
        # TODO: Agent 3 — replace with real MOKO-4B call
        issues = []
        for error in result.errors:
            issues.append({
                "severity": "error" if not error.recoverable else "warning",
                "node_id": error.node_id,
                "description": error.message,
                "fix_suggestion": error.recovery_action or "manual review needed"
            })
        
        if not issues:
            issues.append({
                "severity": "info",
                "description": "No issues detected",
                "fix_suggestion": None
            })
        
        cost = 300
        self.token_usage.add_local(cost)
        
        passed = len([i for i in issues if i["severity"] == "error"]) == 0
        
        return Decision(
            content={
                "passed": passed,
                "quality_score": result.quality_score,
                "issues": issues,
                "summary": f"Review {'passed' if passed else 'failed'} with {len(issues)} issues"
            },
            confidence=self._confidence,
            source=ConfidenceSource.LOCAL_LLM,
            token_cost=cost
        )
    
    def refine(self, issues: list[dict]) -> Decision:
        """Generate fix instructions from review issues.
        
        Args:
            issues: List of issues from review step.
            
        Returns:
            Decision with fix instructions for each issue.
        """
        # TODO: Agent 3 — replace with real MOKO-4B call
        fixes = []
        for issue in issues:
            if issue.get("fix_suggestion"):
                fixes.append({
                    "target": issue.get("node_id", "unknown"),
                    "action": issue["fix_suggestion"],
                    "priority": "high" if issue.get("severity") == "error" else "medium"
                })
        
        cost = 400
        self.token_usage.add_local(cost)
        
        return Decision(
            content={
                "fixes": fixes,
                "requires_re_review": len(fixes) > 0
            },
            confidence=self._confidence,
            source=ConfidenceSource.LOCAL_LLM,
            token_cost=cost
        )
    
    def reset_token_usage(self) -> None:
        """Reset accumulated token counter."""
        self.token_usage = TokenUsage()
    
    def get_token_usage(self) -> TokenUsage:
        """Get accumulated token usage."""
        return self.token_usage
```

### Verifikasi

```python
llm = MandorLLM()
result = llm.analyze_brief("buat video cinematic 30 detik")
assert result.is_reliable()
assert "intent" in result.content
assert "scenes" in result.content
assert result.token_cost > 0
```

---

## Task 1.6 — Workflow Engine

### Instruksi

Buat `auto-editor/orchestrator/workflow_engine.py`.

Ini adalah **DAG (Directed Acyclic Graph) executor** yang menjalankan pipeline editing.
Node bisa jalan paralel jika tidak punya dependency bersama.

### Implementasi

```python
"""DAG-based workflow engine for video editing pipelines.

Executes workflow nodes respecting dependency order.
Supports parallel execution for independent branches.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Any, Callable, Optional
from enum import Enum
import time
import threading
from collections import deque

from ..models import WorkflowResult, EditError, TokenUsage


class NodeStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class WorkflowNode:
    """Single node in the workflow DAG."""
    id: str
    handler: Callable[..., Any]
    deps: list[str] = field(default_factory=list)
    config: dict = field(default_factory=dict)
    retry_count: int = 2
    timeout: int = 300            # seconds
    status: NodeStatus = NodeStatus.PENDING
    result: Any = None
    error: Optional[str] = None
    
    @property
    def is_ready(self, completed: set[str]) -> bool:
        """Check if all dependencies are completed."""
        return all(dep in completed for dep in self.deps)


class WorkflowEngine:
    """Execute DAG-based workflows with parallel support.
    
    Usage:
        engine = WorkflowEngine()
        engine.register("my_workflow", [node1, node2, node3])
        result = engine.run("my_workflow", input_data={})
    """
    
    def __init__(self):
        self._workflows: dict[str, list[WorkflowNode]] = {}
        self._progress_callbacks: list[Callable] = []
    
    def register(self, name: str, nodes: list[WorkflowNode]) -> None:
        """Register a workflow by name.
        
        Validates:
        - No duplicate node IDs
        - All dependencies reference existing nodes
        - No circular dependencies
        """
        self._validate_workflow(name, nodes)
        self._workflows[name] = nodes
    
    def _validate_workflow(self, name: str, nodes: list[WorkflowNode]) -> None:
        """Validate workflow definition."""
        node_ids = {n.id for n in nodes}
        
        # Check duplicate IDs
        if len(node_ids) != len(nodes):
            ids = [n.id for n in nodes]
            duplicates = {id_ for id_ in ids if ids.count(id_) > 1}
            raise ValueError(f"Duplicate node IDs in '{name}': {duplicates}")
        
        # Check dependencies exist
        all_deps = set()
        for n in nodes:
            all_deps.update(n.deps)
        missing = all_deps - node_ids
        if missing:
            raise ValueError(f"Missing dependencies in '{name}': {missing}")
        
        # Check circular dependencies via DFS
        visited = set()
        path = []
        
        def dfs(node_id: str) -> None:
            if node_id in path:
                cycle_start = path[path.index(node_id):]
                raise ValueError(
                    f"Circular dependency in '{name}': {' → '.join(cycle_start + [node_id])}"
                )
            if node_id in visited:
                return
            visited.add(node_id)
            path.append(node_id)
            node = next(n for n in nodes if n.id == node_id)
            for dep in node.deps:
                dfs(dep)
            path.pop()
        
        for n in nodes:
            if n.id not in visited:
                dfs(n.id)
    
    def get_workflow(self, name: str) -> list[WorkflowNode]:
        """Get registered workflow nodes."""
        if name not in self._workflows:
            raise KeyError(f"Workflow '{name}' not registered")
        return self._workflows[name]
    
    def list_workflows(self) -> list[str]:
        """List all registered workflow names."""
        return list(self._workflows.keys())
    
    def on_progress(self, callback: Callable[[str, NodeStatus, float], None]) -> None:
        """Register progress callback.
        
        Callback receives (node_id, status, progress_ratio).
        """
        self._progress_callbacks.append(callback)
    
    def run(self, name: str, input_data: dict) -> WorkflowResult:
        """Execute a workflow.
        
        Args:
            name: Registered workflow name.
            input_data: Input data passed to all nodes.
            
        Returns:
            WorkflowResult with results, errors, and metrics.
        """
        if name not in self._workflows:
            raise KeyError(f"Workflow '{name}' not registered")
        
        nodes = [WorkflowNode(**{**n.__dict__, 'status': NodeStatus.PENDING})
                 for n in self._workflows[name]]
        nodes_map = {n.id: n for n in nodes}
        
        completed: set[str] = set()
        results: dict[str, Any] = {}
        errors: list[EditError] = []
        start_time = time.time()
        token_usage = TokenUsage()
        
        # Process nodes in topological order with parallel execution
        while len(completed) < len(nodes):
            # Find ready nodes
            ready = [
                n for n in nodes
                if n.status == NodeStatus.PENDING and n.is_ready(completed)
            ]
            
            if not ready and len(completed) < len(nodes):
                # Check for stalled nodes (all remaining have unmet deps)
                pending = [n for n in nodes if n.status == NodeStatus.PENDING]
                stalled = [n for n in pending if not n.is_ready(completed)]
                if stalled:
                    errors.append(EditError(
                        node_id=stalled[0].id,
                        error_type="stalled_dependency",
                        message=f"Node '{stalled[0].id}' has unmet dependencies",
                        recoverable=False
                    ))
                    # Mark stalled as skipped
                    for s in stalled:
                        s.status = NodeStatus.SKIPPED
                        completed.add(s.id)
                    continue
                break
            
            # Execute ready nodes (potentially parallel)
            threads = []
            for node in ready:
                node.status = NodeStatus.RUNNING
                self._notify_progress(node.id, NodeStatus.RUNNING, 0.0)
                
                thread = threading.Thread(
                    target=self._execute_node,
                    args=(node, input_data, results, errors, token_usage)
                )
                thread.start()
                threads.append((node, thread))
            
            # Wait for all threads in this batch
            for node, thread in threads:
                thread.join(timeout=node.timeout)
                if thread.is_alive():
                    # Timeout
                    node.status = NodeStatus.FAILED
                    errors.append(EditError(
                        node_id=node.id,
                        error_type="timeout",
                        message=f"Node '{node.id}' timed out after {node.timeout}s",
                        recoverable=True,
                        recovery_action="retry"
                    ))
                
                completed.add(node.id)
                self._notify_progress(
                    node.id,
                    node.status,
                    1.0 if node.status == NodeStatus.SUCCESS else 0.0
                )
        
        end_time = time.time()
        
        success = all(n.status == NodeStatus.SUCCESS for n in nodes)
        quality_score = 1.0 if success else max(0.0, 1.0 - len(errors) * 0.2)
        
        return WorkflowResult(
            success=success,
            output_path=results.get("output_path"),
            token_usage=token_usage,
            errors=errors,
            quality_score=quality_score,
            processing_time=end_time - start_time
        )
    
    def _execute_node(
        self,
        node: WorkflowNode,
        input_data: dict,
        results: dict,
        errors: list,
        token_usage: TokenUsage
    ) -> None:
        """Execute a single node with retry logic."""
        for attempt in range(node.retry_count + 1):
            try:
                # Merge input data with results from previous nodes
                node_input = {
                    **input_data,
                    **{k: v for k, v in results.items() if k in node.deps}
                }
                
                result = node.handler(**node_input)
                
                # Track token usage if provider returns it
                if isinstance(result, dict) and "_token_cost" in result:
                    token_usage.add_local(result["_token_cost"])
                
                results[node.id] = result
                node.status = NodeStatus.SUCCESS
                node.result = result
                return
                
            except Exception as e:
                if attempt < node.retry_count:
                    time.sleep(2 ** attempt)  # exponential backoff
                else:
                    node.status = NodeStatus.FAILED
                    node.error = str(e)
                    errors.append(EditError(
                        node_id=node.id,
                        error_type="execution_error",
                        message=str(e),
                        recoverable=True,
                        recovery_action="retry or skip"
                    ))
    
    def _notify_progress(self, node_id: str, status: NodeStatus, progress: float) -> None:
        """Notify progress callbacks."""
        for cb in self._progress_callbacks:
            try:
                cb(node_id, status, progress)
            except Exception:
                pass  # Don't let callback errors break the engine
    
    def clear(self) -> None:
        """Clear all registered workflows."""
        self._workflows.clear()
        self._progress_callbacks.clear()
```

### Verifikasi

```python
engine = WorkflowEngine()
engine.register("test", [
    WorkflowNode(id="a", handler=lambda **_: {"from_a": True}),
    WorkflowNode(id="b", handler=lambda **_: {"from_b": True}, deps=["a"]),
    WorkflowNode(id="c", handler=lambda **_: {"from_c": True}, deps=["a"]),
])
result = engine.run("test", {})
assert result.success
assert result.processing_time > 0
```

---

## Task 1.7 — Template Database

### Instruksi

Buat `auto-editor/orchestrator/template_db.py` dan folder `auto-editor/config/templates/`.

Template system untuk layout yang bisa di-reuse. Format YAML.

### Implementasi

```python
"""Layout template manager — load, save, search, apply templates.

Templates are YAML files defining reusable coordinate layouts.
Search uses keyword matching (0 token cost).
"""

from __future__ import annotations
from pathlib import Path
from typing import Optional, Any
import yaml
import re

from ..models import CoordinateElement, Position, Size, Timeline, Transform, TextStyle


class TemplateDB:
    """Manage layout templates stored as YAML files.
    
    Templates define reusable CoordinateElement layouts that can be
    applied to any video project with variable substitution.
    """
    
    def __init__(self, templates_dir: str = "config/templates"):
        self._templates_dir = Path(templates_dir)
        self._cache: dict[str, dict] = {}
        self._load_all()
    
    def _load_all(self) -> None:
        """Load all YAML templates from directory."""
        if not self._templates_dir.exists():
            self._templates_dir.mkdir(parents=True, exist_ok=True)
            return
        
        for yaml_file in self._templates_dir.glob("*.yaml"):
            try:
                with open(yaml_file) as f:
                    data = yaml.safe_load(f)
                    if data and "name" in data:
                        self._cache[data["name"]] = data
            except (yaml.YAMLError, IOError) as e:
                print(f"Warning: Failed to load template '{yaml_file}': {e}")
    
    def list_all(self) -> list[dict]:
        """List all available templates with metadata."""
        return [
            {"name": t.get("name"), "description": t.get("description", ""),
             "style": t.get("style", "custom"), "aspect_ratio": t.get("aspect_ratio", "16:9")}
            for t in self._cache.values()
        ]
    
    def get(self, name: str) -> Optional[dict]:
        """Get template by name."""
        return self._cache.get(name)
    
    def find_similar(self, query: str) -> Optional[dict]:
        """Find best matching template by keyword matching.
        
        Args:
            query: Search query (e.g., "cinematic product", "tiktok vertical").
            
        Returns:
            Best matching template or None.
        """
        query_lower = query.lower()
        query_keywords = set(re.findall(r'\w+', query_lower))
        
        best_score = 0
        best_template = None
        
        for name, template in self._cache.items():
            # Score berdasarkan:
            # - keyword match di name (bobot 3)
            # - keyword match di description (bobot 2)
            # - keyword match di style (bobot 2)
            # - keyword match di tags (bobot 1)
            
            searchable = f"{name} {template.get('description', '')} "
            searchable += f"{template.get('style', '')} "
            searchable += " ".join(template.get("tags", []))
            
            tmpl_keywords = set(re.findall(r'\w+', searchable.lower()))
            overlap = query_keywords & tmpl_keywords
            
            if not overlap:
                continue
            
            score = sum(
                3 if kw in name.lower() else
                2 if kw in template.get('description', '').lower() else
                2 if kw in template.get('style', '').lower() else
                1
                for kw in overlap
            )
            
            if score > best_score:
                best_score = score
                best_template = template
        
        return best_template
    
    def apply(self, name: str, variables: dict[str, str]) -> list[CoordinateElement]:
        """Apply template with variable substitution.
        
        Args:
            name: Template name.
            variables: Dict of variable names to values.
                Template variables use {VARIABLE_NAME} syntax.
                
        Returns:
            List of CoordinateElements with resolved positions.
        """
        template = self.get(name)
        if not template:
            raise KeyError(f"Template '{name}' not found")
        
        elements = []
        for track in template.get("tracks", []):
            # Deep copy and substitute variables
            track_str = yaml.dump(track)
            for var_name, var_value in variables.items():
                track_str = track_str.replace(f"{{{var_name}}}", str(var_value))
            resolved = yaml.safe_load(track_str)
            
            element = self._track_to_element(resolved)
            elements.append(element)
        
        return elements
    
    def save(self, name: str, data: dict) -> None:
        """Save a new template to disk."""
        filepath = self._templates_dir / f"{name}.yaml"
        with open(filepath, "w") as f:
            yaml.dump(data, f, default_flow_style=False, allow_unicode=True)
        self._cache[name] = data
    
    def delete(self, name: str) -> bool:
        """Delete a template."""
        filepath = self._templates_dir / f"{name}.yaml"
        if filepath.exists():
            filepath.unlink()
            self._cache.pop(name, None)
            return True
        return False
    
    def _track_to_element(self, track: dict) -> CoordinateElement:
        """Convert YAML track definition to CoordinateElement."""
        pos = track.get("position", {})
        sz = track.get("size", {})
        tml = track.get("timeline", {})
        trf = track.get("transform", {})
        style = track.get("style", {})
        
        element = CoordinateElement(
            id=track.get("id", "untitled"),
            type=track.get("type", "video"),
            position=Position(x=pos.get("x", 0.5), y=pos.get("y", 0.5), z=pos.get("z", 0)),
            size=Size(
                width=sz.get("width", 0.5),
                height=sz.get("height", 0.5),
                unit=sz.get("unit", "normalized")
            ),
            timeline=Timeline(start=tml.get("start", 0.0), end=tml.get("end", 10.0)),
            transform=Transform(
                rotation=trf.get("rotation", 0.0),
                scale=trf.get("scale", 1.0),
                opacity=trf.get("opacity", 1.0),
                anchor=trf.get("anchor", "center")
            ),
        )
        
        # Set style based on element type
        if element.type == "text" and style:
            element.text_style = TextStyle(
                text=style.get("text", ""),
                font_family=style.get("font_family", "Inter"),
                font_size=style.get("font_size", 48),
                font_weight=style.get("font_weight", 400),
                color=style.get("color", "#FFFFFF"),
                text_align=style.get("text_align", "center"),
            )
        
        return element
    
    def reload(self) -> None:
        """Reload all templates from disk."""
        self._cache.clear()
        self._load_all()
```

### Template YAML — Minimal 3 Bawaan

**`config/templates/cinematic.yaml`:**
```yaml
name: "cinematic"
description: "Cinematic widescreen template with title overlay"
style: "cinematic"
aspect_ratio: "16:9"
tags: [cinematic, film, widescreen, professional]

tracks:
  - id: "main_video"
    type: "video"
    position: { x: 0, y: 0, z: 0 }
    size: { width: 1.0, height: 1.0, unit: "normalized" }
    timeline: { start: 0, end: 30 }
    transform: { scale: 1.0, opacity: 1.0 }
    style: { fit: "cover" }

  - id: "title_overlay"
    type: "text"
    position: { x: 0.5, y: 0.45, z: 2 }
    size: { width: 0.8, height: 0.12, unit: "normalized" }
    timeline: { start: 0, end: 5 }
    transform: { opacity: 1.0, anchor: "center" }
    style:
      text: "{TITLE}"
      font_family: "Montserrat"
      font_size: 56
      color: "#FFFFFF"
      font_weight: 700
      text_align: "center"
      shadow: { offset: [2, 2], blur: 4, color: "#00000080" }

  - id: "subtitle"
    type: "text"
    position: { x: 0.5, y: 0.92, z: 3 }
    size: { width: 0.9, height: 0.06, unit: "normalized" }
    timeline: { start: 0, end: 30 }
    transform: { opacity: 0.85 }
    style:
      text: "(auto subtitle)"
      font_family: "Inter"
      font_size: 20
      color: "#FFFFFF"
      text_align: "center"
      shadow: { offset: [1, 1], blur: 2, color: "#000000" }
```

**`config/templates/tiktok_product.yaml`:**
```yaml
name: "tiktok_product"
description: "TikTok 9:16 vertical product review layout"
style: "product"
aspect_ratio: "9:16"
tags: [tiktok, product, review, vertical, social]

tracks:
  - id: "main_video"
    type: "video"
    position: { x: 0.5, y: 0.5, z: 0 }
    size: { width: 1.0, height: 0.7, unit: "normalized" }
    timeline: { start: 0, end: 60 }
    style: { fit: "cover" }

  - id: "product_name"
    type: "text"
    position: { x: 0.5, y: 0.06, z: 2 }
    size: { width: 0.9, height: 0.08, unit: "normalized" }
    timeline: { start: 0, end: 60 }
    transform: { opacity: 1.0 }
    style:
      text: "{PRODUCT_NAME}"
      font_size: 52
      color: "#FFFFFF"
      font_weight: 800
      text_align: "center"

  - id: "price_tag"
    type: "text"
    position: { x: 0.5, y: 0.14, z: 2 }
    size: { width: 0.5, height: 0.06, unit: "normalized" }
    timeline: { start: 0, end: 60 }
    style:
      text: "{PRICE}"
      font_size: 40
      color: "#FF4444"
      font_weight: 700
      text_align: "center"

  - id: "cta_button"
    type: "text"
    position: { x: 0.5, y: 0.90, z: 3 }
    size: { width: 0.7, height: 0.06, unit: "normalized" }
    timeline: { start: 50, end: 60 }
    transform: { scale: 1.0, opacity: 1.0 }
    style:
      text: "⬇ BELI SEKARANG ⬇"
      font_size: 36
      color: "#FF4444"
      font_weight: 700
      text_align: "center"
```

**`config/templates/slideshow.yaml`:**
```yaml
name: "slideshow"
description: "Simple photo/video slideshow with transition effects"
style: "presentation"
aspect_ratio: "16:9"
tags: [slideshow, photo, presentation, gallery]

tracks:
  - id: "slide"
    type: "video"
    position: { x: 0.5, y: 0.5, z: 0 }
    size: { width: 0.9, height: 0.85, unit: "normalized" }
    timeline: { start: 0, end: 30 }
    style: { fit: "contain" }

  - id: "caption"
    type: "text"
    position: { x: 0.5, y: 0.92, z: 1 }
    size: { width: 0.8, height: 0.06, unit: "normalized" }
    timeline: { start: 0, end: 30 }
    style:
      text: "{CAPTION}"
      font_size: 24
      color: "#DDDDDD"
      font_weight: 400
      text_align: "center"
```

### Verifikasi

```python
db = TemplateDB("config/templates")
assert len(db.list_all()) >= 3
assert db.get("cinematic") is not None
result = db.find_similar("tiktok product video")
assert result is not None
```

---

## Task 1.8 — Coordinate System (0-Token Math Engine)

### Instruksi

Buat `auto-editor/workers/layout_engine/coordinate.py`.

Ini adalah **salah satu komponen paling penting** — engine matematika yang menghitung
posisi elemen tanpa melibatkan AI. Semua layout decisions yang bisa dihitung
dengan rumus, harus dihitung di sini (0 token).

### Implementasi

```python
"""Coordinate math engine — pure computation, 0 token cost.

All positioning calculations happen here using normalized coordinates.
AI only provides high-level instructions (e.g., "center the title"),
the math engine computes exact pixel positions.
"""

from __future__ import annotations
from typing import Optional
import math

from ...models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    Keyframe, Animation, AspectRatio
)


class CoordinateEngine:
    """Compute exact positions from normalized coordinates.
    
    All positions are normalized (0.0 - 1.0) and resolution-independent.
    Conversion to pixels happens at render time.
    """
    
    def __init__(self, canvas_width: int = 1920, canvas_height: int = 1080):
        self.canvas_width = canvas_width
        self.canvas_height = canvas_height
        self.aspect_ratio = canvas_width / canvas_height
    
    def to_pixels(self, normalized: float, dimension: int) -> int:
        """Convert normalized coordinate to pixel value."""
        return round(normalized * dimension)
    
    def to_normalized(self, pixels: int, dimension: int) -> float:
        """Convert pixel value to normalized coordinate."""
        if dimension == 0:
            return 0.0
        return pixels / dimension
    
    def get_bounds(self, element: CoordinateElement) -> dict:
        """Get pixel boundaries of an element on canvas.
        
        Returns:
            {left, top, right, bottom, width, height, center_x, center_y}
        """
        pw = self.canvas_width
        ph = self.canvas_height
        
        # Element size in pixels
        if element.size.unit == "normalized":
            w = element.size.width * pw
            h = element.size.height * ph
        elif element.size.unit == "pixel":
            w = element.size.width
            h = element.size.height
        else:  # percent
            w = (element.size.width / 100) * pw
            h = (element.size.height / 100) * ph
        
        # Scale
        w *= element.transform.scale
        h *= element.transform.scale
        
        # Position based on anchor point
        anchor_offsets = {
            "center": (0.5, 0.5),
            "top_left": (0, 0),
            "top_right": (1, 0),
            "bottom_left": (0, 1),
            "bottom_right": (1, 1),
        }
        ox, oy = anchor_offsets.get(element.transform.anchor, (0.5, 0.5))
        
        x = (element.position.x * pw) - (w * ox)
        y = (element.position.y * ph) - (h * oy)
        
        return {
            "left": x,
            "top": y,
            "right": x + w,
            "bottom": y + h,
            "width": w,
            "height": h,
            "center_x": x + w / 2,
            "center_y": y + h / 2,
        }
    
    def check_overlap(self, a: CoordinateElement, b: CoordinateElement) -> bool:
        """Check if two elements visually overlap on canvas."""
        ba = self.get_bounds(a)
        bb = self.get_bounds(b)
        
        return not (
            ba["right"] <= bb["left"] or
            ba["left"] >= bb["right"] or
            ba["bottom"] <= bb["top"] or
            ba["top"] >= bb["bottom"]
        )
    
    def resolve_overlap(
        self, fixed: CoordinateElement, moving: CoordinateElement,
        direction: str = "right"
    ) -> CoordinateElement:
        """Push moving element away from fixed element to resolve overlap.
        
        Args:
            fixed: Element that stays in place.
            moving: Element that gets pushed.
            direction: Push direction ("right", "left", "up", "down").
            
        Returns:
            New CoordinateElement with adjusted position.
        """
        bf = self.get_bounds(fixed)
        bm = self.get_bounds(moving)
        
        import copy
        adjusted = copy.deepcopy(moving)
        
        if direction == "right":
            new_x = (bf["right"] + bm["width"] / 2) / self.canvas_width
            adjusted.position.x = new_x
        elif direction == "left":
            new_x = (bf["left"] - bm["width"] / 2) / self.canvas_width
            adjusted.position.x = new_x
        elif direction == "down":
            new_y = (bf["bottom"] + bm["height"] / 2) / self.canvas_height
            adjusted.position.y = new_y
        elif direction == "up":
            new_y = (bf["top"] - bm["height"] / 2) / self.canvas_height
            adjusted.position.y = new_y
        
        return adjusted
    
    def create_grid(
        self, start: Position, cols: int, rows: int,
        spacing: float = 0.02, element_size: Size = Size(0.2, 0.2)
    ) -> list[CoordinateElement]:
        """Create a grid of elements.
        
        Useful for multi-clip layouts, contact sheets, etc.
        
        Args:
            start: Starting position (top-left of grid).
            cols: Number of columns.
            rows: Number of rows.
            spacing: Gap between elements (normalized).
            element_size: Size of each grid cell.
            
        Returns:
            List of CoordinateElements with computed positions.
        """
        elements = []
        for row in range(rows):
            for col in range(cols):
                x = start.x + col * (element_size.width + spacing)
                y = start.y + row * (element_size.height + spacing)
                
                elements.append(CoordinateElement(
                    id=f"grid_{row}_{col}",
                    type="video",
                    position=Position(x=x, y=y, z=start.z),
                    size=Size(width=element_size.width, height=element_size.height)
                ))
        
        return elements
    
    def split_screen(
        self, count: int, layout: str = "grid"
    ) -> list[Position]:
        """Divide canvas into equal regions for split-screen.
        
        Args:
            count: Number of splits (1, 2, 4, 6, 9).
            layout: "grid", "horizontal", "vertical", "picture_in_picture".
            
        Returns:
            List of Positions (center point for each region).
        """
        positions = []
        
        if layout == "horizontal":
            h = 1.0 / count
            for i in range(count):
                positions.append(Position(x=0.5, y=h * i + h / 2))
        
        elif layout == "vertical":
            w = 1.0 / count
            for i in range(count):
                positions.append(Position(x=w * i + w / 2, y=0.5))
        
        elif layout == "picture_in_picture":
            positions = [
                Position(x=0.5, y=0.5),   # main (full)
                Position(x=0.8, y=0.8),   # pip (small, bottom-right)
            ]
        
        else:  # grid
            sqrt_n = math.ceil(math.sqrt(count))
            w = 1.0 / sqrt_n
            h = 1.0 / sqrt_n
            for row in range(sqrt_n):
                for col in range(sqrt_n):
                    if len(positions) < count:
                        positions.append(Position(
                            x=w * col + w / 2,
                            y=h * row + h / 2
                        ))
        
        return positions
    
    def apply_keyframe(
        self, element: CoordinateElement, time: float
    ) -> CoordinateElement:
        """Apply keyframe animation at a given time.
        
        Interpolates between keyframes using the specified easing function.
        
        Args:
            element: Element with animation keyframes.
            time: Current time in seconds (relative to element start).
            
        Returns:
            New element with interpolated transform values.
        """
        if not element.animation or not element.animation.keyframes:
            return element
        
        kfs = sorted(element.animation.keyframes, key=lambda k: k.time)
        
        if time <= kfs[0].time:
            return self._apply_keyframe_values(element, kfs[0])
        
        if time >= kfs[-1].time:
            return self._apply_keyframe_values(element, kfs[-1])
        
        # Find surrounding keyframes
        for i in range(len(kfs) - 1):
            if kfs[i].time <= time <= kfs[i + 1].time:
                t = (time - kfs[i].time) / (kfs[i + 1].time - kfs[i].time)
                eased_t = self._ease(t, element.animation.easing)
                return self._interpolate(element, kfs[i], kfs[i + 1], eased_t)
        
        return element
    
    def _interpolate(
        self, element: CoordinateElement,
        kf_start: Keyframe, kf_end: Keyframe, t: float
    ) -> CoordinateElement:
        """Linearly interpolate between two keyframes."""
        import copy
        result = copy.deepcopy(element)
        
        # Interpolate position
        if kf_start.x is not None and kf_end.x is not None:
            result.position.x = kf_start.x + (kf_end.x - kf_start.x) * t
        if kf_start.y is not None and kf_end.y is not None:
            result.position.y = kf_start.y + (kf_end.y - kf_start.y) * t
        
        # Interpolate transform
        if kf_start.scale is not None and kf_end.scale is not None:
            result.transform.scale = kf_start.scale + (kf_end.scale - kf_start.scale) * t
        if kf_start.opacity is not None and kf_end.opacity is not None:
            result.transform.opacity = kf_start.opacity + (kf_end.opacity - kf_start.opacity) * t
        if kf_start.rotation is not None and kf_end.rotation is not None:
            result.transform.rotation = kf_start.rotation + (kf_end.rotation - kf_start.rotation) * t
        
        return result
    
    def _apply_keyframe_values(
        self, element: CoordinateElement, kf: Keyframe
    ) -> CoordinateElement:
        """Apply keyframe values directly (no interpolation)."""
        import copy
        result = copy.deepcopy(element)
        
        if kf.x is not None: result.position.x = kf.x
        if kf.y is not None: result.position.y = kf.y
        if kf.scale is not None: result.transform.scale = kf.scale
        if kf.opacity is not None: result.transform.opacity = kf.opacity
        if kf.rotation is not None: result.transform.rotation = kf.rotation
        
        return result
    
    def _ease(self, t: float, easing: str) -> float:
        """Apply easing function to normalized time t (0.0 - 1.0)."""
        if easing == "linear":
            return t
        elif easing == "ease_in":
            return t * t
        elif easing == "ease_out":
            return t * (2 - t)
        elif easing == "ease_in_out":
            return t * t * (3 - 2 * t) if t < 0.5 else 1 - (1 - t) * (1 - t) * (3 - 2 * (1 - t))
        elif easing == "bounce":
            if t < 0.3636:
                return 7.5625 * t * t
            elif t < 0.7273:
                t -= 0.5455
                return 7.5625 * t * t + 0.75
            elif t < 0.9091:
                t -= 0.8182
                return 7.5625 * t * t + 0.9375
            else:
                t -= 0.9545
                return 7.5625 * t * t + 0.984375
        return t
    
    def center_in_canvas(self, size: Size) -> Position:
        """Get position to center an element on canvas."""
        return Position(x=0.5, y=0.5)
    
    def align_to_edge(self, edge: str, margin: float = 0.05) -> Position:
        """Get position aligned to canvas edge.
        
        Args:
            edge: "top-left", "top-right", "bottom-left", "bottom-right", "top", "bottom".
            margin: Normalized margin from edge.
        """
        positions = {
            "top-left": Position(x=margin, y=margin),
            "top-right": Position(x=1 - margin, y=margin),
            "bottom-left": Position(x=margin, y=1 - margin),
            "bottom-right": Position(x=1 - margin, y=1 - margin),
            "top": Position(x=0.5, y=margin),
            "bottom": Position(x=0.5, y=1 - margin),
        }
        return positions.get(edge, Position(x=0.5, y=0.5))
    
    def rule_of_thirds(self, h_pos: str, v_pos: str) -> Position:
        """Position using the rule of thirds.
        
        Args:
            h_pos: "left" | "center" | "right"
            v_pos: "top" | "middle" | "bottom"
        """
        h_map = {"left": 1/3, "center": 0.5, "right": 2/3}
        v_map = {"top": 1/3, "middle": 0.5, "bottom": 2/3}
        return Position(x=h_map.get(h_pos, 0.5), y=v_map.get(v_pos, 0.5))
    
    def golden_ratio_position(self, offset_x: float = 0, offset_y: float = 0) -> Position:
        """Position using golden ratio (≈ 1.618)."""
        phi = (1 + math.sqrt(5)) / 2
        return Position(x=1/phi + offset_x, y=1/phi + offset_y)
    
    def safe_zone(self, margin: float = 0.1) -> dict:
        """Get safe zone boundaries (pixel)."""
        return {
            "left": self.canvas_width * margin,
            "top": self.canvas_height * margin,
            "right": self.canvas_width * (1 - margin),
            "bottom": self.canvas_height * (1 - margin),
            "width": self.canvas_width * (1 - 2 * margin),
            "height": self.canvas_height * (1 - 2 * margin),
        }
```

### Verifikasi

```python
engine = CoordinateEngine(1920, 1080)

# Rule of thirds
pos = engine.rule_of_thirds("left", "top")
assert pos.x == 1/3 and pos.y == 1/3, f"Got {pos}"

# Grid 2x2
grid = engine.create_grid(Position(0, 0), 2, 2)
assert len(grid) == 4

# Overlap detection
a = CoordinateElement("a", "video", size=Size(0.5, 0.5))
b = CoordinateElement("b", "video", position=Position(0.5, 0.5), size=Size(0.5, 0.5))
assert engine.check_overlap(a, b)

# No overlap
c = CoordinateElement("c", "video", position=Position(1, 1), size=Size(0.01, 0.01))
assert not engine.check_overlap(a, c)

# Keyframe interpolation
elem = CoordinateElement(
    "test", "text",
    animation=Animation(keyframes=[
        Keyframe(time=0, opacity=0),
        Keyframe(time=1, opacity=1, scale=0.8),
        Keyframe(time=2, opacity=1, scale=1.0),
    ])
)
at_0 = engine.apply_keyframe(elem, 0)
assert at_0.transform.opacity == 0
```

---

## Task 1.9 — Config System

### Instruksi

Buat `auto-editor/config/settings_loader.py` sebagai loader YAML → dataclass.

### Implementasi

```python
"""Configuration loader — YAML file to AutoEditorConfig dataclass."""

from __future__ import annotations
from pathlib import Path
from typing import Optional
import yaml
import os

from ..models import AutoEditorConfig, Mode, LocalConfig, APIConfig, ResourceConfig, BehaviorConfig


DEFAULT_CONFIG_PATH = Path(__file__).parent / "settings.yaml"


def load_config(path: Optional[str] = None) -> AutoEditorConfig:
    """Load configuration from YAML file with environment variable overrides.
    
    Resolution order (later overrides earlier):
    1. Default config (hardcoded)
    2. YAML file
    3. Environment variables (OPENCUT_*)
    
    Args:
        path: Path to YAML config file. If None, uses default path.
        
    Returns:
        AutoEditorConfig with merged settings.
    """
    config = _default_config()
    
    # Load YAML if exists
    config_path = Path(path) if path else DEFAULT_CONFIG_PATH
    if config_path.exists():
        with open(config_path) as f:
            yaml_data = yaml.safe_load(f)
        if yaml_data:
            config = _merge_yaml(config, yaml_data)
    
    # Environment variable overrides
    config = _apply_env_overrides(config)
    
    return config


def _default_config() -> AutoEditorConfig:
    """Hardcoded default configuration."""
    return AutoEditorConfig(
        mode=Mode.HYBRID,
        local=LocalConfig(),
        api=APIConfig(),
        resources=ResourceConfig(),
        behavior=BehaviorConfig(),
    )


def _merge_yaml(config: AutoEditorConfig, yaml_data: dict) -> AutoEditorConfig:
    """Merge YAML data into config, preserving unset values."""
    if "mode" in yaml_data:
        config.mode = Mode(yaml_data["mode"])
    
    if "local" in yaml_data:
        local = yaml_data["local"]
        if "llm_model" in local: config.local.llm_model = local["llm_model"]
        if "tts_model" in local: config.local.tts_model = local["tts_model"]
        if "asr_model" in local: config.local.asr_model = local["asr_model"]
        if "models_dir" in local: config.local.models_dir = local["models_dir"]
    
    if "api" in yaml_data:
        api = yaml_data["api"]
        if "llm" in api:
            if "provider" in api["llm"]: config.api.llm_provider = api["llm"]["provider"]
            if "model" in api["llm"]: config.api.llm_model = api["llm"]["model"]
            if "max_tokens" in api["llm"]: config.api.llm_max_tokens = api["llm"]["max_tokens"]
        if "pexels" in api and "api_key" in api["pexels"]:
            config.api.pexels_api_key = api["pexels"]["api_key"]
    
    if "resources" in yaml_data:
        res = yaml_data["resources"]
        if "max_vram_gb" in res: config.resources.max_vram_gb = res["max_vram_gb"]
        if "max_ram_gb" in res: config.resources.max_ram_gb = res["max_ram_gb"]
        if "max_threads" in res: config.resources.max_threads = res["max_threads"]
    
    if "behavior" in yaml_data:
        beh = yaml_data["behavior"]
        if "confidence_threshold" in beh: config.behavior.confidence_threshold = beh["confidence_threshold"]
        if "max_retries" in beh: config.behavior.max_retries = beh["max_retries"]
        if "cache_enabled" in beh: config.behavior.cache_enabled = beh["cache_enabled"]
        if "cache_ttl_minutes" in beh: config.behavior.cache_ttl_minutes = beh["cache_ttl_minutes"]
    
    return config


def _apply_env_overrides(config: AutoEditorConfig) -> AutoEditorConfig:
    """Apply OPENCUT_* environment variable overrides."""
    env_map = {
        "OPENCUT_MODE": ("mode", lambda v: Mode(v)),
        "OPENCUT_LLM_MODEL": ("local.llm_model", str),
        "OPENCUT_TTS_MODEL": ("local.tts_model", str),
        "OPENCUT_ASR_MODEL": ("local.asr_model", str),
        "OPENCUT_MODELS_DIR": ("local.models_dir", str),
        "OPENCUT_API_PROVIDER": ("api.llm_provider", str),
        "OPENCUT_API_MODEL": ("api.llm_model", str),
        "OPENCUT_API_MAX_TOKENS": ("api.llm_max_tokens", int),
        "OPENCUT_PEXELS_KEY": ("api.pexels_api_key", str),
        "OPENCUT_MAX_VRAM": ("resources.max_vram_gb", int),
        "OPENCUT_MAX_RAM": ("resources.max_ram_gb", int),
        "OPENCUT_THREADS": ("resources.max_threads", int),
        "OPENCUT_CONFIDENCE": ("behavior.confidence_threshold", float),
        "OPENCUT_CACHE": ("behavior.cache_enabled", lambda v: v.lower() == "true"),
    }
    
    for env_name, (attr_path, converter) in env_map.items():
        env_val = os.environ.get(env_name)
        if env_val is not None:
            parts = attr_path.split(".")
            obj = config
            for part in parts[:-1]:
                obj = getattr(obj, part)
            try:
                setattr(obj, parts[-1], converter(env_val))
            except (ValueError, TypeError):
                pass  # Skip invalid env values
    
    return config


def save_config(config: AutoEditorConfig, path: str) -> None:
    """Save configuration to YAML file."""
    data = {
        "mode": config.mode.value,
        "local": {
            "llm_model": config.local.llm_model,
            "tts_model": config.local.tts_model,
            "asr_model": config.local.asr_model,
            "models_dir": config.local.models_dir,
        },
        "api": {
            "llm": {
                "provider": config.api.llm_provider,
                "model": config.api.llm_model,
                "max_tokens": config.api.llm_max_tokens,
            },
            "pexels": {"api_key": config.api.pexels_api_key if config.api.pexels_api_key else ""},
        },
        "resources": {
            "max_vram_gb": config.resources.max_vram_gb,
            "max_ram_gb": config.resources.max_ram_gb,
            "max_threads": config.resources.max_threads,
        },
        "behavior": {
            "confidence_threshold": config.behavior.confidence_threshold,
            "max_retries": config.behavior.max_retries,
            "cache_enabled": config.behavior.cache_enabled,
            "cache_ttl_minutes": config.behavior.cache_ttl_minutes,
        },
    }
    
    with open(path, "w") as f:
        yaml.dump(data, f, default_flow_style=False, allow_unicode=True)
```

### Verifikasi

```python
config = load_config()
assert config.mode in Mode
assert 0 < config.behavior.confidence_threshold <= 1.0
assert config.resources.max_vram_gb > 0
```

---

## Task 1.10 — CLI Entry Point

### Instruksi

Buat `auto-editor/main.py` — CLI untuk menjalankan auto-editor.

### Implementasi

```python
#!/usr/bin/env python3
"""OpenCut Auto-Editor — CLI entry point.

Usage:
    python -m auto-editor edit <footage_dir> [--script FILE] [--output PATH] [--mode MODE]
    python -m auto-editor batch <projects_dir> [--format FORMAT] [--resolution RES]
    python -m auto-editor voiceover --text FILE [--voice ID] [--output PATH]
    python -m auto-editor subtitle <video_path> [--language LANG] [--output PATH]
    python -m auto-editor estimate <footage_dir> [--script FILE]
    python -m auto-editor list-templates
    python -m auto-editor config [--show] [--set KEY=VALUE]
"""

from __future__ import annotations
import argparse
import sys
from pathlib import Path
from typing import Optional

# Ensure auto-editor is in path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor import __version__
from auto_editor.config.settings_loader import load_config, save_config
from auto_editor.orchestrator.intent_router import IntentRouter


def build_parser() -> argparse.ArgumentParser:
    """Build argument parser with all subcommands."""
    parser = argparse.ArgumentParser(
        prog="opencut-auto",
        description="OpenCut AI Auto-Editor — Token-efficient video editing automation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  opencut-auto edit ./footage/ --script script.txt --output result.mp4
  opencut-auto edit ./footage/ --mode offline
  opencut-auto batch ./projects/ --format mp4
  opencut-auto voiceover --text narasi.txt --voice id
  opencut-auto subtitle video.mp4 --language id
  opencut-auto estimate ./footage/ --script script.txt
        """
    )
    
    parser.add_argument("--version", action="version", version=f"opencut-auto v{__version__}")
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # ── edit ──────────────────────────────────────────────
    edit_parser = subparsers.add_parser("edit", help="Auto-edit video from footage directory")
    edit_parser.add_argument("footage", type=str, help="Path to directory containing footage")
    edit_parser.add_argument("--script", "-s", type=str, help="Path to script/narration file")
    edit_parser.add_argument("--output", "-o", type=str, default="./output.mp4", help="Output video path")
    edit_parser.add_argument("--mode", "-m", type=str, choices=["offline", "hybrid", "cloud"],
                            default=None, help="Processing mode")
    edit_parser.add_argument("--style", type=str, help="Editing style (cinematic, vlog, tutorial)")
    edit_parser.add_argument("--duration", "-d", type=int, help="Target duration in seconds")
    edit_parser.add_argument("--prompt", "-p", type=str, help="Natural language editing instruction")
    
    # ── batch ─────────────────────────────────────────────
    batch_parser = subparsers.add_parser("batch", help="Batch render multiple projects")
    batch_parser.add_argument("projects_dir", type=str, help="Directory containing project folders")
    batch_parser.add_argument("--format", "-f", type=str, default="mp4", choices=["mp4", "mov", "webm"])
    batch_parser.add_argument("--resolution", "-r", type=str, default="1080p", 
                            choices=["720p", "1080p", "4k"])
    batch_parser.add_argument("--mode", "-m", type=str, choices=["offline", "hybrid", "cloud"], default=None)
    
    # ── voiceover ─────────────────────────────────────────
    vo_parser = subparsers.add_parser("voiceover", help="Generate voiceover audio from text")
    vo_parser.add_argument("--text", "-t", type=str, required=True, help="Path to text/script file")
    vo_parser.add_argument("--voice", "-v", type=str, default="default", help="Voice profile ID")
    vo_parser.add_argument("--output", "-o", type=str, default="./voiceover.wav", help="Output audio path")
    vo_parser.add_argument("--language", "-l", type=str, default="id", help="Language code")
    vo_parser.add_argument("--speed", type=float, default=1.0, help="Speech speed (0.5-2.0)")
    
    # ── subtitle ──────────────────────────────────────────
    sub_parser = subparsers.add_parser("subtitle", help="Generate subtitles from video")
    sub_parser.add_argument("video", type=str, help="Path to video file")
    sub_parser.add_argument("--language", "-l", type=str, default="id", help="Language code")
    sub_parser.add_argument("--output", "-o", type=str, help="Output SRT path (default: auto)")
    sub_parser.add_argument("--format", "-f", type=str, default="srt", choices=["srt", "vtt", "ass"])
    
    # ── estimate ──────────────────────────────────────────
    est_parser = subparsers.add_parser("estimate", help="Estimate token cost before running")
    est_parser.add_argument("footage", type=str, help="Path to footage directory")
    est_parser.add_argument("--script", "-s", type=str, help="Path to script file")
    est_parser.add_argument("--prompt", "-p", type=str, help="Editing instruction")
    
    # ── list-templates ────────────────────────────────────
    subparsers.add_parser("list-templates", help="List available layout templates")
    
    # ── config ────────────────────────────────────────────
    config_parser = subparsers.add_parser("config", help="View or modify configuration")
    config_parser.add_argument("--show", action="store_true", help="Show current configuration")
    config_parser.add_argument("--set", "-s", type=str, action="append", 
                             help="Set config value (KEY=VALUE format)")
    
    return parser


def cmd_edit(args: argparse.Namespace) -> int:
    """Handle 'edit' command."""
    print(f"[Edit] Footage: {args.footage}")
    print(f"[Edit] Output: {args.output}")
    print(f"[Edit] Mode: {args.mode or 'hybrid'}")
    
    # Analyze with IntentRouter if prompt given
    if args.prompt:
        router = IntentRouter()
        intent, params = router.classify(args.prompt)
        plan = router.create_plan(args.prompt)
        print(f"[Edit] Intent: {intent.value}")
        print(f"[Edit] Plan: {plan.duration}s, {plan.style.value}, {plan.aspect_ratio.value}")
    
    print("[Edit] Not fully implemented yet — Agent 2 will complete workflow execution.")
    return 0


def cmd_batch(args: argparse.Namespace) -> int:
    """Handle 'batch' command."""
    projects_dir = Path(args.projects_dir)
    if not projects_dir.exists():
        print(f"[Error] Projects directory not found: {args.projects_dir}", file=sys.stderr)
        return 1
    
    project_folders = [f for f in projects_dir.iterdir() if f.is_dir()]
    print(f"[Batch] Found {len(project_folders)} projects in {args.projects_dir}")
    print(f"[Batch] Format: {args.format}, Resolution: {args.resolution}")
    print("[Batch] Not fully implemented yet.")
    return 0


def cmd_voiceover(args: argparse.Namespace) -> int:
    """Handle 'voiceover' command."""
    text_path = Path(args.text)
    if not text_path.exists():
        print(f"[Error] Text file not found: {args.text}", file=sys.stderr)
        return 1
    
    print(f"[Voiceover] Text: {args.text}")
    print(f"[Voiceover] Voice: {args.voice}")
    print(f"[Voiceover] Language: {args.language}")
    print(f"[Voiceover] Output: {args.output}")
    print("[Voiceover] Not fully implemented yet — Agent 2 will complete TTS integration.")
    return 0


def cmd_subtitle(args: argparse.Namespace) -> int:
    """Handle 'subtitle' command."""
    video_path = Path(args.video)
    if not video_path.exists():
        print(f"[Error] Video file not found: {args.video}", file=sys.stderr)
        return 1
    
    output = args.output or Path(args.video).with_suffix(f".{args.format}")
    print(f"[Subtitle] Video: {args.video}")
    print(f"[Subtitle] Language: {args.language}")
    print(f"[Subtitle] Output: {output}")
    print("[Subtitle] Not fully implemented yet — Agent 2 will complete ASR integration.")
    return 0


def cmd_estimate(args: argparse.Namespace) -> int:
    """Handle 'estimate' command — estimate token cost."""
    config = load_config()
    
    footage_path = Path(args.footage)
    if not footage_path.exists():
        print(f"[Error] Footage directory not found: {args.footage}", file=sys.stderr)
        return 1
    
    # Count files
    video_files = list(footage_path.glob("*.mp4")) + list(footage_path.glob("*.mov"))
    audio_files = list(footage_path.glob("*.mp3")) + list(footage_path.glob("*.wav"))
    
    print(f"\n=== Token Estimation ===")
    print(f"Mode: {config.mode.value}")
    print(f"Footage: {len(video_files)} videos, {len(audio_files)} audio files")
    
    # Estimate based on mode
    estimates = {
        "offline": {"planning": 2500, "execution": 0, "total": 2500},
        "hybrid": {"planning": 3500, "execution": 1500, "total": 5000},
        "cloud": {"planning": 8000, "execution": 12000, "total": 20000},
    }
    
    est = estimates.get(config.mode.value, estimates["hybrid"])
    print(f"Planning: ~{est['planning']} tokens")
    print(f"Execution: ~{est['execution']} tokens")
    print(f"Total: ~{est['total']} tokens")
    print(f"Estimated cost: ${est['total'] * 0.00015:.4f} (if API used)")
    
    return 0


def cmd_list_templates(args: argparse.Namespace) -> int:
    """Handle 'list-templates' command."""
    from auto_editor.orchestrator.template_db import TemplateDB
    
    db = TemplateDB("config/templates")
    templates = db.list_all()
    
    if not templates:
        print("No templates found.")
        return 0
    
    print(f"\nAvailable Templates ({len(templates)}):")
    print(f"{'Name':<25} {'Style':<15} {'Aspect':<10} {'Description'}")
    print("-" * 75)
    for t in templates:
        print(f"{t['name']:<25} {t['style']:<15} {t['aspect_ratio']:<10} {t['description']}")
    
    return 0


def cmd_config(args: argparse.Namespace) -> int:
    """Handle 'config' command."""
    config_path = Path(__file__).parent / "config" / "settings.yaml"
    config = load_config()
    
    if args.show:
        print(f"\nCurrent Configuration ({config.mode.value} mode):")
        print(f"  Local LLM: {config.local.llm_model}")
        print(f"  Local TTS: {config.local.tts_model}")
        print(f"  Local ASR: {config.local.asr_model}")
        print(f"  API Provider: {config.api.llm_provider}")
        print(f"  API Model: {config.api.llm_model}")
        print(f"  Max VRAM: {config.resources.max_vram_gb}GB")
        print(f"  Max RAM: {config.resources.max_ram_gb}GB")
        print(f"  Confidence Threshold: {config.behavior.confidence_threshold}")
        print(f"  Cache Enabled: {config.behavior.cache_enabled}")
    
    if args.set:
        for kv in args.set:
            if "=" not in kv:
                print(f"[Error] Invalid format: {kv}. Use KEY=VALUE", file=sys.stderr)
                return 1
            key, value = kv.split("=", 1)
            print(f"[Config] Would set {key}={value}")
    
    return 0


def main() -> int:
    """Main CLI entry point."""
    parser = build_parser()
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 0
    
    # Route to command handler
    handlers = {
        "edit": cmd_edit,
        "batch": cmd_batch,
        "voiceover": cmd_voiceover,
        "subtitle": cmd_subtitle,
        "estimate": cmd_estimate,
        "list-templates": cmd_list_templates,
        "config": cmd_config,
    }
    
    handler = handlers.get(args.command)
    if handler:
        return handler(args)
    
    print(f"[Error] Unknown command: {args.command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
```

Juga buat `auto-editor/__init__.py` dengan versi:

```python
"""OpenCut AI Auto-Editor — Token-efficient video editing automation."""

__version__ = "0.1.0"
__author__ = "MOKO OS Team"
```

### Verifikasi

```bash
python -m auto-editor --help
python -m auto-editor list-templates
python -m auto-editor edit ./footage/ --prompt "buat video cinematic"
python -m auto-editor estimate ./footage/
```

---

## Task 1.11 — Test Suite

### Instruksi

Buat `auto-editor/tests/` dengan test untuk setiap komponen yang sudah dibuat.

### Test Files

**`tests/test_intent_router.py`:**
```python
"""Tests for IntentRouter — rule-based command classification."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.intent_router import IntentRouter
from auto_editor.models import EditingIntent


def test_classify_auto_edit():
    router = IntentRouter()
    tests = [
        ("buat video cinematic", EditingIntent.AUTO_EDIT),
        ("bikin video produk 30 detik", EditingIntent.AUTO_EDIT),
        ("buatkan video promosi", EditingIntent.AUTO_EDIT),
        ("create a video tutorial", EditingIntent.AUTO_EDIT),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected, f"Failed: '{query}' → {intent}, expected {expected}"


def test_classify_voiceover():
    router = IntentRouter()
    tests = [
        ("tambah voiceover", EditingIntent.ADD_VOICEOVER),
        ("buat narasi", EditingIntent.ADD_VOICEOVER),
        ("tambahkan dubbing", EditingIntent.ADD_VOICEOVER),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected


def test_classify_subtitle():
    router = IntentRouter()
    tests = [
        ("buat subtitle", EditingIntent.ADD_SUBTITLE),
        ("tambah teks", EditingIntent.ADD_SUBTITLE),
        ("generate caption", EditingIntent.ADD_SUBTITLE),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected


def test_classify_render():
    router = IntentRouter()
    tests = [
        ("render semua", EditingIntent.BATCH_RENDER),
        ("export video", EditingIntent.RENDER),
        ("simpan hasil", EditingIntent.RENDER),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected


def test_classify_unknown():
    router = IntentRouter()
    tests = [
        "apa kabar",
        "siapa nama kamu",
        "hello world",
        "testing 123",
    ]
    for query in tests:
        intent, _ = router.classify(query)
        assert intent == EditingIntent.UNKNOWN, f"Failed: '{query}' → {intent}"


def test_extract_duration():
    router = IntentRouter()
    assert router.extract_duration("30 detik") == 30
    assert router.extract_duration("2 menit") == 120
    assert router.extract_duration("5 minute video") == 300
    assert router.extract_duration("no duration here") is None


def test_extract_style():
    router = IntentRouter()
    assert router.extract_style("cinematic video") == "cinematic"
    assert router.extract_style("vlog style") == "vlog"
    assert router.extract_style("tutorial content") == "tutorial"
    assert router.extract_style("no style") is None


def test_create_plan():
    router = IntentRouter()
    plan = router.create_plan("buat video tiktok 30 detik produk kopi")
    assert plan.duration == 30
    assert plan.aspect_ratio.value == "9:16"
    assert plan.intent == EditingIntent.AUTO_EDIT
```

**`tests/test_workflow_engine.py`:**
```python
"""Tests for WorkflowEngine — DAG execution with parallel support."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.workflow_engine import WorkflowEngine, WorkflowNode


def test_simple_workflow():
    engine = WorkflowEngine()
    results = {}
    
    def node_a(**kw):
        results["a"] = True
        return {"from_a": "done"}
    
    engine.register("test", [WorkflowNode(id="a", handler=node_a)])
    result = engine.run("test", {})
    assert result.success
    assert results["a"]


def test_sequential_workflow():
    engine = WorkflowEngine()
    order = []
    
    def node_a(**kw):
        order.append("a")
        return {"val": 1}
    
    def node_b(**kw):
        order.append("b")
        assert kw.get("val") == 1
        return {"val": 2}
    
    engine.register("seq", [
        WorkflowNode(id="a", handler=node_a),
        WorkflowNode(id="b", handler=node_b, deps=["a"]),
    ])
    result = engine.run("seq", {})
    assert result.success
    assert order == ["a", "b"]


def test_parallel_workflow():
    engine = WorkflowEngine()
    
    def node_a(**kw):
        return {"from_a": True}
    
    def node_b(**kw):
        return {"from_b": True}
    
    def node_c(**kw):
        return {"merged": {**kw}}
    
    engine.register("parallel", [
        WorkflowNode(id="a", handler=node_a),
        WorkflowNode(id="b", handler=node_b),
        WorkflowNode(id="c", handler=node_c, deps=["a", "b"]),
    ])
    result = engine.run("parallel", {})
    assert result.success
    assert result.processing_time > 0


def test_node_failure():
    engine = WorkflowEngine()
    
    def failing_node(**kw):
        raise RuntimeError("Simulated failure")
    
    engine.register("fail", [WorkflowNode(id="a", handler=failing_node, retry_count=0)])
    result = engine.run("fail", {})
    assert not result.success
    assert len(result.errors) > 0


def test_invalid_dependency():
    engine = WorkflowEngine()
    try:
        engine.register("bad", [
            WorkflowNode(id="a", handler=lambda **_: {}),
            WorkflowNode(id="b", handler=lambda **_: {}, deps=["nonexistent"]),
        ])
        assert False, "Should have raised ValueError"
    except ValueError:
        pass


def test_circular_dependency():
    engine = WorkflowEngine()
    try:
        engine.register("circular", [
            WorkflowNode(id="a", handler=lambda **_: {}, deps=["b"]),
            WorkflowNode(id="b", handler=lambda **_: {}, deps=["a"]),
        ])
        assert False, "Should have raised ValueError"
    except ValueError:
        pass


def test_progress_callback():
    engine = WorkflowEngine()
    updates = []
    
    def callback(node_id, status, progress):
        updates.append((node_id, status))
    
    engine.on_progress(callback)
    engine.register("progress", [
        WorkflowNode(id="a", handler=lambda **_: {}),
    ])
    engine.run("progress", {})
    assert len(updates) > 0
```

**`tests/test_template_db.py`:**
```python
"""Tests for TemplateDB — YAML template management."""

import sys
import tempfile
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.template_db import TemplateDB


def test_list_templates():
    db = TemplateDB("config/templates")
    templates = db.list_all()
    assert len(templates) >= 3  # cinematic, tiktok_product, slideshow


def test_get_template():
    db = TemplateDB("config/templates")
    t = db.get("cinematic")
    assert t is not None
    assert "tracks" in t


def test_find_similar():
    db = TemplateDB("config/templates")
    result = db.find_similar("tiktok product video review")
    assert result is not None


def test_apply_template():
    db = TemplateDB("config/templates")
    elements = db.apply("cinematic", {"TITLE": "Test Video"})
    assert len(elements) > 0
    for el in elements:
        assert hasattr(el, 'position')
        assert hasattr(el, 'timeline')


def test_save_and_delete():
    with tempfile.TemporaryDirectory() as tmpdir:
        db = TemplateDB(tmpdir)
        db.save("test_template", {
            "name": "test_template",
            "tracks": [{"id": "test", "type": "video"}]
        })
        assert db.get("test_template") is not None
        db.delete("test_template")
        assert db.get("test_template") is None


def test_nonexistent_template():
    db = TemplateDB("config/templates")
    assert db.get("nonexistent_template") is None
```

**`tests/test_models.py`:**
```python
"""Tests for data models — dataclass construction and validation."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    EditingPlan, Scene, SceneType, AspectRatio, EditingIntent,
    TokenUsage, WorkflowResult, EditError,
)


def test_coordinate_element_defaults():
    el = CoordinateElement(id="test", type="video")
    assert el.position.x == 0.5
    assert el.position.y == 0.5
    assert el.transform.opacity == 1.0
    assert el.timeline.start == 0.0
    assert el.timeline.end == 10.0


def test_timeline_duration():
    t = Timeline(start=5, end=15)
    assert t.duration == 10


def test_editing_plan_defaults():
    plan = EditingPlan()
    assert plan.duration == 30
    assert plan.intent == EditingIntent.AUTO_EDIT
    assert plan.scenes == []


def test_token_usage():
    usage = TokenUsage()
    usage.add_local(500)
    usage.add_api(300)
    assert usage.local_llm == 500
    assert usage.api_llm == 300
    assert usage.total == 800


def test_workflow_result():
    result = WorkflowResult(success=True)
    assert result.quality_score == 1.0
    assert result.processing_time == 0.0


def test_scene_creation():
    scene = Scene(id=1, scene_type=SceneType.ESTABLISHING, duration=8.0)
    assert scene.id == 1
    assert scene.duration == 8.0
```

**`tests/test_coordinate.py`:**
```python
"""Tests for CoordinateEngine — 0-token math engine."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.workers.layout_engine.coordinate import CoordinateEngine
from auto_editor.models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    Keyframe, Animation,
)


def test_pixel_conversion():
    engine = CoordinateEngine(1920, 1080)
    assert engine.to_pixels(0.5, 1920) == 960
    assert engine.to_pixels(1.0, 1080) == 1080
    assert engine.to_normalized(960, 1920) == 0.5


def test_get_bounds():
    engine = CoordinateEngine(1920, 1080)
    el = CoordinateElement(
        id="test", type="video",
        position=Position(x=0.5, y=0.5),
        size=Size(width=0.5, height=0.5)
    )
    bounds = engine.get_bounds(el)
    assert bounds["width"] == 960  # 0.5 * 1920
    assert bounds["height"] == 540  # 0.5 * 1080


def test_overlap_detection():
    engine = CoordinateEngine(1920, 1080)
    a = CoordinateElement("a", "video", size=Size(0.5, 0.5))
    b = CoordinateElement("b", "video", position=Position(0.5, 0.5), size=Size(0.5, 0.5))
    assert engine.check_overlap(a, b)


def test_no_overlap():
    engine = CoordinateEngine(1920, 1080)
    a = CoordinateElement("a", "video", size=Size(0.1, 0.1))
    b = CoordinateElement("b", "video", position=Position(1, 1), size=Size(0.1, 0.1))
    assert not engine.check_overlap(a, b)


def test_grid_creation():
    engine = CoordinateEngine(1920, 1080)
    grid = engine.create_grid(Position(0, 0), 2, 2)
    assert len(grid) == 4
    # Grid elements should have different x positions
    x_positions = set(el.position.x for el in grid)
    assert len(x_positions) == 2  # 2 columns


def test_rule_of_thirds():
    engine = CoordinateEngine()
    pos = engine.rule_of_thirds("left", "top")
    assert round(pos.x, 4) == round(1/3, 4)
    assert round(pos.y, 4) == round(1/3, 4)


def test_split_screen():
    engine = CoordinateEngine()
    positions = engine.split_screen(4, "grid")
    assert len(positions) == 4


def test_keyframe_interpolation():
    engine = CoordinateEngine()
    elem = CoordinateElement(
        "test", "text",
        animation=Animation(keyframes=[
            Keyframe(time=0, opacity=0),
            Keyframe(time=1, opacity=1),
        ])
    )
    result = engine.apply_keyframe(elem, 0)
    assert result.transform.opacity == 0
    result = engine.apply_keyframe(elem, 1)
    assert result.transform.opacity == 1


def test_safe_zone():
    engine = CoordinateEngine(1920, 1080)
    zone = engine.safe_zone(0.1)
    assert zone["left"] == 192
    assert zone["top"] == 108
    assert zone["width"] == 1536
    assert zone["height"] == 864


def test_center_in_canvas():
    engine = CoordinateEngine()
    pos = engine.center_in_canvas(Size(0.5, 0.5))
    assert pos.x == 0.5
    assert pos.y == 0.5
```

---

## Task 1.12 — Run All Tests & Fix

### Instruksi

Jalankan seluruh test suite dan pastikan semua lulus.

```bash
python -m pytest auto-editor/tests/ -v

# Jika pytest tidak ada:
python -m unittest discover -s auto-editor/tests/ -v
```

### Verifikasi Final

```
=================================== test session starts ===================================
collected 30+ items

tests/test_intent_router.py .........                                               [ 30%]
tests/test_workflow_engine.py .......                                              [ 55%]
tests/test_template_db.py .....                                                    [ 72%]
tests/test_models.py ......                                                        [ 92%]
tests/test_coordinate.py ...                                                       [100%]

=================================== 30 passed in 0.45s ===================================
```

Semua test WAJIB lulus. Jika ada yang gagal, perbaiki kode sampai lulus semua.

---

## DELIVERABLES FINAL AGENT 1

```
Task 1.1  ✅ OpenCut Classic ter-clone dan running
Task 1.2  ✅ Struktur auto-editor/ + worker skeleton
Task 1.3  ✅ Core data models (models.py)
Task 1.4  ✅ Intent Router (intent_router.py)
Task 1.5  ✅ Mandor LLM Bridge (mandor_llm.py) — mock
Task 1.6  ✅ Workflow Engine (workflow_engine.py) — DAG executor
Task 1.7  ✅ Template Database (template_db.py) + 3 template YAML
Task 1.8  ✅ Coordinate Engine (coordinate.py) — 0-token math
Task 1.9  ✅ Config System (settings.yaml + settings_loader.py)
Task 1.10 ✅ CLI Entry Point (main.py)
Task 1.11 ✅ Test Suite (30+ tests)
Task 1.12 ✅ All tests passing
```

**Selesai.** Agent 1 tidak mengerjakan worker implementation detail — itu urusan Agent 2.
Yang penting: semua interface (Protocol), data models, dan orchestration backbone sudah siap
untuk Agent 2 dan Agent 3 bekerja di atasnya.

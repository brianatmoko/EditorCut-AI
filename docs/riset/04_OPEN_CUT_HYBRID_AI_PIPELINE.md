# 04 — OpenCut Hybrid AI Pipeline & Workflow Engine

> **Dokumen spesifikasi pipeline** — bagaimana local LLM (MOKO) dan API eksternal
> bekerja sama dalam satu workflow yang efisien, hemat token, dan resilient.
>
> **Mengadopsi pola:** MOKO Dual-System (Brain + Executor + Guard),
> Hermes AI function calling, OpenClaw pipeline nodes.

---

## 1. Arsitektur Pipeline

### 1.1 Three-Layer Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│                   ORCHESTRATOR LAYER                          │
│  Mandor LLM: Decision making, planning, quality control       │
│  Intent Router: Klasifikasi perintah editing                  │
│  Workflow Engine: Eksekusi pipeline, error handling           │
├──────────────────────────────────────────────────────────────┤
│                    WORKER LAYER                               │
│  Scene Detector  │  Asset Finder  │  Layout Engine            │
│  TTS Engine      │  ASR Engine    │  Compositor               │
│  Effects         │  Color Grade   │  Subtitle                 │
├──────────────────────────────────────────────────────────────┤
│                    RENDER LAYER                               │
│  OpenCut WASM Compositor  │  FFmpeg Encoder  │  MOKO Native   │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 Intent Router untuk Editing

```python
EDITING_INTENTS = {
    "AUTO_EDIT": {
        "keywords": ["buat video", "edit otomatis", "bikin", "buatkan"],
        "handler": "AutoEditWorkflow",
        "token_estimate": 3000
    },
    "ADD_VOICEOVER": {
        "keywords": ["voiceover", "narasi", "suara", "dubbing"],
        "handler": "VoiceoverWorkflow",
        "token_estimate": 500
    },
    "ADD_SUBTITLE": {
        "keywords": ["subtitle", "teks", "caption", "terjemahan"],
        "handler": "SubtitleWorkflow",
        "token_estimate": 200
    },
    "TRIM_VIDEO": {
        "keywords": ["potong", "trim", "cut", "hapus bagian"],
        "handler": "TrimWorkflow",
        "token_estimate": 100
    },
    "CHANGE_LAYOUT": {
        "keywords": ["layout", "tata letak", "posisi", "template"],
        "handler": "LayoutWorkflow",
        "token_estimate": 200
    },
    "ADD_EFFECTS": {
        "keywords": ["efek", "filter", "transisi", "color"],
        "handler": "EffectsWorkflow",
        "token_estimate": 300
    },
    "BATCH_RENDER": {
        "keywords": ["render semua", "batch", "export semua"],
        "handler": "BatchRenderWorkflow",
        "token_estimate": 500
    }
}
```

---

## 2. Token Budget System

### 2.1 Dynamic Token Allocation

Setiap workflow punya **token budget** yang dialokasikan secara dinamis:

```python
class TokenBudget:
    def __init__(self, mode: str = "hybrid"):
        self.mode = mode  # "offline" | "hybrid" | "cloud"
        self.budget = {
            "offline": 0,      # 0 token = pure lokal
            "hybrid": 5000,    # 5K token per project
            "cloud": 20000     # 20K token per project
        }
        self.used = 0
    
    def allocate(self, task_type: str, complexity: float) -> int:
        """
        Alokasikan token berdasarkan tipe task dan kompleksitas.
        complexity: 0.0 (mudah) - 1.0 (sulit)
        """
        base = {
            "planning": 2000,
            "layout": 800,
            "script": 1500,
            "search": 500,
            "review": 800,
            "refine": 2000
        }
        allocated = int(base.get(task_type, 500) * (0.5 + complexity * 0.5))
        
        # Batasi sesuai budget
        remaining = self.budget[self.mode] - self.used
        return min(allocated, remaining)
    
    def spend(self, tokens: int):
        self.used += tokens
    
    def is_exhausted(self) -> bool:
        return self.used >= self.budget[self.mode]
```

### 2.2 Token-Saving Strategies

| Strategy | Description | Token Saved |
|----------|-------------|-------------|
| **Template matching** | Skip AI jika template cocok | ~2000 |
| **Rule-based layout** | Math engine, bukan AI | ~800 |
| **Local TTS/ASR** | 0 token vs cloud TTS | ~500-2000 |
| **Caching** | Cache hasil query RAG | ~200-500 |
| **Progressive refinement** | AI review hanya bagian error | ~1000 |
| **Batch processing** | Satu prompt untuk banyak scene | ~3000 |

---

## 3. Workflow Engine Detail

### 3.1 Workflow Definition

Workflow didefinisikan sebagai DAG (Directed Acyclic Graph):

```python
@dataclass
class WorkflowNode:
    id: str
    type: str  # "planning" | "execution" | "review" | "render"
    handler: str  # nama fungsi worker
    deps: list[str]  # dependency node IDs
    config: dict
    retry_count: int = 2
    timeout: int = 300  # detik

@dataclass
class Workflow:
    id: str
    name: str
    nodes: list[WorkflowNode]
    token_budget: int
    mode: str  # "offline" | "hybrid" | "cloud"
    
class WorkflowEngine:
    def execute(self, workflow: Workflow, input_data: dict) -> dict:
        results = {}
        for node in topological_sort(workflow.nodes):
            # Cek dependency
            for dep in node.deps:
                if dep not in results:
                    raise DependencyError(f"{node.id} waits for {dep}")
            
            # Eksekusi dengan retry
            for attempt in range(node.retry_count + 1):
                try:
                    result = self.run_node(node, input_data, results)
                    results[node.id] = result
                    break
                except Exception as e:
                    if attempt == node.retry_count:
                        raise
                    log(f"Retry {node.id} ({attempt + 1}/{node.retry_count})")
        
        return results
```

### 3.2 Auto-Edit Workflow (Full Pipeline)

```yaml
workflow: "auto-edit"
mode: "hybrid"
token_budget: 5000

nodes:
  - id: "analyze_brief"
    type: "planning"
    handler: "mandor.analyze"
    config:
      prompt_template: "analyze_edit_request"
      max_tokens: 500
    deps: []
    token_cost: 500

  - id: "find_assets"
    type: "execution"
    handler: "workers.asset_finder.search"
    config:
      sources: ["local", "pexels", "pixabay"]
      max_results: 10
    deps: ["analyze_brief"]
    token_cost: 0

  - id: "generate_script"
    type: "planning"
    handler: "mandor.generate_script"
    config:
      language: "auto"
      style: "auto"
      max_tokens: 1000
    deps: ["analyze_brief"]
    token_cost: 800

  - id: "storyboard"
    type: "planning"
    handler: "mandor.storyboard"
    config:
      max_scenes: 12
    deps: ["find_assets", "generate_script"]
    token_cost: 500

  - id: "voiceover"
    type: "execution"
    handler: "workers.audio.tts"
    config:
      engine: "local"  # force local
      voice: "auto"
    deps: ["generate_script"]
    token_cost: 0

  - id: "layout_scenes"
    type: "execution"
    handler: "workers.layout.apply_template"
    config:
      template: "auto_select"
    deps: ["storyboard"]
    token_cost: 0

  - id: "apply_effects"
    type: "execution"
    handler: "workers.effects.auto_apply"
    config:
      color_grade: true
      transitions: true
    deps: ["layout_scenes"]
    token_cost: 0

  - id: "quality_review"
    type: "planning"
    handler: "mandor.review"
    config:
      check: ["lip_sync", "timing", "layout", "audio_level"]
      confidence_threshold: 0.7
    deps: ["voiceover", "layout_scenes", "apply_effects"]
    token_cost: 300

  - id: "render"
    type: "execution"
    handler: "workers.renderer.render"
    config:
      format: "mp4"
      resolution: "1080p"
      codec: "h264"
    deps: ["quality_review"]
    token_cost: 0

  - id: "final_review"
    type: "planning"
    handler: "mandor.final_review"
    config:
      check: ["corruption", "aspect_ratio", "duration_match"]
    deps: ["render"]
    token_cost: 200
```

---

## 4. Mandor LLM Decision System

### 4.1 System Prompt Strategy

Mandor LLM menggunakan **dynamic system prompt** yang berubah sesuai fase:

```python
SYSTEM_PROMPTS = {
    "analyze": """
    Anda adalah analis brief editing video.
    Tugas Anda: ekstrak parameter dari permintaan user.
    
    Output JSON:
    {
        "duration": number (detik),
        "style": string,
        "mood": string,
        "target_platform": string,
        "key_elements": string[],
        "has_voiceover": boolean,
        "music_style": string | null,
        "special_requirements": string[]
    }
    
    JANGAN generate konten. JANGAN tulis skrip. 
    HANYA ekstrak parameter.
    """,
    
    "storyboard": """
    Anda adalah sutradara video yang membuat storyboard.
    Berdasarkan: brief analysis + available assets + script.
    
    Buat scene-by-scene breakdown:
    - Setiap scene: durasi, jenis shot, asset yang dipakai
    - Narrative flow: bagaimana cerita mengalir
    - Transitions: antar scene
    
    Output JSON array of scenes.
    Gunakan bahasa yang SINGKAT. Ini untuk koordinasi, bukan untuk publikasi.
    """,
    
    "review": """
    Anda adalah quality assurance editor.
    Periksa hasil editing dan cari masalah:
    - Timing mismatch (voiceover tidak sinkron)
    - Layout error (elemen bertumpuk)
    - Audio issues (volume tidak seimbang)
    - Missing assets (ada scene tanpa video)
    
    Jika tidak ada masalah: return {"passed": true}
    Jika ada masalah: return {"passed": false, "issues": [...], "fix_plan": "..."}
    
    PRIORITAS: lebih baik false positive (cek ulang) dari pada false negative (rusak).
    """
}
```

### 4.2 Confidence Scoring

Setiap keputusan mandor diberi confidence score:

```python
class Decision:
    content: dict
    confidence: float  # 0.0 - 1.0
    source: str  # "local_llm" | "api_llm" | "rule_engine"
    
    def is_reliable(self) -> bool:
        if self.source == "rule_engine":
            return True  # Rule-based selalu reliable
        return self.confidence >= CONFIDENCE_THRESHOLDS[self.source]

# Threshold
CONFIDENCE_THRESHOLDS = {
    "local_llm": 0.7,  # MOKO-4B
    "api_llm": 0.9,    # GPT-4o-mini
}
```

---

## 5. Caching & Optimization

### 5.1 Multi-Level Cache

```
┌────────────────────────────┐
│     Level 1: Memory Cache   │  ← Dict cache, cepat, limited
│     TTL: 5 menit            │
├────────────────────────────┤
│     Level 2: Disk Cache     │  ← JSON/YAML, persistent
│     TTL: 24 jam             │
├────────────────────────────┤
│     Level 3: Template DB    │  ← YAML templates, permanent
│     TTL: infinite           │
└────────────────────────────┘
```

### 5.2 Cache Keys

```python
def cache_key(workflow: str, input_hash: str) -> str:
    """Generate cache key dari workflow + input"""
    return f"opencut:cache:{workflow}:{input_hash}"

def should_use_cache(query: str, context: dict) -> bool:
    """Cek apakah boleh pakai cache"""
    # Jangan cache jika user explicit minta fresh
    if any(kw in query.lower() for kw in ["baru", "fresh", "jangan cache"]):
        return False
    # Cache untuk template matching selalu
    if context.get("task") == "template_match":
        return True
    # Cache untuk RAG dengan threshold similarity
    return context.get("similarity", 0) > 0.85
```

### 5.3 Progressive Refinement

Daripada regenerate semuanya, sistem hanya refine bagian yang salah:

```
ITERASI 1: Full generate (3000 token)
  → Hasil: 80% bagus, 20% perlu perbaikan
    
ITERASI 2: Refine only (500 token)
  → Hanya bagian yang error diperbaiki
  → LLM dikirimi: "Bagian [scene 3, 5, 7] perlu diperbaiki karena ..."
  → Token hemat: 2500 (5x lebih hemat)
    
ITERASI N: Sampai quality review passed atau budget habis
```

---

## 6. Error Handling & Resilience

### 6.1 Error Types & Recovery

| Error | Penyebab | Recovery | Token Impact |
|-------|----------|----------|--------------|
| **Asset not found** | Video/gambar tidak ada di source | Cari alternatif, atau prompt user | +100 |
| **TTS failed** | Model TTS crash | Fallback ke TTS lain, atau simpan sebagai teks | 0 |
| **Scene detect timeout** | Video terlalu panjang | Split video, proses paralel | 0 |
| **Layout overflow** | Elemen bertumpuk/tumpang tindih | Auto-adjust posisi, push ke safe zone | 0 |
| **Quality review fail** | Timing/layout tidak sesuai | Loop refine (maks 3x) | +500/loop |
| **Render error** | FFmpeg/OpenCut crash | Fallback ke encoder lain | 0 |
| **LLM timeout** | Model lokal terlalu lambat | Fallback ke rule-based | 0 |

### 6.2 Degradation Path

```
HIGH QUALITY (Hybrid mode)
  → Local LLM + API quality boost
  → Jika API down: turun ke OFFLINE
  → Jika local LLM down: turun ke RULE-ONLY
  
OFFLINE MODE
  → Local LLM + Local TTS + Local ASR
  → Jika local LLM down: turun ke RULE-ONLY
  
RULE-ONLY MODE
  → Template matching + Rule engine
  → Tidak ada AI sama sekali
  → Tetap bisa edit, tapi tanpa inteligence
  
EMERGENCY MODE
  → Hanya render ulang project terakhir
  → Output: last known good config
```

---

## 7. Monitoring & Logging

### 7.1 Telemetry per Project

```json
{
  "project_id": "abc123",
  "duration": 60,
  "mode": "hybrid",
  "token_usage": {
    "total": 4200,
    "local_llm": 3200,
    "api_llm": 1000,
    "by_phase": {
      "analyze": 400,
      "storyboard": 800,
      "script": 1200,
      "review": 800,
      "refine": 1000
    }
  },
  "timing": {
    "total_seconds": 120,
    "planning": 15,
    "execution": 90,
    "render": 15
  },
  "errors": [],
  "cache_hits": 3,
  "quality_score": 0.92
}
```

### 7.2 Token Analytics Dashboard

```
TOKEN USAGE per PROJECT (rata-rata):
├─ Planning: 1,200 token (28%)
│  ├─ Brief analysis: 400
│  └─ Storyboard: 800
├─ Script generation: 1,500 token (36%)
├─ Quality review: 600 token (14%)
├─ Refinement: 800 token (19%)
└─ Other: 100 token (3%)

SAVINGS:
├─ Template matching: -2,000 token
├─ Rule-based layout: -800 token
├─ Local TTS (instead of API): -1,500 token
├─ Caching: -500 token
└─ TOTAL SAVED: ~5,000 token per project (70% lebih hemat)
```

---

## 8. Command Line Interface

### 8.1 CLI Usage

```bash
# Auto-edit dari folder footage
opencut-auto edit ./footage/ --script script.txt --output result.mp4

# Batch render dari direktori
opencut-auto batch ./projects/ --format mp4 --resolution 4k

# Voiceover only
opencut-auto voiceover --text "narasi.txt" --voice id --output audio.wav

# Subtitle from video
opencut-auto subtitle video.mp4 --language id --output subtitle.srt

# Mode offline (0 token)
opencut-auto edit ./footage/ --mode offline

# Preview token cost before running
opencut-auto estimate ./footage/ --script script.txt
```

### 8.2 API Usage

```python
from opencut_auto import AutoEditor

editor = AutoEditor(mode="hybrid")

# Auto edit
result = editor.edit(
    footage_dir="./footage/",
    script_path="script.txt",
    output="result.mp4",
    style="cinematic"
)

# Batch
results = editor.batch([
    {"footage": "./project1/", "script": "s1.txt"},
    {"footage": "./project2/", "script": "s2.txt"},
], output_dir="./output/")

# Pipeline custom
editor.run_pipeline([
    ("analyze", {"brief": "Buat video produk kopi"}),
    ("find_assets", {"keywords": ["kopi", "coffee", "barista"]}),
    ("layout", {"template": "product-showcase"}),
    ("render", {"format": "mp4"})
])
```

---

## 9. Integrasi Mapping ke MOKO OS

| Komponen MOKO OS | Padanan di Auto-Editor | Fungsi |
|-----------------|------------------------|--------|
| `dual_system/brain_node.py` | `mandor_llm` (System 2) | Planning, reasoning, review |
| `dual_system/executor_node.py` | `workers/*` (System 1) | Eksekusi editing, render |
| `dual_system/runtime_guard.py` | `quality_review` | Validasi hasil, deteksi error |
| `dual_system/orchestrator.py` | `workflow_engine` | Koordinasi semua node |
| `moko_rag/` | `asset_finder/rag_search.py` | Cari asset dari database lokal |
| `moko_crawler/` | `asset_finder/crawler.py` | Cari asset dari web |
| `moko_native/` | `renderer/ffmpeg_pipeline.py` | Akselerasi native compositing |
| `moko_agents/intent_router.py` | `intent_router.py` | Klasifikasi perintah editing |
| `Byte-Q quantization` | `models/` (compressed) | Model TTS/ASR terkompresi |
| `marathon_engine` | `batch_render` | Batch processing auto-continue |

---

## 10. Ringkasan

Hybrid AI Pipeline ini dirancang untuk:

1. **Zero-waste token usage** — token cuma dipakai untuk decision making
2. **Graceful degradation** — dari hybrid → offline → rule-only, tidak pernah crash total
3. **Progressive refinement** — refine bagian error, bukan regenerate semua
4. **Full lokal capability** — 100% fungsi tanpa internet, tanpa API key
5. **MOKO OS native** — sharing model, sharing infrastructure, sharing philosophy

> "Bukan AI yang mengedit video. Tapi AI yang mengoordinasikan alat-alat editing
> dengan cara yang paling hemat token."

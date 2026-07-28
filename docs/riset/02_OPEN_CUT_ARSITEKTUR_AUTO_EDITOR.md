# 02 — OpenCut AI Auto-Editor: Arsitektur Teknis

> **Dokumen arsitektur** — rancangan teknis sistem auto-editing yang bekerja
> seperti editor manusia: mencari bahan, membuat voiceover, menata layout,
> dan render — tanpa ketergantungan berlebihan pada token AI.
>
> **Base platform:** OpenCut Classic (opencut-app/opencut-classic)
> **AI orchestrator:** MOKO OS hybrid LLM system
> **License:** MIT

---

## 1. Struktur Folder Proyek

```
Opencut/
├── apps/
│   └── web/                    # OpenCut web app (React + Vite)
├── auto-editor/                # NEW: Sistem auto-editing
│   ├── orchestrator/           # Mandor AI orchestrator
│   │   ├── mandor_llm.py       # Local LLM bridge (MOKO-AI-4B)
│   │   ├── intent_router.py    # Route perintah editing
│   │   ├── workflow_engine.py  # Pipeline orchestrator
│   │   └── template_db.py      # Template editing & layout
│   │
│   ├── workers/                # Pekerja spesialis
│   │   ├── scene_detector/     # Deteksi scene otomatis
│   │   │   ├── detector.py     # FFmpeg scene detect
│   │   │   └── classifier.py   # Klasifikasi jenis scene
│   │   ├── asset_finder/       # Pencari bahan
│   │   │   ├── crawler.py      # Web crawler (Pexels, Pixabay)
│   │   │   ├── rag_search.py   # RAG dari library lokal
│   │   │   └── downloader.py   # Download & cache asset
│   │   ├── layout_engine/      # Coordinate-based layout
│   │   │   ├── coordinate.py   # Sistem koordinat 4D (x,y,z,t)
│   │   │   ├── compositor.py   # Compositing logic
│   │   │   └── template.py     # Template layout siap pakai
│   │   ├── audio_pipeline/     # Pipeline audio
│   │   │   ├── tts_engine.py   # Local TTS (CosyVoice/Bark)
│   │   │   ├── asr_whisper.py  # Local ASR (Whisper.cpp)
│   │   │   ├── alignment.py    # Align voiceover ke timeline
│   │   │   └── mixer.py        # Audio mixing & mastering
│   │   ├── effects/            # Efek & transisi
│   │   │   ├── color_grade.py  # Auto color grading
│   │   │   ├── transition.py   # Transisi cerdas
│   │   │   └── text_overlay.py # Text & subtitle generator
│   │   └── renderer/           # Render engine
│   │       ├── opencut_bridge.py   # Bridge ke OpenCut render
│   │       └── ffmpeg_pipeline.py  # FFmpeg fallback render
│   │
│   ├── api/                    # API endpoint (opsional)
│   │   ├── routes.py           # REST API untuk trigger editing
│   │   └── websocket.py        # Real-time progress
│   │
│   ├── models/                 # Local AI models
│   │   ├── tts/                # Model TTS lokal (GGUF format)
│   │   ├── asr/                # Model Whisper (GGUF)
│   │   └── scene/              # Model scene detection (ONNX)
│   │
│   └── config/                 # Konfigurasi
│       ├── settings.yaml       # Global settings
│       ├── templates/          # Template YAML layout
│       └── providers.yaml      # API provider config
│
├── moko_bridge/                # NEW: Bridge ke MOKO OS
│   ├── moko_client.py          # Client komunikasi ke MOKO OS
│   └── moko_models.py          # Shared model definitions
│
├── docs/                       # Dokumentasi
│   └── riset/                  # Dokumentasi riset
│       ├── 01_*                # Visi & filosofi
│       ├── 02_*                # Arsitektur (ini)
│       ├── 03_*                # Coordinate layout
│       └── 04_*                # Hybrid AI pipeline
│
└── README.md
```

---

## 2. Alur Kerja Auto-Editing (Seperti Manusia)

### 2.1 Flowchart Lengkap

```
USER INPUT: "Buat video cinematic 60 detik tentang pantai"
     │
     ▼
┌─────────────────────────────────────┐
│  PHASE 1: BRIEF ANALYSIS            │ ◄── MANDOR (Local LLM)
│  ────────────────────                │
│  • Intent routing                    │     Token: ~500
│  • Ekstrak parameter (durasi, tema)  │
│  • Tentukan template editing         │
│  • Output: EditingPlan{JSON}         │
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│  PHASE 2: MATERIAL GATHERING        │ ◄── PEKERJA (Parallel)
│  ─────────────────────                │
│  ├─ Scene Detector: Scan footage     │     Token: 0
│  │   → Shot list + timestamps        │     (FFmpeg rules)
│  ├─ Asset Finder: Cari B-Roll        │
│  │   → URL/cache video pantai        │     Token: ~300 (RAG)
│  └─ Audio Check: Analisis audio      │
│      → Noise level, speech detect    │     Token: 0
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│  PHASE 3: STORYBOARD ASSEMBLY       │ ◄── MANDOR + LAYOUT
│  ─────────────────────                │
│  • Urutkan scene berdasarkan narasi  │     Token: ~800
│  • Tentukan durasi per scene         │
│  • Layout setiap shot di coordinate  │     Token: 0
│  • Output: Storyboard{Timeline}      │
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│  PHASE 4: PRODUCTION                │ ◄── PEKERJA (Sequential)
│  ───────────────────                  │
│  ├─ Voiceover: Local TTS generate    │     Token: 0
│  │   → Audio file + timing           │
│  ├─ Layout Engine: Apply coordinate  │     Token: 0
│  │   → Posisi video, teks, efek      │
│  ├─ Effects: Auto color + transition │     Token: 0
│  └─ Subtitle: ASR → .srt file        │     Token: 0
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│  PHASE 5: QUALITY REVIEW            │ ◄── MANDOR
│  ───────────────────                  │
│  • Cek timing voiceover vs visual    │     Token: ~300
│  • Deteksi anomali layout            │
│  • Jika OK → render. Jika tidak →    │
│    loop ke fase 3 dengan koreksi     │
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│  PHASE 6: RENDER                    │ ◄── RENDERER
│  ───────────────                      │
│  • OpenCut WASM composite            │     Token: 0
│  • FFmpeg encode H.264/H.265         │
│  • Output: final_video.mp4           │
└─────────────────────────────────────┘
```

### 2.2 Data Flow Antar Fase

```
EditingPlan {
  intent: "cinematic_beach_video",
  duration: 60,           // detik
  aspect_ratio: "16:9",
  style: "cinematic",
  voiceover: {
    language: "id",
    style: "narasi_tenang",
    script: "..."         // di-generate atau dari user
  },
  scenes: [
    {
      id: 1,
      type: "establishing",
      duration: 8,
      source: "auto_find",
      layout: { /* coordinate */ },
      voiceover: { start: 0, end: 8, text: "..." }
    },
    ...
  ],
  audio: {
    background_music: "auto_select",
    volume: { music: 0.3, voiceover: 1.0 }
  },
  effects: {
    color_grade: "warm_cinematic",
    transitions: "crossfade"
  }
}
```

---

## 3. Material Gathering (Seperti Editor Mencari Bahan)

### 3.1 Sumber Asset

| Sumber | Metode | Token Cost | Prioritas |
|--------|--------|------------|-----------|
| **Local Library** | RAG search dari folder asset lokal | ~300 (embedding) | Tertinggi |
| **Pexels API** | REST API, filter by keyword/color | 0 (REST) | Tinggi |
| **Pixabay API** | REST API, video + audio | 0 (REST) | Tinggi |
| **Internet Archive** | Crawl public domain footage | 0 | Sedang |
| **User Upload** | Drag-drop langsung ke OpenCut | 0 | Tertinggi |

### 3.2 Scene Detection & Klasifikasi

```
INPUT: video.mp4
  │
  ▼
FFmpeg scene detect (scene_threshold=0.3)
  │
  ├─ Shot 1: [0:00-0:05] → Landscape (establishing)
  ├─ Shot 2: [0:05-0:12] → Close-up object
  ├─ Shot 3: [0:12-0:20] → People talking
  └─ Shot N: ...
  │
  ▼
Klasifikasi per scene (rule-based + vision model kecil):
  • establishing / wide / medium / closeup / detail
  • interior / exterior
  • day / night
  • motion: static / pan / zoom
```

### 3.3 Asset Matching Logic

Mandor LLM menentukan kebutuhan per scene, lalu asset finder mencocokkan:

```
Scene: "sunset beach establishing shot"
  → Query RAG: "sunset beach wide shot" 
  → Filter: durasi > 5s, resolusi > 1080p
  → Rank: cosine similarity
  → Return: URL/path terbaik
```

---

## 4. Voiceover Pipeline (Lokal, 0 Token)

### 4.1 Alur

```
SCRIPT INPUT: "Pantai ini terletak di selatan pulau Jawa..."
  │
  ▼
┌─────────────────────┐
│ Text Processor      │ ← Bersihkan teks, tambah SSML tags
│  • Segmentasi kalimat│
│  • Deteksi emosi    │
│  • Tambah pause tags│
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│ Local TTS Engine    │ ← CosyVoice / Bark / XTTS (GGUF)
│  • Model: ~500MB-1GB │     Token: 0
│  • Voice: pilih dari│
│    library suara     │
│  • Speed: 1.0x      │
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│ Audio Post-Process  │ ← Equalizer, noise gate, compressor
│  • Normalize -3dB   │     Token: 0
│  • Add reverb (jika │
│    cinematic)       │
└─────────┬───────────┘
          ▼
    voiceover.wav + timing.json
```

### 4.2 Timing Alignment

Voiceover dihasilkan dengan timing per kata/kalimat:

```json
{
  "segments": [
    {"text": "Pantai ini terletak", "start": 0.0, "end": 1.5},
    {"text": "di selatan pulau Jawa", "start": 1.5, "end": 3.2},
    {"text": "dengan pasir putih yang memukau", "start": 3.2, "end": 5.8}
  ]
}
```

Timeline editor otomatis menyesuaikan durasi visual berdasarkan timing voiceover.

---

## 5. Layout Engine (Coordinate-Based)

> Detail lengkap ada di `03_OPEN_CUT_KOORDINAT_LAYOUT.md`

### 5.1 Prinsip

Setiap elemen visual diposisikan dengan koordinat absolut:

```
Element = {
  type: "video" | "text" | "image" | "effect",
  x: 0.0,        // 0.0 = kiri, 1.0 = kanan (relative)
  y: 0.0,        // 0.0 = atas, 1.0 = bawah
  z: 0,          // Layer stacking (0 = background)
  t_start: 0.0,  // Muncul di detik ke-
  t_end: 5.0,    // Hilang di detik ke-
  width: 0.5,    // Lebar relatif
  height: 0.5,   // Tinggi relatif
  rotation: 0,   // Derajat
  opacity: 1.0,  // 0.0 - 1.0
  scale: 1.0,    // Scale factor
  anchor: "center", // Pivot point
  easing: "ease-in-out" // Animasi
}
```

### 5.2 Contoh Layout "Cinematic Product Showcase"

```
Track 1 (z=0): Video background fullscreen [0:00 - 30:00]
  x:0, y:0, w:1.0, h:1.0

Track 2 (z=1): Product overlay [5:00 - 15:00]
  x:0.7, y:0.7, w:0.25, h:0.25, rotation:15

Track 3 (z=2): Text title [0:00 - 5:00]
  x:0.5, y:0.2, w:0.8, h:0.15
  text: "Produk Kopi Terbaik", font_size: 48, color: white

Track 4 (z=3): Subtitle [0:00 - 30:00]
  x:0.5, y:0.9, w:0.9, h:0.1
  text: "(auto dari ASR)", font_size: 24
```

---

## 6. Efek & Transisi Cerdas

### 6.1 Auto Color Grading

```
Analisis histogram per scene:
  ├─ Underexposed → Brightness + contrast fix
  ├─ Warm scene → Tingkatkan orange/red tone
  ├─ Cool scene → Tingkatkan blue tone
  └─ Flat profile → Apply LUT "cinematic" / "vintage"
  
Rule-based, 0 token. GPU-accelerated via OpenCut WASM.
```

### 6.2 Smart Transitions

```
Antara Scene A dan Scene B:
  ├─ Jika location berbeda → Crossfade 0.5s
  ├─ Jika mood berubah drastis → Dip to black
  ├─ Jika same scene cut → Hard cut
  └─ Jika montage → Spin/blur transition
  
Template matching, 0 token.
```

---

## 7. Render Pipeline

### 7.1 OpenCut Integration

Kita tidak membangun renderer dari nol — kita memanfaatkan **OpenCut WASM compositor**:

```
┌─────────────────────────────────────────┐
│  auto-editor → Output: Project File      │
│  Format: JSON timeline + asset references│
└─────────────────┬───────────────────────┘
                  ▼
┌─────────────────────────────────────────┐
│  OpenCut Render Bridge                   │
│  ──────────────────────                   │
│  • Load project ke OpenCut internal format│
│  • Trigger compositor (WASM)             │
│  • Stream progress ke user               │
└─────────────────┬───────────────────────┘
                  ▼
┌─────────────────────────────────────────┐
│  FFmpeg Encode                           │
│  ──────────────                           │
│  • H.264/H.265 hardware encoding          │
│  • Output format sesuai target            │
│  • Web: 1080p@30fps, H.264                │
└─────────────────────────────────────────┘
```

### 7.2 Fallback Runtimes

Ketika OpenCut WASM compositor belum support fitur tertentu:

```
OpenCut WASM → Compositor support?
  ├─ YES → Pakai OpenCut native
  ├─ NO → Fallback ke FFmpeg filter graph
  └─ PARTIAL → Hybrid: sebagian WASM, sebagian FFmpeg
```

---

## 8. Mode Operasi

### 8.1 Tiga Mode Utama

| Mode | Deskripsi | Token | Internet | Cocok Untuk |
|------|-----------|-------|----------|-------------|
| **Offline** | 100% lokal, MOKO + TTS + ASR semuanya lokal | 0 | Tidak | Daily use, privasi |
| **Hybrid** | Mandor lokal, API untuk quality boost | ~1-3K | Ya | Hasil optimal |
| **Cloud** | Full API, MOKO sebagai router saja | ~5-10K | Ya | Hardware rendah |

### 8.2 Konfigurasi Provider

```yaml
# providers.yaml
mode: "hybrid"  # offline | hybrid | cloud

local:
  llm: "moko/MOKO-AI-4B-Q3_K_M.gguf"
  tts: "cosyvoice/CosyVoice-300M.gguf"
  asr: "whisper/whisper-small.gguf"

api:
  llm:
    provider: "openai"  # atau "openrouter" / "claude"
    model: "gpt-4o-mini"
    max_tokens: 2000
  tts: null  # gunakan lokal
  scene_detect: null  # gunakan FFmpeg

fallback:
  confidence_threshold: 0.7  # jika local < 0.7, panggil API
  retry_count: 2
```

---

## 9. Ringkasan

Arsitektur ini dirancang agar:
1. **Seperti editor manusia** — setiap fase terdefinisi jelas, dari brief hingga export
2. **Token-hemat** — AI hanya untuk decision making, komputasi untuk rendering
3. **Modular** — setiap worker bisa diganti/diupgrade tanpa mengganggu yang lain
4. **Local-first** — berfungsi penuh tanpa internet, privasi terjaga
5. **OpenCut compatible** — memanfaatkan ekosistem OpenCut yang sudah mature

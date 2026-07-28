# 01 — OpenCut AI Auto-Editor: Visi & Filosofi Sistem

> **Dokumen master visi** — mendefinisikan filosofi, pendekatan token-efficient,
> dan arsitektur hybrid lokal + API untuk sistem auto-editing video berbasis AI
> yang bekerja seperti editor manusia sungguhan.
>
> **Terinspirasi dari:** OpenClaw, Hermes AI, MOKO OS Dual-System Architecture,
> Fugu/Sakana AI Orchestration, dan prinsip ekonomi token MOKO Byte-Q.

---

## 1. Filosofi Dasar: "Bekerja Seperti Manusia, Berpikir Seperti AI"

Kebanyakan AI video editor saat ini bergantung pada satu model besar yang mencoba
melakukan semuanya dalam satu prompt raksasa — mahal, boros token, dan sering
kehilangan konteks. Sistem ini mengambil pendekatan berbeda:

### 1.1 Prinsip Kerja Seperti Editor Manusia

| Tahap | Manusia | AI Kita |
|-------|---------|---------|
| **1. Paham Brief** | Baca deskripsi project, catat kebutuhan | Local LLM (MOKO-AI-4B) analisis intent, ekstrak parameter |
| **2. Cari Bahan** | Google, ambil video/audio/gambar dari library | Web crawler + local asset database + RAG search |
| **3. Storyboard** | Sketch kasar timeline di kertas | Coordinate-based layout engine + template matching |
| **4. Voiceover** | Rekam suara, edit audio | Local TTS (CosyVoice/Bark/XTTS) + ASR alignment |
| **5. Layout** | Drag-drop elemen ke timeline | Coordinate positioning (x, y, z, waktu) |
| **6. Export** | Render final | FFmpeg/OpenCut render pipeline |

### 1.2 Token-Efficient: Prinsip Ekonomi Token

Sistem ini dirancang untuk **tidak membuang token pada hal yang bisa dihitung**.

```
INPUT: Video mentah 10 menit + skrip 500 kata
┌─────────────────────────────────────────────────────┐
│  NAIF (1 model besar, 1 prompt):                    │
│  └─ Prompt 32K token → $0.30/per video ❌           │
│                                                      │
│  KITA (Hybrid, orchestrated):                       │
│  ├─ Local LLM: 2K token → analisis tujuan           │
│  ├─ Rules engine: 0 token → scene detect             │
│  ├─ Coordinate math: 0 token → layout hitung         │
│  ├─ Local TTS: 0 token → voiceover generate          │
│  └─ API (optional): 1K token → refine jika perlu    │
│  Total: ~3K token ($0.02) + local compute (free) ✅  │
└─────────────────────────────────────────────────────┘
```

**Strategi utama:**
1. **0-token operations** — apa yang bisa dihitung dengan rumus/rule, jangan pakai AI
2. **Local-first** — TTS, ASR, scene detection jalan di lokal (WASM/C++/Rust)
3. **Hybrid API** — API hanya dipanggil untuk tugas yang benar-benar perlu nalar tinggi
4. **Caching cerdas** — template layout, efek, transisi di-cache, tidak digenerate ulang

---

## 2. Arsitektur Hybrid: Mandor Lokal + Pekerja Spesialis

Mengadopsi pola **Mandor–Pekerja** dari MOKO OS (`docs/riset/22_HYBRID_ORCHESTRATION_MANDOR_PEKERJA.md`):

```
┌──────────────────────────────────────────────────┐
│                   MANDOR (Local LLM)               │
│  MOKO-AI-4B / MOKO-Coder-1.5B                     │
│  ─────────────────────────────                      │
│  • Analisis intent editing                          │
│  • Buat rencana kerja (step-by-step)                │
│  • Delegasi ke pekerja spesialis                    │
│  • Review & refine hasil                            │
│  • 100% lokal — 0 token cost                        │
└──────────────┬───────────────────────────────────┘
               │ orchestrates
    ┌──────────┼──────────┬──────────┬──────────┐
    ▼          ▼          ▼          ▼          ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│SCENE    │ │ASSET    │ │LAYOUT   │ │AUDIO    │ │RENDER   │
│DETECTOR│ │FINDER   │ │ENGINE   │ │PIPELINE │ │ENGINE   │
├─────────┤ ├─────────┤ ├─────────┤ ├─────────┤ ├─────────┤
│FFmpeg   │ │Web      │ │Canvas   │ │Local TTS│ │FFmpeg   │
│+ Python │ │Crawler  │ │2D Coord │ │+ ASR    │ │OpenCut  │
│Rule-based│ │+ RAG    │ │Math     │ │Whisper  │ │WASM     │
│0 token  │ │0-1K tok │ │0 token  │ │0 token  │ │0 token  │
└─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

### 2.1 Kapan API Eksternal Dipanggil?

API (OpenAI/Claude/fal.ai) HANYA dipanggil dalam 3 skenario:
1. **Quality boost** — Local LLM kurang yakin dengan hasil (confidence < 0.7)
2. **Creative generation** — Butuh ide kreatif di luar pattern database
3. **Script polishing** — User minta bahasa atau tone tertentu yang sulit lokal

---

## 3. Integrasi dengan MOKO OS Ecosystem

Sistem ini bukan standalone — ia adalah **modul dari MOKO OS**:

| Komponen MOKO OS | Peran di Auto-Editor |
|------------------|----------------------|
| `dual_system/` | Brain (System 2) rencanakan editing, Executor (System 1) eksekusi |
| `moko_rag/` | Cari asset video/audio dari database lokal |
| `moko_crawler/` | Cari footage gratis dari web (Pexels, Pixabay) |
| `moko_agents/` | Intent router untuk perintah editing |
| `Byte-Q` | Kompresi model TTS/ASR agar muat di RAM 4GB |
| `native_accel/` | Akselerasi C++/Rust untuk compositing & rendering |
| `marathon_engine/` | Batch rendering multi-project (auto-continue) |

---

## 4. Target Kinerja

| Metrik | Target | Keterangan |
|--------|--------|------------|
| **Token per project** | < 5K token | 10x lebih hemat dari naif |
| **Waktu editing 5 menit video** | < 3 menit | Local pipeline parallel |
| **VRAM usage** | < 4GB | Berjalan di GPU entry-level |
| **Voiceover quality** | MOS > 3.5 | Local TTS setara API |
| **Layout accuracy** | > 95% | Coordinate-based, precision pixel |

---

## 5. Perbandingan dengan Pendekatan Lain

| Aspek | CapCut AI | OpenClaw | Hermes AI | **Kita (MOKO+OpenCut)** |
|-------|-----------|----------|-----------|--------------------------|
| **Cloud dependency** | Wajib | Minimal | Partial | **0% (opsional)** |
| **Token cost** | Tinggi | Sedang | Sedang | **Rendah (< 5K/project)** |
| **Layout system** | Timeline | Node-based | Script | **Coordinate-based (x,y,z,t)** |
| **Voiceover** | Cloud TTS | Eksternal | Lokal | **Local TTS + ASR** |
| **Asset sourcing** | Library sendiri | Manual | AI search | **Crawler + RAG + manual** |
| **License** | Proprietary | ? | ? | **MIT (OpenCut) + MOKO** |
| **Offline capable** | Tidak | Sebagian | Sebagian | **Ya, full offline mode** |

---

## 6. Contoh Skenario: "Buat video TikTok 30 detik"

**INPUT USER:** "Buat video promosi produk kopi, 30 detik, gaya cinematic dengan
voiceover bahasa Indonesia, tampilkan produk dari berbagai sudut"

**ALUR KERJA (TANPA API):**
```
1. Mandor (MOKO-4B): Analisis → "TikTok 30s, produk kopi, cinematic, voiceover ID"
   → Token: ~800 (local, gratis)

2. Scene Detector (Rule-based): Bagi 30 detik → 6 segmen @ 5 detik
   → Token: 0

3. Asset Finder (RAG + Crawler): Cari video kopi, biji kopi, seduhan dari library lokal
   → Token: ~400 (embedding search)

4. Layout Engine (Coordinate): Posisi elemen berdasarkan template "product showcase"
   → Token: 0 (rumus matematika)

5. Voiceover (Local TTS): Generate narasi 30 detik dari script
   → Token: 0 (CosyVoice lokal)

6. Compositor (OpenCut + FFmpeg): Gabung semua, render 1080p
   → Token: 0

TOTAL TOKEN: ~1.200 (≈ $0.001)
TANPA API EKSTERNAL SAMA SEKALI
```

---

## 7. Inspirasi OpenClaw & Hermes AI

### OpenClaw-style:
- **Modular pipeline** — Setiap tahap editing adalah node yang bisa di-swap
- **Scriptable** — User bisa override keputusan AI dengan skrip Lua/Python
- **Deterministic fallback** — Jika AI error, pakai rule-based yang stabil

### Hermes AI-style:
- **Function calling** — LLM memanggil tool editing spesifik (bukan generate semuanya)
- **Chain-of-thought** — LLM menunjukkan reasoning sebelum action
- **Self-correction** — Deteksi error layout/audio dan fix otomatis

---

## 8. Ringkasan

Sistem ini bukan "AI yang mengedit video" — tapi **orchestrator yang mengoordinasikan
tool editing** dengan cara yang sangat efisien. Bedanya:
- AI cuma dipakai untuk **decision making**, bukan **content generation**
- Selebihnya: **rule engine + coordinate math + local model spesialis**
- Hasil: **Editing cerdas dengan biaya mendekati nol**

> "Jangan pakai AI untuk sesuatu yang bisa dihitung dengan rumus.
> Pakai AI hanya untuk sesuatu yang perlu dipahami."

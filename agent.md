# OpenCut AI Auto-Editor — Project Hub

> **Pusat informasi proyek** — bacaan wajib sebelum semua agent mulai.
> Baca ini dulu, baru lihat agent-specific task di `agent1.md`, `agent2.md`, `agent3.md`.

---

## 1. Visi Singkat

Bangun **sistem auto-editing video berbasis AI yang bekerja seperti editor manusia** — 
mencari bahan, bikin voiceover, nata layout berdasarkan koordinat, render — 
dengan **token seminimal mungkin** (target < 5K token per project).

## 2. Arsitektur Inti

```
Opencut (base editor) ←── auto-editor/ (sistem kita) ←── moko_bridge/ (ke MOKO OS)
```

- **OpenCut Classic** — base video editor (web, WASM compositor, timeline)
- **auto-editor/** — sistem AI automation kita (orchestrator, workers, pipeline)
- **moko_bridge/** — jembatan ke MOKO OS untuk local LLM, RAG, native acceleration

## 3. Tech Stack

| Lapisan | Teknologi |
|---------|-----------|
| **Base Editor** | OpenCut Classic (React + Vite + WASM) |
| **Auto-Editor Core** | Python 3.12 |
| **AI Orchestrator** | MOKO-AI-4B (local GGUF) via moko_bridge |
| **TTS** | CosyVoice/Bark lokal (format GGUF) |
| **ASR** | Whisper.cpp lokal (GGUF) |
| **Layout** | Coordinate 4D (x,y,z,t) — pure math, 0 token |
| **Render** | OpenCut WASM compositor + FFmpeg fallback |
| **Asset Search** | RAG lokal + Pexels/Pixabay REST API |

## 4. Filosofi Token-Efficient

```
○ AI DIGUNAKAN UNTUK: decision making, planning, quality review
✗ AI TIDAK DIGUNAKAN UNTUK: hitung posisi, render, cari asset (REST)
```

Prioritas: **Rule engine → Template → Local LLM → API** (paling hemat dulu).

## 5. Struktur Folder

```
Opencut/
├── auto-editor/            # Sistem auto-editing (semua agent kerja di sini)
│   ├── orchestrator/       # Mandor LLM + workflow engine
│   ├── workers/            # Worker spesialis
│   │   ├── scene_detector/
│   │   ├── asset_finder/
│   │   ├── layout_engine/
│   │   ├── audio_pipeline/
│   │   ├── effects/
│   │   └── renderer/
│   ├── api/                # REST API opsional
│   ├── models/             # Local model files (GGUF/ONNX)
│   └── config/             # Settings, templates, providers
├── moko_bridge/            # Bridge ke MOKO OS
├── docs/riset/             # Dokumen riset (01-04)
└── agent.md                ← Kamu di sini
```

## 6. Multi-Agent Pembagian Tugas

| Agent | Fokus | Target Folder |
|-------|-------|--------------|
| **Agent 1** | Foundation — clone OpenCut + setup project + orchestrator core | `auto-editor/orchestrator/` |
| **Agent 2** | Workers — semua worker spesialis (scene, asset, audio, layout, effects, render) | `auto-editor/workers/*` |
| **Agent 3** | Integration — MOKO bridge + API + template system + testing | `moko_bridge/` + `auto-editor/api/` |

## 7. Konvensi Kode

- **Bahasa:** Python 3.12 (auto-editor), TypeScript (jika menyentuh OpenCut)
- **Style:** type hints wajib, docstring minimalis, no comments kode trivial
- **Config:** YAML untuk template/settings, JSON untuk data exchange
- **Testing:** Tiap worker wajib punya test (`test_*.py`)
- **Error handling:** graceful degradation — jangan crash, fallback ke mode lebih rendah

## 8. Referensi Cepat

| File | Isi |
|------|-----|
| `docs/riset/01_*` | Visi & filosofi sistem |
| `docs/riset/02_*` | Arsitektur teknis lengkap |
| `docs/riset/03_*` | Spesifikasi coordinate layout system |
| `docs/riset/04_*` | Hybrid AI pipeline & workflow engine |
| `moko_bridge/` | (akan dibuat Agent 3) |

## 9. Mode Operasi

`offline` → `hybrid` → `cloud` — makin kanan makin banyak token, makin kiri makin mandiri.
Default: **hybrid** (local LLM + API quality boost jika perlu).

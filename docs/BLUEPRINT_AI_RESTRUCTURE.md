# OpenCut AI — Blueprint Restrukturasi Total

## Filosofi: AI-Native Editing

OpenCut AI bukan editor video biasa yang ditambah fitur AI.
Ini adalah **AI Orchestrator** yang kebetulan punya video editor.
Setiap interaksi user → AI → timeline → rendering.

---

## Arsitektur Baru: 5 Lapisan

```
┌─────────────────────────────────────────────────────┐
│                    UI LAYER (apps/web)               │
│  ┌──────────┐ ┌───────────┐ ┌───────────────────┐  │
│  │ Dashboard │ │   Editor  │ │  AI Command Bar   │  │
│  │   AI      │ │  Layout   │ │  (Cmd+K)          │  │
│  └──────────┘ └───────────┘ └───────────────────┘  │
├─────────────────────────────────────────────────────┤
│                 ORCHESTRATOR LAYER                    │
│  ┌──────────────────────────────────────────────┐   │
│  │          AIOrchestrator (EditorCore+)         │   │
│  │  ┌─────────┐ ┌────────┐ ┌────────────────┐   │   │
│  │  │ Context │ │ Intent │ │ Workflow Engine │   │   │
│  │  │ Manager │ │ Router │ │ (DAG)           │   │   │
│  │  └─────────┘ └────────┘ └────────────────┘   │   │
│  │  ┌─────────┐ ┌────────┐ ┌────────────────┐   │   │
│  │  │ Timeline│ │ Asset  │ │ Smart Render   │   │   │
│  │  │ AI      │ │ AI     │ │ Optimizer      │   │   │
│  │  └─────────┘ └────────┘ └────────────────┘   │   │
│  └──────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────┤
│                AI INFERENCE LAYER                     │
│  ┌──────────────────┐ ┌────────────────────────┐   │
│  │ Rust/WASM AI     │ │ Python AI Server        │   │
│  │ • Smart trim     │ │ • LLM (Mandor)          │   │
│  │ • Scene detect   │ │ • TTS / ASR             │   │
│  │ • Auto-transition │ │ • Asset search         │   │
│  │ • Real-time       │ │ • Template gen         │   │
│  └──────────────────┘ └────────────────────────┘   │
├─────────────────────────────────────────────────────┤
│              RUST CORE (Business Logic)              │
│  ┌──────────┐ ┌────────┐ ┌──────┐ ┌────────────┐  │
│  │Compositor│ │Effects │ │Masks │ │ Time/AI     │  │
│  │ (wgpu)   │ │(blur..)│ │(SDF) │ │ + AI ops    │  │
│  └──────────┘ └────────┘ └──────┘ └────────────┘  │
├─────────────────────────────────────────────────────┤
│             STORAGE / PERSISTENCE                    │
│  ┌──────────┐ ┌────────┐ ┌────────────────────┐   │
│  │IndexedDB │ │ OPFS   │ │ AI Model Cache     │   │
│  └──────────┘ └────────┘ └────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

---

## 1. AIOrchestrator — Jantung Baru

Menggantikan `EditorCore` sebagai central mediator. Semua operasi AI dan non-AI melalui sini.

### Struktur Baru `src/core/`

```
src/core/
  index.ts                  # AIOrchestrator class (dulu EditorCore)
  managers/
    playback-manager.ts     # Existing
    timeline-manager.ts     # Existing (diperkuat AI)
    scenes-manager.ts       # Existing
    project-manager.ts      # Existing
    media-manager.ts        # Existing
    renderer-manager.ts     # Existing
    save-manager.ts         # Existing
    audio-manager.ts        # Existing
    selection-manager.ts    # Existing
    clipboard-manager.ts    # Existing
    diagnostics-manager.ts  # Existing
    ai/
      context-manager.ts    # NEW - tracking konteks editing
      intent-router.ts      # NEW - NLP ke action
      workflow-engine.ts    # NEW - DAG pipeline
      timeline-ai.ts        # NEW - AI timeline ops
      asset-ai.ts           # NEW - AI asset suggestion
      smart-render.ts       # NEW - AI render optimization
```

### AIOrchestrator Class

```typescript
export class AIOrchestrator {
  // Existing managers
  timeline: TimelineManager
  command: CommandManager
  playback: PlaybackManager
  scenes: ScenesManager
  project: ProjectManager
  media: MediaManager
  renderer: RendererManager
  save: SaveManager
  audio: AudioManager
  selection: SelectionManager
  clipboard: ClipboardManager
  diagnostics: DiagnosticsManager

  // NEW AI managers
  ai: {
    context: AIContextManager       // Konteks editing real-time
    intent: AIIntentRouter          // Routing perintah natural language
    workflow: AIWorkflowEngine      // DAG workflow execution
    timeline: AITimelineAssistant   // Smart timeline suggestions
    asset: AIAssetAdvisor           // Asset recommendations
    render: AISmartRender           // Render optimization
  }
}
```

---

## 2. Dashboard AI — Smart Project Hub

### Layout Baru

```
┌──────────────────────────────────────────────────────┐
│  [Logo] OpenCut AI                      [Search] [+] │
│                                                      │
│  ┌── AI Quick Actions ──────────────────────────────┐│
│  │  [🎬 Buat video dari script]  [🎙️ Voiceover]     ││
│  │  [✂️ Auto-edit footage]  [📝 Generate dari teks] ││
│  └──────────────────────────────────────────────────┘│
│                                                      │
│  ┌── Smart Suggestions ────────────────────────────┐│
│  │  "Lanjutkan proyek Travel Vlog?"                ││
│  │  "Mau bikin video untuk TikTok dari footage?"   ││
│  └──────────────────────────────────────────────────┘│
│                                                      │
│  ┌── Recent Projects ──────────────────────────────┐│
│  │  [Card] [Card] [Card] [Card]                    ││
│  └──────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────┘
```

### File baru:
- `src/app/dashboard/page.tsx` — Dashboard utama AI
- `src/dashboard/ai-quick-actions.tsx` — Tombol AI cepat
- `src/dashboard/smart-suggestions.tsx` — Rekomendasi AI
- `src/dashboard/project-templates.tsx` — Template AI

### AI Quick Actions
```typescript
type AIQuickAction = {
  id: string
  icon: ReactNode
  title: string
  description: string
  intent: EditingIntent
  onActivate: () => void
}
```

---

## 3. Editor Layout AI-Native

### Layout Baru 4-Panel

```
┌─────────────────────────────────────────────────────────┐
│  Editor Header — [Project]  [AI Command: Cmd+K]  [...] │
├──────────┬──────────────────────────────┬───────────────┤
│          │                              │               │
│  ASSETS  │        PREVIEW               │  AI ASSISTANT │
│  Panel   │        Canvas                │  Panel (NEW)  │
│          │                              │               │
│  AI Src  │  [Smart Guides] [Overlays]   │  • Chat/CMD   │
│  Filter  │                              │  • Suggestions│
│          │                              │  • Timeline   │
│          │                              │  • Effects    │
├──────────┴──────────────────────────────┴───────────────┤
│                    TIMELINE (AI-powered)                 │
│  ┌────────────────────────────────────────────────────┐ │
│  │ [AI Auto-arrange] [Smart Trim] [Auto-Transition]  │ │
│  ├────────────────────────────────────────────────────┤ │
│  │ [Track 1: Video] ▓▓▓▓▓▓▓▓░░░░▓▓▓▓▓▓▓▓▓▓          │ │
│  │ [Track 2: Audio] ░░░░▓▓▓▓▓▓░░░░░░▓▓▓▓▓▓          │ │
│  │ [Track 3: Text]  ░░░░░░░░▓▓▓▓░░░░░░░░░░          │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### AI Assistant Panel (menggantikan Properties Panel)

```
┌── AI Assistant ──────────────────────────────────┐
│ [Chat] [Suggest] [Auto] [Template]                │
│                                                    │
│ ┌─ Chat ───────────────────────────────────────┐  │
│ │ "buat video 30 detik dari footage ini"        │  │
│ │ "tambah transisi fade di semua klip"         │  │
│ │ "atur layout untuk TikTok"                   │  │
│ └──────────────────────────────────────────────┘  │
│                                                    │
│ ┌─ Context Suggestions ────────────────────────┐  │
│ │ 🎯 Clip terpilih: 3 video, 2 audio           │  │
│ │ 💡 Saran: Auto-arrange timeline              │  │
│ │ 💡 Saran: Tambah background music            │  │
│ │ 💡 Saran: Generate subtitles otomatis        │  │
│ └──────────────────────────────────────────────┘  │
│                                                    │
│ ┌─ Quick AI Actions ───────────────────────────┐  │
│ │ [🎬 Auto Edit] [🎙️ Voiceover] [📝 Caption]  │  │
│ │ [🎨 Color Grade] [✨ Effects] [🔄 Transisi]  │  │
│ └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

---

## 4. Smart Timeline — AI Timeline Engine

### Kemampuan Baru
1. **Auto-arrange clips** — AI nata timeline berdasarkan konten
2. **Smart transitions** — AI pilih transisi cocok antar scene
3. **Auto-trim** — AI potong bagian tidak penting
4. **Scene detection** — Deteksi scene change otomatis
5. **AI pacing** — Atur durasi klip berdasarkan mood

### File baru:
- `src/timeline/ai/timeline-ai-manager.ts` — AI inti timeline
- `src/timeline/ai/auto-arrange.ts` — Auto-arrange logic
- `src/timeline/ai/transition-suggester.ts` — Rekomendasi transisi
- `src/timeline/ai/smart-trim.ts` — Smart trim logic
- `src/timeline/ai/scene-detector.ts` — Scene detection
- `src/timeline/ai/pacing-optimizer.ts` — Pacing optimization

### Alur Auto-Edit (end-to-end)

```
User: "buat video cinematic 30 detik dari footage ini"
  │
  ▼
AI Intent Router ──► Intent: AUTO_EDIT
  │                   Params: style=cinematic, duration=30
  ▼
AIOrchestrator
  ├── 1. Scene Detection (Rust/WASM)
  ├── 2. Asset Selection (Python/LLM)
  ├── 3. Storyboard Generation (Python/LLM)
  ├── 4. Timeline Assembly (TypeScript)
  ├── 5. Transition Setup (TypeScript → Rust)
  ├── 6. Preview Render (Rust/WASM wgpu)
  └── 7. User Review → Refine → Export
```

---

## 5. AI Command Bar (Cmd+K)

Baru: Command palette AI yang paham konteks timeline.

```
┌── AI Command ─────────────────────────────┐
│ 🔍 [Buat video 30 detik dari...       ]  │
├──────────────────────────────────────────┤
│ ◉ Buat video dari footage               │
│   "buat cinematic 30 detik"              │
│ ◉ Tambah voiceover                       │
│   "voiceover pake bahasa indonesia"       │
│ ◉ Generate subtitle                      │
│   "subtitle otomatis dari video"          │
│ ◉ Atur layout untuk TikTok               │
│ ◉ Auto color grade                       │
│ ──────────────────────────────────────── │
│ Template:                              ▼ │
│ [Cinematic] [Vlog] [Tutorial] [Produk]  │
└──────────────────────────────────────────┘
```

---

## 6. Integrasi Python Auto-Editor → WASM

### Arsitektur Hybrid

```
Browser (WASM) ◄──── WebSocket/HTTP ────► Python Server
  │                                            │
  │ Real-time AI ops                      Heavy AI ops
  │ • Scene detection                     • LLM reasoning
  │ • Smart trim                          • TTS generation
  │ • Auto transitions                    • ASR/transcription
  │ • Frame analysis                      • Asset search (Pexels)
  │ • Color analysis                      • Template generation
```

### WASM AI Module (Baru di Rust)
- `rust/crates/ai/` — AI inference di Rust
- `rust/crates/ai/src/scene.rs` — Scene detection
- `rust/crates/ai/src/classify.rs` — Frame classification
- `rust/crates/ai/src/optimize.rs` — Render optimization

---

## 7. Peta Migrasi Step-by-Step

### Fase 1: Foundation (Sekarang)
- [x] Rename project ke OpenCut AI
- [x] Update brand, domain, metadata
- [x] Buat blueprint ini

### Fase 2: AI Core
- [ ] Buat `AIContextManager` — tracking konteks editing
- [ ] Buat `AIIntentRouter` — NLP ke action
- [ ] Buat `AIWorkflowEngine` — DAG pipeline
- [ ] Integrasi `auto_editor/api/` dengan web app via WebSocket

### Fase 3: Smart Timeline
- [ ] Buat `AITimelineAssistant`
- [ ] Auto-arrange clips
- [ ] Smart transitions
- [ ] Scene detection (Rust/WASM)
- [ ] AI pacing

### Fase 4: Dashboard AI
- [ ] Rombak `/projects` → `/dashboard`
- [ ] AI Quick Actions
- [ ] Smart Suggestions
- [ ] Project templates AI

### Fase 5: AI Command Bar & Assistant
- [ ] Cmd+K AI Command palette
- [ ] AI Assistant panel (ganti Properties)
- [ ] Natural language editing

### Fase 6: Rust AI Module
- [ ] `rust/crates/ai/` — WASM AI ops
- [ ] Scene detection native
- [ ] Smart trim native
- [ ] Auto-transition native

### Fase 7: Full Pipeline
- [ ] End-to-end auto-edit: Keyword → Video
- [ ] Real-time AI suggestions
- [ ] Smart export optimization
- [ ] Template marketplace

---

## 8. File Structure Baru (Ringkasan)

```
src/
  core/
    index.ts                              # AIOrchestrator
    managers/
      ai/
        context-manager.ts                # NEW
        intent-router.ts                  # NEW
        workflow-engine.ts                 # NEW
        timeline-ai.ts                    # NEW
        asset-ai.ts                       # NEW
        smart-render.ts                   # NEW
  dashboard/                              # NEW
    ai-quick-actions.tsx
    smart-suggestions.tsx
    project-templates.tsx
    dashboard-store.ts
  timeline/
    ai/                                   # NEW
      timeline-ai-manager.ts
      auto-arrange.ts
      transition-suggester.ts
      smart-trim.ts
      scene-detector.ts
      pacing-optimizer.ts
  ai/                                     # NEW
    command-bar.tsx
    assistant-panel.tsx
    context-aware.ts
    suggestions.tsx
    quick-actions.tsx
  app/
    dashboard/                            # NEW
      page.tsx
    editor/[project_id]/
      page.tsx                            # Layout baru

rust/
  crates/
    ai/                                   # NEW
      src/scene.rs
      src/classify.rs
      src/optimize.rs
      Cargo.toml

auto_editor/
  api/
    routes_ws.py                          # Perkuat WebSocket
  orchestrator/
    intent_router.py                      # Integrasi dgn TS version
    workflow_engine.py                    # Sinkron dgn TS version
```

---

## Prinsip Desain

1. **AI dulu, UI kedua** — Setiap komponen punya `aiContext` dan `aiSuggestions`
2. **Data-driven** — Semua keputusan AI dicatat, di-log, bisa di-review
3. **Graceful degradation** — AI offline → fallback ke rule-based
4. **Privacy-first** — AI lokal diutamakan (WASM), API optional
5. **Real-time** — AI ops di WASM untuk latensi rendah
6. **Progressive** — Setiap fase bisa jalan sendiri, tidak blokir fitur lain

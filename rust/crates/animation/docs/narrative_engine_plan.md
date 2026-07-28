# Narrative Engine & Branching Storylines — Extended Plan

## Ringkasan

Dokumen ini adalah **extended plan** di atas `implementation_plan.md`. Fokusnya: **branching storylines**, **narrative state**, **character-driven arcs**, **engagement mechanics**, dan **produksi animasi 2D berkualitas film**.

---

## Arsitektur Narrative Engine

### Story Graph (Bukan Linear Episode Lagi)

Ganti model `Vec<CinematicAct>` linear dengan **Directed Acyclic Graph (DAG)** — setiap node adalah *beat* cerita, setiap edge adalah kondisi transisi.

```
[START] ──► [BEAT A] ──► [BEAT B] ──► [BEAT C] ──► [ENDING 1]
                      │                            │
                      └──► [BEAT D] ──► [BEAT E] ──┤
                                                   ├──► [ENDING 2]
                                                   └──► [ENDING 3]
```

### Data Model Baru (`rust/crates/animation/src/narrative/`)

```
rust/crates/animation/src/narrative/
├── mod.rs              # Re-export
├── graph.rs            # StoryGraph, StoryNode, StoryEdge
├── state.rs            # NarrativeState — flags, relationships, variables
├── conditions.rs       # Condition evaluator (gerbang logika)
├── consequences.rs     # Apa yang berubah setelah beat dieksekusi
├── character_arc.rs    # Arc development per karakter
├── engagement.rs       # Pacing & tension tracker
├── dialogue.rs         # Dialog tree system
└── generator.rs        # AI/LLM story graph generator
```

#### `graph.rs` — StoryGraph

```rust
pub enum NodeType {
    Opening,
    Scene,           // Adegan biasa dengan entities + camera
    Decision,        // Pemain memilih opsi → branching
    ActionSequence,  // Adegan aksi cepat, tempo tinggi
    QuietMoment,     // Momen tenang — karakter refleksi, dialog
    Twist,           // Plot twist — otomatis triggered
    Climax,          // Klimaks cerita
    Ending,          // Akhir cerita (bisa multiple)
}

pub struct StoryNode {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub description: String,
    pub dialogue_lines: Vec<DialogueLine>,      // Dialog yang terjadi di node ini
    pub camera_script: Vec<CameraDirective>,     // Arahan sinematik
    pub required_flags: Vec<FlagCondition>,      // Syarat masuk node ini
    pub set_flags: Vec<String>,                  // Flag yang di-set setelah node
    pub emotion_arc: Vec<EmotionDirective>,      // Perubahan emosi per karakter
    pub animation_override: Option<String>,      // Override pose spesifik
    pub duration_seconds: f64,
    pub choices: Vec<StoryChoice>,               // Opsi pemain (hanya untuk Decision)
    pub children: Vec<String>,                   // Node ID tujuan
}

pub struct StoryEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<Condition>,
    pub priority: u32,     // Semakin kecil, semakin diprioritaskan
    pub label: String,     // "if hero_health > 50", "if trust_budi > 3"
}

pub struct StoryGraph {
    pub id: String,
    pub title: String,
    pub nodes: HashMap<String, StoryNode>,
    pub edges: Vec<StoryEdge>,
    pub root_node_id: String,
    pub starting_flags: HashMap<String, FlagValue>,
    pub meta: StoryMeta,
}
```

#### `state.rs` — NarrativeState

```rust
pub struct NarrativeState {
    pub current_node_id: String,
    pub visited_nodes: Vec<String>,
    pub flags: HashMap<String, FlagValue>,
    pub character_relationships: HashMap<(String, String), i32>, // (A, B) → trust score
    pub character_arc_progress: HashMap<String, ArcStage>,
    pub tension_level: f32,          // 0.0 (santai) — 1.0 (tegang)
    pub elapsed_story_time: f64,      // Waktu dalam dunia cerita
    pub global_event_log: Vec<StoryEvent>,
    pub active_foreshadowing: Vec<ForeshadowingHint>,
}

pub enum FlagValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

pub enum ArcStage {
    Introduction,
    RisingAction,
    Crisis,
    Climax,
    Resolution,
}
```

#### `conditions.rs` — Sistem Gerbang Logika

```rust
pub enum Condition {
    FlagEquals { flag: String, value: FlagValue },
    FlagGreaterThan { flag: String, value: f64 },
    RelationshipAtLeast { char_a: String, char_b: String, min: i32 },
    VisitedNode { node_id: String },
    NotVisitedNode { node_id: String },
    TensionAbove(f32),
    TensionBelow(f32),
    AllOf(Vec<Condition>),
    AnyOf(Vec<Condition>),
    NoneOf(Vec<Condition>),
}
```

#### `character_arc.rs` — Arc Progression per Karakter

```rust
pub struct CharacterArc {
    pub character_id: String,
    pub arc_type: ArcType,       // Hero, Mentor, FallFromGrace, Redemption, etc.
    pub stages: Vec<ArcStage>,
    pub current_stage: usize,
    pub defining_moments: Vec<String>,  // Node IDs yang mengubah arc
}

pub enum ArcType {
    HeroJourney,       // Biasa → Berani → Pahlawan
    FallFromGrace,     // Baik → Godaan → Jatuh → (Redemption?)
    Redemption,        // Jahat → Penyesalan → Tebus Dosa
    MentorSacrifice,   // Bijak → Membimbing → Mengorbankan diri
    ComicRelief,       // Pelawak → (bisa serius di klimaks)
    LoveInterest,      // Muncul → Dekat → (Berhasil/Gagal)
    Antagonist,        // Lawan → Motivasi terungkap → (Kalah/Menang)
}
```

#### `engagement.rs` — Tension & Pacing Engine

```rust
pub struct PacingProfile {
    pub tension_curve: Vec<TensionPoint>,  // Titik-titik tension di timeline
    pub beat_rhythm: BeatRhythm,           // Cepat-lambat-cepat-lambat
    pub twist_frequency: TwistFreq,        // Seberapa sering plot twist muncul
}

pub enum BeatRhythm {
    Standard,       // Naik-turun wajar
    ActionPacked,   // Cepat terus, jarang jeda
    SlowBurn,       // Pelan menanjak, ledakan di akhir
    Rollercoaster,  // Ekstrem naik-turun-naik-turun
}

/// Aturan sinematik "naik-turun" yang mencegah kebosanan:
/// 1. Setiap adegan aksi berat harus diikuti momen tenang (refleksi)
/// 2. Tension tidak boleh flat > 3 beat berturut-turut
/// 3. Setiap 3-4 beat harus ada "kejutan kecil" (twist/reveal)
/// 4. Emosi berganti setiap beat untuk variasi
```

---

## Branching Storylines: Karakter → Cerita

### Tahap 1: Story Path per Karakter (Sekarang)

Setiap karakter punya **jalan cerita sendiri** yang branching berdasarkan interaksi:

```
Karakter: Budi (anak kecil)
├── Jalan A: Budi diculik → Polisi menyelamatkan → Happy Ending
├── Jalan B: Budi kabur sendiri → Bertemu preman → Jalan gelap
└── Jalan C: Budi melawan penculik → Tersesat di hutan → Survival

Karakter: Kapten Polisi
├── Jalan A: Menyelamatkan sandera → Naik pangkat → Pahlawan
├── Jalan B: Gagal menyelamatkan → Depresi → Mencari penebusan
└── Jalan C: Korupsi dalam tim → Dikhianati → Balas dendam
```

### Tahap 2: Multi-Character Crossover (Nanti)

Karakter-karakter dari jalan cerita berbeda bisa **bertemu** di node tertentu. Pertemuan ini membuka *branch path baru* yang tidak bisa diakses dari single path saja.

```
Budi Jalan A (selamat)  ──┐
                          ├──► Node Pertemuan ──► Aliansi ──► Ending Epik
Kapten Jalan B (depresi) ──┘
```

---

## AI Story Generator — `narrative_generator.py`

### Input → Output Flow

```
[Prompt User] 
    │
    ▼
[LLM Generator] ←── [Story Graph Template Library]
    │                        │
    ├── StoryGraph           ├── "Hero Rescue" template
    ├── CharacterArcs        ├── "Zombie Survival" template  
    ├── Conditions           ├── "School Drama" template
    └── PacingProfile        └── "Space Odyssey" template
    │
    ▼
[NarrativeState] ──► [Animation Engine] ──► [Render]
```

### Template-Based Generation + Dynamic Expansion

```python
class NarrativeGenerator:
    """
    1. Ambil template story graph dari library
    2. Inject karakter user ke dalam graph
    3. LLM generate node descriptions, dialogue, conditions
    4. Validasi graph (no dead ends, achievable endings)
    5. Simpan sebagai StoryGraph JSON
    """
```

---

## Dialog & Subtitle Engine

### Dialogue Tree (`dialogue.rs`)

```rust
pub struct DialogueLine {
    pub speaker_id: String,
    pub text: String,                    // Teks utama
    pub emotion: EmotionState,          // Ekspresi saat bicara
    pub voiceover_key: Option<String>,  // Key untuk TTS audio
    pub duration_seconds: f64,
    pub subtitle_style: SubtitleStyle,
}

pub struct DialogueBranch {
    pub lines: Vec<DialogueLine>,
    pub choices: Vec<DialogueChoice>,    // Opsi respons pemain
}

pub struct DialogueChoice {
    pub text: String,                    // Teks tombol pilihan
    pub response: Vec<DialogueLine>,     // Response AI setelah pilih
    pub set_flags: Vec<String>,          // Flag berubah
    pub relationship_delta: Vec<(String, String, i32)>, // Relasi berubah
    pub goto_node: Option<String>,       // Loncat ke node tertentu
}
```

### Subtitle Styling

```rust
pub struct SubtitleStyle {
    pub font_size: f32,
    pub color: String,          // Hex
    pub position: SubtitlePos,  // Bottom, Top, Left, Right
    pub background: bool,       // Background box
    pub effect: SubtitleEffect, // Typewriter, Fade, Slide, Glitch
}
```

---

## Engagement Mechanics — Agar Tidak Membosankan

### Aturan Emosi Berganti (Emotion Rotation)

```
Setiap beat, EMOSI DOMINAN harus BERGANTI:
  Senang  →  Tegang  →  Sedih  →  Takut  →  Marah  →  Lega  →  Senang lagi
```

Jika 2 beat berturut-turut emosinya sama → **otomatis sisipkan "twist kecil"**.

### Pacing Formula

| Waktu Cerita | Tension | Event |
|---|---|---|
| 0% — 15% | 0.2 — 0.3 | Pengenalan karakter & dunia |
| 15% — 30% | 0.3 — 0.6 | Konflik pertama muncul |
| 30% — 45% | 0.5 — 0.7 | Plot twist / komplikasi |
| 45% — 55% | 0.3 — 0.4 | Momen tenang (refleksi) |
| 55% — 75% | 0.6 — 0.9 | Aksi meningkat, krisis |
| 75% — 85% | 0.4 — 0.5 | False resolution / twist |
| 85% — 100% | 0.7 — 1.0 | Klimaks & ending |

### Visual Engagement Boosts

1. **Camera Shake** — proporsional dengan tension (0.0 — 0.3)
2. **Speed Variation** — scene aksi pakai fast cuts (2-3s per shot), scene tenang pakai slow pans (5-8s)
3. **Color Grading** — sesuai emosi: hangat (senang), dingin (sedih), merah (marah), gelap (takut)
4. **Parallax Multi-Layer** — background bergerak lambat dari foreground, efek depth sinematik
5. **Dynamic Lighting** — Cahaya bergeser mengikuti mood, bayangan muncul saat tegang

---

## Animasi 2D Quality Boost — Sinematik

### Di atas existing skeletal animation:

#### Facial Close-Up System

```
[CinematicAct dengan shot_type: CloseUp]
    │
    ▼
[Render mode beralih ke "Facial CloseUp"]
    │
    ├── Wajah tampil di 60% layar
    ├── Mata: pupil bergerak (mata "hidup")
    ├── Alis: ekspresif (naik/turun/miring)
    ├── Mulut: sync dengan dialogue duration
    └── Background: blur/darken (depth of field efek)
```

#### Dynamic Camera Movement

```rust
pub struct CameraDirective {
    pub from: CameraShot,
    pub to: CameraShot,
    pub transition: CameraTransition,  // Pan, Zoom, Dolly, Tilt, Follow
    pub duration: f64,
    pub easing: EasingCurve,           // Smooth in/out
}

pub enum CameraTransition {
    Cut,              // Langsung pindah
    FadeToBlack,
    FadeFromBlack,
    CrossFade { overlap: f64 },
    Wipe(Direction),
    ZoomBlur,
}
```

#### Procedural Secondary Animation

Sudah ada di `physics_secondary.rs` — pastikan **diaktifkan selalu**:
- Rambut bergoyang saat karakter bergerak (follow-through)
- Pakaian berayun (overlapping action)
- Squash & stretch saat landing/lompat
- Anticipation sebelum gerakan besar

#### Timing & Spacing — Prinsip Animasi 12 Frame

Integrasikan 12 Prinsip Animasi Disney ke dalam state machine:

| Prinsip | Implementasi |
|---|---|
| Squash & Stretch | Otomatis di landing/jump |
| Anticipation | Tambahkan anticipation_pose sebelum run/jump/attack |
| Staging | Camera angle otomatis fokus ke karakter penting |
| Straight Ahead / Pose-to-Pose | Keyframe interpolation (Bezier) |
| Follow Through | Physics engine (sudah ada) |
| Slow In & Out | Easing curves di setiap transisi state |
| Arc | Trajectory melengkung, bukan linear |
| Secondary Action | Rambut, baju (sudah ada) |
| Timing | Duration per pose sesuai berat karakter |
| Exaggeration | Over-express emotion — squash 20% lebih |
| Solid Drawing | Bone proportions konsisten |
| Appeal | Karakter punya silhouette unik |

---

## Modul Baru di Rust

### `rust/crates/narrative/` — Crate Baru

```
rust/crates/narrative/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── graph.rs         # StoryGraph, StoryNode, StoryEdge
│   ├── state.rs         # NarrativeState
│   ├── conditions.rs    # Condition evaluator
│   ├── character_arc.rs # Character arc tracking
│   ├── engagement.rs    # Tension & pacing
│   ├── dialogue.rs      # Dialogue trees
│   ├── generator.rs     # Template-based + LLM graph gen
│   └── validator.rs     # Graph validation (no dead ends)
```

**Alasan crate baru (bukan di `animation/`):** Sesuai AGENTS.md, `animation/` urus gerakan. Narrative adalah domain terpisah — bisa digunakan tanpa animation engine (misal untuk text-based story atau visual novel di masa depan).

### Modifikasi `rust/crates/animation/src/cinematic.rs`

```rust
pub struct CinematicMovie {
    pub title: String,
    pub summary: String,
    pub total_duration: f64,
    pub acts: Vec<CinematicAct>,
    // BARU:
    pub story_graph_id: Option<String>,        // Link ke StoryGraph
    pub narrative_state: Option<NarrativeState>, // Snapshot state
    pub character_arcs: Vec<CharacterArc>,     // Arc per karakter
}
```

### Modifikasi `apps/desktop/src/state/editor.rs`

```rust
// Tambahan di EditorState:
pub struct EditorState {
    // ... existing ...
    pub narrative_state: Option<NarrativeState>,
    pub story_graph: Option<StoryGraph>,
    pub available_endings: Vec<String>,      // Ending apa saja yang mungkin
    pub unlocked_endings: Vec<String>,        // Ending yang sudah dicapai
    pub current_character_perspective: String, // Cerita dari sisi karakter mana
}
```

---

## AI Conversation Flow — Branching Story

### Flow Baru (Ganti Linear Episode)

```
[User: "Buat cerita polisi"]
    │
    ▼
[AI: "Mau dari sisi siapa?"]
    ├── [Kapten Polisi] ──► Jalan Heroik
    ├── [Teroris]         ──► Jalan Antagonist (pahami motivasi mereka)
    └── [Anak Kecil]      ──► Jalan Survival/Korban
    │
    ▼
[AI Generate StoryGraph untuk path tersebut]
    │
    ▼
[Node 1 dimainkan]
    │
    ▼
[Di Node Decision: "Kapten melihat teroris — apa tindakan?"]
    ├── [Langsung serbu]  ──► Jalan A: Action cepat, risiko tinggi
    └── [Memantau dulu]   ──► Jalan B: Intelijen, aman tapi lambat
    │
    ▼
[Terus sampai salah satu ending tercapai]
```

### Quick Replies Dinamis

Tombol pilihan di UI tidak hardcoded — diambil dari `StoryNode.choices` → dirender sebagai tombol. Setiap pilihan mengubah `NarrativeState` dan meloncat ke node berikutnya.

---

## Story Graph Template Library

Template disimpan di `config/story_templates/` sebagai YAML:

```yaml
# config/story_templates/hero_rescue.yaml
id: hero_rescue
title: "Misi Penyelamatan"
characters_required: ["hero", "villain", "victim"]
default_arcs:
  hero: HeroJourney
  villain: Antagonist
  victim: ComicRelief
nodes:
  - id: intro
    type: Opening
    title: "Hari yang Cerah"
    duration: 10.0
    children: ["kidnap", "patrol"]
    
  - id: kidnap
    type: Scene
    title: "Penculikan"
    duration: 15.0
    required_flags: []
    set_flags: ["event_kidnap_happened"]
    children: ["decision_pursue"]
    
  - id: decision_pursue
    type: Decision
    title: "Kejar atau Tidak?"
    choices:
      - text: "Kejar sekarang juga!"
        goto: "chase_highway"
        set_flags: ["choice_aggressive"]
      - text: "Lacak dulu dari markas"
        goto: "track_base"
        set_flags: ["choice_careful"]
    children: ["chase_highway", "track_base"]
    
  - id: chase_highway
    type: ActionSequence
    title: "Pengejaran di Jalan Raya"
    duration: 20.0
    required_flags: ["choice_aggressive"]
    children: ["confrontation"]
    
  - id: track_base
    type: Scene
    title: "Penyusupan Markas"
    duration: 25.0
    required_flags: ["choice_careful"]
    children: ["confrontation"]
    
  # ... more nodes ...
endings:
  - id: heroic_end
    title: "Pahlawan Kota"
    condition: { flag_equals: { flag: "victim_saved", value: true } }
  - id: tragic_end
    title: "Pengorbanan"
    condition: { flag_equals: { flag: "hero_fallen", value: true } }
```

---

## Roadmap Implementasi

### Fase 5: Narrative Foundation
1. Buat crate `rust/crates/narrative/` dengan `graph.rs`, `state.rs`, `conditions.rs`
2. Migrasi `CinematicMovie` → contains `StoryGraph` reference
3. Implementasi `NarrativeState` tracker
4. Unit test: graph traversal, condition evaluation, flag system

### Fase 6: Character Arcs & Dialogue
1. Implementasi `character_arc.rs` — progression per karakter
2. Implementasi `dialogue.rs` — dialog tree + subtitle styling
3. Generator LLM — story graph dari template + prompt
4. Integrasi dengan `gemini_director.py` — output StoryGraph bukan linear

### Fase 7: Branching UI & Player Choices
1. Modifikasi Desktop app — render Choice buttons di sidebar/overlay
2. Modifikasi Preview — animasi transisi antar node
3. Implementasi Quick Replies dari `StoryNode.choices`
4. Save/Load narrative state

### Fase 8: Engagement & Cinematic Quality
1. Pacing engine — tension curve otomatis
2. CameraDirective — transisi sinematik antar shot
3. Color grading dinamis sesuai mood
4. Facial Close-Up system
5. Story Graph template library (5-10 templates awal)

### Fase 9: Multi-Character Crossover (Masa Depan)
1. Merge multiple StoryGraphs dari path berbeda
2. Conditional cross-node yang hanya aktif jika dua karakter tertentu bertemu
3. Super-ending yang menggabungkan resolusi semua arc

---

## Metrik Kesuksesan

| Metrik | Target |
|---|---|
| Story paths per karakter | Minimal 3 per karakter |
| Unique endings | Minimal 2 per story graph |
| Tension variation | Tidak flat > 2 beat berturut |
| Emotion variety | Berganti setiap beat |
| Player choice impact | Setiap pilihan mengubah ending |
| Animasi FPS | 24 FPS cinematic, 60 FPS interaktif |
| Loading time antar node | < 500ms |

---

## Integrasi dengan Animation Engine

```
NarrativeState ──(current_node_id)──► StoryNode
                                          │
                                          ├── emotions_arc ──► EmotionState ──► apply_emotion_overlay()
                                          ├── camera_script ──► CameraDirective ──► CameraShot
                                          ├── dialogue_lines ──► DialogueRenderer (overlay)
                                          └── animation_override ──► AnimationName ──► state_machine.transition_to()
                                                │
                                                ▼
                                          pose.rs → SticmanPose → render.rs → segments
                                                │
                                                ▼
                                          physics_secondary.rs → hair/cloth physics
```

---

Dokumen ini adalah **living plan** — akan diperbarui seiring implementasi. Setiap fase menghasilkan PR yang bisa di-review dan di-merge secara independen.

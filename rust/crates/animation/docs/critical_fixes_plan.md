# Critical Fixes: Cinematic Rendering & Storytelling Quality

## Daftar Masalah & Root Cause

### Masalah 1: Polisi lari di tempat, teroris mundur ke belakang

**Root Cause** — `apps/desktop/src/ui/preview.rs:691-789`

Camera lock-on tracking menempatkan **target entity di tengah layar**. Saat polisi berlari dari x=-0.4 ke x=0.5, kamera mengikuti persis sehingga polisi tetap di tengah — terlihat seperti lari di tempat. Teroris yang tidak di-track kamera bergeser relatif terhadap kamera, menciptakan ilusi bergerak mundur.

```
Yang terjadi:
  Frame 1: Polisi di center (0), Teroris di kanan (+0.7)
  Frame 2: Polisi masih di center (0), Teroris bergeser (+0.5) ← sebenarnya kamera ikut polisi
  Terlihat: Polisi diam, teroris jalan mundur
```

**Fix**: Kamera harus menempatkan tracked entity di posisi **asimetris** tergantung ShotType:
- `ActionFollow` → tracked entity di 1/3 kiri layar, target di depan (2/3 kanan)
- `Wide` → kamera di tengah-tengah semua entity (group shot)
- `CloseUp` → tracked entity di center, zoom tinggi
- `Medium` → tracked entity di center, zoom sedang

### Masalah 2: Background tidak berubah per babak

**Root Cause** — `apps/desktop/src/ui/preview.rs:1330-1333`

Background di-load dari `EditorState::scene_theme`, **bukan dari act yang sedang diputar**. Jadi meskipun Act 1 theme="city" dan Act 2 theme="cyberpunk", background tetap "city" karena setting editor.

### Masalah 3: Tidak ada gelembung teks (speech bubble)

**Root Cause** — Data model `CinematicAct` tidak punya field dialog. Renderer tidak punya sistem text overlay.

### Masalah 4: Tidak ada transisi kamera antar babak

**Root Cause** — Kamera langsung snap dari `CameraShot` act sebelumnya ke act berikutnya. Tidak ada interpolasi/transition.

---

## Fix Plan Detail

### Fix 1: Camera Offset System (Chase/Framing)

#### Di `preview.rs` — ganti logika camera target:

```rust
// HITUNG CAMERA CENTER BERDASARKAN SHOT TYPE
let cam_center_x = match cam.shot_type {
    ShotType::ActionFollow => {
        // Tracked entity di 1/3 kiri layar
        // Offset = 0.33 * screen_width_in_world_units
        let screen_offset = 0.33; // 1/3 layar
        target_pos_x + cam.pan_x - screen_offset
    }
    ShotType::Wide => {
        // Kamera di tengah-tengah semua entity
        let avg_x = act.entities.iter()
            .map(|e| {
                let progress = /* ... */;
                e.pos_x + e.end_x.map(|ex| (ex - e.pos_x) * progress).unwrap_or(0.0)
            })
            .sum::<f32>() / act.entities.len() as f32;
        avg_x + cam.pan_x
    }
    ShotType::CloseUp | ShotType::Medium => {
        target_pos_x + cam.pan_x // center framing (seperti sekarang)
    }
};
```

#### Dynamic Framing Offset per Role:

Buat sistem sederhana: **setiap entity punya "screen target fraction"** — di mana mereka harus muncul di layar:

| Peran | Posisi Layar |
|---|---|
| Hero (dikejar) | 0.30 (kiri) |
| Hero (mengejar) | 0.25 (kiri, lebih ekstrim) |
| Antagonis | 0.70 (kanan) |
| Target (dialog) | 0.50 (tengah) |
| Wide shot | 0.50 (tengah - group) |

### Fix 2: Background Per Act

```rust
// Di render(), ganti:
- let theme = self.editor.borrow().scene_theme.clone();
- self.get_or_load_bg(&theme)

// Jadi:
+ let theme = if let Some(ref movie) = self.cinematic_movie {
+     let current_act = movie.acts.iter()
+         .find(|a| current_time >= a.start_time && current_time < a.start_time + a.duration)
+         .or_else(|| movie.acts.first());
+     current_act.map(|a| a.theme.clone()).unwrap_or(self.editor.borrow().scene_theme.clone())
+ } else {
+     self.editor.borrow().scene_theme.clone()
+ };
+ self.get_or_load_bg(&theme)
```

**Cross-fade antar background**: Saat act berganti, simpan `prev_bg` + `prev_theme` dan lakukan alpha blend:

```rust
struct Preview {
    // ... existing fields ...
    prev_bg_cache: Option<(String, Arc<RenderImage>)>, // (theme, image)
    act_transition_progress: f64, // 0.0 → 1.0 selama N detik transisi
}
```

Di render: paint `prev_bg` dengan alpha (1 - progress) lalu `current_bg` dengan alpha (progress).

### Fix 3: Speech Bubble System

#### Data Model — tambah ke `CinematicAct`:

```rust
pub struct DialogueLine {
    pub entity_id: String,
    pub text: String,
    pub start_time: f64,   // offset dalam act
    pub duration: f64,
    pub emotion: String,   // "normal", "shout", "whisper"
}

// Tambah ke CinematicAct:
pub struct CinematicAct {
    // ... existing ...
    pub dialogues: Vec<DialogueLine>,
}
```

#### Render — speech bubble di `preview.rs`:

```rust
// ── 4. Dialogue / Speech Bubbles ─────────────────────────────
for d in &act.dialogues {
    let rel_time = current_time - act.start_time;
    if rel_time >= d.start_time && rel_time <= d.start_time + d.duration {
        if let Some(entity) = act.entities.iter().find(|e| e.id == d.entity_id) {
            let bubble_x = /* posisi entity di screen */;
            let bubble_y = /* di atas kepala entity */;
            let alpha = /* fade in/out */;
            
            // Render rounded rect background
            // Render teks di dalamnya
            // Render tail/arrow ke arah karakter
        }
    }
}
```

Speech bubble styling:
- Background: putih/putih-translucent, rounded corners
- Text: hitam, center, max width ~200px
- Tail: segitiga kecil mengarah ke karakter
- Emotion variants: normal (putih), shout (merah/kuning), whisper (abu-abu)

### Fix 4: Camera Transitions

#### Interpolasi kamera antar act:

```rust
struct Preview {
    // ... existing ...
    prev_camera: Option<CameraShot>,
    camera_transition_timer: f64,
    camera_transition_duration: f64, // default 1.0s
}

// Di render():
if let Some(prev_cam) = &self.prev_camera {
    if self.camera_transition_timer < self.camera_transition_duration {
        let t = self.camera_transition_timer / self.camera_transition_duration;
        let ease = smoothstep(t); // ease-in-out
        
        // Lerp semua property kamera
        let current_zoom = lerp(prev_cam.zoom, cam.zoom, ease);
        let current_pan_x = lerp(prev_cam.pan_x, cam.pan_x, ease);
        let current_shake = lerp(prev_cam.shake, cam.shake, ease);
        // ... gunakan interpolated values ...
    }
}
```

### Fix 5: Entity Movement — Per-Entity Timing

**Masalah tambahan**: Semua entity bergerak dengan progress yang sama (act_elapsed / act_duration). Entity yang mulai bergerak di tengah act akan langsung loncat.

**Fix**: Gunakan `end_x` sebagai target absolut, hitung kecepatan per entity:

```rust
// Untuk setiap entity, hitung progress DURATION SENDIRI
// Jika entity punya end_x, dia bergerak dari pos_x ke end_x
// dalam durasi yang proporsional dengan jaraknya
let move_duration = if let (Some(ex), Some(ey)) = (entity.end_x, entity.end_y) {
    let dist = ((ex - entity.pos_x).powi(2) + (ey - entity.pos_y).powi(2)).sqrt();
    (dist / 0.5).min(act.duration) // 0.5 unit/detik kecepatan default
} else {
    act.duration
};
let entity_progress = ((act_elapsed) / move_duration).min(1.0);
```

### Fix 6: Dialog Subtitle Style di Data Model

```rust
pub struct SubtitleConfig {
    pub font_size: f32,    // 0.0-1.0 relatif
    pub color_hex: String, // "#FFFFFF"
    pub background: bool,
    pub effect: String,    // "typewriter", "fade", "none"
    pub position: String,  // "bottom", "top", "left", "right"
}
```

---

## Task List Immediate

```
[P1] Fix camera offset per ShotType (ActionFollow, Wide, CloseUp, Medium)
[P1] Fix background loading from current act theme + cross-fade transition
[P2] Add DialogueLine to CinematicAct data model
[P2] Implement speech bubble renderer (rounded rect + text)
[P2] Camera transition interpolation between acts
[P3] Per-entity movement timing (bukan seragam per act)
[P3] Camera shake enhancements (perlin noise, direction-aware)
[P3] Subtitle system (bottom text overlay)
```

---

## Contoh Hasil Setelah Fix

### Sebelum (saat ini):
```
Act 1: Polisi lari (x:-0.4 → x:0.5), Teroris (x:0.3, idle)
→ Layar: Polisi di tengah lari di tempat, teroris di kanan bergerak mundur perlahan
→ Background: city (statis, tidak berubah dari editor setting)
→ Tidak ada teks dialog
→ Kamera snap kasar antar act
```

### Sesudah:
```
Act 1: Polisi lari mengejar
→ Layar: Polisi di 1/3 kiri lari dengan benar, teroris di 2/3 kanan semakin membesar
→ Background: city dengan parallax (gedung belakang gerak lambat)
→ Speech bubble: "Berhenti! / Stop!" di atas polisi
→ Kamera: smooth pan mengikuti aksi

Act 2: Masuk ke cyberpunk
→ Background cross-fade dari city → cyberpunk (1 detik transisi)
→ Kamera zoom out untuk wide shot markas teroris
→ Polisi di kiri, teroris di kanan, anak kecil di tengah
```

# OpenCut — Full System Plan v2: Matematika Koordinat & Proyeksi Kamera

*(Revisi dari FULL_SYSTEM_PLAN.md — memperdalam ke matematika rendering)*

---

## 1. Konsep Fundamental: Kamera Sebagai Proyeksi, Bukan Scaling

### 1.1 Masalah Model Kamera Saat Ini

Kode rendering saat ini menggunakan formula:

```
desired_h = base_h * zoom * depth_scale
screen_x = mon_cx + (world_x - cam_center_x) * mon_w * 0.38 * zoom
screen_y = ground_y - depth_offset - (world_y * mon_h * 0.35)
```

Ini adalah **multiplicative scaling**, bukan **perspective projection**. Akibatnya:
- Saat zoom in (`zoom > 1`), **organisasi karakter diperbesar** secara bersamaan — Karakter A dan Karakter B sama-sama membesar. Karakter yang dekat dengan kamera dan karakter yang jauh sama-sama diperbesar, tanpa mempertimbangkan jarak mereka dari target kamera.
- Tidak ada efek **parallax** — gerakan horizontal kamera tidak menyebabkan objek jauh bergerak lebih lambat dari objek dekat.
- **Tidak ada efek fokus kamera** — tidak ada jendela pandang yang secara alami "memotong" area. Seluruh stage terlihat, hanya dimampatkan/diperluas.

### 1.2 Model Yang Diinginkan: Pinhole-Camera 2.5D

Kita ingin mensimulasikan sebuah **pinhole camera** yang diperbolehkan hanya bergerak pada sumbu X dan Z (horizontal/depth), dengan zoom yang dikendalikan oleh FOV atau posisi Z. Ini memberikan:

1. **Efek zoom yang "benar"**: Saat zoom in, kamera secara fisik bergeser lebih dekat ke karakter target, dan karakter yang jauh dari pusat fokus akan keluar dari jandela (out of frame) karena menjadi terlalu lebar — persis seperti kamera nyata.

2. **Efek parallax antar karakter**:Karakter di foreground (Z kecil, dekat kamera) akan bergerak lebih banyak pada layar dibanding karakter di background (Z besar, jauh dari kamera) saat kamera melakukan panning, menciptakan kesan kedalaman yang alami.

3. **Efek karakter yang terpotong secara **: Hanya karakter yang berada dalam frustum kamera yang terlihat. Sisanya terpotong. Ini memberikan efek fokus yang membuat animasi terasa lebih sinematik dan profesional.

---

## 2. Perumusan Matematika Rendering

### 2.1 Parameter Dasar

```
World coordinates: unit world, 2.5D
  - X: horizontal, range ~ -5 to +5 (disesuaikan per scene)
  - Y: vertical, 0 = ground level, >0 = terrain / platform
  - Z: depth, 1.0 = foreground (dekat kamera), 3.0 = background (jauh kamera)
  - Z tidak boleh negatif (kamera hanya melihat ke +Z)

Camera target: posisi di dunia yang ingin kamera lihat
  - target_world = (center_x, center_y)
  - camera_depth = seberapa jauh kamera dari "kanvas" — dikontrol oleh zoom
  - FOV = sudut pandang horizontal kamera (default = 60°)

Screen coordinates: ruang layar piksel
  - screen_w = lebar kanvas piksel (mon_w)
  - screen_h = tinggi kanvas piksel (mon_h)
```

```
### 2.2 Transformasi 3 langkah dari World ke Screen

**Langkah 1: World → Camera space**
O Jacarakter di P_world = (x_w, y_w, z_w)
O Kamera di P_cam = (cam_x, cam_y, cam_z)
O Kamera menghadap ke +X (ke kanan, sumbu horizontal)
Untuk dunia 2D, kita kolaps dimensi Y menjadi hanya posisi objek, bukan kamera.

```
camera_x = (x_w - cam_x) / z_w   ← proyeksi horizontal, dibagi oleh Z
camera_y = (y_w - cam_y)   / z_w   ← proyeksi vertikal (dapat diabaikan jika hanya ground level)
```

**Langkah 2: Camera → Projection**

Panjang frustum pada bidang proyeksi (dimana objek berada):

```
proj_w = z_w * fov_scale
proj_h = z_w * fov_scale / aspect_ratio
```

Dimana:
```
fov_scale = 2 * z_proj * tan(FOV / 2) = 2 * cam_z * tan(noFOV/2)
```

Untuk Z di mana , `z_proj` = 1.0 (standing), proy_pw adalah ukuran lensa normal.

**Langkah 3: Projection → Screen ()
Hubungan dari camera_x ke screen_x_M menggunakan perspektif linear:
```
screen_x = screen_cx + (camera_x * screen_w) / (z_w * fov_scale)
screen_y = screen_cy - (camera_y * screen_h) / (z_w * fov_scale)
```

Di mana:
```
screen_cx = center horizontal dari kanvas layar
screen_cy = center vercal dari kanvas layar (atau posisi "ground" untuk karakter)
fov_scale = 1.0 untuk proporsi standar
```

**Aternatif sederhana** (digunakan dalam game 2.5D seperti Stardew Valley / Inside):
```
base_size_px = 200/screen_w           ← Ukuran karakter "normal" dlm piksel per satuan (adjustable)
pixel_per_unit = fov_scale / camera_z   ← scaling perspektif per unit jarak
rel_x· = (x_world - cam_x) / z_w          ← koordinat sudut horizontal
x_screen = screen_cx + rel_x * pixel_per_unit * screen_w / 2.0  ← ignore z untuk sementara
width_screen = base_size_px / z_w * zoom_factor
height_screen = width_screen / aspect
```

Di mana `zoom_factor` menggantikan `zoom_hint()` saat ini.

---

### 2.3 Implementasi Praktis (Rumus Final untuk Preview)

Setelah analisis dan penyederhanaan, rumus praktis yang kita gunakan:

```rust
fn world_to_screen(entity_x: f32, entity_z: f32, cam: &CameraState, screen: &Screen) -> (f32, f32, f32) {

    // Parallax-enhanced Z: lebih dekat → lebih sensitif ke pergerakan kamera
    let parallax_scale = 1.0 / entity_z.max(0..5); // karakter foreground bergerak lebih banyak

    let relative_x = (entity_x - cam.center_x) * parallax_scale;
    let screen_x = screen.cx + (relative_x * screen.w * 0.3 * cam.zoom) * parallax_scale;

    let y_offset = entity_pos_y * screen.h * 0.02; // Y world → screen (minor elevation)
    let screen_y = screen.ground - (cam.depth * 0.3 * screen.h) - y_offset;

    // Size: perspektif = semakin dekat ke kamera → semakin besar
    let character_pixel_height = base_px * (1.0 - entity_z * 0.15) * cam.zoom / entity_z;

    let (screen_x, screen_y, character_height)
}
```

Persis dengan unit compute:
- `entity_z` = 1.0 (foreground, terdekat) = karakter normal/besar
- `entity_z` = 3·0 (background) = karakter kecil
- `cam.zoom` > 1 = semua diperbesar (efek lensa) — karakter DEKAT kamera terlihat lebih dramatis
- `cam.center_x` = dimana kamera melihat
- `cam.pan_x` = offset lateral dari center

**Kunci perbaikan**: saat zoom-in, karakter dengan z rendah (dekat) akan menjadi SANGAT besar, karakter dengan z tinggi (jauh) menjadi lebih kecil — menciptakan efek gradien fokus yang natural.

---

## 3. Koordinat World — Konsistensiik  Data

### 3.1 Problem: Entity pos_x/pos_z tidak konsisten

- AI menghasilkan posisi dalam format **world units** (mis: `pos_x: 0.5`, `pos_z:` 1.0`)
T- StageEntity end_x/end_y menyimpan target dalam world units
- rendering # mnaemolus-world_screen → perlu konversi ke screen px

### 3.2 Steering Rules untuk AI Director

AI harus diberi constrain **unit world yang benar**:
```
COORDINATE RULES (world 2.5D):
  - X range: -3.0 (kiri panggung, di luar jendela) to +3.0 (kanan panggung)
  - Karakter berkolokasi di sekitar X = -2 hingga +2 (visible area)
  - Gunakan Z = 1.0 untuk forefront (aktif), Z = 2.0 untuk supports, Z = 3.0 untuk background
  - Z harus konsisten antar aksiakt` — (misal; character "police" → Z=1.0, background "witness" → Z=2.5)
  - Tanpa Z = 0.0 (artinya karakter berada di depan kamerakk)
  - Setiap dialog harus dalam Z yang terlihat (di antara 1.0-2.5)
```

---

## 5 Rencana Implementasi Langsung

### 5.1 Phase 1: Refactoringentir Sistem Proyeksi Kamera

_File terdampak_: `apps/desktop/src/ui/preview.rs` (block rendering cinmatik, lines 734–880)

1. **Bangun struct `LayerState` **
   ```rust
   pub struct LayerState {
       pub mon_w: f32,    // lebar `mon` (canvas viewport)
       pub mon_h: f32,    // tinggi mon
       pub mon_cx, cy: f32, center x/y viewport  
       pub ground_y: f32,
       pub zoom: f32,    // kamera zoom
       pub depth_scale, parallax_sling, dll
   }
   impl LayerState {
       fn project(world_×: f32, world_y:f32, wworld_z: f32, cam_center_X: f32) →
           (Ob screen_x: f32, screen_y: f32, sprite _pixel_w: f3, sprite_pixel_h: f32)
   }
   ```

2. **Ganti kode rendering** yang menggunakan perkalian `* mon_w * 0.38 * zoom` menjadi `layer.project(...)`

3. **Validasi visual**: test zoom 0.5 (FullShot), zoom 1.0 (Medium), zoom 1.6 (ExtremeCloseUp)

### 5.2 Phase 2: Parallax Depth

4. **Mungkinkan paralaks kamera**: koordinat X pada background depth (Z > 2) bergerak lebih lambat saat panning.
   ```
   """
   background_layer_x *= azx_scale * (Z – 1) // 2.0 + 0.5
   """

### 5.3 Phase 3: COORDINATE FOR AI

5. **Perbaiki prompt gemini_director.py**: Tambahkan `COORDINATE_RULES` (dium di §3.2).

6. **Verifikasi bahwa AI di mengeluarkan koordinat dalam range yang valid** (tidak ada pos_x 0.9 di3 saat kamera seharusnya terfokus pada sesuatu yang lain).

### 5.4 Phase 4: DutchAngle visual (GPU Rotate)

7. Tambahkan `window.rotate(tilt_angle)` dalam blok cinematic.

---

## 6. Perbaikan General Play Issues

### 6.1 Sprint Character     fiz Effect
- Director ertp AI memilih `".run"` action → executor: gerakan CEPAT (speed) di λ=1.2
- Shaade fizik::pt juga: ak tara Z-depth = 1.0

### 62. Dikasi "C" Total Cerita

- `PACING` di prompt → paksa AI menggue **p leh moment teks tar animasi** alih-alih gerakan train

---

## inal]

Prioritas = render proyekt translation → kode LayerProjecter → test→ COROENAI prompt.

Lanjutkan ke Phase 1?: Dari atas Plan final: Apakah Anda setuju kita pindah next ke babakan implementasi matematika rendering (Phase 1 ~ 1), atau Anda ingin tambahkan sub.php terlebih dahulu?
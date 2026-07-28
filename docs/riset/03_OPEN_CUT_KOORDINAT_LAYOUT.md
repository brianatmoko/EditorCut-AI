# 03 — OpenCut Coordinate Layout System (CLS)

> **Dokumen spesifikasi teknis** — sistem koordinat 4D (x, y, z, t) untuk
> positioning elemen video secara presisi, menggantikan timeline-only approach
> dengan pendekatan compositing 2D + timeline yang terintegrasi.
>
> **Terinspirasi dari:** CSS Box Model, OpenClaw node system, Hermes AI spatial
> reasoning, After Effects coordinate system, MOKO CausalVisualFlowCompressor.

---

## 1. Filosofi: Kenapa Coordinate-Based?

### 1.1 Masalah Timeline-Only

Timeline tradisional (CapCut, Premiere) hanya punya **satu dimensi: waktu (t)**.
Posisi elemen di layar ditentukan secara manual atau via keyframe — tidak ada
reusable coordinate system.

```
Timeline: [─── clip A ───][── clip B ───][─── clip C ───]
             ↑ hanya waktu, layout manual
```

### 1.2 Solusi: Coordinate Layout System (CLS)

Kita tambahkan **3 dimensi spasial (x, y, z)** ke dalam timeline, sehingga setiap
elemen punya posisi absolut di layar yang bisa dihitung secara matematis.

```
CLS 4D Space:
  x : 0.0 - 1.0 (horizontal, relatif terhadap canvas)
  y : 0.0 - 1.0 (vertikal)
  z : 0 - N     (layer depth / stacking order)
  t : 0 - M     (waktu dalam detik)
```

Ini memungkinkan:
- **Reusable templates** — simpan koordinat sebagai template, apply ke video lain
- **Mathematical positioning** — hitung posisi dengan rumus, bukan manual
- **Auto-layout** — AI cukup tentukan rule, komputasi dilakukan oleh math engine (0 token)
- **Precision** — pixel-perfect positioning tanpa trial-error

---

## 2. Sistem Koordinat

### 2.1 Normalized Coordinate Space

Semua koordinat menggunakan **normalized space (0.0 - 1.0)** agar resolusi-independent:

```
(0,0) ───────────────────────────────── (1,0)
  │                                          │
  │                                          │
  │            CANVAS SPACE                  │
  │       (16:9, 9:16, 1:1, etc)           │
  │                                          │
  │                                          │
(0,1) ───────────────────────────────── (1,1)
```

Konversi ke pixel:
```
pixel_x = normalized_x * canvas_width
pixel_y = normalized_y * canvas_height
```

### 2.2 Spesifikasi Elemen

```typescript
interface CoordinateElement {
  // Identitas
  id: string;
  type: 'video' | 'image' | 'text' | 'shape' | 'effect';
  
  // Posisi Spasial (normalized 0.0-1.0)
  position: {
    x: number;        // 0.0 = kiri, 0.5 = tengah, 1.0 = kanan
    y: number;        // 0.0 = atas, 0.5 = tengah, 1.0 = bawah
    z: number;        // layer index (0 = paling belakang)
  };
  
  // Ukuran
  size: {
    width: number;    // normalized width (0.0 - 1.0)
    height: number;   // normalized height (0.0 - 1.0)
    unit: 'normalized' | 'pixel' | 'percent';
  };
  
  // Timeline (kapan muncul/hilang)
  timeline: {
    start: number;    // detik
    end: number;      // detik
    duration: number; // otomatis dari end - start
  };
  
  // Transformasi
  transform: {
    rotation: number;     // derajat (-360 to 360)
    scale: number;        // 0.0 - 10.0
    opacity: number;      // 0.0 - 1.0
    anchor: AnchorPoint;  // pivot: 'center' | 'top-left' | etc
  };
  
  // Animasi (opsional)
  animation?: {
    keyframes: Keyframe[];
    easing: 'linear' | 'ease-in' | 'ease-out' | 'ease-in-out';
  };
  
  // Visual properties
  style: {
    // Untuk video/image
    crop?: { x: number; y: number; width: number; height: number };
    flip?: { horizontal: boolean; vertical: boolean };
    borderRadius?: number;
    border?: { width: number; color: string };
    shadow?: { offset: number[]; blur: number; color: string };
    
    // Untuk text
    text?: string;
    fontFamily?: string;
    fontSize?: number;      // dalam px
    fontWeight?: number;
    color?: string;
    textAlign?: 'left' | 'center' | 'right';
    lineHeight?: number;
    letterSpacing?: number;
  };
  
  // Efek (opsional)
  effects?: Effect[];
}
```

---

## 3. Layout Templates

### 3.1 Definisi Template

Template adalah kumpulan koordinat yang bisa di-reuse:

```yaml
# template: product-showcase.yaml
name: "Product Showcase Cinematic"
description: "Tampilkan produk dari berbagai sudut dengan overlay informasi"
aspect_ratio: "16:9"
duration: 30  # detik, otomatis disesuaikan

tracks:
  - id: "bg"
    type: "video"
    position: { x: 0, y: 0, z: 0 }
    size: { width: 1.0, height: 1.0 }
    timeline: { start: 0, end: 30 }
    style: { fit: "cover" }

  - id: "product_main"
    type: "video"
    position: { x: 0.5, y: 0.5, z: 1 }
    size: { width: 0.6, height: 0.6 }
    timeline: { start: 0, end: 30 }
    transform: { scale: 1.0, opacity: 1.0 }
    animation:
      keyframes:
        - { time: 0, scale: 0.8, opacity: 0 }
        - { time: 1, scale: 1.0, opacity: 1.0 }
        - { time: 28, scale: 1.0, opacity: 1.0 }
        - { time: 30, scale: 1.2, opacity: 0 }
      easing: "ease-in-out"

  - id: "title"
    type: "text"
    position: { x: 0.5, y: 0.15, z: 2 }
    size: { width: 0.8, height: 0.15 }
    timeline: { start: 2, end: 12 }
    style:
      text: "{TITLE}"
      fontFamily: "Montserrat"
      fontSize: 48
      color: "#FFFFFF"
      textAlign: "center"
      fontWeight: 700
      shadow: { offset: [2, 2], blur: 4, color: "#00000080" }
    animation:
      keyframes:
        - { time: 2, y: 0.3, opacity: 0 }
        - { time: 3, y: 0.15, opacity: 1.0 }

  - id: "price_tag"
    type: "text"
    position: { x: 0.82, y: 0.75, z: 2 }
    size: { width: 0.3, height: 0.1 }
    timeline: { start: 5, end: 25 }
    style:
      text: "{PRICE}"
      fontSize: 36
      color: "#FFD700"
      fontWeight: 700

  - id: "info_bottom"
    type: "text"
    position: { x: 0.5, y: 0.85, z: 1 }
    size: { width: 0.8, height: 0.08 }
    timeline: { start: 5, end: 30 }
    style:
      text: "{DESCRIPTION}"
      fontSize: 20
      color: "#CCCCCC"
      textAlign: "center"

  - id: "subtitle"
    type: "text"
    position: { x: 0.5, y: 0.92, z: 3 }
    size: { width: 0.9, height: 0.06 }
    timeline: { start: 0, end: 30 }
    style:
      text: "(auto subtitle)"
      fontSize: 16
      color: "#FFFFFF"
      textAlign: "center"
      shadow: { offset: [1, 1], blur: 2, color: "#000000" }
```

### 3.2 Template Variables

Template bisa memiliki variable yang diisi otomatis:

```yaml
variables:
  - name: "TITLE"
    source: "user_input"   # dari user atau auto-generate
    fallback: "Product Video"
  - name: "PRICE" 
    source: "metadata"
    fallback: "Rp 0"
  - name: "DESCRIPTION"
    source: "ai_generate"  # mandor LLM generate deskripsi
    fallback: "Product description"
```

### 3.3 Template Library

```
templates/
├── cinematic/           # Template cinematic
│   ├── travel.yaml      # Video perjalanan
│   ├── product.yaml     # Product showcase
│   └── cinematic.yaml   # Cinematic generic
├── social/              # Template sosial media
│   ├── tiktok.yaml      # 9:16 vertical
│   ├── reels.yaml       # Instagram Reels
│   ├── shorts.yaml      # YouTube Shorts
│   └── story.yaml       # Instagram Story
├── presentation/        # Template presentasi
│   ├── slideshow.yaml   # Foto slideshow
│   └── tutorial.yaml    # Tutorial screen recording
├── music/               # Template musik
│   ├── lyric.yaml       # Lyric video
│   └── album.yaml       # Album teaser
└── custom/              # Template custom user
    └── *.yaml
```

---

## 4. Auto-Layout Engine: Matematika Tanpa Token

### 4.1 Core Algorithm

Layout engine menghitung posisi final setiap elemen **tanpa melibatkan AI**.
AI hanya menentukan **rule tingkat tinggi**, engine mengerjakan matematikanya.

```
AI (Mandor): "Letakkan teks judul di sepertiga atas layar"
  ↓
Layout Engine:
  ├─ Parser: "sepertiga atas" = y: 0.0 to 0.33
  ├─ Alignment: center = x: 0.5
  ├─ Padding: 10% dari lebar = width: 0.8
  ├─ Font size: 5% dari tinggi canvas = 48px (1080p)
  └─ Output: { x: 0.5, y: 0.15, width: 0.8, height: 0.15 }
  ↓
Token cost: 0 (pure computation)
```

### 4.2 Layout Rules Engine

Rules engine menginterpretasikan instruksi AI ke koordinat konkret:

```python
class LayoutRuleEngine:
    def interpret(self, instruction: str, context: dict) -> CoordinateElement:
        """
        Contoh instruksi AI:
        - "judul di atas, rata tengah"
        - "video utama fullscreen"
        - "overlay logo di pojok kanan bawah"
        - "subtitle di bawah, teks putih"
        - "dua video side-by-side"
        """
        match instruction:
            case "judul di atas, rata tengah":
                return {
                    "x": 0.5, "y": 0.15,
                    "width": 0.8, "height": 0.12,
                    "text_align": "center"
                }
            case "video utama fullscreen":
                return {
                    "x": 0, "y": 0,
                    "width": 1.0, "height": 1.0,
                    "fit": "cover"
                }
            case "overlay logo di pojok kanan bawah":
                return {
                    "x": 0.85, "y": 0.85,
                    "width": 0.12, "height": 0.12 * aspect_ratio
                }
            case "subtitle di bawah":
                return {
                    "x": 0.5, "y": 0.90,
                    "width": 0.9, "height": 0.08
                }
            case "side-by-side":
                return [
                    {"x": 0, "y": 0, "width": 0.5, "height": 1.0},  # left
                    {"x": 0.5, "y": 0, "width": 0.5, "height": 1.0}  # right
                ]
            case "picture-in-picture":
                return [
                    {"x": 0, "y": 0, "width": 1.0, "height": 1.0},  # bg
                    {"x": 0.7, "y": 0.65, "width": 0.25, "height": 0.25}  # pip
                ]
```

### 4.3 Safe Zone & Margin

Setiap template punya **safe zone** untuk memastikan konten tidak terpotong:

```
┌─────────────────────────────────────────┐
│  ░░░░░░░░░░░ MARGIN ░░░░░░░░░░░░░░░░   │ 10%
│  ░░ ┌──────────────────────────────┐ ░░ │
│  ░░ │        SAFE ZONE             │ ░░ │
│  ░░ │    (konten utama di sini)    │ ░░ │
│  ░░ │                              │ ░░ │
│  ░░ └──────────────────────────────┘ ░░ │
│  ░░░░░░░░░░░ MARGIN ░░░░░░░░░░░░░░░░   │
│  ░░ ┌──────────────────────────────┐ ░░ │
│  ░░ │   SUBTITLE / CAPTION         │ ░░ │
│  ░░ └──────────────────────────────┘ ░░ │
└─────────────────────────────────────────┘
```

### 4.4 Smart Positioning Macros

```python
# Makro positioning yang bisa dipanggil AI

def rule_of_thirds(horizontal: str, vertical: str) -> (float, float):
    """Terapkan rule of thirds photography"""
    h_map = {"kiri": 0.33, "tengah": 0.5, "kanan": 0.66}
    v_map = {"atas": 0.33, "tengah": 0.5, "bawah": 0.66}
    return h_map[horizontal], v_map[vertical]

def golden_ratio(offset: float = 0.0) -> (float, float):
    """Posisi golden ratio (≈ 1.618)"""
    phi = 1.618
    return 1.0 / phi + offset, 1.0 / phi + offset

def center() -> (float, float):
    return 0.5, 0.5

def top_left(padding: float = 0.05) -> (float, float):
    return padding, padding

def bottom_right(padding: float = 0.05) -> (float, float):
    return 1.0 - padding, 1.0 - padding
```

---

## 5. Keyframe & Animasi System

### 5.1 Keyframe Interpolation

```
Keyframe: [{t:0, x:0, opacity:0}, {t:1, x:0.5, opacity:1}]
  │
  │  Easing: ease-in-out
  │
  ▼
  t=0:   x=0.0,   opacity=0.0
  t=0.2: x=0.15,  opacity=0.3   (ease-in: lambat di awal)
  t=0.5: x=0.5,   opacity=0.7   (linear: kecepatan konstan)
  t=0.8: x=0.75,  opacity=0.9   (ease-out: melambat)
  t=1.0: x=1.0,   opacity=1.0
```

### 5.2 Animation Presets

```yaml
presets:
  fade_in:
    keyframes:
      - { time: 0, opacity: 0 }
      - { time: 0.5, opacity: 1 }
    easing: ease-out

  slide_up:
    keyframes:
      - { time: 0, y: "+0.2", opacity: 0 }
      - { time: 0.5, y: 0, opacity: 1 }
    easing: ease-out

  zoom_in:
    keyframes:
      - { time: 0, scale: 0.8, opacity: 0 }
      - { time: 0.5, scale: 1.0, opacity: 1 }
    easing: ease-out

  bounce:
    keyframes:
      - { time: 0, y: "-0.3", opacity: 0 }
      - { time: 0.4, y: 0, opacity: 1 }
      - { time: 0.5, y: "-0.02", opacity: 1 }
      - { time: 0.6, y: 0, opacity: 1 }
    easing: ease-out
```

---

## 6. AI → Coordinate Translation

### 6.1 Natural Language to Layout

Ini adalah **satu-satunya bagian yang pakai AI** untuk layout:

```
USER: "Buat video opening dengan logo di tengah, teks judul di bawah logo,
       dan background gradasi biru"
  │
  ▼
Mandor LLM (MOKO-4B):
  ├─ Intent: "opening_video"
  ├─ Elements: [logo, title, background]
  ├─ Layout pattern: center-top-down
  └─ Output: layout_plan (JSON)
  │
  ▼
Layout Engine (0 token):
  ├─ Background: fullscreen, gradient blue
  ├─ Logo: x=0.5, y=0.35, w=0.3, h=0.3, anim=fade_in
  ├─ Title: x=0.5, y=0.7, w=0.8, h=0.1, font=48px
  └─ Timing: logo 0-5s, title 0.5-5s
  │
  ▼
Hasil: CoordinateElement[] lengkap dengan koordinat eksak
```

### 6.2 Confidence-Based API Call

```python
def layout_from_nl(instruction: str) -> list[CoordinateElement]:
    """Natural language → layout coordinates"""
    
    # Step 1: Coba rule matching dulu (0 token)
    rule_result = LayoutRuleEngine.match(instruction)
    if rule_result and rule_result.confidence > 0.85:
        return rule_result.elements  # ✅ 100% lokal
    
    # Step 2: Coba template matching
    template = TemplateDB.find_similar(instruction)
    if template:
        return template.apply(instruction)  # ✅ Template match
    
    # Step 3: Baru panggil LLM (kecil dulu)
    local_result = MandorLLM.generate_layout(instruction)
    if local_result.confidence > 0.7:
        return local_result.elements  # ✅ Lokal cukup yakin
    
    # Step 4: API fallback (hanya jika perlu)
    api_result = APIModel.generate_layout(instruction)
    return api_result.elements  # ⚠️ Token terpakai
```

---

## 7. Compositing & Rendering

### 7.1 Coordinate to Canvas

Setelah layout selesai, compositor mengubah koordinat menjadi frame:

```python
def composite_frame(
    elements: list[CoordinateElement], 
    assets: dict[str, Asset],
    time: float
) -> Frame:
    """
    Composite satu frame dari daftar elemen.
    - elements: daftar semua elemen di timeline
    - assets: video/image/audio assets
    - time: current time in seconds
    
    Returns: Frame (RGBA buffer)
    """
    # Filter elemen yang visible di time ini
    visible = [e for e in elements if e.timeline.start <= time <= e.timeline.end]
    
    # Sort by z-index (ascending)
    visible.sort(key=lambda e: e.position.z)
    
    # Composite setiap layer
    canvas = Canvas()
    for element in visible:
        # Hitung posisi pixel
        px = element.position.x * canvas.width
        py = element.position.y * canvas.height
        pw = element.size.width * canvas.width
        ph = element.size.height * canvas.height
        
        # Apply transform & animation
        transform = calculate_transform(element, time)
        
        # Blend ke canvas
        canvas.blit(assets[element.id], px, py, pw, ph, transform)
    
    return canvas.to_frame()
```

### 7.2 Hardware Acceleration

Compositing di-accelerate via:
- **OpenCut WASM** — compositor existing (prioritas)
- **WebGL/WebGPU** — GPU acceleration via browser
- **FFmpeg filter graph** — fallback untuk batch rendering
- **MOKO native_accel** — C++/Rust untuk compositing berat

---

## 8. Contoh Template Lengkap

### 8.1 TikTok 9:16 — Product Review

```yaml
name: "tiktok-product-review"
aspect_ratio: "9:16"
canvas: { width: 1080, height: 1920 }

tracks:
  - id: "main_video"
    type: "video"
    z: 0
    position: { x: 0.5, y: 0.45 }
    size: { width: 1.0, height: 0.7, unit: "normalized" }
    fit: "cover"
    
  - id: "reviewer_face"
    type: "video"
    z: 1
    position: { x: 0.22, y: 0.78 }
    size: { width: 0.25, height: 0.25, unit: "normalized" }
    border: { width: 3, color: "#FFFFFF" }
    border_radius: 50  # circular
    
  - id: "product_name"
    type: "text"
    z: 2
    position: { x: 0.5, y: 0.06 }
    size: { width: 0.9, height: 0.08, unit: "normalized" }
    style:
      text: "{PRODUCT_NAME}"
      font_size: 64
      color: "#FFFFFF"
      text_align: "center"
      font_weight: 800
    animation: { preset: "fade_in" }
    
  - id: "price"
    type: "text"
    z: 2
    position: { x: 0.5, y: 0.14 }
    size: { width: 0.5, height: 0.06, unit: "normalized" }
    style:
      text: "{PRICE}"
      font_size: 48
      color: "#FF4444"
      text_align: "center"
      font_weight: 700
    animation: { preset: "slide_up", delay: 0.5 }
    
  - id: "rating_stars"
    type: "text"
    z: 2
    position: { x: 0.5, y: 0.20 }
    size: { width: 0.4, height: 0.05, unit: "normalized" }
    style:
      text: "⭐⭐⭐⭐⭐"
      font_size: 32
      text_align: "center"
    animation: { preset: "zoom_in", delay: 1.0 }
    
  - id: "cta_button"
    type: "shape"
    z: 3
    position: { x: 0.5, y: 0.92 }
    size: { width: 0.7, height: 0.06, unit: "normalized" }
    style:
      background_color: "#FF4444"
      border_radius: 30
    timeline: { start: 20, end: 60 }
    animation: { preset: "bounce", delay: 20 }
    
  - id: "cta_text"
    type: "text"
    z: 4
    position: { x: 0.5, y: 0.92 }
    size: { width: 0.7, height: 0.06, unit: "normalized" }
    style:
      text: "BELI SEKARANG →"
      font_size: 36
      color: "#FFFFFF"
      text_align: "center"
      font_weight: 700
    timeline: { start: 20, end: 60 }
    animation: { preset: "bounce", delay: 20 }
```

---

## 9. Integrasi dengan MOKO CausalVisualFlowCompressor

Sistem koordinat ini kompatibel dengan **MOKO CausalVisualFlowCompressor** 
(dari `moko_llm_runtime_guard.py`) untuk kompresi visual layout:

```
COORD_2D Format:
  [OPTICAL_ANCHOR: x:y-width:height@t_start-t_end]

Contoh:
  - Logo di tengah: [OPTICAL_ANCHOR: 0.35:0.2-0.3:0.3@0-30]
  - Subtitle di bawah: [OPTICAL_ANCHOR: 0.05:0.85-0.9:0.1@5-60]
  - Product overlay: [OPTICAL_ANCHOR: 0.7:0.65-0.25:0.25@10-50]

Ini memungkinkan MOKO OS memahami tata letak visual video secara
terkompresi, tanpa perlu mengirim full frame ke LLM.
```

---

## 10. Ringkasan

Coordinate Layout System adalah inti dari efisiensi token sistem ini:

1. **AI tidak perlu menghitung posisi** — cukup bilang "letakkan di pojok kanan"
2. **Engine menghitung matematikanya** — 0 token, precision pixel
3. **Template bisa di-reuse** — sekali buat, ribuan video
4. **Keyframe system** — animasi smooth tanpa AI
5. **MOKO compatible** — format COORD_2D bisa langsung diproses MOKO OS

> "AI menentukan apa yang mau diletakkan di mana — 
>  matematika yang menghitung posisi pastinya."

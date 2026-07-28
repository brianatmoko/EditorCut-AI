use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gpui::{
    canvas, div, prelude::*, px, point, Bounds, ClickEvent, Context, Corners, IntoElement,
    ParentElement, PathBuilder, Render, RenderImage, SharedString, Styled, Window,
};

use animation::{stickman_to_segments, StickmanRenderData, get_character, ActionBeat, ColorRole, ShapeKind, StageEntity, SceneExecutor};
use crate::state::EditorState;
use crate::ui::theme::Theme;
use spine_runtime::{ScmlData, parse_scml, evaluate_pose, get_file_name, get_file_info};

pub struct CharacterSkin {
    pub id: String,
    pub name: String,
    /// SCML bone animation data (loaded from scml/ directory)
    pub scml_data: Option<Arc<ScmlData>>,
    pub scml_entity_name: Option<String>,
    /// Loaded body part PNG images (filename → RenderImage, unrotated)
    pub scml_parts: std::sync::Mutex<HashMap<String, Arc<RenderImage>>>,
    /// Raw RGBA images for per-bone rotation (filename → RgbaImage)
    pub scml_raw_parts: std::sync::Mutex<HashMap<String, image::RgbaImage>>,
    /// Cache of rotated/transformed images (key: filename+angle+flip → (image, w, h))
    pub rotated_cache: std::sync::Mutex<HashMap<(String, i32, bool), (Arc<RenderImage>, f32, f32)>>,
}

impl CharacterSkin {
    /// Map a director action name to an SCML animation name.
    pub fn action_to_scml_anim(&self, action: &str) -> &str {
        match action.to_lowercase().as_str() {
            "idle" | "stand" | "waiting" | "wave" | "point" | "taunt" | "cheer"
            | "surrender" | "cower" | "cry" | "laugh" | "triumph" | "nod"
            | "shake_head" | "shrug" | "salute" | "bow" | "talk" | "give_up" => "Idle",
            "walk" | "walking" | "slow_walk" | "sad_walk" | "stealth_walk" => "Walk",
            "run" | "running" | "sprint" | "panic_run" => "Run",
            "jump" | "jumping" | "hop" | "happy_hop" | "vault" | "climb" => "Jump",
            "hurt" | "hit" | "hurt_heavy" | "stunned" | "stumble" | "down" | "dead" => "Hurt",
            "fall" | "jatuh" | "terluka" => "Hurt",
            "attack" | "attack1" | "punch" | "jab" | "cross" | "uppercut" | "hook" => "Attack",
            "attack2" | "kick" | "roundhouse" | "side_kick" => "Attack1",
            "attack3" | "shoot" | "shooting" | "aim" | "shoot_pistol" => "Attack2",
            "attack4" | "melee_swing" | "melee_stab" => "Attack3",
            "fun" | "gift" | "celebrate" => "Fun",
            "dodge" | "roll" | "dive" | "slide" | "cover" | "duck" | "weave" => "Hit",
            "block" | "parry" | "defend" => "Hit",
            _ => "Idle",
        }
    }
}

pub struct CharacterRegistry {
    pub skins: HashMap<String, CharacterSkin>,
}

// ════════════════════════════════════════════════════════════════
// CAMERA PROJECTION — Pinhole Camera 2.5D Model
// ════════════════════════════════════════════════════════════════
//
// World: units where X is horizontal (-3..+3 visible), Z is depth (1..3).
// Camera has a center_x in world, and a zoom (1.0 = neutral, >1 = tighter).
// Parallax = foreground (low Z) appears to move faster horizontally than background (high Z).
// Perspective = characters at low Z are bigger (camera closer to them), high Z = smaller.

pub struct CameraProjection {
    pub mon_cx: f32,         // screen center x (px)
    pub ground_y: f32,       // ground level (screen y px)
    pub view_scale: f32,     // pixels per world unit at Z=1.0 (determines "lens" focal length)
    pub zoom: f32,           // camera zoom-in factor (>1 = tighter shot)
    pub cam_center_x: f32,   // world x the camera looks at (center of viewport in world units)
    pub cam_center_y: f32,   // world y the camera looks at
    pub shake_x: f32,
    pub shake_y: f32,
}

pub struct ProjectedEntity {
    pub screen_x: f32,
    pub screen_y: f32,
    pub sprite_w: f32,
    pub sprite_h: f32,
}

impl CameraProjection {
    /// Project (world_x, world_y, world_z) → screen coords + sprite size.
    /// Uses parallax: `relative_x = (entity_x - cam_center_x) / entity_z`
    /// Perspective: `size_scale = zoom / entity_z` → foreground grows larger on zoom-in.
    pub fn project(
        &self,
        entity_x: f32,
        entity_y: f32,
        entity_z: f32,
        base_char_height: f32,
    ) -> ProjectedEntity {
        // Clamp Z to avoid divide-by-zero / extreme foreground
        let z = entity_z.max(0.5);

        // Parallax: foreground entities move more relative to camera center than background
        let relative_x = (entity_x - self.cam_center_x) / z;

        // Screen X: relative position × pixels-per-unit × zoom
        // (zoom multiplies the whole view scale → tighter shot = larger coords)
        let screen_x = self.mon_cx + relative_x * self.view_scale * self.zoom + self.shake_x;

        // Y world (height above ground) — minor visual lift, also perspective-scaled
        let y_world_lift = entity_y * self.view_scale * 0.35 / z * self.zoom;

        // Ground is bottom of "visible plane" — depth makes entities sit slightly higher
        let depth_lift = (1.0 / z - 1.0).max(0.0) * base_char_height * 0.18;

        let screen_y = self.ground_y - y_world_lift + depth_lift + self.shake_y;

        // Perspective size: zoom × (1/z). Foreground = bigger, background = smaller.
        // This is the KEY difference from old system — no more uniform zoom scaling.
        let size_scale = self.zoom / z;
        let sprite_h = base_char_height * size_scale;
        let sprite_w = sprite_h; // square aspect base — adjusted by caller with actual aspect

        ProjectedEntity {
            screen_x,
            screen_y,
            sprite_w,
            sprite_h,
        }
    }

    /// Helper: visible frustum check (cull entities off-screen for efficiency)
    pub fn is_visible(&self, entity_x: f32, entity_z: f32, margin_factor: f32) -> bool {
        let z = entity_z.max(0.5);
        let relative_x = (entity_x - self.cam_center_x) / z;
        let half_unit_view = self.zoom * margin_factor;
        relative_x.abs() < half_unit_view
    }
}

/// Rotate a point around (cx, cy) by `angle_deg` degrees (clockwise).
/// Used for Dutch angle camera tilt.
fn dutch_rotate(x: f32, y: f32, cx: f32, cy: f32, angle_deg: f32) -> (f32, f32) {
    if angle_deg == 0.0 { return (x, y); }
    let rad = angle_deg.to_radians();
    let dx = x - cx;
    let dy = y - cy;
    let cos = rad.cos();
    let sin = rad.sin();
    (cx + dx * cos - dy * sin, cy + dx * sin + dy * cos)
}

/// Render a character using SCML bone animation instead of frame sequences.
/// Returns `true` if rendered successfully, `false` if SCML data is not available.
fn render_scml_character(
    scml_data: &ScmlData,
    entity_name: &str,
    anim_name: &str,
    current_time: f64,
    skin: &CharacterSkin,
    proj: &ProjectedEntity,
    _projector: &CameraProjection,
    mon_cx: f32,
    ground_y: f32,
    tilt_angle: f32,
    facing_left: bool,
    window: &mut gpui::Window,
) -> bool {
    use std::collections::hash_map::Entry;

    let entity = match scml_data.entities.iter().find(|e| e.name == entity_name) {
        Some(e) => e,
        None => return false,
    };
    let anim = match entity.animations.iter().find(|a| a.name == anim_name)
        .or_else(|| entity.animations.first())
    {
        Some(a) => a,
        None => return false,
    };

    let time_ms = ((current_time * 1000.0) as u32) % anim.length.max(1);
    let pose = evaluate_pose(scml_data, entity, anim, time_ms, None);

    // ── SCML pixel-units → screen pixels ──────────────────────────────
    // Airama karakter preview: tinggi sprite setinggi `proj.sprite_h` pixel.
    // Karakter SCML pakai pixel koordinate orphan. Pilih referensi: tinggi
    // rata-rata objek "body" dari SCML mainline ~ 200px. Kalibrasi scale:
    let ref_height = 500.0_f32; // total karakter ~500px di SCML space
    let char_scale = (proj.sprite_h / ref_height).max(0.05);

    // Karakter center di screen:
    // proj.screen_y = feet/ground level. Karakter root bone ada di tengah
    // (anatomi pivot). Untuk alignment yang natural, pakai proyeksi origin
    // di midpoint vertikal sprite, supaya World Y=0 di SCML ada di center.
    let char_cx = proj.screen_x;
    let char_cy = proj.screen_y - proj.sprite_h * 0.5;

    for obj in &pose.objects {
        if obj.alpha < 0.01 { continue; }

        let file_name = get_file_name(&scml_data.folders, obj.folder, obj.file);
        let file_info = get_file_info(&scml_data.folders, obj.folder, obj.file);

        let fw = file_info.map_or(100.0, |f| f.width) as f32;
        let fh = file_info.map_or(100.0, |f| f.height) as f32;
        let pivot_x = file_info.map_or(0.5, |f| f.pivot_x) as f32;
        let pivot_y = file_info.map_or(0.5, |f| f.pivot_y) as f32;

        // Get/cache raw RGBA dan rotated RenderImage
        let angle_deg = obj.world_angle as f32;
        let angle_rounded = (angle_deg / 5.0).round() as i32 * 5; // quantum 5°
        let cache_key = (file_name.clone(), angle_rounded, facing_left);

        let (img, out_w, out_h) = {
            let cache = skin.rotated_cache.lock().unwrap();
            if let Some(&(ref ri, w, h)) = cache.get(&cache_key) {
                (Arc::clone(ri), w, h)
            } else {
                drop(cache);
                // Lazy-load raw RGBA
                let raw = {
                    let mut raws = skin.scml_raw_parts.lock().unwrap();
                    if let Entry::Occupied(o) = raws.entry(file_name.clone()) {
                        o.get().clone()
                    } else {
                        // Load via helper
                        let loaded = load_raw_rgba(&skin.id, &file_name);
                        if let Some(img) = loaded {
                            raws.insert(file_name.clone(), img.clone());
                            img
                        } else {
                            continue;
                        }
                    }
                };

                // Rotate + flip → RenderImage
                let (ri, w, h) = rotate_rgba_image(&raw, if angle_rounded == 0 { 0.01 } else { angle_rounded as f32 }, facing_left);
                let mut cache = skin.rotated_cache.lock().unwrap();
                cache.insert(cache_key.clone(), (Arc::clone(&ri), w, h));
                (ri, w, h)
            }
        };

        // ── Compute screen position: SCML pixel-space → screen-space ──
        let sx = char_cx + obj.world_x * char_scale;
        let sy = char_cy - obj.world_y * char_scale; // SCML Y-up → screen Y-down

        let (rx, ry) = dutch_rotate(sx, sy, mon_cx, ground_y, tilt_angle);

        // Pivot di SCML: (0,1) = bottom-left. (0.5,0.5) = center.
        // Untuk rotating part centered pada pivot point:
        // draw_x/y = world_pos − pivot * rotated_size (di screen space)
        let draw_w = out_w * char_scale * (obj.scale_x.abs() as f32).max(0.05);
        let draw_h = out_h * char_scale * (obj.scale_y.abs() as f32).max(0.05);

        // Center part di (rx, ry) — pivot point tadi tinggal di pusat part rotated
        let draw_x = rx - draw_w * 0.5;
        let draw_y = ry - draw_h * 0.5;

        let bounds = gpui::Bounds {
            origin: point(px(draw_x), px(draw_y)),
            size: gpui::size(px(draw_w.max(1.0)), px(draw_h.max(1.0))),
        };
        let _ = window.paint_image(bounds, Corners::all(px(0.0)), Arc::clone(&img), 0, false);
    }

    true
}

/// Load raw RGBA image from the SCML directory for `skin_id` / `file_name`.
fn load_raw_rgba(skin_id: &str, file_name: &str) -> Option<image::RgbaImage> {
    let base_paths = [
        std::path::PathBuf::from("."),
        std::path::PathBuf::from("/home/brianatmokoo/Documents/Linux/Opencut"),
    ];
    let root = base_paths.iter().find(|p| p.exists())?;

    let (pack_dir, scml_subdir) = match skin_id {
        s if s.starts_with("police") => {
            let num = s.trim_start_matches("police_");
            ("craftpix-543219-2d-game-police-character-free-sprite-sheets", num)
        }
        s if s.starts_with("terrorist") => {
            let num = s.trim_start_matches("terrorist_");
            ("craftpix-485144-2d-game-terrorists-character-free-sprites-sheets", num)
        }
        s if s.starts_with("chibi") => {
            let season = s.trim_start_matches("chibi_");
            ("craftpix-955440-2d-game-chibi-boy-free-character-sprite-sheet", season)
        }
        _ => return None,
    };

    let scml_dir = root.join(pack_dir).join("scml").join(scml_subdir);
    let file_path = scml_dir.join(file_name);

    let actual_path = if file_path.exists() {
        file_path
    } else {
        // Try fallbacks
        let fallbacks = [
            root.join(pack_dir).join("scml").join("1").join(file_name),
            root.join(pack_dir).join("scml").join("summer").join(file_name),
        ];
        fallbacks.into_iter().find(|p| p.exists())?
    };

    image::open(&actual_path).ok().map(|i| i.to_rgba8())
}

pub struct Preview {
    editor: Rc<RefCell<EditorState>>,
    is_playing: bool,
    current_time: f64,
    total_duration: f64,
    render_data: Option<Arc<StickmanRenderData>>,
    /// Cache of decoded background images keyed by theme name
    bg_cache: HashMap<String, Arc<RenderImage>>,
    /// GPU textures created in the last frame to be dropped in the next frame
    prev_frame_gpu_images: Rc<RefCell<Vec<Arc<RenderImage>>>>,
    /// Multi-character 2D sprite registry
    character_registry: Option<Arc<CharacterRegistry>>,
    /// Active Cinematic Movie script (multi-act, multi-entity, virtual camera)
    pub cinematic_movie: Option<Arc<animation::CinematicMovie>>,
    /// Camera transition: previous camera shot (for smooth interpolation)
    prev_camera: Option<animation::CameraShot>,
    /// Camera transition progress (0.0 → 1.0)
    camera_transition_t: f64,
    /// Previous theme for background cross-fade
    prev_theme: Option<String>,
    /// Background cross-fade progress (0.0 → 1.0)
    bg_transition_t: f64,
    /// Smart camera director for dynamic shot selection
    smart_director: animation::SmartCameraDirector,
    /// Cached action beats for current act (avoids recomputing every frame)
    cached_beats: Vec<animation::ActionBeat>,
    /// Last act number we cached beats for
    last_act_number: u32,
    /// Current dynamically-selected camera shot from SmartCameraDirector
    dynamic_camera: Option<animation::CameraShot>,
    /// Previous dynamic camera shot (for smooth interpolation on shot change)
    prev_dynamic_camera: Option<animation::CameraShot>,
    /// Interpolation progress between prev and current dynamic camera (0.0→1.0)
    dynamic_transition_t: f64,
    /// Scene executor for character movement at the cinematic rendering layer
    scene_executor: Option<Arc<Mutex<animation::SceneExecutor>>>,
}

impl Preview {
    pub fn new(editor: Rc<RefCell<EditorState>>, _cx: &mut Context<Self>) -> Self {
        // Lazy-load assets from disk on startup
        let registry = load_character_registry();
        let character_registry = if registry.skins.is_empty() { None } else { Some(Arc::new(registry)) };
        Self {
            editor,
            is_playing: false,
            current_time: 0.0,
            total_duration: 60.0,
            render_data: None,
            bg_cache: HashMap::new(),
            prev_frame_gpu_images: Rc::new(RefCell::new(Vec::new())),
            character_registry,
            cinematic_movie: None,
            prev_camera: None,
            camera_transition_t: 0.0,
            prev_theme: None,
            bg_transition_t: 0.0,
            smart_director: animation::SmartCameraDirector::default(),
            cached_beats: Vec::new(),
            last_act_number: 0,
            dynamic_camera: None,
            prev_dynamic_camera: None,
            dynamic_transition_t: 1.0,
            scene_executor: None,
        }
    }

    fn format_time(secs: f64) -> String {
        let mins = (secs / 60.0).floor() as u32;
        let s = (secs % 60.0) as u32;
        let cs = ((secs - secs.floor()) * 100.0) as u32;
        format!("{:02}:{:02}.{:02}", mins, s, cs)
    }

    fn seek_by(editor: &Rc<RefCell<EditorState>>, delta: f64) {
        let editor = editor.borrow_mut();
        let mut anim = editor.animator.borrow_mut();
        let new_time = (anim.time() + delta).max(0.0).min(anim.total_duration());
        anim.seek(new_time);
    }

    fn skip_to_start(editor: &Rc<RefCell<EditorState>>) {
        let editor = editor.borrow_mut();
        let mut anim = editor.animator.borrow_mut();
        anim.seek(0.0);
    }

    fn skip_to_end(editor: &Rc<RefCell<EditorState>>) {
        let editor = editor.borrow_mut();
        let anim = editor.animator.borrow();
        let end = anim.total_duration();
        drop(anim);
        let mut anim = editor.animator.borrow_mut();
        anim.seek(end);
    }

    /// Load (or return cached) background RenderImage for a theme.
    fn get_or_load_bg(&mut self, theme: &str) -> Option<Arc<RenderImage>> {
        if let Some(cached) = self.bg_cache.get(theme) {
            return Some(Arc::clone(cached));
        }
        let img = load_bg_image(theme)?;
        self.bg_cache.insert(theme.to_string(), Arc::clone(&img));
        Some(img)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_stickman(
        data: &StickmanRenderData,
        bounds: Bounds<gpui::Pixels>,
        window: &mut Window,
        bg_image: Option<Arc<RenderImage>>,
        prev_bg_image: Option<Arc<RenderImage>>,
        bg_transition_t: f64,
        character_registry: Option<&CharacterRegistry>,
        cinematic_movie: Option<&animation::CinematicMovie>,
        current_time: f64,
        camera_transition_t: f64,
        prev_camera: Option<&animation::CameraShot>,
        dynamic_camera: Option<&animation::CameraShot>,
        prev_dynamic_camera: Option<&animation::CameraShot>,
        camera_dynamic_transition_t: f64,
        scene_executor: Option<&Mutex<animation::SceneExecutor>>,
        beats: &[animation::ActionBeat],
    ) {
        // ── Coordinate setup ────────────────────────────────────────────────
        let aspect_ratio = 16.0_f32 / 9.0_f32;
        let bounds_w: f32 = bounds.size.width.into();
        let bounds_h: f32 = bounds.size.height.into();
        let origin_x: f32 = bounds.origin.x.into();
        let origin_y: f32 = bounds.origin.y.into();

        let (mon_w, mon_h) = if bounds_w / bounds_h > aspect_ratio {
            (bounds_h * aspect_ratio, bounds_h)
        } else {
            (bounds_w, bounds_w / aspect_ratio)
        };

        let mon_x = origin_x + (bounds_w - mon_w) / 2.0;
        let mon_y = origin_y + (bounds_h - mon_h) / 2.0;
        let mon_cx = mon_x + mon_w / 2.0;
        let _mon_cy = mon_y + mon_h / 2.0;
        let cam_x = data.pos_x as f32;
        // Ground = bottom 15% of screen. With bg image, put at 87% (on the asphalt).
        let ground_y = if bg_image.is_some() { mon_y + mon_h * 0.87 } else { mon_y + mon_h * 0.85 };
        // scale: one Spine "unit" → pixels. Spineboy is ~515 Spine units tall (foot bottom to head top).
        // We want the character to be ~60% of frame height.
        let spine_char_height = 515.0_f32; // Spine units from foot bottom (~-30) to head top (~485)
        let desired_char_screen_h = mon_h * 0.60; // character takes 60% of screen height
        let spine_px_per_unit = desired_char_screen_h / spine_char_height;
        // Legacy stickman scale (different coord system)
        let scale = mon_h * 0.75;

        // ── Drawing helpers ─────────────────────────────────────────────────
        let line = |x1: f32, y1: f32, x2: f32, y2: f32, thick: f32,
                    col: gpui::Rgba, win: &mut Window| {
            let mut p = PathBuilder::stroke(px(thick));
            p.move_to(point(px(x1), px(y1)));
            p.line_to(point(px(x2), px(y2)));
            if let Ok(p) = p.build() { win.paint_path(p, col); }
        };

        // Draw a filled axis-aligned rect
        let rect = |x: f32, y: f32, w: f32, h: f32, col: gpui::Rgba, win: &mut Window| {
            let mut p = PathBuilder::fill();
            p.move_to(point(px(x), px(y)));
            p.line_to(point(px(x + w), px(y)));
            p.line_to(point(px(x + w), px(y + h)));
            p.line_to(point(px(x), px(y + h)));
            p.line_to(point(px(x), px(y)));
            if let Ok(p) = p.build() { win.paint_path(p, col); }
        };

        // Draw a filled polygon from list of (x,y) points
        let poly = |pts: &[(f32, f32)], col: gpui::Rgba, win: &mut Window| {
            if pts.len() < 2 { return; }
            let mut p = PathBuilder::fill();
            p.move_to(point(px(pts[0].0), px(pts[0].1)));
            for &(x, y) in &pts[1..] { p.line_to(point(px(x), px(y))); }
            p.line_to(point(px(pts[0].0), px(pts[0].1)));
            if let Ok(p) = p.build() { win.paint_path(p, col); }
        };

        // Draw a filled circle approximation
        let circle = |cx: f32, cy: f32, r: f32, col: gpui::Rgba, win: &mut Window| {
            let n = 20_u32;
            let mut p = PathBuilder::fill();
            p.move_to(point(px(cx + r), px(cy)));
            for i in 1..=n {
                let a = std::f32::consts::TAU * i as f32 / n as f32;
                p.line_to(point(px(cx + r * a.cos()), px(cy + r * a.sin())));
            }
            if let Ok(p) = p.build() { win.paint_path(p, col); }
        };

        // Draw a bone capsule from p1 to p2 with given radius (filled pill shape)
        let _capsule = |x1: f32, y1: f32, x2: f32, y2: f32, r: f32,
                       col: gpui::Rgba, win: &mut Window| {
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            let nx = -dy / len * r;
            let ny = dx / len * r;
            let pts = [
                (x1 + nx, y1 + ny),
                (x2 + nx, y2 + ny),
                (x2 - nx, y2 - ny),
                (x1 - nx, y1 - ny),
            ];
            poly(&pts, col, win);
            // end-cap circles
            circle(x1, y1, r, col, win);
            circle(x2, y2, r, col, win);
        };

        // Map model space → screen space (for legacy stickman)
        let mx = |x: f64| -> f32 { mon_cx + (x as f32 - cam_x) * scale };
        let my = |y: f64| -> f32 { ground_y - y as f32 * scale };
        // Map Spine world coords (Y-up, 0=ground) → screen space
        let _sx = |x: f64| -> f32 { mon_cx + (x as f32) * spine_px_per_unit };
        let _sy = |y: f64| -> f32 { ground_y - y as f32 * spine_px_per_unit };

        // ── 1. Background (image or procedural fallback) with cross-fade ────
        // Parallax: background shifts slower than foreground when camera pans.
        // cam_x (entity pos + camera pan offset) drives the shift direction.
        let pan_offset = dynamic_camera.map(|cam| cam.pan_x as f64).unwrap_or(0.0_f64);
        let parallax_cam = data.pos_x + pan_offset;
        let bg_parallax_factor = 4.0; // higher = slower background movement
        let parallax_px = (-parallax_cam * (mon_w as f64) * 0.03 / bg_parallax_factor) as f32;
        let bg_x = mon_x + parallax_px;
        if bg_transition_t < 1.0 {
            // Transisi: paint previous background with fade-out
            if let Some(ref prev_bg) = prev_bg_image {
                let alpha = (1.0 - bg_transition_t).clamp(0.0, 1.0) as f32;
                let image_bounds = gpui::Bounds {
                    origin: point(px(bg_x), px(mon_y)),
                    size: gpui::size(px(mon_w), px(mon_h)),
                };
                let _ = window.paint_image(image_bounds, Corners::all(px(0.0)), Arc::clone(prev_bg), 0, false);
            }
        }
        if let Some(ref bg) = bg_image {
            let image_bounds = gpui::Bounds {
                origin: point(px(bg_x), px(mon_y)),
                size: gpui::size(px(mon_w), px(mon_h)),
            };
            let _ = window.paint_image(image_bounds, Corners::all(px(0.0)), Arc::clone(bg), 0, false);
        } else {
            // Dark base fill for procedural themes
            rect(mon_x, mon_y, mon_w, mon_h, gpui::rgba(0x111827ff), window);
        }

        // ── 2. Themed environment overlay (only for procedural/no-image themes) ─
        if bg_image.is_none() { match data.scene_theme.as_str() {
            "city" => {
                // Sky gradient simulation (light blue)
                rect(mon_x, mon_y, mon_w, mon_h * 0.65, gpui::rgba(0x87ceebff), window);
                // Clouds
                circle(mon_cx - mon_w * 0.2, mon_y + mon_h * 0.12, mon_w * 0.07, gpui::rgba(0xffffffff), window);
                circle(mon_cx - mon_w * 0.12, mon_y + mon_h * 0.10, mon_w * 0.09, gpui::rgba(0xffffffff), window);
                circle(mon_cx + mon_w * 0.25, mon_y + mon_h * 0.08, mon_w * 0.06, gpui::rgba(0xf8f8f8ff), window);
                circle(mon_cx + mon_w * 0.32, mon_y + mon_h * 0.07, mon_w * 0.08, gpui::rgba(0xffffffff), window);
                // Buildings (far)
                let bld_col_far = gpui::rgba(0x8090a0ff);
                for i in 0..8_i32 {
                    let bx = mon_x + mon_w * (i as f32 * 0.14 + 0.02);
                    let bh = mon_h * (0.25 + (i as f32 * 0.07 + 0.3).sin().abs() * 0.20);
                    rect(bx, ground_y - bh, mon_w * 0.11, bh, bld_col_far, window);
                    // windows
                    let win_col = gpui::rgba(0xd4eaf7ff);
                    for row in 0..4_i32 {
                        for col in 0..3_i32 {
                            rect(bx + mon_w * 0.016 + col as f32 * mon_w * 0.028,
                                 ground_y - bh + mon_h * 0.04 + row as f32 * mon_h * 0.045,
                                 mon_w * 0.018, mon_h * 0.025, win_col, window);
                        }
                    }
                }
                // Buildings (near, darker)
                let bld_col = gpui::rgba(0x4a5568ff);
                for i in 0..5_i32 {
                    let bx = mon_x + mon_w * (i as f32 * 0.22 - 0.05);
                    let bh = mon_h * (0.30 + (i as f32 * 0.53 + 1.1).sin().abs() * 0.18);
                    rect(bx, ground_y - bh, mon_w * 0.18, bh, bld_col, window);
                    let win_col2 = gpui::rgba(0xfef3c7ff);
                    for row in 0..5_i32 {
                        for col in 0..3_i32 {
                            rect(bx + mon_w * 0.025 + col as f32 * mon_w * 0.045,
                                 ground_y - bh + mon_h * 0.05 + row as f32 * mon_h * 0.04,
                                 mon_w * 0.025, mon_h * 0.022, win_col2, window);
                        }
                    }
                }
                // Road
                rect(mon_x, ground_y, mon_w, mon_h * 0.35, gpui::rgba(0x374151ff), window);
                // Sidewalk stripe
                rect(mon_x, ground_y, mon_w, mon_h * 0.05, gpui::rgba(0x9ca3afff), window);
                // Road markings
                let dash_col = gpui::rgba(0xfbbf24ff);
                for i in 0..12_i32 {
                    rect(mon_x + mon_w * (i as f32 * 0.085 + 0.01), ground_y + mon_h * 0.15,
                         mon_w * 0.05, mon_h * 0.012, dash_col, window);
                }
                // Streetlights
                for i in [0.1_f32, 0.5, 0.9] {
                    let lx = mon_x + mon_w * i;
                    line(lx, ground_y + mon_h * 0.02, lx, ground_y - mon_h * 0.22,
                         2.0, gpui::rgba(0x6b7280ff), window);
                    line(lx, ground_y - mon_h * 0.22, lx + mon_w * 0.04, ground_y - mon_h * 0.22,
                         2.0, gpui::rgba(0x6b7280ff), window);
                    circle(lx + mon_w * 0.04, ground_y - mon_h * 0.22, mon_w * 0.008,
                           gpui::rgba(0xfde68aff), window);
                }
            }

            "cyberpunk" => {
                // Night sky
                rect(mon_x, mon_y, mon_w, mon_h, gpui::rgba(0x050a1aff), window);
                // Neon skyscrapers
                let bld_colors = [0x1e1b4bff, 0x1e1b4bff, 0x14012bff, 0x0a0a2eff];
                for i in 0..7_i32 {
                    let bx = mon_x + mon_w * (i as f32 * 0.155);
                    let bh = mon_h * (0.40 + (i as f32 * 0.8).sin().abs() * 0.35);
                    let col = gpui::rgba(bld_colors[(i % 4) as usize]);
                    rect(bx, ground_y - bh, mon_w * 0.13, bh, col, window);
                    // neon window strips
                    let strip_colors = [0xff00ffff, 0x00f3ffff, 0xff2d78ff];
                    let scol = gpui::rgba(strip_colors[(i % 3) as usize]);
                    for row in 0..8_i32 {
                        rect(bx + mon_w * 0.01, ground_y - bh + row as f32 * mon_h * 0.045 + mon_h * 0.02,
                             mon_w * 0.11, mon_h * 0.012, scol, window);
                    }
                }
                // Neon ground (rain reflections)
                rect(mon_x, ground_y, mon_w, mon_h * 0.35, gpui::rgba(0x0a0a1aff), window);
                // Neon horizon line
                line(mon_x, ground_y, mon_x + mon_w, ground_y, 3.0, gpui::rgba(0xff2d78ff), window);
                line(mon_x, ground_y, mon_x + mon_w, ground_y, 1.0, gpui::rgba(0xffffffff), window);
                // Neon road lines
                for i in 0..8_i32 {
                    line(mon_x + mon_w * (i as f32 * 0.13 + 0.03), ground_y + mon_h * 0.12,
                         mon_x + mon_w * (i as f32 * 0.13 + 0.03), ground_y + mon_h * 0.35,
                         1.5, gpui::rgba(0x00f3ff44), window);
                }
            }

            "forest" => {
                // Sky
                rect(mon_x, mon_y, mon_w, mon_h * 0.55, gpui::rgba(0x7dd3fcff), window);
                // Ground/grass
                rect(mon_x, ground_y, mon_w, mon_h * 0.45, gpui::rgba(0x166534ff), window);
                // Grass top strip
                rect(mon_x, ground_y - mon_h * 0.02, mon_w, mon_h * 0.04, gpui::rgba(0x22c55eff), window);
                // Background trees (far, lighter)
                for i in 0..10_i32 {
                    let tx = mon_x + mon_w * (i as f32 * 0.11 + 0.01);
                    let th = mon_h * (0.25 + (i as f32 * 0.7).sin().abs() * 0.12);
                    // trunk
                    rect(tx + mon_w * 0.025, ground_y - th * 0.3, mon_w * 0.02, th * 0.3, gpui::rgba(0x92400eff), window);
                    // foliage
                    circle(tx + mon_w * 0.035, ground_y - th * 0.4, mon_w * 0.045, gpui::rgba(0x16a34aff), window);
                    circle(tx + mon_w * 0.025, ground_y - th * 0.55, mon_w * 0.038, gpui::rgba(0x15803dff), window);
                }
                // Foreground trees (near, darker)
                for i in 0..5_i32 {
                    let tx = mon_x + mon_w * (i as f32 * 0.22 + 0.04);
                    let th = mon_h * (0.55 + (i as f32 * 1.1).sin().abs() * 0.12);
                    rect(tx + mon_w * 0.035, ground_y - th * 0.45, mon_w * 0.03, th * 0.45, gpui::rgba(0x78350fff), window);
                    circle(tx + mon_w * 0.05, ground_y - th * 0.6, mon_w * 0.07, gpui::rgba(0x14532dff), window);
                    circle(tx + mon_w * 0.035, ground_y - th * 0.78, mon_w * 0.055, gpui::rgba(0x166534ff), window);
                    circle(tx + mon_w * 0.065, ground_y - th * 0.72, mon_w * 0.05, gpui::rgba(0x15803dff), window);
                }
                // Flowers
                for i in [0.1_f32, 0.3, 0.6, 0.8] {
                    circle(mon_x + mon_w * i, ground_y - mon_h * 0.03, mon_h * 0.015, gpui::rgba(0xfbbf24ff), window);
                }
            }

            "room" => {
                // Floor
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0xc8a26aff), window);
                // Back wall
                rect(mon_x, mon_y, mon_w, ground_y - mon_y, gpui::rgba(0xf5f0e8ff), window);
                // Carpet
                rect(mon_cx - mon_w * 0.3, ground_y + mon_h * 0.05, mon_w * 0.6, mon_h * 0.15,
                     gpui::rgba(0x8b5cf6aa), window);
                // Wall baseboard
                rect(mon_x, ground_y - mon_h * 0.02, mon_w, mon_h * 0.02, gpui::rgba(0xe8dcc8ff), window);
                // Window (left)
                let wx = mon_x + mon_w * 0.08;
                let wy = mon_y + mon_h * 0.12;
                rect(wx, wy, mon_w * 0.18, mon_h * 0.3, gpui::rgba(0x93c5fdff), window);
                rect(wx, wy, mon_w * 0.18, mon_h * 0.3, gpui::rgba(0x6b7280ff), window); // frame is thin border via line
                line(wx, wy, wx + mon_w * 0.18, wy, 3.0, gpui::rgba(0x6b7280ff), window);
                line(wx, wy + mon_h * 0.3, wx + mon_w * 0.18, wy + mon_h * 0.3, 3.0, gpui::rgba(0x6b7280ff), window);
                line(wx, wy, wx, wy + mon_h * 0.3, 3.0, gpui::rgba(0x6b7280ff), window);
                line(wx + mon_w * 0.18, wy, wx + mon_w * 0.18, wy + mon_h * 0.3, 3.0, gpui::rgba(0x6b7280ff), window);
                // Bookshelf (right)
                let shx = mon_x + mon_w * 0.72;
                rect(shx, mon_y + mon_h * 0.08, mon_w * 0.24, mon_h * 0.55, gpui::rgba(0x92400eff), window);
                let book_cols = [0x6366f1ff, 0x10b981ff, 0xf59e0bff, 0xef4444ff, 0x06b6d4ff, 0x8b5cf6ff];
                for row in 0..3_i32 {
                    for col in 0..4_i32 {
                        let bkcol = gpui::rgba(book_cols[((row * 4 + col) % 6) as usize]);
                        rect(shx + mon_w * 0.02 + col as f32 * mon_w * 0.055,
                             mon_y + mon_h * 0.11 + row as f32 * mon_h * 0.15,
                             mon_w * 0.045, mon_h * 0.11, bkcol, window);
                    }
                }
                // Sofa (center-left)
                let sx = mon_cx - mon_w * 0.35;
                rect(sx, ground_y - mon_h * 0.15, mon_w * 0.38, mon_h * 0.15, gpui::rgba(0xf59e0bff), window);
                rect(sx, ground_y - mon_h * 0.25, mon_w * 0.38, mon_h * 0.12, gpui::rgba(0xd97706ff), window);
                rect(sx, ground_y - mon_h * 0.25, mon_w * 0.06, mon_h * 0.25, gpui::rgba(0xb45309ff), window);
                rect(sx + mon_w * 0.32, ground_y - mon_h * 0.25, mon_w * 0.06, mon_h * 0.25, gpui::rgba(0xb45309ff), window);
                // Potted plant
                circle(mon_x + mon_w * 0.6, ground_y - mon_h * 0.12, mon_h * 0.1, gpui::rgba(0x16a34aff), window);
                rect(mon_x + mon_w * 0.585, ground_y - mon_h * 0.05, mon_w * 0.03, mon_h * 0.06,
                     gpui::rgba(0x92400eff), window);
            }

            "school" => {
                // Room
                rect(mon_x, mon_y, mon_w, ground_y - mon_y, gpui::rgba(0xf9fafbff), window);
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0xe5e7ebff), window);
                // Blackboard
                rect(mon_cx - mon_w * 0.3, mon_y + mon_h * 0.08, mon_w * 0.6, mon_h * 0.35, gpui::rgba(0x166534ff), window);
                // Chalk text lines (decorative)
                for i in 0..4_i32 {
                    line(mon_cx - mon_w * 0.25, mon_y + mon_h * 0.16 + i as f32 * mon_h * 0.06,
                         mon_cx + mon_w * 0.1, mon_y + mon_h * 0.16 + i as f32 * mon_h * 0.06,
                         1.5, gpui::rgba(0xd1fae5ff), window);
                }
                // Desks
                for i in 0..3_i32 {
                    let dx = mon_x + mon_w * (0.1 + i as f32 * 0.3);
                    rect(dx, ground_y - mon_h * 0.12, mon_w * 0.2, mon_h * 0.06, gpui::rgba(0xd97706ff), window);
                    line(dx + mon_w * 0.04, ground_y - mon_h * 0.06, dx + mon_w * 0.04, ground_y, 2.0, gpui::rgba(0x92400eff), window);
                    line(dx + mon_w * 0.14, ground_y - mon_h * 0.06, dx + mon_w * 0.14, ground_y, 2.0, gpui::rgba(0x92400eff), window);
                }
                // Windows
                for i in [0.05_f32, 0.85] {
                    let wx2 = mon_x + mon_w * i;
                    rect(wx2, mon_y + mon_h * 0.12, mon_w * 0.08, mon_h * 0.25, gpui::rgba(0x93c5fdff), window);
                }
            }

            "space" => {
                // Deep space gradient
                rect(mon_x, mon_y, mon_w, mon_h, gpui::rgba(0x020617ff), window);
                // Stars
                let star_positions = [(0.1_f32, 0.1_f32), (0.85, 0.15), (0.3, 0.05), (0.6, 0.2),
                    (0.15, 0.4), (0.75, 0.35), (0.5, 0.08), (0.9, 0.5), (0.4, 0.55),
                    (0.2, 0.65), (0.7, 0.6), (0.45, 0.3), (0.8, 0.08), (0.05, 0.75)];
                for &(sx, sy) in &star_positions {
                    circle(mon_x + mon_w * sx, mon_y + mon_h * sy, mon_h * 0.004, gpui::rgba(0xffffffff), window);
                }
                // Planet (large, Saturn-like)
                circle(mon_x + mon_w * 0.15, mon_y + mon_h * 0.25, mon_h * 0.12, gpui::rgba(0xc4a35aff), window);
                line(mon_x + mon_w * 0.04, mon_y + mon_h * 0.27, mon_x + mon_w * 0.26, mon_y + mon_h * 0.24,
                     5.0, gpui::rgba(0xa07b3baa), window);
                // Small moon
                circle(mon_x + mon_w * 0.78, mon_y + mon_h * 0.18, mon_h * 0.055, gpui::rgba(0x6b7280ff), window);
                // Nebula (colored glow blobs)
                circle(mon_cx + mon_w * 0.2, mon_y + mon_h * 0.4, mon_h * 0.15, gpui::rgba(0x6366f133), window);
                circle(mon_cx + mon_w * 0.25, mon_y + mon_h * 0.35, mon_h * 0.10, gpui::rgba(0xec489955), window);
                // Asteroid ground
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0x374151ff), window);
                circle(mon_cx - mon_w * 0.2, ground_y + mon_h * 0.05, mon_h * 0.06, gpui::rgba(0x4b5563ff), window);
                circle(mon_cx + mon_w * 0.3, ground_y + mon_h * 0.04, mon_h * 0.04, gpui::rgba(0x4b5563ff), window);
            }

            "desert" => {
                // Sky (warm orange/gold)
                rect(mon_x, mon_y, mon_w, mon_h * 0.55, gpui::rgba(0xfde68aff), window);
                // Sun
                circle(mon_x + mon_w * 0.82, mon_y + mon_h * 0.12, mon_h * 0.08, gpui::rgba(0xf59e0bff), window);
                // Distant dune horizon
                let dune_pts = [
                    (mon_x, ground_y - mon_h * 0.06),
                    (mon_cx - mon_w * 0.25, ground_y - mon_h * 0.20),
                    (mon_cx, ground_y - mon_h * 0.10),
                    (mon_cx + mon_w * 0.3, ground_y - mon_h * 0.25),
                    (mon_x + mon_w, ground_y - mon_h * 0.08),
                    (mon_x + mon_w, ground_y),
                    (mon_x, ground_y),
                ];
                poly(&dune_pts, gpui::rgba(0xfbbf24ff), window);
                // Sand ground
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0xf59e0bff), window);
                // Cacti
                for i in [0.12_f32, 0.68] {
                    let cx2 = mon_x + mon_w * i;
                    let ch = mon_h * 0.25;
                    rect(cx2, ground_y - ch, mon_w * 0.025, ch, gpui::rgba(0x15803dff), window);
                    rect(cx2 - mon_w * 0.04, ground_y - ch * 0.65, mon_w * 0.04, mon_h * 0.06, gpui::rgba(0x15803dff), window);
                    rect(cx2 + mon_w * 0.025, ground_y - ch * 0.5, mon_w * 0.04, mon_h * 0.06, gpui::rgba(0x15803dff), window);
                }
            }

            "ocean" => {
                // Sky
                rect(mon_x, mon_y, mon_w, ground_y - mon_y, gpui::rgba(0x38bdf8ff), window);
                // Ocean
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0x0369a1ff), window);
                // Ocean lighter top strip
                rect(mon_x, ground_y, mon_w, mon_h * 0.06, gpui::rgba(0x0ea5e9ff), window);
                // Waves
                for i in 0..5_i32 {
                    let wx2 = mon_x + mon_w * (i as f32 * 0.22 - 0.05);
                    line(wx2, ground_y + mon_h * 0.04, wx2 + mon_w * 0.12, ground_y + mon_h * 0.02,
                         2.0, gpui::rgba(0x7dd3fcff), window);
                }
                // Beach/sand strip
                rect(mon_x, ground_y - mon_h * 0.05, mon_w, mon_h * 0.05, gpui::rgba(0xfde68aff), window);
                // Palm tree (left)
                let ptx = mon_x + mon_w * 0.08;
                line(ptx, ground_y - mon_h * 0.01, ptx + mon_w * 0.04, ground_y - mon_h * 0.35,
                     6.0, gpui::rgba(0x92400eff), window);
                circle(ptx + mon_w * 0.04, ground_y - mon_h * 0.35, mon_h * 0.08, gpui::rgba(0x15803dff), window);
                // Sun/sky
                circle(mon_x + mon_w * 0.75, mon_y + mon_h * 0.12, mon_h * 0.07, gpui::rgba(0xfef08aff), window);
            }

            "arctic" => {
                // Aurora sky
                rect(mon_x, mon_y, mon_w, mon_h * 0.6, gpui::rgba(0x0f172aff), window);
                // Aurora bands
                poly(&[
                    (mon_x, mon_y + mon_h * 0.15),
                    (mon_x + mon_w, mon_y + mon_h * 0.25),
                    (mon_x + mon_w, mon_y + mon_h * 0.35),
                    (mon_x, mon_y + mon_h * 0.28),
                ], gpui::rgba(0x34d39955), window);
                poly(&[
                    (mon_x + mon_w * 0.2, mon_y + mon_h * 0.1),
                    (mon_x + mon_w, mon_y + mon_h * 0.18),
                    (mon_x + mon_w, mon_y + mon_h * 0.26),
                    (mon_x + mon_w * 0.2, mon_y + mon_h * 0.2),
                ], gpui::rgba(0x818cf866), window);
                // Stars
                for &(sx, sy) in &[(0.1_f32, 0.05_f32), (0.4, 0.08), (0.7, 0.04), (0.9, 0.12)] {
                    circle(mon_x + mon_w * sx, mon_y + mon_h * sy, mon_h * 0.004, gpui::rgba(0xffffffff), window);
                }
                // Snow ground
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0xf0f9ffff), window);
                // Snow hills
                let hill1 = [(mon_x, ground_y), (mon_cx - mon_w * 0.15, ground_y - mon_h * 0.18),
                    (mon_cx + mon_w * 0.1, ground_y - mon_h * 0.08), (mon_x + mon_w, ground_y)];
                poly(&hill1, gpui::rgba(0xdbeafeff), window);
                // Iceberg
                let ice = [(mon_x + mon_w * 0.6, ground_y - mon_h * 0.15),
                    (mon_x + mon_w * 0.75, ground_y - mon_h * 0.22),
                    (mon_x + mon_w * 0.85, ground_y - mon_h * 0.1),
                    (mon_x + mon_w * 0.8, ground_y)];
                poly(&ice, gpui::rgba(0xbae6fdff), window);
            }

            "volcano" => {
                // Dark sky
                rect(mon_x, mon_y, mon_w, mon_h, gpui::rgba(0x1c0a00ff), window);
                // Lava glow at horizon
                rect(mon_x, ground_y - mon_h * 0.08, mon_w, mon_h * 0.08, gpui::rgba(0xea580c44), window);
                // Volcano silhouette
                let volc = [(mon_cx - mon_w * 0.35, ground_y),
                    (mon_cx - mon_w * 0.05, ground_y - mon_h * 0.55),
                    (mon_cx + mon_w * 0.05, ground_y - mon_h * 0.55),
                    (mon_cx + mon_w * 0.35, ground_y)];
                poly(&volc, gpui::rgba(0x292524ff), window);
                // Lava at top
                circle(mon_cx, ground_y - mon_h * 0.55, mon_h * 0.05, gpui::rgba(0xf97316ff), window);
                // Lava streams
                for i in [-0.03_f32, 0.0, 0.03] {
                    line(mon_cx + i * mon_w, ground_y - mon_h * 0.55,
                         mon_cx + i * mon_w * 3.0, ground_y - mon_h * 0.35,
                         5.0, gpui::rgba(0xef4444cc), window);
                }
                // Ground
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0x1c0a00ff), window);
            }

            _ => {
                // Generic studio / fallback: clean gradient background
                rect(mon_x, mon_y, mon_w, ground_y - mon_y, gpui::rgba(0x1e293bff), window);
                rect(mon_x, ground_y, mon_w, mon_h * 0.4, gpui::rgba(0x0f172aff), window);
                // Studio floor reflection
                line(mon_x, ground_y, mon_x + mon_w, ground_y, 2.0, gpui::rgba(0x38bdf8ff), window);
            }
        } } // end if bg_image.is_none() + match

        // ── 2b. Ground shadow band (always shown, blends with image bg too) ──
        // Semi-transparent dark strip at the bottom ground line
        rect(mon_x, ground_y, mon_w, mon_h - (ground_y - mon_y), gpui::rgba(0x00000066), window);
        // Thin bright ground line for visual clarity
        line(mon_x, ground_y, mon_x + mon_w, ground_y, 1.5, gpui::rgba(0xffffff44), window);

        // ── 3. Data-Driven Character Rendering ─────────────────────────────────
        // Uses CharacterDef from animation::character module.
        // Each part is attached to a bone and drawn in z-order.

        let char_type = data.character_type.as_str();

        // ── Helper: bone lookup by label ─────────────────────────────────────
        let find_bone = |label: &str| -> Option<&animation::BoneTransform> {
            data.bones.iter().find(|b| b.label == label)
        };

        // Screen-space accessors for bone endpoints
        let bx1 = |b: &animation::BoneTransform| mx(b.x1);
        let by1 = |b: &animation::BoneTransform| my(b.y1);
        let blen = |b: &animation::BoneTransform| b.length as f32 * scale;
        let bangle = |b: &animation::BoneTransform| -b.angle as f32;

        // Rotate local-space (lx,ly) around (0,0) → translate to (ox,oy)
        let rpt = |ox: f32, oy: f32, lx: f32, ly: f32, angle: f32| -> (f32, f32) {
            let c = angle.cos();
            let s = angle.sin();
            (ox + lx * c - ly * s, oy + lx * s + ly * c)
        };

        // ── Color palette resolution ─────────────────────────────────────────
        let palette = match char_type {
            "casual" => (
                gpui::rgba(0xfde8d0ff), // skin
                gpui::rgba(0xd4a07aff), // skin shadow
                gpui::rgba(0x78350fff), // hair
                gpui::rgba(0x3b82f6ff), // cloth main (blue T-shirt)
                gpui::rgba(0x1e3a8aff), // cloth dark (denim shorts)
                gpui::rgba(0xfef08aff), // cloth accent (yellow trim)
                gpui::rgba(0xef4444ff), // shoe (red sneaker)
                gpui::rgba(0xf8f8f8ff), // shoe sole (white)
                gpui::rgba(0x1a1a2eff), // outline
                gpui::rgba(0xffffffff), // white
            ),
            "robot" => (
                gpui::rgba(0x94a3b8ff), // skin (silver chassis)
                gpui::rgba(0x64748bff), // skin shadow
                gpui::rgba(0x0f172aff), // hair (dark panel)
                gpui::rgba(0x334155ff), // cloth main (steel grey)
                gpui::rgba(0x1e293bff), // cloth dark
                gpui::rgba(0x38bdf8ff), // cloth accent (neon blue)
                gpui::rgba(0x0f172aff), // shoe
                gpui::rgba(0x1e293bff), // shoe sole
                gpui::rgba(0x0f172aff), // outline
                gpui::rgba(0xffffffff), // white
            ),
            _ => {
                let k = gpui::rgba(0x1a1a2eff);
                (k, k, k, k, k, k, k, k, k, gpui::rgba(0xffffffff))
            }
        };

        let resolve_color = |role: &ColorRole| -> gpui::Rgba {
            match role {
                ColorRole::Skin => palette.0,
                ColorRole::SkinShadow => palette.1,
                ColorRole::Hair => palette.2,
                ColorRole::ClothMain => palette.3,
                ColorRole::ClothDark => palette.4,
                ColorRole::ClothAccent => palette.5,
                ColorRole::Shoe => palette.6,
                ColorRole::ShoeSole => palette.7,
                ColorRole::Outline => palette.8,
                ColorRole::White => palette.9,
                ColorRole::LightGray => gpui::rgba(0xd0d0d0ff),
                ColorRole::Custom(hex) => gpui::rgba(*hex),
            }
        };

        // ── 3a. Multi-Character & Virtual Camera Cinematic Studio Rendering ────
        if let Some(registry) = character_registry {
            if let Some(movie) = cinematic_movie {
                let act = movie.acts.iter()
                    .find(|a| current_time >= a.start_time && current_time < a.start_time + a.duration)
                    .or_else(|| movie.acts.first());

                if let Some(act) = act {
                    // Use dynamic camera from SmartCameraDirector if available, else fall back to act camera
                    let cam = dynamic_camera.unwrap_or(&act.camera);

                    // Smooth interpolation between dynamic camera shots when they change
                    let (use_zoom, use_pan_x, use_pan_y, use_shake) = if dynamic_camera.is_some() && prev_dynamic_camera.is_some() && camera_dynamic_transition_t < 1.0 {
                        let t = smoothstep(camera_dynamic_transition_t as f32);
                        let prev = prev_dynamic_camera.unwrap();
                        (
                            lerp(prev.zoom, cam.zoom, t).clamp(0.5, 3.0),
                            lerp(prev.pan_x, cam.pan_x, t),
                            lerp(prev.pan_y, cam.pan_y, t),
                            lerp(prev.shake, cam.shake, t),
                        )
                    } else if camera_transition_t < 1.0 {
                        if let Some(prev) = prev_camera {
                            let t = smoothstep(camera_transition_t as f32);
                            (
                                lerp(prev.zoom, cam.zoom, t).clamp(0.5, 3.0),
                                lerp(prev.pan_x, cam.pan_x, t),
                                lerp(prev.pan_y, cam.pan_y, t),
                                lerp(prev.shake, cam.shake, t),
                            )
                        } else {
                            (cam.zoom.clamp(0.5, 3.0), cam.pan_x, cam.pan_y, cam.shake)
                        }
                    } else {
                        (cam.zoom.clamp(0.5, 3.0), cam.pan_x, cam.pan_y, cam.shake)
                    };

                    let zoom = use_zoom;
                    // Shake decays exponentially from nearest impact/epic beat time;
                    // only active within 0.4s of a beat — no continuous vibration.
                    let shake_envelope = if use_shake > 0.0 {
                        beats.iter()
                            .filter(|b| b.is_impact || b.is_epic)
                            .map(|b| (current_time - b.time).abs())
                            .filter(|&dt| dt < 0.4)
                            .map(|dt| (-dt * 10.0).exp())
                            .fold(0.0_f64, f64::max)
                    } else {
                        0.0
                    };
                    let shake_amp = use_shake.clamp(0.0, 1.5) * shake_envelope as f32;
                    let shake_x = if shake_amp > 0.0 { (current_time * 35.0).sin() as f32 * shake_amp * 8.0 } else { 0.0 };
                    let shake_y = if shake_amp > 0.0 { (current_time * 28.0).cos() as f32 * shake_amp * 5.0 } else { 0.0 };

                    let entity_pos_data = scene_executor.and_then(|mtx| mtx.lock().ok());
                    let entity_pos = |e: &StageEntity| -> (f32, f32) {
                        // Trust the executor's interpolated position entirely.
                        // If the executor hasn't initialized this entity yet (e.g. first
                        // frame of a new act), fall back to the entity's start position
                        // rather than doing our own interpolation (which would conflict
                        // with the executor's ease_in_out_cubic and cause visual jumps).
                        if let Some(ref exec) = entity_pos_data.as_ref() {
                            if let Some(pose) = exec.get_entity_pose(&e.id) {
                                return (pose.pos_x as f32, pose.pos_y as f32);
                            }
                        }
                        // Also check if there's an end_x — if the act is near the end of
                        // its duration, show the end position (avoids entities stuck at start).
                        let act_elapsed = (current_time - act.start_time).max(0.0);
                        let p = if act.duration > 0.0 { (act_elapsed / act.duration).min(1.0) as f32 } else { 1.0 };
                        // If executor not ready, use simple start position (not eased)
                        // so we don't conflict with executor's own interpolation.
                        let _ = p; // suppress unused warning
                        (e.pos_x, e.pos_y)
                    };

                    let target_pos_x = if let Some(ref tid) = cam.target_entity_id {
                        if let Some(e) = act.entities.iter().find(|e| &e.id == tid) {
                            entity_pos(e).0
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                    let cam_center_x = match cam.shot_type {
                        animation::ShotType::ExtremeWide
                        | animation::ShotType::Wide
                        | animation::ShotType::Establishing
                        | animation::ShotType::FullShot
                        | animation::ShotType::GroupShot => {
                            if !act.entities.is_empty() {
                                // Use midpoint of min/max entity X — centers frame on action span
                                let xs: Vec<f32> = act.entities.iter()
                                    .map(|e| entity_pos(e).0)
                                    .collect();
                                let min_x = xs.iter().cloned().fold(f32::INFINITY, f32::min);
                                let max_x = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                                let mid_x = (min_x + max_x) * 0.5;
                                mid_x + cam.pan_x
                            } else {
                                target_pos_x + cam.pan_x
                            }
                        }
                        animation::ShotType::ActionFollow => {
                            let offset = cam.shot_type.framing_offset_x();
                            target_pos_x + cam.pan_x + offset / zoom.clamp(0.5, 2.0)
                        }
                        _ => {
                            let offset = cam.shot_type.framing_offset_x();
                            target_pos_x + cam.pan_x + offset / zoom.clamp(0.5, 2.0)
                        }
                    };

                    // DutchAngle: tilt scene around screen center for disorienting effect
                    let tilt_angle = cam.tilt_angle;

                    // ── Build the camera projection ────────────────────────────
                    let view_scale = mon_w * 0.32; // tune: pixels per world unit at Z=1
                    let base_char_h = mon_h * 0.45; // base character sprite height (at Z=1, zoom=1)
                    let projector = CameraProjection {
                        mon_cx: mon_cx as f32,
                        ground_y: ground_y as f32,
                        view_scale,
                        zoom: zoom as f32,
                        cam_center_x: cam_center_x as f32,
                        cam_center_y: 0.0,
                        shake_x,
                        shake_y,
                    };

                    let mut sorted_entities = act.entities.clone();
                    sorted_entities.sort_by(|a, b| b.pos_z.partial_cmp(&a.pos_z).unwrap_or(std::cmp::Ordering::Equal));

                    // Depth of field: compute focus distance from camera target
                    let dof = cam.depth_of_field;
                    let focus_z = if let Some(ref tid) = cam.target_entity_id {
                        act.entities.iter().find(|e| &e.id == tid).map(|e| e.pos_z as f32).unwrap_or(1.0)
                    } else { 1.0 };

                    for entity in &sorted_entities {
                        let skin_key = match entity.character_skin_id.as_str() {
                            "police" | "police_1" => "police_1",
                            "police_2" => "police_2",
                            "police_3" => "police_3",
                            "terrorist" | "terrorist_1" => "terrorist_1",
                            "terrorist_2" => "terrorist_2",
                            "terrorist_3" => "terrorist_3",
                            "chibi" | "chibi_summer" => "chibi_summer",
                            "chibi_autumn" => "chibi_autumn",
                            "chibi_winter" => "chibi_winter",
                            other => other,
                        };

                        if let Some(skin) = registry.skins.get(skin_key).or_else(|| registry.skins.get("police_1")) {
                            let (entity_cur_x, entity_cur_y) = entity_pos(entity);

                            // Frustum cull: skip far-off entities (margin 4.0 = generous, no false culls)
                            if !projector.is_visible(entity_cur_x, entity.pos_z as f32, 4.0) {
                                continue;
                            }

                            // Project world → screen base
                            let proj = projector.project(
                                entity_cur_x, entity_cur_y,
                                entity.pos_z as f32, base_char_h,
                            );

                            // SCML bone rendering only: skip entities without SCML data.
                            // (No stickman fallback — user requires SCML-only rendering.)
                            if let Some(ref scml_data) = skin.scml_data {
                                let scml_entity_name = skin.scml_entity_name.as_deref().unwrap_or("");
                                let anim_name = skin.action_to_scml_anim(&entity.action);
                                render_scml_character(
                                    scml_data, scml_entity_name, anim_name,
                                    current_time, skin,
                                    &proj, &projector, mon_cx as f32, ground_y as f32,
                                    tilt_angle, entity.facing_left, window,
                                );
                                // Depth of field overlay
                                if dof > 0.0 {
                                    let z_dist = (entity.pos_z as f32 - focus_z).abs();
                                    let blur_strength = (z_dist * dof * 2.0).min(1.0).max(0.0);
                                    if blur_strength > 0.1 {
                                        let alpha = (blur_strength * 120.0) as u8;
                                        let overlay = gpui::rgba((alpha as u32) << 24 | 0x000000);
                                        let desired_h = proj.sprite_h;
                                        let desired_w = proj.sprite_h * 0.6;
                                        let (cx, cy) = dutch_rotate(proj.screen_x, proj.screen_y, mon_cx as f32, ground_y as f32, tilt_angle);
                                        rect(cx - desired_w / 2.0, cy - desired_h, desired_w, desired_h, overlay, window);
                                    }
                                }
                            }
                        }
                    }

                    // ── 3b. Vignette (cinematic edge darkening) ─────────────
                    // Intensifies with shot intensity — creates focus on center
                    {
                        let vignette_strength = (cam.shake * 2.0 + act.intensity as f32 * 0.5).min(0.7);
                        if vignette_strength > 0.1 {
                            let edge_w = mon_w * 0.12;
                            let alpha = (vignette_strength * 200.0) as u8;
                            let vcol = gpui::rgba((alpha as u32) << 24);
                            // Top edge
                            rect(mon_x, mon_y, mon_w, edge_w, vcol, window);
                            // Bottom edge
                            rect(mon_x, mon_y + mon_h - edge_w, mon_w, edge_w, vcol, window);
                            // Left edge
                            rect(mon_x, mon_y, edge_w, mon_h, vcol, window);
                            // Right edge
                            rect(mon_x + mon_w - edge_w, mon_y, edge_w, mon_h, vcol, window);
                        }
                    }

                    // ── 4. Speech Bubbles ────────────────────────────────────
                    for d in &act.dialogues {
                        let rel_time = current_time - act.start_time;
                        if rel_time >= d.start_time && rel_time <= d.start_time + d.duration {
                            if let Some(entity) = act.entities.iter().find(|e| e.id == d.entity_id) {
                                let (ex, ey) = entity_pos(entity);
                                // Same projection as entity rendering
                                let proj = projector.project(ex, ey, entity.pos_z as f32, base_char_h);
                                let (bubble_cx, bubble_cy) = dutch_rotate(proj.screen_x, proj.screen_y - proj.sprite_h - mon_h * 0.04, mon_cx as f32, ground_y as f32, tilt_angle);

                                // Bubble width berdasarkan teks
                                let text_len = d.text.len().max(4) as f32;
                                let bubble_w = (text_len * 8.0).min(mon_w * 0.35).max(mon_w * 0.08);
                                let bubble_h = mon_h * 0.065;
                                let bubble_x = bubble_cx - bubble_w / 2.0;
                                let bubble_y = bubble_cy - bubble_h;

                                // Bubble background color berdasarkan emotion
                                let (bg_col, text_col) = match d.emotion.as_str() {
                                    "shout" => (gpui::rgba(0xef4444ff), gpui::rgba(0xffffffff)),
                                    "whisper" => (gpui::rgba(0x6b728088), gpui::rgba(0xe2e8f0ff)),
                                    _ => (gpui::rgba(0xffffffff), gpui::rgba(0x1a1a2eff)),
                                };
                                // Bubble border
                                let border_col = gpui::rgba(0x1a1a2e44);

                                // Rounded rectangle bubble
                                let corner_r = mon_h * 0.015;
                                let segs = 12;
                                // Top edge
                                let mut bp = PathBuilder::fill();
                                bp.move_to(point(px(bubble_x + corner_r), px(bubble_y)));
                                // Top-right corner
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp.line_to(point(
                                        px(bubble_x + bubble_w - corner_r + corner_r * a.cos()),
                                        px(bubble_y + corner_r - corner_r * a.sin()),
                                    ));
                                }
                                // Bottom-right corner
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp.line_to(point(
                                        px(bubble_x + bubble_w - corner_r + corner_r * (std::f32::consts::FRAC_PI_2 + a).cos()),
                                        px(bubble_y + bubble_h - corner_r + corner_r * (std::f32::consts::FRAC_PI_2 + a).sin()),
                                    ));
                                }
                                // Bottom-left corner
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp.line_to(point(
                                        px(bubble_x + corner_r + corner_r * (std::f32::consts::PI + a).cos()),
                                        px(bubble_y + bubble_h - corner_r + corner_r * (std::f32::consts::PI + a).sin()),
                                    ));
                                }
                                // Top-left corner
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp.line_to(point(
                                        px(bubble_x + corner_r + corner_r * (3.0 * std::f32::consts::FRAC_PI_2 + a).cos()),
                                        px(bubble_y + corner_r + corner_r * (3.0 * std::f32::consts::FRAC_PI_2 + a).sin()),
                                    ));
                                }
                                if let Ok(p) = bp.build() {
                                    window.paint_path(p, bg_col);
                                }

                                // Bubble border outline
                                let mut bp2 = PathBuilder::stroke(px(1.5));
                                bp2.move_to(point(px(bubble_x + corner_r), px(bubble_y)));
                                // Top-right
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp2.line_to(point(
                                        px(bubble_x + bubble_w - corner_r + corner_r * a.cos()),
                                        px(bubble_y + corner_r - corner_r * a.sin()),
                                    ));
                                }
                                // Bottom-right
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp2.line_to(point(
                                        px(bubble_x + bubble_w - corner_r + corner_r * (std::f32::consts::FRAC_PI_2 + a).cos()),
                                        px(bubble_y + bubble_h - corner_r + corner_r * (std::f32::consts::FRAC_PI_2 + a).sin()),
                                    ));
                                }
                                // Bottom-left
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp2.line_to(point(
                                        px(bubble_x + corner_r + corner_r * (std::f32::consts::PI + a).cos()),
                                        px(bubble_y + bubble_h - corner_r + corner_r * (std::f32::consts::PI + a).sin()),
                                    ));
                                }
                                // Top-left
                                for i in 0..=segs {
                                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / segs as f32;
                                    bp2.line_to(point(
                                        px(bubble_x + corner_r + corner_r * (3.0 * std::f32::consts::FRAC_PI_2 + a).cos()),
                                        px(bubble_y + corner_r + corner_r * (3.0 * std::f32::consts::FRAC_PI_2 + a).sin()),
                                    ));
                                }
                                if let Ok(p) = bp2.build() {
                                    window.paint_path(p, border_col);
                                }

                                // Bubble tail (triangle pointing down to character)
                                let tail_w = mon_h * 0.02;
                                let tail_h = mon_h * 0.025;
                                let mut tp = PathBuilder::fill();
                                tp.move_to(point(px(bubble_cx - tail_w), px(bubble_y + bubble_h)));
                                tp.line_to(point(px(bubble_cx + tail_w), px(bubble_y + bubble_h)));
                                tp.line_to(point(px(bubble_cx), px(bubble_y + bubble_h + tail_h)));
                                if let Ok(p) = tp.build() {
                                    window.paint_path(p, bg_col);
                                }

                                // ── Render text ──────────────────────────────
                                // GPUI tidak punya text rendering langsung di canvas.
                                // Untuk sementara, gunakan pendekatan simplified:
                                // Gunakan font rendering via div overlay sebagai fallback.
                                // TAPI karena kita di canvas, kita tidak bisa pakai div.
                                // Solusi: gambarkan text sebagai approximate dots.
                                // Untuk full text rendering, nanti integrasikan text shader.
                                // Sementara ini, text hanya tampil sebagai placeholder dot.

                                // Fallback: gambar title bar kecil dengan color-coded indicator
                                let indicator_w = bubble_w * 0.6;
                                let indicator_h = mon_h * 0.02;
                                let ind_x = bubble_cx - indicator_w / 2.0;
                                let ind_y = bubble_y + bubble_h * 0.25;
                                let ind_col = match d.emotion.as_str() {
                                    "shout" => gpui::rgba(0xffffffcc),
                                    "whisper" => gpui::rgba(0xe2e8f088),
                                    _ => gpui::rgba(0x1a1a2e88),
                                };
                                rect(ind_x, ind_y, indicator_w, indicator_h, ind_col, window);
                            }
                        }
                    }
                    return;
                }
            }

            // modular bone animation — SCML — handled in cinematic block.
            // legacy PNG spritesheet frame rendering removed.
        }

        // Spine skeleton rendering removed — using SCML modular bone animation.



        if char_type == "stickman" {
            // ── Simple stickman line drawing ──────────────────────────────
            let outline_col = gpui::rgba(0x1a1a2eff);
            for seg in &data.segments {
                let mut p = PathBuilder::stroke(px(3.0));
                p.move_to(point(px(mx(seg.x1)), px(my(seg.y1))));
                p.line_to(point(px(mx(seg.x2)), px(my(seg.y2))));
                if let Ok(p) = p.build() { window.paint_path(p, outline_col); }
            }
            // Head circle
            let hx = mx(data.head_cx);
            let hy = my(data.head_cy);
            let hr = data.head_r as f32 * scale;
            let n = 24_u32;
            let mut path = PathBuilder::stroke(px(3.0));
            path.move_to(point(px(hx + hr), px(hy)));
            for i in 1..=n {
                let a = std::f32::consts::TAU * i as f32 / n as f32;
                path.line_to(point(px(hx + hr * a.cos()), px(hy + hr * a.sin())));
            }
            if let Ok(p) = path.build() { window.paint_path(p, outline_col); }
        } else {
            // ══════════════════════════════════════════════════════════════
            // DATA-DRIVEN CHARACTER RENDERING
            // ══════════════════════════════════════════════════════════════
            // Select character view based on facing_angle:
            // facing_angle=0 = front, PI=back, +PI/2=right side, -PI/2=left side
            let view_suffix = {
                let fa = data.facing_angle;
                // 4 sectors, each 90°
                if fa.abs() < std::f64::consts::FRAC_PI_4 {
                    ""  // front
                } else if (fa - std::f64::consts::PI).abs() < std::f64::consts::FRAC_PI_4
                       || (fa + std::f64::consts::PI).abs() < std::f64::consts::FRAC_PI_4 {
                    "_back"  // back
                } else if fa > 0.0 {
                    "_right"  // right side
                } else {
                    "_left"   // left side
                }
            };
            let lookup_type = format!("{}{}", char_type, view_suffix);
            let char_def = get_character(&lookup_type);
            let mut parts = char_def.parts.clone();

            // Modulate z-order based on facing_angle for 2D parallel rotation.
            // Left-side parts (positive z) swap depth order with right-side parts (negative z)
            // as the character turns.
            let facing_angle = data.facing_angle;
            let left_pref = facing_angle.cos();
            let right_pref = -facing_angle.cos();
            parts.sort_by(|a, b| {
                let az = a.z_order as f64;
                let bz = b.z_order as f64;
                let za = if az > 0.0 && az < 40.0 { az + left_pref * 50.0 } else if az < 0.0 { az + right_pref * 50.0 } else { az };
                let zb = if bz > 0.0 && bz < 40.0 { bz + left_pref * 50.0 } else if bz < 0.0 { bz + right_pref * 50.0 } else { bz };
                za.partial_cmp(&zb).unwrap_or(std::cmp::Ordering::Equal)
            });

            for part in &parts {
                let bone = match find_bone(&part.bone) {
                    Some(b) => b,
                    None => continue,
                };

                // Character rendering: no debug output
                let ox = bx1(bone);
                let oy = by1(bone);
                let len = blen(bone);
                let ang = bangle(bone);
                let col = resolve_color(&part.color);

                match &part.shape {
                    ShapeKind::TaperedLimb { start, end, w1, w2, cap_start, cap_end } => {
                        // Draw a smooth tapered limb shape using multi-point polygon
                        // for organic curves (not just a rectangle).
                        let s0 = *start * len;
                        let s1 = *end * len;
                        let hw0 = *w1 * len;
                        let hw1 = *w2 * len;

                        // Generate smooth contour with 8 segments per side
                        let segments = 8;
                        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(segments * 2 + 2);

                        // Left edge (proximal → distal)
                        for i in 0..=segments {
                            let t = i as f32 / segments as f32;
                            let lx = s0 + (s1 - s0) * t;
                            // Very subtle muscle curve — 4% belly bulge only
                            let belly = (t * std::f32::consts::PI).sin() * 0.04;
                            let hw = hw0 + (hw1 - hw0) * t + belly * hw0;
                            let ly = -hw;
                            pts.push(rpt(ox, oy, lx, ly, ang));
                        }
                        // Right edge (distal → proximal)
                        for i in (0..=segments).rev() {
                            let t = i as f32 / segments as f32;
                            let lx = s0 + (s1 - s0) * t;
                            let belly = (t * std::f32::consts::PI).sin() * 0.04;
                            let hw = hw0 + (hw1 - hw0) * t + belly * hw0;
                            let ly = hw;
                            pts.push(rpt(ox, oy, lx, ly, ang));
                        }

                        // Draw filled shape
                        if pts.len() >= 3 {
                            let mut p = PathBuilder::fill();
                            p.move_to(point(px(pts[0].0), px(pts[0].1)));
                            for &(x, y) in &pts[1..] { p.line_to(point(px(x), px(y))); }
                            p.line_to(point(px(pts[0].0), px(pts[0].1)));
                            if let Ok(p) = p.build() { window.paint_path(p, col); }
                        }

                        // End cap circles for smooth joint transitions
                        if *cap_start {
                            let (cx, cy) = rpt(ox, oy, s0, 0.0, ang);
                            circle(cx, cy, hw0, col, window);
                        }
                        if *cap_end {
                            let (cx, cy) = rpt(ox, oy, s1, 0.0, ang);
                            circle(cx, cy, hw1, col, window);
                        }

                        // Optional outline
                        if let Some(ref outline_role) = part.outline {
                            let ocol = resolve_color(outline_role);
                            let thick = (len * 0.025).max(0.8).min(1.8);
                            if pts.len() >= 3 {
                                let mut p = PathBuilder::stroke(px(thick));
                                p.move_to(point(px(pts[0].0), px(pts[0].1)));
                                for &(x, y) in &pts[1..] { p.line_to(point(px(x), px(y))); }
                                p.line_to(point(px(pts[0].0), px(pts[0].1)));
                                if let Ok(p) = p.build() { window.paint_path(p, ocol); }
                            }
                        }
                    }

                    ShapeKind::Ellipse { rx, ry } => {
                        // Convert bone-local offset to screen coordinates
                        let (cx, cy) = rpt(ox, oy, part.offset_x * len, part.offset_y * len, ang);
                        let srx = *rx * len;
                        let sry = *ry * len;
                        // Draw ellipse as polygon
                        let n = 20_u32;
                        let mut p = PathBuilder::fill();
                        let rc = ang.cos();
                        let rs = ang.sin();
                        p.move_to(point(px(cx + srx * rc), px(cy + srx * rs)));
                        for i in 1..=n {
                            let a = std::f32::consts::TAU * i as f32 / n as f32;
                            // Apply rotation to make ellipse follow bone angle
                            let ex = srx * a.cos();
                            let ey = sry * a.sin();
                            let rc = ang.cos();
                            let rs = ang.sin();
                            let rx2 = ex * rc - ey * rs;
                            let ry2 = ex * rs + ey * rc;
                            p.line_to(point(px(cx + rx2), px(cy + ry2)));
                        }
                        if let Ok(p) = p.build() { window.paint_path(p, col); }

                        // Draw outline if present
                        if let Some(ref outline_role) = part.outline {
                            let ocol = resolve_color(outline_role);
                            let thick = (len * 0.025).max(1.0).min(2.2);
                            let mut p = PathBuilder::stroke(px(thick));
                            let rc = ang.cos();
                            let rs = ang.sin();
                            p.move_to(point(px(cx + srx * rc), px(cy + srx * rs)));
                            for i in 1..=n {
                                let a = std::f32::consts::TAU * i as f32 / n as f32;
                                let ex = srx * a.cos();
                                let ey = sry * a.sin();
                                let rc = ang.cos();
                                let rs = ang.sin();
                                let rx2 = ex * rc - ey * rs;
                                let ry2 = ex * rs + ey * rc;
                                p.line_to(point(px(cx + rx2), px(cy + ry2)));
                            }
                            if let Ok(p) = p.build() { window.paint_path(p, ocol); }
                        }
                    }

                    ShapeKind::Circle { r } => {
                        let (cx, cy) = rpt(ox, oy, part.offset_x * len, part.offset_y * len, ang);
                        let sr = *r * len;
                        circle(cx, cy, sr, col, window);

                        // Draw outline if present
                        if let Some(ref outline_role) = part.outline {
                            let ocol = resolve_color(outline_role);
                            let thick = (len * 0.025).max(1.0).min(2.2);
                            let n = 20_u32;
                            let mut p = PathBuilder::stroke(px(thick));
                            p.move_to(point(px(cx + sr), px(cy)));
                            for i in 1..=n {
                                let a = std::f32::consts::TAU * i as f32 / n as f32;
                                p.line_to(point(px(cx + sr * a.cos()), px(cy + sr * a.sin())));
                            }
                            if let Ok(p) = p.build() { window.paint_path(p, ocol); }
                        }
                    }

                    ShapeKind::Polygon { points } => {
                        if points.len() < 3 { continue; }
                        // Transform from bone-local fraction coords to screen
                        let screen_pts: Vec<(f32, f32)> = points.iter()
                            .map(|&(lx, ly)| rpt(ox, oy, lx * len, ly * len, ang))
                            .collect();

                        let mut p = PathBuilder::fill();
                        p.move_to(point(px(screen_pts[0].0), px(screen_pts[0].1)));
                        for &(x, y) in &screen_pts[1..] { p.line_to(point(px(x), px(y))); }
                        p.line_to(point(px(screen_pts[0].0), px(screen_pts[0].1)));
                        if let Ok(p) = p.build() { window.paint_path(p, col); }

                        // Optional outline
                        if let Some(ref outline_role) = part.outline {
                            let ocol = resolve_color(outline_role);
                            let thick = (len * 0.04).max(1.0).min(2.5);
                            let mut p = PathBuilder::stroke(px(thick));
                            p.move_to(point(px(screen_pts[0].0), px(screen_pts[0].1)));
                            for &(x, y) in &screen_pts[1..] { p.line_to(point(px(x), px(y))); }
                            p.line_to(point(px(screen_pts[0].0), px(screen_pts[0].1)));
                            if let Ok(p) = p.build() { window.paint_path(p, ocol); }
                        }
                    }
                }
            }

            // ── Face features (dynamic: eye blink, eyebrow, mouth) ───────
            if char_type != "robot" {
                if let Some(hb) = find_bone("head") {
                    let hx = mx(data.head_cx);
                    let hy = my(data.head_cy);
                    let hr = data.head_r as f32 * scale;
                    let head_angle = bangle(hb);

                    let face_col = gpui::rgba(0x1e1b4bff);

                    // 1. Eyes (Normal or Blinking)
                    let eye_y = 0.05 * hr; // slightly above head center
                    if data.eye_blink > 0.5 {
                        // Drawn as horizontal lines
                        for eye_x in [-0.20_f32 * hr, 0.20 * hr] {
                            let (x1s, y1s) = rpt(hx, hy, eye_y, eye_x - hr * 0.10, head_angle);
                            let (x2s, y2s) = rpt(hx, hy, eye_y, eye_x + hr * 0.10, head_angle);
                            let mut p = PathBuilder::stroke(px(hr * 0.06));
                            p.move_to(point(px(x1s), px(y1s)));
                            p.line_to(point(px(x2s), px(y2s)));
                            if let Ok(p) = p.build() { window.paint_path(p, face_col); }
                        }
                    } else {
                        // Normal eyes drawn as filled circles
                        let eye_r = hr * 0.06;
                        let (e1x, e1y) = rpt(hx, hy, eye_y, -0.20 * hr, head_angle);
                        circle(e1x, e1y, eye_r, face_col, window);
                        let (e2x, e2y) = rpt(hx, hy, eye_y, 0.20 * hr, head_angle);
                        circle(e2x, e2y, eye_r, face_col, window);
                    }

                    // 2. Eyebrows
                    let eb_y = hr * (0.22 + data.eyebrow as f32 * 0.10);
                    for ex in [-0.20_f32, 0.20] {
                        let (x1s, y1s) = rpt(hx, hy, eb_y, (ex - 0.10) * hr, head_angle);
                        let (x2s, y2s) = rpt(hx, hy, eb_y, (ex + 0.10) * hr, head_angle);
                        let mut p = PathBuilder::stroke(px(hr * 0.05));
                        p.move_to(point(px(x1s), px(y1s)));
                        p.line_to(point(px(x2s), px(y2s)));
                        if let Ok(p) = p.build() { window.paint_path(p, face_col); }
                    }

                    // 3. Mouth (Smile or Open)
                    // lx = along-bone (vertical), ly = perpendicular (horizontal)
                    // mouth is BELOW center → lx is negative (down on screen)
                    let mouth_lx = -0.22 * hr;
                    if data.mouth > 0.18 {
                        let mr = (data.mouth as f32 * 0.18 * hr).max(hr * 0.06);
                        let (mcx, mcy) = rpt(hx, hy, mouth_lx, 0.0, head_angle);
                        circle(mcx, mcy, mr, face_col, window);
                        circle(mcx, mcy, mr * 0.60, gpui::rgba(0xff7070ff), window);
                    } else {
                        // Closed smile line
                        let (mx1, my1) = rpt(hx, hy, mouth_lx, -0.12 * hr, head_angle);
                        let (mx2, my2) = rpt(hx, hy, mouth_lx,  0.12 * hr, head_angle);
                        let mut p = PathBuilder::stroke(px(hr * 0.04));
                        p.move_to(point(px(mx1), px(my1)));
                        p.line_to(point(px(mx2), px(my2)));
                        if let Ok(p) = p.build() { window.paint_path(p, face_col); }
                    }
                }
            }
        } // end character rendering

        // ════════════════════════════════════════════════════════════════════
        // LAYER 2: SKELETON DEBUG OVERLAY (hidden in production)
        // Set SHOW_SKELETON_DEBUG = true to re-enable for debugging.
        // ════════════════════════════════════════════════════════════════════
        const SHOW_SKELETON_DEBUG: bool = false;
        if SHOW_SKELETON_DEBUG {
            let debug_bone_col = gpui::rgba(0xfde04788);
            let debug_prox_col = gpui::rgba(0xef4444cc);
            let debug_dist_col = gpui::rgba(0x60a5facc);
            let debug_head_col = gpui::rgba(0xe879f9cc);
            let dot_r = scale * 0.012;
            let dot_r_big = scale * 0.016;

            for bone in &data.bones {
                if bone.label == "head" { continue; }
                let (x1s, y1s) = (mx(bone.x1), my(bone.y1));
                let (x2s, y2s) = (mx(bone.x2), my(bone.y2));
                let mut bp = PathBuilder::stroke(px(1.5));
                bp.move_to(point(px(x1s), px(y1s)));
                bp.line_to(point(px(x2s), px(y2s)));
                if let Ok(p) = bp.build() { window.paint_path(p, debug_bone_col); }
                circle(x1s, y1s, dot_r_big, debug_prox_col, window);
                circle(x2s, y2s, dot_r, debug_dist_col, window);
            }
            circle(mx(data.head_cx), my(data.head_cy), dot_r_big, debug_head_col, window);
        }

        // ── 6. Monitor frame border ──────────────────────────────────────────
        let mut border_path = PathBuilder::stroke(px(3.0));
        border_path.move_to(point(px(mon_x), px(mon_y)));
        border_path.line_to(point(px(mon_x + mon_w), px(mon_y)));
        border_path.line_to(point(px(mon_x + mon_w), px(mon_y + mon_h)));
        border_path.line_to(point(px(mon_x), px(mon_y + mon_h)));
        border_path.line_to(point(px(mon_x), px(mon_y)));
        if let Ok(path) = border_path.build() {
            window.paint_path(path, gpui::rgba(0x1e293bff));
        }
    }
}

impl Render for Preview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();

        // 1. Cleanup GPU textures from the previous frame to avoid atlas leak
        if let Ok(mut prev_images) = self.prev_frame_gpu_images.try_borrow_mut() {
            for img in prev_images.drain(..) {
                let _ = window.drop_image(img);
            }
        }

        let (current_part, total_parts) = {
            let ed = self.editor.borrow();
            (ed.current_part, ed.story_episodes.len())
        };

        let anim = self.editor.borrow().animator.clone();
        let render_data = {
            let editor = self.editor.borrow();
            let mut a = anim.borrow_mut();
            self.is_playing = a.is_playing();
            let pose = a.update();
            // C1 fix: read time AFTER update() so current_time matches the clock we just advanced
            self.current_time = a.time();
            self.total_duration = a.total_duration();
            let action = a.active_action();
            stickman_to_segments(&pose, &editor.scene_theme, &editor.character_type, &action)
        };
        if let Some(ref exec_mtx) = self.scene_executor {
            if let Ok(mut guard) = exec_mtx.lock() {
                guard.update(self.current_time);
            }
        }
        self.render_data = Some(Arc::new(render_data));
        self.cinematic_movie = self.editor.borrow().cinematic_movie.clone();
        if let Some(ref movie) = self.cinematic_movie {
            self.total_duration = movie.total_duration;
            // Debug: only log every ~60 frames to avoid spam
            if (self.current_time * 10.0) as u64 % 60 == 0 {
                let pct = (self.current_time / movie.total_duration * 100.0).min(100.0);
                eprintln!("[Cinematic] t={:.1}s / {:.0}s ({:.0}%)", self.current_time, movie.total_duration, pct);
            }
        }

        // ── Poll pending AI result (set by background thread) ────────────────
        let pending_arc = {
            // Borrow dropped immediately after .clone()
            self.editor.borrow().pending_movie.clone()
        };
        if let Ok(mut guard) = pending_arc.try_lock() {
            if guard.is_some() {
                let (movie, parsed) = guard.take().unwrap();
                drop(guard); // release lock before borrowing editor
                let total_dur = movie.total_duration;
                let clips = parsed.clips.clone();
                let mut ed = self.editor.borrow_mut();
                
                let movie_arc = std::sync::Arc::new(movie);
                let mut executor = animation::SceneExecutor::new((*movie_arc).clone());
                executor.initialize_act(0);
                self.scene_executor = Some(Arc::new(Mutex::new(executor)));
                ed.cinematic_movie = Some(movie_arc.clone());
                ed.studio_script = Some(parsed);
                ed.is_generating_movie = false;
                ed.movie_status = "Film siap diputar!".to_string();
                ed.animator.borrow_mut().set_timeline(clips);
                ed.animator.borrow_mut().override_duration = Some(total_dur);
                
                let part_num = (ed.story_episodes.len() + 1) as u32;
                ed.story_episodes.push(crate::state::editor::StoryEpisode {
                    part_number: part_num,
                    title: movie_arc.title.clone(),
                    summary: movie_arc.summary.clone(),
                    movie: movie_arc.clone(),
                });
                ed.current_part = part_num;
                ed.ai_state = crate::state::editor::AiConversationState::EpisodeDone;
                
                ed.chat_history.push(crate::state::editor::ChatMessage {
                    sender: crate::state::editor::ChatSender::AI,
                    text: format!(
                        "🎬 **Part {}: {}** selesai di-generate!\n\n▶ Tekan tombol Play di bawah layar monitor untuk memutar.\n\nApakah kamu ingin lanjut membuat episode berikutnya?",
                        part_num, movie_arc.title
                    ),
                });
                ed.chat_quick_replies = vec![
                    "Ya, Lanjut ke Part berikutnya!".to_string(),
                    "Tidak, saya ingin edit ini".to_string(),
                ];

                // ── AUTO-CONTINUE: generate next episode automatically ───────
                let next_part_num = ed.story_episodes.len() as u32 + 1;
                let max_eps = ed.auto_continue_max_episodes;
                let total_eps_done = ed.story_episodes.len();
                if ed.auto_continue
                    && next_part_num <= max_eps
                {
                    eprintln!(
                        "[Preview] Auto-continue: scheduling Part {} of {}",
                        next_part_num, max_eps
                    );
                    ed.movie_status = format!(
                        "Auto-generate Part {} dari {}...",
                        next_part_num, max_eps
                    );
                    ed.chat_history.push(crate::state::editor::ChatMessage {
                        sender: crate::state::editor::ChatSender::AI,
                        text: format!(
                            "🔄 **Auto-continue aktif**: Saya sedang generate Part {} dari target {} ...",
                            next_part_num, max_eps
                        ),
                    });
                    // Set flag for sidebar to pick up and trigger next generation
                    ed.pending_auto_continue = true;
                } else if ed.auto_continue {
                    // Reached max episodes: finalize the series
                    ed.auto_continue = false;
                    ed.movie_status = "Seri lengkap selesai!".to_string();
                    ed.chat_history.push(crate::state::editor::ChatMessage {
                        sender: crate::state::editor::ChatSender::AI,
                        text: format!(
                            "🎉 **Seri lengkap selesai!** Total {} episode berhasil di-generate.\n\nTekan tombol Play untuk menonton, atau reset untuk membuat cerita baru.",
                            total_eps_done
                        ),
                    });
                    ed.chat_quick_replies =
                        vec!["🎬 Tonton dari awal".to_string(), "↺ Buat cerita baru".to_string()];
                }
                
                eprintln!("[Preview] AI movie applied from background thread. Duration={:.1}s", total_dur);
            }
        }

        // ── Poll real-time progress from background thread ─────────────────
        if let Ok(progress_guard) = self.editor.borrow().pending_progress.try_lock() {
            let p = progress_guard.trim().to_string();
            if !p.is_empty() && !p.contains("Timeout") && !p.contains("killing") {
                let mut ed = self.editor.borrow_mut();
                if ed.is_generating_movie && ed.movie_status != p {
                    ed.movie_status = p;
                }
            }
        }

        let is_generating = self.editor.borrow().is_generating_movie;
        let movie_status = self.editor.borrow().movie_status.clone();


        let bg_theme = {
            if let Some(ref movie) = self.cinematic_movie {
                let current_act = movie.acts.iter()
                    .find(|a| self.current_time >= a.start_time && self.current_time < a.start_time + a.duration)
                    .or_else(|| movie.acts.first());
                current_act.map(|a| a.theme.clone()).unwrap_or_else(|| self.editor.borrow().scene_theme.clone())
            } else {
                self.editor.borrow().scene_theme.clone()
            }
        };

        // Background cross-fade: deteksi perubahan theme
        if self.prev_theme.as_deref() != Some(&bg_theme) {
            // Theme baru: reset transisi
            self.prev_theme = Some(bg_theme.clone());
            self.bg_transition_t = 0.0;
        } else {
            self.bg_transition_t = (self.bg_transition_t + 0.016).min(1.0);
        }
        // Prev theme untuk cross-fade: kita clone dulu untuk menghindari borrow conflict
        let prev_bg_theme_str = self.prev_theme.clone();
        let prev_bg_theme = prev_bg_theme_str.as_deref().and_then(|t| {
            if t != &bg_theme { Some(t.to_string()) } else { None }
        });
        let prev_bg_image: Option<Arc<RenderImage>> = prev_bg_theme.as_ref().and_then(|t| self.get_or_load_bg(t));
        let bg_image: Option<Arc<RenderImage>> = self.get_or_load_bg(&bg_theme);

        // Camera transition tracking
        let prev_camera: Option<animation::CameraShot> = {
            if let Some(ref movie) = self.cinematic_movie {
                let current_act = movie.acts.iter()
                    .find(|a| self.current_time >= a.start_time && self.current_time < a.start_time + a.duration);
                if let Some(act) = current_act {
                    // Detect act change — start transition from previous camera to new act's camera
                    let act_changed = self.last_act_number != act.act_number && self.last_act_number != 0;
                    if act_changed {
                        self.camera_transition_t = 0.0;
                    }
                    if self.prev_camera.is_none() {
                        // First frame ever: no transition
                        self.prev_camera = Some(act.camera.clone());
                        self.camera_transition_t = 1.0;
                    } else if act_changed {
                        // prev_camera keeps the OLD act's camera (already set)
                        // camera_transition_t set to 0 above — let lerp run from 0 → 1
                        self.camera_transition_t = (self.camera_transition_t + 0.016).min(1.0);
                    } else {
                        // Same act — advance transition, promote prev_camera when done
                        self.camera_transition_t = (self.camera_transition_t + 0.016).min(1.0);
                        if self.camera_transition_t >= 1.0 {
                            self.prev_camera = Some(act.camera.clone());
                        }
                    }
                }
                self.prev_camera.clone()
            } else {
                None
            }
        };
        let camera_transition_t = self.camera_transition_t;
        let bg_transition_t = self.bg_transition_t;

        // ── Smart Camera Director: dynamic shot selection per-frame ──────────
        if let Some(ref movie) = self.cinematic_movie.clone() {
            let current_act = movie.acts.iter()
                .find(|a| self.current_time >= a.start_time && self.current_time < a.start_time + a.duration);
            if let Some(act) = current_act {
                // Re-cache beats when entering a new act
                if act.act_number != self.last_act_number {
                    self.cached_beats = animation::SmartCameraDirector::extract_beats(act);
                    self.last_act_number = act.act_number;
                }
                let old_shot = self.dynamic_camera.clone();
                self.dynamic_camera = Some(self.smart_director.select_shot(
                    act,
                    self.current_time,
                    self.prev_camera.as_ref(),
                    &self.cached_beats,
                ));
                // Detect shot change for smooth interpolation
                let shot_changed = match (&old_shot, &self.dynamic_camera) {
                    (Some(old), Some(new)) => {
                        old.shot_type != new.shot_type
                            || old.target_entity_id != new.target_entity_id
                    }
                    (None, Some(_)) => true,
                    _ => false,
                };
                if shot_changed {
                    self.prev_dynamic_camera = old_shot;
                    self.dynamic_transition_t = 0.0;
                } else {
                    self.dynamic_transition_t = (self.dynamic_transition_t + 0.016).min(1.0);
                }
            }
        }
        let dynamic_camera = self.dynamic_camera.clone();
        let prev_dynamic_camera = self.prev_dynamic_camera.clone();
        let camera_dynamic_transition_t = self.dynamic_transition_t;
        let _cached_beats = self.cached_beats.clone();

        let time_str = SharedString::from(Self::format_time(self.current_time));
        let dur_str = SharedString::from(Self::format_time(self.total_duration));
        let play_icon = if self.is_playing { "⏸" } else { "▶" };

        let stickman_arc = self.render_data.clone();
        let has_data = stickman_arc.is_some() || self.cinematic_movie.is_some();

        // Compute cinematic progress for transport bar overlay
        let cinematic_progress: Option<(String, String, f64)> = self.cinematic_movie.as_ref().map(|m| {
            let act = m.acts.iter()
                .find(|a| self.current_time >= a.start_time && self.current_time < a.start_time + a.duration)
                .or_else(|| m.acts.first());
            let act_title = act.map(|a| format!("Babak {}: {}", a.act_number, a.title.clone())).unwrap_or_default();
            let pct = if m.total_duration > 0.0 { (self.current_time / m.total_duration * 100.0).min(100.0) } else { 0.0 };
            (m.title.clone(), act_title, pct)
        });

        let registry_arc = self.character_registry.clone();
        let movie_arc = self.cinematic_movie.clone();
        let cur_time = self.current_time;
        let scene_executor_mtx = self.scene_executor.clone();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(Theme::bg_base())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(Theme::bg_elevated())
                    .m(px(8.0))
                    .rounded(px(8.0))
                    .child(
                        if is_generating {
                            // ── AI Generation Loading — full black overlay ──────────────────
                            let status_str = SharedString::from(movie_status.clone());
                            div()
                                .size_full()
                                .relative()
                                .rounded(px(8.0))
                                .bg(gpui::rgba(0x000000ff))  // pure black background
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(16.0))
                                .child(
                                    div()
                                        .text_size(px(48.0))
                                        .child(SharedString::from("🎬"))
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .gap(px(10.0))
                                        .child(
                                            div()
                                                .text_size(px(16.0))
                                                .text_color(gpui::rgba(0xe2e8f0ff))
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .child(SharedString::from("AI Sedang Menyusun Film..."))
                                        )
                                        .child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(gpui::rgba(0x94a3b8ff))
                                                .child(status_str)
                                        )
                                )
                                .child(
                                    // Animated loading bar
                                    div()
                                        .w(px(200.0))
                                        .h(px(4.0))
                                        .bg(gpui::rgba(0x1e293bff))
                                        .rounded_full()
                                        .child(
                                            div()
                                                .h_full()
                                                .w(px(80.0))
                                                .bg(gpui::rgba(0x6366f1ff))
                                                .rounded_full()
                                        )
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(10.0))
                                        .child(div().w(px(10.0)).h(px(10.0)).rounded_full().bg(gpui::rgba(0x6366f1ff)))
                                        .child(div().w(px(10.0)).h(px(10.0)).rounded_full().bg(gpui::rgba(0x8b5cf6ff)))
                                        .child(div().w(px(10.0)).h(px(10.0)).rounded_full().bg(gpui::rgba(0xa855f7ff)))
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(gpui::rgba(0x475569ff))
                                        .child(SharedString::from(format!("🎬 {} — AI Director", movie_status)))
                                )
                                .into_any_element()
                        } else if has_data {
                            canvas(
                                move |bounds, _window, _cx| { let _ = bounds; },
                                move |bounds, (), window, _cx| {
                                    if let Some(ref data) = stickman_arc {
                                        Self::draw_stickman(
                                            data,
                                            bounds,
                                            window,
                                            bg_image.clone(),
                                            prev_bg_image.clone(),
                                            bg_transition_t,
                                            registry_arc.as_deref(),
                                            movie_arc.as_deref(),
                                            cur_time,
                                            camera_transition_t,
                                            prev_camera.as_ref(),
                                            dynamic_camera.as_ref(),
                                            prev_dynamic_camera.as_ref(),
                                            camera_dynamic_transition_t,
                                            scene_executor_mtx.as_deref(),
                                            &_cached_beats[..],
                                        );
                                    }
                                },
                            )
                            .size_full()
                            .rounded(px(8.0))
                            .into_any_element()
                        } else {
                            // ── Empty state placeholder ────────────────────────────────
                            div()
                                .size_full()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(12.0))
                                .bg(gpui::rgba(0x0d1117ff))
                                .rounded(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(36.0))
                                        .child(SharedString::from("🎥"))
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .text_size(px(14.0))
                                                .text_color(gpui::rgba(0x94a3b8ff))
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .child(SharedString::from("Cinematic Studio"))
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .text_color(gpui::rgba(0x475569ff))
                                                .child(SharedString::from("Buka tab Studio → Ceritakan ide → Tekan Generate"))
                                        )
                                )
                                .into_any_element()
                        },

                    ),
            )
            .child(
                div()
                    .h(px(if cinematic_progress.is_some() { 60.0 } else { 40.0 }))
                    .bg(Theme::bg_surface())
                    .border_t_1()
                    .border_color(Theme::border_subtle())
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(2.0))
                    .px(px(12.0))
                    .when_some(cinematic_progress, |d, (title, act_title, pct)| {
                        d.child(
                            div()
                                .w_full()
                                .flex()
                                .justify_between()
                                .text_size(px(9.0))
                                .text_color(Theme::text_disabled())
                                .child(SharedString::from(format!("\u{1f3a5} {}", act_title)))
                                .child(SharedString::from(format!("{:.0}%", pct))),
                        )
                        .child(
                            div()
                                .w_full()
                                .h(px(4.0))
                                .bg(Theme::bg_elevated())
                                .rounded(px(2.0))
                                .child(
                                    div()
                                        .h(px(4.0))
                                        .rounded(px(2.0))
                                        .bg(Theme::accent())
                                        .w(gpui::relative(pct as f32 / 100.0)),
                                )
                        )
                        .child(
                            div()
                                .text_size(px(8.0))
                                .text_color(gpui::rgba(0x6b7280ff))
                                .child(SharedString::from(title))
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child(
                                transport_btn("\u{23ee}", "skip-start")
                                    .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                        Self::skip_to_start(&this.editor);
                                    })),
                            )
                            .child(
                                transport_btn("\u{23ea}", "rew-5")
                                    .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                        Self::seek_by(&this.editor, -5.0);
                                    })),
                            )
                            .child(
                                div()
                                    .id("play-pause")
                                    .h(px(28.0))
                                    .px(px(12.0))
                                    .bg(Theme::accent())
                                    .rounded(px(6.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(gpui::white())
                                    .text_size(px(12.0))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(Theme::accent_hover()))
                                    .child(SharedString::from(play_icon))
                                    .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                        let editor = this.editor.borrow();
                                        let mut anim = editor.animator.borrow_mut();
                                        if anim.is_playing() { anim.pause(); } else { anim.play(); }
                                    })),
                            )
                            .child(
                                transport_btn("\u{23e9}", "fwd-5")
                                    .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                        Self::seek_by(&this.editor, 5.0);
                                    })),
                            )
                            .child(
                                transport_btn("\u{23ed}", "skip-end")
                                    .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                                        Self::skip_to_end(&this.editor);
                                    })),
                            )
                            .when(current_part > 0 && current_part < total_parts as u32, |d| {
                                d.child(
                                    div()
                                        .id("next-part-btn")
                                        .h(px(22.0))
                                        .px(px(8.0))
                                        .bg(Theme::accent())
                                        .rounded(px(4.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(Theme::accent_pressed()))
                                        .text_size(px(9.0))
                                        .text_color(gpui::white())
                                        .child(SharedString::from("Next Part →"))
                                        .on_click(cx.listener(move |this, _: &ClickEvent, _w, _cx| {
                                            let next_p = current_part + 1;
                                            let mut ed = this.editor.borrow_mut();
                                            if (next_p as usize) <= ed.story_episodes.len() {
                                                let movie = ed.story_episodes[(next_p - 1) as usize].movie.clone();
                                                ed.current_part = next_p;
                                                ed.cinematic_movie = Some(movie.clone());
                                                ed.animator.borrow_mut().override_duration = Some(movie.total_duration);
                                                ed.animator.borrow_mut().seek(0.0);
                                                ed.animator.borrow_mut().play();
                                            }
                                        }))
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .text_size(px(11.0))
                            .font_family("monospace")
                            .when(current_part > 0, |d| {
                                let badge_text = if total_parts > 0 {
                                    format!("PART {} OF {}", current_part, total_parts)
                                } else {
                                    format!("PART {}", current_part)
                                };
                                d.child(
                                    div()
                                        .bg(gpui::rgba(0x6366f1ff))
                                        .text_color(gpui::white())
                                        .px(px(6.0))
                                        .py(px(1.5))
                                        .rounded(px(4.0))
                                        .text_size(px(8.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .mr(px(4.0))
                                        .child(SharedString::from(badge_text))
                                )
                            })
                            .child(div().text_color(Theme::text_primary()).child(time_str))
                            .child(div().text_color(Theme::text_secondary()).child(SharedString::from("/")))
                            .child(div().text_color(Theme::text_secondary()).child(dur_str)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child(
                                transport_btn("\u{1f50a}", "volume")
                                    .on_click(cx.listener(|_this, _: &ClickEvent, _w, _cx| {})),
                            )
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(Theme::text_disabled())
                                    .px(px(4.0))
                                    .child(SharedString::from("16:9")),
                            ),
                    ),
            )
    }
}


/// Smooth Hermite interpolation (ease-in-out)
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Decode an embedded background PNG into a `RenderImage` (BGRA format as required by GPUI).
/// Images are embedded at compile time using `include_bytes!`.
fn load_bg_image(theme: &str) -> Option<Arc<RenderImage>> {
    let bytes: &[u8] = match theme {
        "city"      => include_bytes!("../../assets/backgrounds/bg_city.png"),
        "cyberpunk" => include_bytes!("../../assets/backgrounds/bg_cyberpunk.png"),
        "forest"    => include_bytes!("../../assets/backgrounds/bg_forest.png"),
        "room"      => include_bytes!("../../assets/backgrounds/bg_room.png"),
        "school"    => include_bytes!("../../assets/backgrounds/bg_school.png"),
        "space"     => include_bytes!("../../assets/backgrounds/bg_space.png"),
        _ => return None,
    };

    // Decode PNG/JPEG → RGBA8
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    let mut data = img.into_raw();

    // GPUI expects BGRA format: swap R ↔ B channels
    for chunk in data.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    // Wrap in an image::ImageBuffer (pixel type stays Rgba<u8> structurally)
    let buf = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, data)?;
    let frame = image::Frame::new(buf);
    Some(Arc::new(RenderImage::new(vec![frame])))
}

/// Load CraftPix Multi-Character 2D Sprite Sheets from workspace asset directories (lazy metadata scan).
fn load_character_registry() -> CharacterRegistry {
    let mut skins = HashMap::new();

    let pack_candidates = [
        ("craftpix-543219-2d-game-police-character-free-sprite-sheets", "police", "Police"),
        ("craftpix-485144-2d-game-terrorists-character-free-sprites-sheets", "terrorist", "Terrorist"),
        ("craftpix-955440-2d-game-chibi-boy-free-character-sprite-sheet", "chibi", "Chibi Boy"),
    ];

    let base_paths = [
        std::path::PathBuf::from("."),
        std::path::PathBuf::from("/home/brianatmokoo/Documents/Linux/Opencut"),
    ];

    let root_path = base_paths.iter().find(|p| p.exists()).unwrap_or(&base_paths[0]);

    for (pack_dir_name, pack_prefix, pack_label) in &pack_candidates {
        let scml_root = root_path.join(pack_dir_name).join("scml");
        if !scml_root.exists() { continue; }

        let skin_entries = match std::fs::read_dir(&scml_root) {
            Ok(r) => r.filter_map(|e| e.ok()).collect::<Vec<_>>(),
            Err(_) => continue,
        };

        for skin_entry in skin_entries {
            let skin_dir = skin_entry.path();
            if !skin_dir.is_dir() { continue; }

            let skin_name = skin_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let skin_id = format!("{}_{}", pack_prefix, skin_name.to_lowercase());
            let full_name = format!("{} ({})", pack_label, skin_name);

            // Load SCML data for this skin
            let scml_data = load_skin_scml(&skin_dir);

            if let Some((data, entity_name)) = scml_data {
                skins.insert(skin_id.clone(), CharacterSkin {
                    id: skin_id,
                    name: full_name,
                    scml_data: Some(data),
                    scml_entity_name: Some(entity_name),
                    scml_parts: std::sync::Mutex::new(HashMap::new()),
                    scml_raw_parts: std::sync::Mutex::new(HashMap::new()),
                    rotated_cache: std::sync::Mutex::new(HashMap::new()),
                });
            }
        }
    }

    eprintln!("[Registry] Registered {} 2D character skins (SCML bone animation only)", skins.len());

    CharacterRegistry { skins }
}

/// Load SCML animation data from a character directory.
/// Returns (ScmlData wrapped in Arc, entity name) if successful.
fn load_skin_scml(scml_dir: &std::path::Path) -> Option<(Arc<ScmlData>, String)> {
    if !scml_dir.exists() { return None; }

    // Find the SCML file (there should be exactly one .scml file)
    let scml_file = std::fs::read_dir(scml_dir).ok()?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|s| s == "scml").unwrap_or(false));

    let scml_path = match scml_file {
        Some(f) => f.path(),
        None => return None,
    };

    let xml = std::fs::read_to_string(&scml_path).ok()?;
    let data = parse_scml(&xml, scml_dir.to_str().unwrap_or("")).ok()?;

    // Get the first entity name
    let entity_name = data.entities.first().map(|e| e.name.clone())?;

    eprintln!("[SCML] Loaded {} with {} entities from {:?}",
        entity_name, data.entities.len(), scml_path);

    Some((Arc::new(data), entity_name))
}

/// Rotate and optionally horizontally flip a small RGBA sub-texture.
fn rotate_rgba_image(
    raw: &image::RgbaImage,
    angle_deg: f32,
    flip_x: bool,
) -> (Arc<RenderImage>, f32, f32) {
    let src = if flip_x {
        image::imageops::flip_horizontal(raw)
    } else {
        raw.clone()
    };

    if angle_deg.abs() < 1.0 || src.width() == 0 || src.height() == 0 {
        let mut data = src.into_raw();
        for chunk in data.chunks_exact_mut(4) { chunk.swap(0, 2); }
        let buf = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(raw.width(), raw.height(), data).unwrap();
        return (Arc::new(RenderImage::new(vec![image::Frame::new(buf)])), raw.width() as f32, raw.height() as f32);
    }

    let rad = angle_deg.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();

    let src_w = src.width();
    let src_h = src.height();
    let cx = src_w as f32 / 2.0;
    let cy = src_h as f32 / 2.0;

    let corners = [(-cx, -cy), (cx, -cy), (cx, cy), (-cx, cy)];
    let mut min_x = f32::MAX; let mut max_x = f32::MIN;
    let mut min_y = f32::MAX; let mut max_y = f32::MIN;
    for (x, y) in corners {
        let rx = x * cos - y * sin;
        let ry = x * sin + y * cos;
        min_x = min_x.min(rx); max_x = max_x.max(rx);
        min_y = min_y.min(ry); max_y = max_y.max(ry);
    }

    let out_w = (max_x - min_x).ceil() as u32;
    let out_h = (max_y - min_y).ceil() as u32;

    if out_w == 0 || out_h == 0 {
        let mut data = src.into_raw();
        for chunk in data.chunks_exact_mut(4) { chunk.swap(0, 2); }
        let buf = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(raw.width(), raw.height(), data).unwrap();
        return (Arc::new(RenderImage::new(vec![image::Frame::new(buf)])), raw.width() as f32, raw.height() as f32);
    }

    let out_cx = out_w as f32 / 2.0;
    let out_cy = out_h as f32 / 2.0;

    let mut out_data = vec![0u8; (out_w * out_h * 4) as usize];
    let src_bytes = src.as_raw();

    for oy in 0..out_h {
        for ox in 0..out_w {
            let dx = ox as f32 - out_cx;
            let dy = oy as f32 - out_cy;
            let sx_coord = dx * cos + dy * sin + cx;
            let sy_coord = -dx * sin + dy * cos + cy;

            let ix = sx_coord.round() as i32;
            let iy = sy_coord.round() as i32;

            if ix >= 0 && ix < src_w as i32 && iy >= 0 && iy < src_h as i32 {
                let src_idx = ((iy as u32 * src_w + ix as u32) * 4) as usize;
                let out_idx = ((oy * out_w + ox) * 4) as usize;
                out_data[out_idx]     = src_bytes[src_idx + 2]; // B
                out_data[out_idx + 1] = src_bytes[src_idx + 1]; // G
                out_data[out_idx + 2] = src_bytes[src_idx];     // R
                out_data[out_idx + 3] = src_bytes[src_idx + 3]; // A
            }
        }
    }

    if let Some(buf) = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(out_w, out_h, out_data) {
        (Arc::new(RenderImage::new(vec![image::Frame::new(buf)])), out_w as f32, out_h as f32)
    } else {
        let mut data = src.into_raw();
        for chunk in data.chunks_exact_mut(4) { chunk.swap(0, 2); }
        let buf = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(raw.width(), raw.height(), data).unwrap();
        (Arc::new(RenderImage::new(vec![image::Frame::new(buf)])), raw.width() as f32, raw.height() as f32)
    }
}



fn transport_btn(icon: &str, id: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .h(px(28.0))
        .w(px(28.0))
        .rounded(px(6.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.0))
        .text_color(Theme::text_secondary())
        .hover(|s| s.bg(Theme::bg_hover()).text_color(Theme::text_primary()))
        .cursor_pointer()
        .child(SharedString::from(icon.to_string()))
}

use std::collections::HashMap;
use crate::render::BoneTransform;

// ─── Color Palette ──────────────────────────────────────────────────────────

pub type Rgba = u32; // 0xRRGGBBAA

#[derive(Clone, Debug, PartialEq)]
pub enum ColorRole {
    Skin,
    SkinShadow,
    Hair,
    ClothMain,
    ClothDark,
    ClothAccent,
    Shoe,
    ShoeSole,
    White,
    LightGray,
    Outline,
    Custom(Rgba),
}

pub struct Palette {
    pub skin: Rgba,
    pub hair: Rgba,
    pub cloth_main: Rgba,
    pub cloth_dark: Rgba,
    pub cloth_accent: Rgba,
    pub shoe: Rgba,
    pub outline: Rgba,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            skin:        0xf5c5a3ff,
            hair:        0x3d2b1fff,
            cloth_main:  0x2563ebff,
            cloth_dark:  0x1e3a8aff,
            cloth_accent:0xeab308ff,
            shoe:        0xef4444ff,
            outline:     0x1a1a2eff,
        }
    }
}

impl Palette {
    pub fn resolve(&self, role: &ColorRole) -> Rgba {
        match role {
            ColorRole::Skin        => self.skin,
            ColorRole::SkinShadow  => 0xd4a688ff,
            ColorRole::Hair        => self.hair,
            ColorRole::ClothMain   => self.cloth_main,
            ColorRole::ClothDark   => self.cloth_dark,
            ColorRole::ClothAccent => self.cloth_accent,
            ColorRole::Shoe        => self.shoe,
            ColorRole::ShoeSole    => 0x333333ff,
            ColorRole::White       => 0xffffffff,
            ColorRole::LightGray   => 0xd0d0d0ff,
            ColorRole::Outline     => self.outline,
            ColorRole::Custom(c)   => *c,
        }
    }
}

// ─── Shape Definitions ───────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ShapeKind {
    TaperedLimb {
        start: f32,
        end: f32,
        w1: f32,
        w2: f32,
        cap_start: bool,
        cap_end: bool,
    },
    Polygon {
        points: Vec<(f32, f32)>,
    },
    Ellipse {
        rx: f32,
        ry: f32,
    },
    Circle {
        r: f32,
    },
}

#[derive(Clone, Debug)]
pub struct PartShape {
    pub name: String,
    pub bone: String,
    pub shape: ShapeKind,
    pub color: ColorRole,
    pub offset_x: f32,
    pub offset_y: f32,
    pub z_order: i32,
    pub outline: Option<ColorRole>,
}

pub struct CharacterDef {
    pub name: String,
    pub parts: Vec<PartShape>,
}

pub struct ResolvedShape {
    pub points: Vec<(f64, f64)>,
    pub fill: Rgba,
    pub outline: Option<Rgba>,
    pub z_order: i32,
}

// ─── Convenience builders ───────────────────────────────────────────────────

pub fn limb(
    name: &str,
    bone: &str,
    start: f32,
    end: f32,
    w1: f32,
    w2: f32,
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::TaperedLimb { start, end, w1, w2, cap_start: true, cap_end: true },
        color,
        offset_x: 0.0,
        offset_y: 0.0,
        z_order: z,
        outline: Some(ColorRole::Outline),
    }
}

pub fn limb_nc(
    name: &str,
    bone: &str,
    start: f32,
    end: f32,
    w1: f32,
    w2: f32,
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::TaperedLimb { start, end, w1, w2, cap_start: false, cap_end: false },
        color,
        offset_x: 0.0,
        offset_y: 0.0,
        z_order: z,
        outline: None,
    }
}

pub fn circle_part_joint(
    name: &str,
    bone: &str,
    ox: f32,
    oy: f32,
    r: f32,
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::Circle { r },
        color,
        offset_x: ox,
        offset_y: oy,
        z_order: z,
        outline: Some(ColorRole::Outline),
    }
}

pub fn circle_part(
    name: &str,
    bone: &str,
    ox: f32,
    oy: f32,
    r: f32,
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::Circle { r },
        color,
        offset_x: ox,
        offset_y: oy,
        z_order: z,
        outline: None,
    }
}

pub fn polygon(
    name: &str,
    bone: &str,
    pts: &[(f32, f32)],
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::Polygon { points: pts.to_vec() },
        color,
        offset_x: 0.0,
        offset_y: 0.0,
        z_order: z,
        outline: Some(ColorRole::Outline),
    }
}

pub fn polygon_no_outline(
    name: &str,
    bone: &str,
    pts: &[(f32, f32)],
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::Polygon { points: pts.to_vec() },
        color,
        offset_x: 0.0,
        offset_y: 0.0,
        z_order: z,
        outline: None,
    }
}

pub fn ellipse(
    name: &str,
    bone: &str,
    ox: f32,
    oy: f32,
    rx: f32,
    ry: f32,
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::Ellipse { rx, ry },
        color,
        offset_x: ox,
        offset_y: oy,
        z_order: z,
        outline: Some(ColorRole::Outline),
    }
}

pub fn ellipse_nc(
    name: &str,
    bone: &str,
    ox: f32,
    oy: f32,
    rx: f32,
    ry: f32,
    color: ColorRole,
    z: i32,
) -> PartShape {
    PartShape {
        name: name.to_string(),
        bone: bone.to_string(),
        shape: ShapeKind::Ellipse { rx, ry },
        color,
        offset_x: ox,
        offset_y: oy,
        z_order: z,
        outline: None,
    }
}

// ─── World-space resolver ────────────────────────────────────────────────────

fn local_to_world(bone: &BoneTransform, lx: f64, ly: f64) -> (f64, f64) {
    let (sin_a, cos_a) = bone.angle.sin_cos();
    let x = bone.x1 + lx * bone.length * cos_a + ly * bone.length * sin_a;
    let y = bone.y1 + lx * bone.length * sin_a - ly * bone.length * cos_a;
    (x, y)
}

const CAP_SEGMENTS: usize = 10;
const ELLIPSE_SEGMENTS: usize = 32;

fn capsule_polygon(
    bone: &BoneTransform,
    start: f64,
    end: f64,
    w1: f64,
    w2: f64,
    cap_start: bool,
    cap_end: bool,
) -> Vec<(f64, f64)> {
    let mut pts = Vec::with_capacity(CAP_SEGMENTS * 2 + 4);
    if cap_start {
        for i in 0..=CAP_SEGMENTS {
            let t = std::f64::consts::PI * i as f64 / CAP_SEGMENTS as f64;
            pts.push(local_to_world(bone, start - w1 * t.sin(), -w1 * t.cos()));
        }
    } else {
        pts.push(local_to_world(bone, start, -w1));
        pts.push(local_to_world(bone, start, w1));
    }
    pts.push(local_to_world(bone, end, w2));
    if cap_end {
        for i in 0..=CAP_SEGMENTS {
            let t = std::f64::consts::PI * i as f64 / CAP_SEGMENTS as f64;
            pts.push(local_to_world(bone, end + w2 * t.sin(), w2 * t.cos()));
        }
    } else {
        pts.push(local_to_world(bone, end, -w2));
    }
    pts
}

fn ellipse_polygon(bone: &BoneTransform, ox: f64, oy: f64, rx: f64, ry: f64) -> Vec<(f64, f64)> {
    let (cx, cy) = local_to_world(bone, ox, oy);
    let (sin_a, cos_a) = bone.angle.sin_cos();
    let lx_hat = (cos_a, sin_a);
    let ly_hat = (sin_a, -cos_a);
    (0..ELLIPSE_SEGMENTS)
        .map(|i| {
            let t = 2.0 * std::f64::consts::PI * i as f64 / ELLIPSE_SEGMENTS as f64;
            let dx = rx * bone.length * t.cos();
            let dy = ry * bone.length * t.sin();
            (
                cx + dx * lx_hat.0 + dy * ly_hat.0,
                cy + dx * lx_hat.1 + dy * ly_hat.1,
            )
        })
        .collect()
}

fn circle_polygon(bone: &BoneTransform, ox: f64, oy: f64, r: f64) -> Vec<(f64, f64)> {
    let (cx, cy) = local_to_world(bone, ox, oy);
    (0..ELLIPSE_SEGMENTS)
        .map(|i| {
            let t = 2.0 * std::f64::consts::PI * i as f64 / ELLIPSE_SEGMENTS as f64;
            (cx + r * bone.length * t.cos(), cy + r * bone.length * t.sin())
        })
        .collect()
}

pub fn resolve_character_shapes(
    bones: &[BoneTransform],
    character: &CharacterDef,
    palette: &Palette,
) -> Vec<ResolvedShape> {
    let bone_map: HashMap<&str, &BoneTransform> = bones.iter().map(|b| (b.label, b)).collect();

    let mut parts: Vec<&PartShape> = character.parts.iter().collect();
    parts.sort_by_key(|p| p.z_order);

    parts
        .iter()
        .filter_map(|p| {
            let bone = *bone_map.get(p.bone.as_str())?;
            let points = match &p.shape {
                ShapeKind::TaperedLimb { start, end, w1, w2, cap_start, cap_end } =>
                    capsule_polygon(bone, *start as f64, *end as f64, *w1 as f64, *w2 as f64, *cap_start, *cap_end),
                ShapeKind::Polygon { points } =>
                    points.iter().map(|(lx, ly)| local_to_world(bone, *lx as f64, *ly as f64)).collect(),
                ShapeKind::Ellipse { rx, ry } =>
                    ellipse_polygon(bone, p.offset_x as f64, p.offset_y as f64, *rx as f64, *ry as f64),
                ShapeKind::Circle { r } =>
                    circle_polygon(bone, p.offset_x as f64, p.offset_y as f64, *r as f64),
            };
            let fill = palette.resolve(&p.color);
            let outline = p.outline.as_ref().map(|c| palette.resolve(c));
            Some(ResolvedShape { points, fill, outline, z_order: p.z_order })
        })
        .collect()
}
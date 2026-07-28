//! Design tokens — colors, spacing, and typography for OpenCut AI desktop.

#![allow(dead_code)]

/// Application color palette.
pub struct Theme;

impl Theme {
    // ── Background layers ────────────────────────────────────────────────────
    /// Deepest background (window chrome, empty areas).
    pub fn bg_base() -> gpui::Rgba {
        gpui::rgba(0x0a0a0aff)
    }
    /// Primary surface (panels, sidebars).
    pub fn bg_surface() -> gpui::Rgba {
        gpui::rgba(0x111111ff)
    }
    /// Elevated surface (cards, dropdowns).
    pub fn bg_elevated() -> gpui::Rgba {
        gpui::rgba(0x1a1a1aff)
    }
    /// Hover state for interactive elements.
    pub fn bg_hover() -> gpui::Rgba {
        gpui::rgba(0x242424ff)
    }
    /// Active/pressed state.
    pub fn bg_active() -> gpui::Rgba {
        gpui::rgba(0x2d2d2dff)
    }

    // ── Borders ───────────────────────────────────────────────────────────────
    /// Subtle dividers between panels.
    pub fn border_subtle() -> gpui::Rgba {
        gpui::rgba(0x1e1e1eff)
    }
    /// Standard border for components.
    pub fn border_default() -> gpui::Rgba {
        gpui::rgba(0x2a2a2aff)
    }
    /// Focus ring.
    pub fn border_focus() -> gpui::Rgba {
        gpui::rgba(0x3b82f6ff)
    }

    // ── Text ─────────────────────────────────────────────────────────────────
    /// Primary readable text.
    pub fn text_primary() -> gpui::Rgba {
        gpui::rgba(0xf2f2f2ff)
    }
    /// Secondary / muted text.
    pub fn text_secondary() -> gpui::Rgba {
        gpui::rgba(0x888888ff)
    }
    /// Disabled text.
    pub fn text_disabled() -> gpui::Rgba {
        gpui::rgba(0x444444ff)
    }

    // ── Accent ────────────────────────────────────────────────────────────────
    /// Primary accent — blue (buttons, selections, playhead).
    pub fn accent() -> gpui::Rgba {
        gpui::rgba(0x3b82f6ff)
    }
    /// Accent hover.
    pub fn accent_hover() -> gpui::Rgba {
        gpui::rgba(0x60a5faff)
    }
    /// Accent pressed.
    pub fn accent_pressed() -> gpui::Rgba {
        gpui::rgba(0x2563ebff)
    }

    // ── Status colors ─────────────────────────────────────────────────────────
    /// Success — green.
    pub fn success() -> gpui::Rgba {
        gpui::rgba(0x22c55eff)
    }
    /// Warning — amber.
    pub fn warning() -> gpui::Rgba {
        gpui::rgba(0xf59e0bff)
    }
    /// Error — red.
    pub fn error() -> gpui::Rgba {
        gpui::rgba(0xef4444ff)
    }
    /// Error color for text.
    pub fn error_color() -> gpui::Rgba {
        gpui::rgba(0xef4444ff)
    }
    /// Success hover.
    pub fn success_hover() -> gpui::Rgba {
        gpui::rgba(0x16a34aff)
    }

    // ── Timeline track colors ─────────────────────────────────────────────────
    /// Video track fill.
    pub fn track_video() -> gpui::Rgba {
        gpui::rgba(0x1d4ed8ff)
    }
    /// Audio track fill.
    pub fn track_audio() -> gpui::Rgba {
        gpui::rgba(0x15803dff)
    }
    /// Effects track fill.
    pub fn track_effects() -> gpui::Rgba {
        gpui::rgba(0x7c3aedff)
    }
    /// Clip block background.
    pub fn clip_bg() -> gpui::Rgba {
        gpui::rgba(0x1e40afff)
    }
    /// Playhead line color.
    pub fn playhead() -> gpui::Rgba {
        gpui::rgba(0xef4444ff)
    }
}

/// Convert an `Rgba` value (0x_RR_GG_BB_AA) to gpui::Rgba.
///
/// gpui::rgb() only takes 0xRRGGBB — this helper adds alpha support.
pub fn rgba(hex: u32) -> gpui::Rgba {
    let r = ((hex >> 24) & 0xff) as f32 / 255.0;
    let g = ((hex >> 16) & 0xff) as f32 / 255.0;
    let b = ((hex >> 8) & 0xff) as f32 / 255.0;
    let a = (hex & 0xff) as f32 / 255.0;
    gpui::Rgba { r, g, b, a }
}

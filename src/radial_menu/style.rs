//! Radial-menu presentation tokens and authored-color resolution.

use bevy::prelude::*;

use super::model::RadialMenu;
use crate::widgets::parse_hex_color;

/// Extra radius of the circular background behind the base outer ring.
pub const RADIAL_MENU_BACKGROUND_BLEED: f32 = 4.0;

/// Extra radius of the retained outer-ring node beyond the authored radius.
pub const RADIAL_MENU_OUTER_RING_OUTSET: f32 = 1.0;

/// Fallback radial-menu background; its alpha comes from `bg_opacity`.
pub const RADIAL_MENU_BACKGROUND: Color = Color::srgb(0.096, 0.118, 0.157);

/// Fallback base outer-ring stroke.
pub const RADIAL_MENU_OUTER_BORDER: Color = Color::srgba(0.38, 0.39, 0.39, 0.90);

/// Fill of an unselected sector.
pub const RADIAL_MENU_SECTOR: Color = Color::srgba(0.10, 0.11, 0.12, 1.0);

/// Stroke of an unselected sector divider.
pub const RADIAL_MENU_DIVIDER: Color = Color::srgba(0.30, 0.32, 0.33, 0.85);

/// Fallback hub fill; its alpha comes from `hub_opacity`.
pub const RADIAL_MENU_HUB: Color = Color::srgb(0.10, 0.10, 0.10);

/// Fallback hub border.
pub const RADIAL_MENU_HUB_BORDER: Color = Color::srgba(0.34, 0.35, 0.35, 1.0);

/// Color of visible sector labels.
pub const RADIAL_MENU_LABEL: Color = Color::srgba(0.91, 0.91, 0.89, 1.0);

/// Color of the highlighted sector's name inside the hub.
pub(crate) const RADIAL_MENU_HUB_LABEL: Color = Color::srgba(0.65, 0.65, 0.64, 1.0);

/// Icon badge fills, cycled by sector index.
pub(crate) const RADIAL_MENU_ICON_COLORS: [Color; 6] = [
    Color::srgb(0.68, 1.0, 0.08),
    Color::srgb(0.05, 0.20, 1.0),
    Color::srgb(1.0, 0.10, 0.12),
    Color::srgb(0.02, 0.72, 0.66),
    Color::srgb(0.08, 0.72, 0.98),
    Color::srgb(0.55, 0.55, 0.55),
];

/// Fraction of the sector thickness used for label sizing.
pub const RADIAL_MENU_LABEL_SIZE_RATIO: f32 = 0.18;

/// Minimum and maximum sector-label font sizes in logical pixels.
pub const RADIAL_MENU_LABEL_SIZE_RANGE: (f32, f32) = (9.0, 13.0);

/// Fraction of the sector panel height used for its icon badge.
pub const RADIAL_MENU_ICON_SIZE_RATIO: f32 = 0.42;

/// Minimum and maximum icon badge dimensions in logical pixels.
pub const RADIAL_MENU_ICON_SIZE_RANGE: (f32, f32) = (24.0, 44.0);

/// Padding applied around sector labels and icon badges.
pub const RADIAL_MENU_SECTOR_CONTENT_PADDING: f32 = 6.0;

/// Width of the selected-sector native UI outline.
pub const RADIAL_MENU_SELECTED_OUTLINE_WIDTH: f32 = 2.0;

/// Width of sector divider strokes.
pub const RADIAL_MENU_DIVIDER_WIDTH: f32 = 1.5;

/// Weight used when blending the configured highlight into a selected sector.
pub const RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT: f32 = 0.34;

/// Default fallback width for both the base outer and hub border strokes.
pub const RADIAL_MENU_DEFAULT_BORDER_WIDTH: f32 = 1.0;

/// Blends an overlay color into a sector fill without changing its opacity.
pub(crate) fn blend_radial_menu_colors(base: Color, overlay: Color, weight: f32) -> Color {
    let base = base.to_srgba();
    let overlay = overlay.to_srgba();
    let weight = weight.clamp(0.0, 1.0);
    Color::srgb(
        base.red * (1.0 - weight) + overlay.red * weight,
        base.green * (1.0 - weight) + overlay.green * weight,
        base.blue * (1.0 - weight) + overlay.blue * weight,
    )
}

/// Uses an authored `#rrggbb` color, or `fallback` when none is set.
fn authored_color(hex: &str, fallback: Color, alpha: f32) -> Color {
    if hex.is_empty() {
        fallback
    } else {
        parse_hex_color(hex, alpha)
    }
}

/// Uses the authored border width, or the default when no border color is set.
fn authored_border_width(hex: &str, width: f32) -> f32 {
    if hex.is_empty() {
        RADIAL_MENU_DEFAULT_BORDER_WIDTH
    } else {
        width.max(0.0)
    }
}

/// Colors and stroke widths of one menu, resolved from its authored fields.
pub(crate) struct RadialMenuColors {
    pub(crate) background: Color,
    pub(crate) highlight: Color,
    pub(crate) selected_fill: Color,
    pub(crate) outer_border: Color,
    pub(crate) outer_border_width: f32,
    pub(crate) hub: Color,
    pub(crate) hub_border: Color,
    pub(crate) hub_border_width: f32,
}

impl RadialMenuColors {
    pub(crate) fn new(menu: &RadialMenu) -> Self {
        let highlight = parse_hex_color(&menu.highlight_color, 1.0);
        Self {
            background: authored_color(
                &menu.bg_color,
                RADIAL_MENU_BACKGROUND.with_alpha(menu.bg_opacity),
                menu.bg_opacity,
            ),
            highlight,
            selected_fill: blend_radial_menu_colors(
                RADIAL_MENU_SECTOR,
                highlight,
                RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT,
            ),
            outer_border: authored_color(&menu.outer_border, RADIAL_MENU_OUTER_BORDER, 1.0),
            outer_border_width: authored_border_width(&menu.outer_border, menu.outer_border_width),
            hub: radial_menu_hub_color(menu),
            hub_border: authored_color(&menu.inner_border, RADIAL_MENU_HUB_BORDER, 1.0),
            hub_border_width: authored_border_width(&menu.inner_border, menu.inner_border_width),
        }
    }
}

/// Fill color of the menu hub.
pub(crate) fn radial_menu_hub_color(menu: &RadialMenu) -> Color {
    authored_color(
        &menu.hub_color,
        RADIAL_MENU_HUB.with_alpha(menu.hub_opacity),
        menu.hub_opacity,
    )
}

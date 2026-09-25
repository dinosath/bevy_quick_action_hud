//! Radial-menu geometry: sector angles, anchors, hit testing, and panel shapes.

use bevy::prelude::*;

use super::model::RadialMenu;

/// Extra radial extent applied to the highlighted sector.
///
/// The reference menu keeps the unselected sectors on the base outer ring,
/// while the selected sector projects beyond that ring.  The inner radius is
/// intentionally unchanged so its inner edge remains the hub's circular arc.
pub const HIGHLIGHTED_SECTOR_OUTSET: f32 = 38.0;

/// Amount by which the hub overlaps sector geometry to guarantee a clean
/// circular inner edge over the conic-gradient sector wedges.
pub const RADIAL_MENU_HUB_OVERLAP: f32 = 4.0;

/// Returns the radial extent used by a sector in the retained UI scene.
///
/// Normal sectors terminate at the authored outer radius.  The highlighted
/// sector extends beyond that base ring as measured from the reference UI.
pub(crate) fn rendered_sector_outer_radius(menu: &RadialMenu, highlighted: bool) -> f32 {
    menu.outer_radius * menu.segment_scale
        + if highlighted {
            HIGHLIGHTED_SECTOR_OUTSET
        } else {
            0.0
        }
}

/// Returns the hub radius that clips the sector panels into an inner arc.
pub(crate) fn radial_menu_hub_radius(menu: &RadialMenu) -> f32 {
    (menu.inner_radius - RADIAL_MENU_HUB_OVERLAP).max(8.0)
}

/// Returns the full radial panel height required for a sector to reach the hub.
///
/// The hub clips this panel into an annular sector.  It must not be shortened
/// to the visible annulus thickness because that creates a flat inner edge.
pub(crate) fn radial_menu_sector_panel_height(outer_radius: f32) -> f32 {
    outer_radius.max(24.0)
}

/// Calculates the start and end angles of a sector, including its gap.
pub fn slice_angles(menu: &RadialMenu, index: usize) -> (f32, f32) {
    let sector_count = menu.slots.len().max(1);
    let sector_angle = menu.arc_span / sector_count as f32;
    let half_gap = if menu.overlap { 0.0 } else { menu.gap / 2.0 };
    let start = menu.arc_offset + index as f32 * sector_angle + half_gap;
    let end = menu.arc_offset + (index + 1) as f32 * sector_angle - half_gap;
    (start, end)
}

/// Returns the center position of a sector in logical pixels.
pub fn slice_center(menu: &RadialMenu, index: usize) -> Vec2 {
    let (start, end) = slice_angles(menu, index);
    let center_angle = (start + end) / 2.0;
    let center_radius = (menu.inner_radius + menu.outer_radius) / 2.0;
    Vec2::new(
        center_angle.cos() * center_radius,
        center_angle.sin() * center_radius,
    )
}

/// Named points on a sector used to place overlay controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorAnchor {
    /// Midpoint of the sector's leading radial edge.
    StartEdge,
    /// Midpoint of the sector's trailing radial edge.
    EndEdge,
    /// Middle of the inner arc, just outside the hub.
    InnerArc,
    /// Middle of the outer arc, just outside the ring.
    OuterArc,
}

/// Returns the menu-local position of `anchor` on sector `index`, in logical pixels.
pub fn sector_anchor(menu: &RadialMenu, index: usize, anchor: SectorAnchor) -> Vec2 {
    let (start, end) = slice_angles(menu, index);
    let middle = (start + end) * 0.5;
    let band_center = (menu.inner_radius + menu.outer_radius) * 0.5;
    let (angle, radius) = match anchor {
        SectorAnchor::StartEdge => (start, band_center),
        SectorAnchor::EndEdge => (end, band_center),
        SectorAnchor::InnerArc => (middle, menu.inner_radius + 2.0),
        SectorAnchor::OuterArc => (middle, menu.outer_radius + 2.0),
    };
    Vec2::from_angle(angle) * radius
}

/// Resolves `angle` in menu-local radians to the sector it occupies.
///
/// This is the input counterpart of [`slice_angles`]: it observes the authored
/// arc span and inter-sector gaps, returning `None` for a direction outside the
/// menu or inside a visible gap.
pub(crate) fn sector_index_at_angle(menu: &RadialMenu, angle: f32) -> Option<usize> {
    let sector_count = menu.slots.len();
    let arc_span = menu.arc_span.clamp(0.0, std::f32::consts::TAU);
    if sector_count == 0 || arc_span <= f32::EPSILON {
        return None;
    }

    let relative = (angle - menu.arc_offset).rem_euclid(std::f32::consts::TAU);
    if relative >= arc_span {
        return None;
    }

    let sector_span = arc_span / sector_count as f32;
    let within_sector = relative.rem_euclid(sector_span);
    let half_gap = if menu.overlap {
        0.0
    } else {
        (menu.gap * 0.5).min(sector_span * 0.5)
    };
    if within_sector < half_gap || within_sector > sector_span - half_gap {
        return None;
    }

    Some((relative / sector_span).floor() as usize)
}

/// Returns the conic-gradient `(start, span)` of sector `index`.
///
/// Conic gradients run clockwise from twelve o'clock; menu angles run
/// counter-clockwise from three o'clock.
pub(crate) fn conic_sector(menu: &RadialMenu, index: usize) -> (f32, f32) {
    let (start, end) = slice_angles(menu, index);
    let begin = (std::f32::consts::FRAC_PI_2 - end).rem_euclid(std::f32::consts::TAU);
    (begin, (end - start).max(0.0))
}

/// Rotated hit-test and content panel of one sector, in hub-local UI coordinates.
#[derive(Clone, Copy)]
pub(crate) struct SectorPanel {
    pub(crate) center: Vec2,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) rotation: f32,
}

impl SectorPanel {
    pub(crate) fn new(menu: &RadialMenu, index: usize, selected: bool) -> Self {
        let (start, end) = slice_angles(menu, index);
        let span = (end - start).max(0.0);
        let middle = (start + end) * 0.5;
        let outer_radius = rendered_sector_outer_radius(menu, selected);
        let height = radial_menu_sector_panel_height(outer_radius);
        Self {
            center: Vec2::new(middle.cos() * height * 0.5, middle.sin() * height * 0.5),
            width: (2.0 * outer_radius * (span * 0.5).sin().abs()).max(32.0),
            height,
            rotation: std::f32::consts::FRAC_PI_2 - middle,
        }
    }
}

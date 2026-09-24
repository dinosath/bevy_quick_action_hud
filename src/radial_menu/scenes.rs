//! Reusable BSN scene fragments for radial-menu presentation.

use crate::*;
use bevy::prelude::*;

/// Helper to calculate slice angles with gap.
pub fn slice_angles(menu: &RadialMenu, index: usize) -> (f32, f32) {
    let n = menu.slots.len().max(1);
    let slice_angle = menu.arc_span / n as f32;
    let half_gap = if menu.overlap { 0.0 } else { menu.gap / 2.0 };
    let a0 = menu.arc_offset + index as f32 * slice_angle + half_gap;
    let a1 = menu.arc_offset + (index + 1) as f32 * slice_angle - half_gap;
    (a0, a1)
}

/// Helper to get the center position of a slice (for placing icons/text).
pub fn slice_center(menu: &RadialMenu, index: usize) -> Vec2 {
    let (a0, a1) = slice_angles(menu, index);
    let center_angle = (a0 + a1) / 2.0;
    let center_radius = (menu.inner_radius + menu.outer_radius) / 2.0;
    Vec2::new(
        center_angle.cos() * center_radius,
        center_angle.sin() * center_radius,
    )
}

/// Returns a full-screen `bevy_ui` overlay [`Node`] that centers its children,
/// authored with the [`bsn!`] macro.
///
/// Spawn it with `commands.spawn_scene(wheel_overlay())` and attach the
/// wheel-menu logic components ([`RadialMenu`] and [`RadialMenuState`]) to the
/// resulting entity.
pub fn wheel_overlay() -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(0.)},
            top: {px(0.)},
            width: {percent(100.)},
            height: {percent(100.)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

/// Returns a zero-size hub [`Node`] used as the positioning origin for slices,
/// authored with the [`bsn!`] macro.
///
/// Spawn it as a child of [`wheel_overlay`] and parent each slice panel to it so
/// the absolutely-positioned panels are laid out relative to the screen center.
pub fn wheel_hub() -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node { width: {px(0.)}, height: {px(0.)} }
    }
}

/// Returns an absolutely-positioned, rounded slice panel centered on the radial
/// position of slice `index`, authored with the [`bsn!`] macro.
///
/// `size` is the panel's width/height in logical pixels and `color` its
/// background color. The panel is laid out as a centered vertical column so
/// icons and labels can be added as children.
pub fn wheel_slice_panel(
    menu: &RadialMenu,
    index: usize,
    size: f32,
    color: Color,
) -> impl bevy::scene::prelude::Scene {
    wheel_slice_panel_styled(menu, index, size, color, size * 0.18)
}

/// Like [`wheel_slice_panel`] but with an explicit `corner_radius`, letting the
/// caller pick a slot shape: `size * 0.5` ≈ round, `size * 0.18` ≈ rounded,
/// `0.0` = square.
pub fn wheel_slice_panel_styled(
    menu: &RadialMenu,
    index: usize,
    size: f32,
    color: Color,
    corner_radius: f32,
) -> impl bevy::scene::prelude::Scene {
    let center = slice_center(menu, index);
    // Math coordinates are y-up and centered on the wheel; UI is y-down relative
    // to the hub, so flip the y axis and offset by half the panel size.
    let left = center.x - size / 2.0;
    let top = -center.y - size / 2.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(left)},
            top: {px(top)},
            width: {px(size)},
            height: {px(size)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: {px(2.)},
            border_radius: {BorderRadius::all(px(corner_radius))},
        }
        BackgroundColor({color})
    }
}

/// Returns a circular center-disc [`Node`] of diameter `radius * 2`, centered
/// on the hub via absolute positioning.
///
/// Spawn as a child of [`wheel_hub`].  Add label or icon children afterward
/// with `commands.entity(disc).add_child(...)`.
pub fn wheel_center_disc(radius: f32, color: Color) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(-radius)},
            top: {px(-radius)},
            width: {px(radius * 2.0)},
            height: {px(radius * 2.0)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(radius))},
        }
        BackgroundColor({color})
    }
}

/// Like [`wheel_center_disc`] but draws a coloured ring border around the hub.
///
/// Use this instead of [`wheel_center_disc`] to get the golden ring shown in
/// the reference screenshots.
pub fn wheel_center_ring(
    radius: f32,
    bg: Color,
    ring_color: Color,
    ring_width: f32,
) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(-radius)},
            top: {px(-radius)},
            width: {px(radius * 2.0)},
            height: {px(radius * 2.0)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(radius))},
            border: {UiRect::all(px(ring_width))},
        }
        BackgroundColor({bg})
        BorderColor::all(ring_color)
    }
}

/// A large dark disc that fills the full wheel area, placed behind all slices.
///
/// Spawn as a child of [`wheel_hub`] **before** the slices so it sits at the
/// back of the z-order.
pub fn wheel_bg_disc(outer_radius: f32, color: Color) -> impl bevy::scene::prelude::Scene {
    let r = outer_radius + 4.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {Val::Px(-r)},
            top: {Val::Px(-r)},
            width: {Val::Px(r * 2.0)},
            height: {Val::Px(r * 2.0)},
            border_radius: {BorderRadius::all(Val::Px(r))},
        }
        BackgroundColor({color})
    }
}

/// A thin amber/gold ring just outside the wheel — approximates the dashed
/// outer border visible in the reference screenshots.
pub fn wheel_outer_ring(
    outer_radius: f32,
    color: Color,
    border_w: f32,
) -> impl bevy::scene::prelude::Scene {
    let r = outer_radius + 1.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {Val::Px(-r)},
            top: {Val::Px(-r)},
            width: {Val::Px(r * 2.0)},
            height: {Val::Px(r * 2.0)},
            border_radius: {BorderRadius::all(Val::Px(r))},
            border: {UiRect::all(Val::Px(border_w))},
        }
        BackgroundColor({Color::NONE})
        BorderColor::all(color)
    }
}

/// Absolutely-positioned rectangular slice panel, sized to better fill a
/// segment of the wheel than the square [`wheel_slice_panel_styled`].
///
/// `width` and `height` are in logical pixels.  Use `corner_radius` ≈
/// `min(width, height) * 0.15` for the rounded look shown in the screenshots.
pub fn wheel_slice_panel_rect(
    menu: &RadialMenu,
    index: usize,
    width: f32,
    height: f32,
    color: Color,
    corner_radius: f32,
) -> impl bevy::scene::prelude::Scene {
    let center = slice_center(menu, index);
    let left = center.x - width / 2.0;
    let top = -center.y - height / 2.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(left)},
            top: {px(top)},
            width: {px(width)},
            height: {px(height)},
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            padding: {UiRect::all(px(6.))},
            border_radius: {BorderRadius::all(px(corner_radius))},
        }
        BackgroundColor({color})
    }
}

/// Returns a [`Text`] node sized for a slice **icon** (typically an emoji or
/// large glyph).
///
/// Spawn as a child of [`wheel_slice_panel`] or insert marker components with
/// `.insert(MyMarker)` on the returned `EntityCommands`.
pub fn wheel_slice_icon(
    icon: String,
    font_size: f32,
    color: Color,
) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Text({icon})
        TextFont { font_size: {FontSize::Px(font_size)} }
        TextColor({color})
    }
}

/// Returns a [`Text`] node sized for a slice **label** (name, count, cooldown,
/// etc.).
///
/// Spawn as a child of [`wheel_slice_panel`] or insert marker components with
/// `.insert(MyMarker)` on the returned `EntityCommands`.
pub fn wheel_slice_label(
    text: String,
    font_size: f32,
    color: Color,
) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Text({text})
        TextFont { font_size: {FontSize::Px(font_size)} }
        TextColor({color})
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// QUICK-ACTION CONFIG — data model for the editor and the HUD
// ─────────────────────────────────────────────────────────────────────────────────

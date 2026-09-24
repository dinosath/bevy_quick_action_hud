//! Retained HUD canvas rendering and HUD-specific runtime systems.

use crate::*;
use bevy::prelude::*;

pub(crate) fn hud_child(
    commands: &mut Commands,
    parent: Entity,
    scene: impl bevy::scene::prelude::Scene,
) -> Entity {
    commands
        .spawn_scene(bsn! {
            { scene }
            ChildOf(parent)
        })
        .id()
}

pub(crate) fn hud_text(s: &str, size: f32, color: Color) -> impl bevy::scene::prelude::Scene {
    super::super::bsn::text(s.to_owned(), size, color)
}

pub(crate) fn hud_wheel_icon(
    commands: &mut Commands,
    parent: Entity,
    icon: &str,
    size: f32,
    index: usize,
) {
    let colors = [
        Color::srgb(0.68, 1.0, 0.08),
        Color::srgb(0.05, 0.20, 1.0),
        Color::srgb(1.0, 0.10, 0.12),
        Color::srgb(0.02, 0.72, 0.66),
        Color::srgb(0.08, 0.72, 0.98),
        Color::srgb(0.55, 0.55, 0.55),
    ];
    let badge = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                width: {Val::Px(size)}, height: {Val::Px(size)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: {BorderRadius::all(Val::Px(size * 0.5))},
            }
            BackgroundColor({colors[index % colors.len()]})
        },
    );
    hud_child(commands, badge, hud_text(icon, size * 0.48, Color::WHITE));
}

pub(crate) fn hud_clickable(
    commands: &mut Commands,
    parent: Entity,
    scene: impl bevy::scene::prelude::Scene,
    action: WheelHudAction,
    base: Color,
) -> Entity {
    let e = commands
        .spawn_scene(scene)
        .insert(WheelHudButton { action, base })
        .id();
    commands.entity(parent).add_child(e);
    e
}

/// Parse `#rrggbb` hex string into a Bevy [`Color`].
pub fn parse_hex_color(hex: &str, alpha: f32) -> Color {
    let s = hex.trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return Color::srgba(r as f32 / 255., g as f32 / 255., b as f32 / 255., alpha);
        }
    }
    Color::srgba(0.23, 0.51, 0.96, alpha)
}

pub fn hud_label_or(key: &str) -> String {
    if key.is_empty() {
        "—".into()
    } else {
        key.into()
    }
}

// ── canvas root ──────────────────────────────────────────────────────────────────

pub(crate) fn hud_canvas_root() -> impl bevy::scene::prelude::Scene {
    bsn! {
        super::super::bsn::canvas()
        BackgroundColor({ HUD_BG.with_alpha(1.0) })
    }
}

// ── main HUD build ───────────────────────────────────────────────────────────────

//! Feature-agnostic UI building blocks shared by every HUD component.

use bevy::prelude::*;
use bevy::scene::prelude::Scene;

use crate::{WheelHudAction, WheelHudButton};

/// Full-size absolute node used by HUD layers and component slots.
pub(crate) fn hud_layer() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.),
        top: Val::Px(0.),
        right: Val::Px(0.),
        bottom: Val::Px(0.),
        ..default()
    }
}

/// Spawns `scene` under `parent` as a HUD control that emits `action`.
pub(crate) fn hud_clickable(
    commands: &mut Commands,
    parent: Entity,
    scene: impl Scene,
    action: WheelHudAction,
    base: Color,
) -> Entity {
    commands
        .spawn_scene(scene)
        .insert((WheelHudButton { action, base }, ChildOf(parent)))
        .id()
}

/// Returns `key`, or an em dash when no binding is assigned.
pub(crate) fn hud_label_or(key: &str) -> String {
    if key.is_empty() {
        "—".into()
    } else {
        key.into()
    }
}

/// Spawns `scene` as a child of `parent` and returns the new entity.
pub(crate) fn spawn_child(commands: &mut Commands, parent: Entity, scene: impl Scene) -> Entity {
    commands
        .spawn_scene(bsn! {
            @{ scene }
            ChildOf(parent)
        })
        .id()
}

/// Declares a single-line text label.
pub(crate) fn text_label(value: &str, size: f32, color: Color) -> impl Scene {
    let value = value.to_owned();
    bsn! {
        Text({ value })
        TextFont { font_size: { FontSize::Px(size) } }
        TextColor({ color })
    }
}

/// Square image node used for icons and gamepad glyphs.
pub(crate) fn image_icon(image: Handle<Image>, size: f32, color: Color) -> (Node, ImageNode) {
    (
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            ..default()
        },
        ImageNode {
            image,
            color,
            ..default()
        },
    )
}

/// Asset path of an embedded editor icon such as `"cil-trash"`.
pub(crate) fn editor_icon_path(name: &str) -> String {
    format!("embedded://bevy_quick_action_hud/embedded/icons/editor/{name}.png")
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

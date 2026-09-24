//! Declarative Bevy Scene Notation fragments owned by the HUD feature.
//!
use bevy::prelude::*;
use bevy::scene::prelude::Scene;

pub(crate) fn canvas() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(0.) }, top: { Val::Px(0.) },
            right: { Val::Px(0.) }, bottom: { Val::Px(0.) },
        }
        Visibility::Hidden
    }
}

pub(crate) fn text(value: String, size: f32, color: Color) -> impl Scene {
    bsn! {
        Text({ value })
        TextFont { font_size: { FontSize::Px(size) } }
        TextColor({ color })
    }
}

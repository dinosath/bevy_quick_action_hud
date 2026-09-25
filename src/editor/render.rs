//! Retained editor presentation refresh systems.

use bevy::feathers::controls::ButtonVariant;
use bevy::prelude::*;

pub(super) fn fix_plain_button_initial_bg(
    q: Query<(Entity, &ButtonVariant), Added<ButtonVariant>>,
    mut commands: Commands,
) {
    for (e, variant) in q.iter() {
        if *variant == ButtonVariant::Plain {
            commands.entity(e).insert(BackgroundColor(Color::NONE));
        }
    }
}

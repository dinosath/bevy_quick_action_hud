//! Retained editor presentation refresh systems.

use super::{EditorUiState, WheelHudState};
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

#[allow(clippy::too_many_arguments)]
pub(super) fn rebuild_editor(mut ui: ResMut<EditorUiState>, hud: Res<WheelHudState>) {
    if !ui.dirty {
        return;
    }
    ui.dirty = false;

    debug!(
        "[editor] rebuild_editor — editor_open={} hud_open={} selection={:?} editing={:?} undo={} redo={}",
        hud.editor_open, hud.open, ui.selection, ui.editing,
        ui.undo_stack.len(),
        ui.redo_stack.len(),
    );

    // The editor is an in-canvas retained UI. The HUD renderer owns the
    // selected component's settings card; no second sidebar is rebuilt here.
    if hud.editor_open {
        ui.nav_count = 0;
        return;
    }
    ui.nav_count = 0;
}

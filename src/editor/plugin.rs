//! Bevy plugin wiring for the editor feature.
//!
//! This module owns registration and scheduling only. Editor behavior remains
//! in focused systems modules, which keeps the plugin boundary declarative and
//! makes the runtime order easy to audit.

use super::EditorAction;
use super::{
    apply_action, apply_set_shortcuts, check_edit_shortcut, click_hud_segments,
    editor_capture_gamepad, editor_capture_key, editor_gamepad_nav, editor_keyboard_radial_nav,
    editor_middle_drag, editor_text_input, editor_toolbar_shortcuts, editor_touch_drag,
    editor_undo_redo_shortcuts, fix_plain_button_initial_bg, hud_button_action_shortcuts,
    hud_wheel_nav, is_nav_only_action, process_hud_buttons, rebuild_editor, scroll_editor_to_focus,
    validate_config, ConfigValidation, EditorButton, EditorUiState, HudMiddleDrag,
    QuickActionConfig, WheelHudState,
};
use crate::QuickActionHudPlugin;
use bevy::prelude::*;
use bevy::text::TextEditChange;
use bevy::ui_widgets::Activate;
use bevy::ui_widgets::ValueChange;

fn on_editor_activate(
    trigger: On<Activate>,
    btns: Query<&EditorButton>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
) {
    if let Ok(btn) = btns.get(trigger.event_target()) {
        let action = btn.action.clone();
        apply_action(&action, &mut cfg, &mut ui, &mut hud);
        ui.dirty = true;
        if !is_nav_only_action(&action) {
            hud.dirty = true;
        }
    }
}

fn on_editor_slider(
    trigger: On<ValueChange<f32>>,
    sliders: Query<&super::components::EditorSlider>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
) {
    if let Ok(field) = sliders.get(trigger.event_target()) {
        let action = match &field.0 {
            EditorAction::SetWheelCooldown { .. }
            | EditorAction::SetWheelInnerRadius { .. }
            | EditorAction::SetWheelOpacity { .. }
            | EditorAction::SetActionCooldown { .. }
            | EditorAction::SetActionOpacity { .. } => Some(trigger.value),
            _ => None,
        };
        if let Some(value) = action {
            let edit = match &field.0 {
                EditorAction::SetWheelCooldown { .. } => EditorAction::SetWheelCooldown { value },
                EditorAction::SetWheelInnerRadius { .. } => {
                    EditorAction::SetWheelInnerRadius { value }
                }
                EditorAction::SetWheelOpacity { .. } => EditorAction::SetWheelOpacity { value },
                EditorAction::SetActionCooldown { set, entry, .. } => {
                    EditorAction::SetActionCooldown {
                        set: *set,
                        entry: *entry,
                        value,
                    }
                }
                EditorAction::SetActionOpacity { set, entry, .. } => {
                    EditorAction::SetActionOpacity {
                        set: *set,
                        entry: *entry,
                        value,
                    }
                }
                _ => unreachable!(),
            };
            apply_action(&edit, &mut cfg, &mut ui, &mut hud);
        }
        ui.dirty = true;
        hud.dirty = true;
    }
}

fn on_editor_text_change(
    trigger: On<TextEditChange>,
    fields: Query<(
        &super::components::EditorTextValue,
        &bevy::text::EditableText,
    )>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
) {
    if let Ok((field, text)) = fields.get(trigger.event_target()) {
        let value = text.value().to_string();
        let action = match &field.0 {
            EditorAction::SetActionName { set, entry, .. } => EditorAction::SetActionName {
                set: *set,
                entry: *entry,
                value,
            },
            EditorAction::SetWheelName { .. } => EditorAction::SetWheelName { value },
            _ => return,
        };
        apply_action(&action, &mut cfg, &mut ui, &mut hud);
        ui.dirty = true;
        hud.dirty = true;
    }
}

/// Registers all editor resources and systems into `app`.
pub(crate) fn register_editor_systems(app: &mut App) {
    app.init_resource::<EditorUiState>()
        .init_resource::<HudMiddleDrag>()
        .init_resource::<ConfigValidation>()
        .add_message::<crate::touch::TouchDragEvent>()
        .add_observer(on_editor_activate)
        .add_observer(on_editor_slider)
        .add_observer(on_editor_text_change)
        .add_systems(
            Update,
            (
                // Capture first so captured input is consumed before normal
                // editor/HUD shortcut systems see it.
                editor_capture_key,
                editor_capture_gamepad,
                editor_toolbar_shortcuts,
                editor_keyboard_radial_nav,
                editor_gamepad_nav,
                click_hud_segments,
                process_hud_buttons,
                editor_text_input,
                apply_set_shortcuts,
                hud_button_action_shortcuts,
                hud_wheel_nav,
                editor_middle_drag,
                check_edit_shortcut,
                editor_undo_redo_shortcuts,
                editor_touch_drag,
                validate_config,
                rebuild_editor,
            )
                .chain()
                .in_set(crate::scheduling::EditorSet::Runtime),
        )
        .add_systems(
            PostUpdate,
            (
                fix_plain_button_initial_bg,
                scroll_editor_to_focus.after(bevy::ui::UiSystems::Layout),
            ),
        );
}

/// Convenience plugin equivalent to `QuickActionHudPlugin::with_editor()`.
///
/// Use this plugin when the application wants the standard retained HUD and
/// the built-in Feathers-based editor together.
pub struct QuickActionEditorPlugin;

impl Plugin for QuickActionEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickActionHudPlugin::with_editor());
    }
}

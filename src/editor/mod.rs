//! Editor feature module for the quick-action HUD.
//!
//! The module root contains only feature wiring and re-exports. Runtime logic
//! is split by responsibility so plugin registration, input, navigation,
//! mutation, presentation, and persistence remain independently auditable.

use crate::*;
use bevy::prelude::*;

mod action_apply;
mod actions;
pub(crate) mod components;
mod drag;
mod helpers;
mod hud_interaction;
mod input;
mod navigation;
pub(crate) mod overlays;
mod persistence;
mod plugin;
mod render;
mod shortcuts;
mod state;
mod toolbar;
mod validation;

use action_apply::apply_action;
pub use actions::EditorAction;
pub use components::{EditorButton, FocusedEditorItem, SegmentHoverColor, WheelSettingsPanel};
use drag::{editor_middle_drag, editor_touch_drag};
use helpers::{action_at, sync_wheelset_visuals, wheel_at};
use hud_interaction::{click_hud_segments, process_hud_buttons};
use input::{editor_capture_gamepad, editor_capture_key, editor_text_input, shortcut_just_pressed};
use navigation::{
    editor_gamepad_nav, editor_keyboard_radial_nav, editor_toolbar_shortcuts,
    scroll_editor_to_focus,
};
use persistence::save_config;
pub(crate) use plugin::register_editor_systems;
pub use plugin::QuickActionEditorPlugin;
use render::fix_plain_button_initial_bg;
use shortcuts::{
    apply_set_shortcuts, check_edit_shortcut, editor_undo_redo_shortcuts,
    hud_button_action_shortcuts, hud_wheel_nav,
};
pub use state::{EditFocus, EditorUiState, Selection};
use state::{HudMiddleDrag, MiddleDragTarget};
pub use toolbar::EditMode;

/// Registers the observer that fills the [`EditMode`] slot.
pub(crate) fn hud_plugin(app: &mut App) {
    app.add_observer(toolbar::fill_edit_mode);
    crate::hud::slot::register::<EditMode>(app);
}
use validation::{validate_config, ConfigValidation};

/// Public touch sizing helper retained for the wasm/mobile integration.
pub use helpers::touch_safe_button_size;

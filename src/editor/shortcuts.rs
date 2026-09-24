//! Runtime and editor keyboard/gamepad shortcut systems.

use super::{
    apply_action, shortcut_just_pressed, EditFocus, EditorAction, EditorUiState, Selection,
    WheelHudState,
};
use crate::*;
use bevy::prelude::*;

pub(super) fn apply_set_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    cfg: Res<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    ui: Res<EditorUiState>,
) {
    // Don't switch sets while the editor sidebar is open — that would be confusing
    // and could conflict with the editor's own DPad navigation.
    if ui.capture_consumed || !hud.open || hud.editor_open || ui.editing != EditFocus::None {
        return;
    }
    let pages = enabled_hud_pages(&cfg);
    let Some(pos) = pages.iter().position(|&i| i == hud.active_set) else {
        return;
    };
    if shortcut_just_pressed(&cfg.next_set_key, &keys, &gamepads) {
        if let Some(&next) = pages.get(pos + 1) {
            hud.active_set = next;
        } else if cfg.cycle_sets {
            hud.active_set = pages[0];
        }
        hud.active_wheel_entry = 0;
        hud.dirty = true;
    }
    if shortcut_just_pressed(&cfg.prev_set_key, &keys, &gamepads) {
        if pos > 0 {
            hud.active_set = pages[pos - 1];
        } else if cfg.cycle_sets {
            hud.active_set = *pages.last().unwrap_or(&hud.active_set);
        }
        hud.active_wheel_entry = 0;
        hud.dirty = true;
    }
}

/// When the HUD is open, checks if any button's shortcut was just pressed.
///
/// **Normal mode** (`editor_open = false`): if `close_on_select` is set, closes the HUD.
///
/// **Dry-run mode** (`editor_open = true`): shows a brief visual flash on the matching
/// button but does **not** close the HUD and does **not** execute the action.
pub(super) fn hud_button_action_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    cfg: Res<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    ui: Res<EditorUiState>,
) {
    if ui.capture_consumed || !hud.open || ui.editing != EditFocus::None {
        return;
    }
    let Some(set) = cfg.sets.get(hud.active_set) else {
        return;
    };
    for (ei, entry) in set.entries.iter().enumerate() {
        if let SetEntry::Action(qa) = entry {
            if qa.key.is_empty() {
                continue;
            }
            if shortcut_just_pressed(&qa.key, &keys, &gamepads) {
                if hud.editor_open {
                    // Dry-run: flash the button, no action, no close.
                    hud.flash_action_entry = Some(ei);
                    hud.flash_action_ttl = 0.25;
                    hud.dirty = true;
                } else if qa.close_on_select {
                    hud.open = false;
                    hud.dirty = true;
                }
            }
        } else if let SetEntry::HudSwitch(hs) = entry {
            if hs.enabled
                && !hs.key.is_empty()
                && shortcut_just_pressed(&hs.key, &keys, &gamepads)
                && cfg
                    .sets
                    .get(hs.target_page)
                    .is_some_and(|page| page.enabled)
            {
                hud.active_set = hs.target_page;
                hud.active_wheel_entry = 0;
                hud.active_wheel_index = 0;
                hud.dirty = true;
            }
        }
    }
}

/// Navigates between legacy top-level wheel entries or, for the active
/// `RadialMenuSetState`, between that set's internal wheels.
pub(super) fn hud_wheel_nav(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    cfg: Res<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    ui: Res<EditorUiState>,
) {
    // Don't navigate wheels while the editor is open.
    if ui.capture_consumed || !hud.open || hud.editor_open || ui.editing != EditFocus::None {
        return;
    }
    let Some(set) = cfg.sets.get(hud.active_set) else {
        return;
    };
    let n = count_wheel_entries(set);
    if n == 0 {
        return;
    }
    // Clamp in case the set shrank since last frame.
    if hud.active_wheel_entry >= n {
        hud.active_wheel_entry = 0;
        hud.active_wheel_index = 0;
        hud.dirty = true;
    }
    let mut wheel_entry = 0usize;
    let mut switch_config: Option<(&str, &str, bool, usize, bool)> = None;
    for entry in &set.entries {
        match entry {
            SetEntry::Wheel(_) => {
                if wheel_entry == hud.active_wheel_entry {
                    // Standalone wheels retain the legacy set-level navigation.
                    switch_config = Some((
                        &set.next_wheel_key,
                        &set.prev_wheel_key,
                        set.cycle_wheels,
                        n,
                        false,
                    ));
                    break;
                }
                wheel_entry += 1;
            }
            SetEntry::RadialMenuSet(ws) => {
                if wheel_entry == hud.active_wheel_entry {
                    let next = if ws.next_wheel_key.is_empty() {
                        &set.next_wheel_key
                    } else {
                        &ws.next_wheel_key
                    };
                    let prev = if ws.prev_wheel_key.is_empty() {
                        &set.prev_wheel_key
                    } else {
                        &ws.prev_wheel_key
                    };
                    // A wheel set's shortcuts switch its internal wheels, not
                    // unrelated wheel components on the HUD page.
                    let wheel_count = ws.wheels.len();
                    switch_config = Some((next, prev, ws.cycle_wheels, wheel_count, true));
                    break;
                }
                wheel_entry += 1;
            }
            _ => {}
        }
    }
    let Some((next_key, prev_key, cycle_wheels, wheel_count, is_wheelset)) = switch_config else {
        return;
    };
    if wheel_count < 2 {
        return;
    }
    if shortcut_just_pressed(next_key, &keys, &gamepads) {
        if !is_wheelset {
            if hud.active_wheel_entry + 1 < n {
                hud.active_wheel_entry += 1;
            } else if cycle_wheels {
                hud.active_wheel_entry = 0;
            }
        } else if hud.active_wheel_index + 1 < wheel_count {
            hud.active_wheel_index += 1;
        } else if cycle_wheels {
            hud.active_wheel_index = 0;
        }
        hud.highlighted = None;
        hud.dirty = true;
    }
    if shortcut_just_pressed(prev_key, &keys, &gamepads) {
        if !is_wheelset {
            if hud.active_wheel_entry > 0 {
                hud.active_wheel_entry -= 1;
            } else if cycle_wheels && n > 0 {
                hud.active_wheel_entry = n - 1;
            }
        } else if hud.active_wheel_index > 0 {
            hud.active_wheel_index -= 1;
        } else if cycle_wheels {
            hud.active_wheel_index = wheel_count - 1;
        }
        hud.highlighted = None;
        hud.dirty = true;
    }
}

/// Handles Ctrl+Z (undo) and Ctrl+Shift+Z / Ctrl+Y (redo) keyboard shortcuts
/// in the editor. These only fire when the editor sidebar is open.
pub(super) fn editor_undo_redo_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
) {
    if ui.capture_consumed || !hud.editor_open {
        return;
    }
    // Block while a key capture is in progress.
    if ui.capture_consumed || ui.editing != EditFocus::None {
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let action = if shift {
            EditorAction::Redo
        } else {
            EditorAction::Undo
        };
        apply_action(&action, &mut cfg, &mut ui, &mut hud);
        hud.dirty = true;
        ui.dirty = true;
    }
    if keys.just_pressed(KeyCode::KeyY) {
        let action = EditorAction::Redo;
        apply_action(&action, &mut cfg, &mut ui, &mut hud);
        hud.dirty = true;
        ui.dirty = true;
    }
}

/// Toggles the editor sidebar when the configured edit shortcut is pressed.
/// Only fires while the HUD overlay is open — gameplay can reuse the same
/// buttons without conflict.
pub(super) fn check_edit_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    cfg: Res<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
) {
    // The HUD open shortcut owns HUD visibility. The edit shortcut must never
    // implicitly open the HUD; edit mode is only meaningful while the HUD is
    // already open.
    if !hud.open {
        return;
    }
    if cfg.edit_shortcut.is_empty() {
        return;
    }
    // Block while a key/gamepad capture is in progress.
    if ui.capture_consumed || ui.editing != EditFocus::None {
        debug!(
            "[editor] edit shortcut blocked — capture in progress ({:?})",
            ui.editing
        );
        return;
    }
    if shortcut_just_pressed(&cfg.edit_shortcut, &keys, &gamepads) {
        if hud.editor_open {
            // Close editor.
            info!(
                "[editor] closing editor via shortcut (open={}, editor_open={})",
                hud.open, hud.editor_open
            );
            hud.editor_open = false;
            hud.edit_control_focus = None;
            hud.settings_open = false;
            ui.selection = Selection::None;
            ui.editing = EditFocus::None;
        } else {
            // Open editor only inside an already-open HUD.
            info!(
                "[editor] opening editor via shortcut (hud open={}, editor_open={})",
                hud.open, hud.editor_open
            );
            hud.editor_open = true;
            hud.edit_control_focus = Some(0);
        }
        hud.dirty = true;
        ui.dirty = true;
    }
}

// ─── middle-button HUD dragging ────────────────────────────────────────────────

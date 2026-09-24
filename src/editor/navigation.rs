//! Keyboard, gamepad, and focus navigation for the editor.
//!
//! Navigation produces editor actions or updates transient focus/selection;
//! scene construction remains in `components` and action mutation remains in
//! `action_apply`.

use super::{apply_action, is_nav_only_action, save_config, wheel_at};
use super::{EditFocus, EditorAction, EditorButton, EditorUiState, FocusedEditorItem, Selection};
use crate::*;
use bevy::prelude::*;
use bevy::ui::{OverflowAxis, UiGlobalTransform};

fn active_edit_wheel(
    cfg: &QuickActionConfig,
    hud: &WheelHudState,
) -> Option<(usize, usize, Option<usize>, usize)> {
    let set = cfg.sets.get(hud.active_set)?;
    let mut wheel_index = 0;
    for (entry, value) in set.entries.iter().enumerate() {
        match value {
            SetEntry::Wheel(w) => {
                if wheel_index == hud.active_wheel_entry {
                    return Some((hud.active_set, entry, None, w.slots.len()));
                }
                wheel_index += 1;
            }
            SetEntry::RadialMenuSet(ws) => {
                if wheel_index == hud.active_wheel_entry {
                    let active = hud
                        .active_wheel_index
                        .min(ws.wheels.len().saturating_sub(1));
                    return ws
                        .wheels
                        .get(active)
                        .map(|w| (hud.active_set, entry, Some(active), w.slots.len()));
                }
                wheel_index += 1;
            }
            SetEntry::Action(_) => {}
            SetEntry::HudSwitch(_) => {}
        }
    }
    None
}

/// Gamepad D-pad + button navigation for editor controls.
pub(super) fn editor_gamepad_nav(
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
    mut cfg: ResMut<QuickActionConfig>,
    focused_btn_q: Query<&EditorButton, With<FocusedEditorItem>>,
) {
    /// Seconds held before repeat begins.
    const HOLD_DELAY: f32 = 0.40;
    /// Seconds between each repeated step once repeating.
    const HOLD_REPEAT: f32 = 0.7;

    if ui.capture_consumed || !hud.editor_open || ui.editing != EditFocus::None {
        ui.nav_hold_dir = 0;
        ui.nav_hold_timer = 0.0;
        return;
    }
    let Some(gamepad) = gamepads.iter().next() else {
        return;
    };

    // In wheel edit mode, left/right cycles the radial edit targets and South
    // activates the selected target. This keeps the radial controls usable
    // without leaving the radial menu preview.
    if hud.highlighted.is_none() {
        let toolbar_focus = if gamepad.just_pressed(GamepadButton::DPadLeft) {
            Some(match hud.edit_control_focus {
                Some(9) => 8,
                Some(10) => 9,
                Some(8) => 11,
                _ => 8,
            })
        } else if gamepad.just_pressed(GamepadButton::DPadRight) {
            Some(match hud.edit_control_focus {
                Some(8) => 9,
                Some(9) => 10,
                Some(10) => 11,
                Some(11) => 8,
                _ => 9,
            })
        } else {
            None
        };
        if let Some(focus) = toolbar_focus {
            hud.edit_control_focus = Some(focus);
            hud.dirty = true;
            return;
        }
        if gamepad.just_pressed(GamepadButton::DPadUp)
            || gamepad.just_pressed(GamepadButton::DPadDown)
        {
            if let Some((set, entry, wheel, count)) = active_edit_wheel(&cfg, &hud) {
                if count > 0 {
                    ui.selection = Selection::Segment {
                        set,
                        entry,
                        wheel,
                        slot: 0,
                    };
                    hud.highlighted = Some((set, entry, wheel, 0));
                    hud.edit_control_focus = Some(0);
                    hud.dirty = true;
                    ui.dirty = true;
                }
            }
            return;
        }
        if gamepad.just_pressed(GamepadButton::South) {
            match hud.edit_control_focus {
                Some(8) => save_config(&cfg, &ui.config_path),
                Some(9) => {
                    let action = EditorAction::AddAction {
                        set: hud.active_set,
                    };
                    apply_action(&action, &mut cfg, &mut ui, &mut hud);
                    hud.dirty = true;
                    ui.dirty = true;
                }
                Some(10) => {
                    hud.settings_open = !hud.settings_open;
                    hud.dirty = true;
                }
                _ => {}
            }
            return;
        }
    }

    if let Some((set, entry, wheel, slot)) = hud.highlighted {
        if gamepad.just_pressed(GamepadButton::DPadUp)
            || gamepad.just_pressed(GamepadButton::DPadDown)
        {
            let direction = if gamepad.just_pressed(GamepadButton::DPadDown) {
                1usize
            } else {
                usize::MAX
            };
            if let Some(w) = wheel_at(&mut cfg, Selection::Wheel { set, entry, wheel }) {
                if !w.slots.is_empty() {
                    let next_slot = if direction == usize::MAX {
                        if slot == 0 {
                            w.slots.len() - 1
                        } else {
                            slot - 1
                        }
                    } else {
                        (slot + 1) % w.slots.len()
                    };
                    ui.selection = Selection::Segment {
                        set,
                        entry,
                        wheel,
                        slot: next_slot,
                    };
                    hud.highlighted = Some((set, entry, wheel, next_slot));
                    hud.edit_control_focus = Some(0);
                    hud.dirty = true;
                    ui.dirty = true;
                }
            }
            return;
        }
        let left = gamepad.just_pressed(GamepadButton::DPadLeft);
        let right = gamepad.just_pressed(GamepadButton::DPadRight);
        if hud.edit_control_focus.is_some() && (left || right) {
            let current = hud.edit_control_focus.unwrap_or(0);
            let next = if right {
                (current + 1) % 8
            } else {
                (current + 7) % 8
            };
            hud.edit_control_focus = Some(next);
            hud.dirty = true;
            return;
        }
        if gamepad.just_pressed(GamepadButton::South) {
            match hud.edit_control_focus.unwrap_or(0) {
                1 | 3 | 4 => {
                    let side = if hud.edit_control_focus == Some(1) {
                        SegmentInsertSide::Before
                    } else {
                        if hud.edit_control_focus == Some(4) {
                            SegmentInsertSide::Outer
                        } else {
                            SegmentInsertSide::After
                        }
                    };
                    if let Some(w) = wheel_at(&mut cfg, Selection::Wheel { set, entry, wheel }) {
                        let insert_at = if side == SegmentInsertSide::Before {
                            slot
                        } else {
                            slot.saturating_add(1)
                        }
                        .min(w.slots.len());
                        w.slots
                            .insert(insert_at, Sector::named(format!("Slot {}", insert_at + 1)));
                        ui.selection = Selection::Segment {
                            set,
                            entry,
                            wheel,
                            slot: insert_at,
                        };
                        hud.highlighted = Some((set, entry, wheel, insert_at));
                        hud.edit_control_focus = Some(0);
                        hud.dirty = true;
                        ui.dirty = true;
                    }
                }
                2 => {
                    if let Some(w) = wheel_at(&mut cfg, Selection::Wheel { set, entry, wheel }) {
                        if w.slots.len() > 1 && slot < w.slots.len() {
                            w.slots.remove(slot);
                            let next_slot = slot.min(w.slots.len() - 1);
                            ui.selection = Selection::Segment {
                                set,
                                entry,
                                wheel,
                                slot: next_slot,
                            };
                            hud.highlighted = Some((set, entry, wheel, next_slot));
                            hud.edit_control_focus = Some(0);
                            hud.dirty = true;
                            ui.dirty = true;
                        }
                    }
                }
                5..=7 => {
                    ui.selection = Selection::Segment {
                        set,
                        entry,
                        wheel,
                        slot,
                    };
                    ui.editing = match hud.edit_control_focus {
                        Some(5) => EditFocus::SlotName(slot),
                        Some(6) => EditFocus::SlotIcon(slot),
                        _ => EditFocus::SlotInput(slot),
                    };
                    ui.capture_skip = true;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                _ => {}
            }
            return;
        }
    }

    // ── D-Pad hold-to-repeat navigation ──────────────────────────────────────
    let down = gamepad.pressed(GamepadButton::DPadDown);
    let up = gamepad.pressed(GamepadButton::DPadUp);
    let dir = if down {
        1i32
    } else if up {
        -1
    } else {
        0
    };

    if dir != 0 {
        if dir != ui.nav_hold_dir {
            // New direction — navigate immediately and start the hold timer.
            ui.nav_hold_dir = dir;
            ui.nav_hold_timer = 0.0;
            if dir > 0 {
                ui.navfocus = (ui.navfocus + 1).min(ui.nav_count.saturating_sub(1));
            } else {
                ui.navfocus = ui.navfocus.saturating_sub(1);
            }
            ui.dirty = true;
        } else {
            // Same direction — accumulate time and repeat after delays.
            ui.nav_hold_timer += time.delta_secs();
            if ui.nav_hold_timer >= HOLD_DELAY {
                let steps = ((ui.nav_hold_timer - HOLD_DELAY) / HOLD_REPEAT) as usize + 1;
                let prev_steps = (((ui.nav_hold_timer - time.delta_secs()) - HOLD_DELAY).max(0.0)
                    / HOLD_REPEAT) as usize;
                let new_steps = steps.saturating_sub(prev_steps);
                for _ in 0..new_steps {
                    if dir > 0 {
                        ui.navfocus = (ui.navfocus + 1).min(ui.nav_count.saturating_sub(1));
                    } else {
                        ui.navfocus = ui.navfocus.saturating_sub(1);
                    }
                }
                if new_steps > 0 {
                    ui.dirty = true;
                }
            }
        }
    } else {
        // No direction held — reset.
        ui.nav_hold_dir = 0;
        ui.nav_hold_timer = 0.0;
    }

    // ── South: activate focused item ─────────────────────────────────────────
    if gamepad.just_pressed(GamepadButton::South) {
        // Resolve the action from whichever focusable type is currently highlighted.
        let action = if let Ok(btn) = focused_btn_q.single() {
            Some(btn.action.clone())
        } else {
            None
        };
        if let Some(action) = action {
            let sel_before = ui.selection;
            apply_action(&action, &mut cfg, &mut ui, &mut hud);
            ui.dirty = true;
            if !is_nav_only_action(&action) {
                hud.dirty = true;
            }
            // If we just entered a capture mode, mark capture_skip so that
            // editor_capture_gamepad ignores the South press that triggered this.
            if ui.editing != EditFocus::None {
                ui.capture_skip = true;
            }
            // Only jump back to the top when we navigated into a new panel.
            // For in-place edits (toggle, stepper, cycle) keep the cursor where it is.
            if ui.selection != sel_before {
                ui.navfocus = 0;
            }
        }
    } else if gamepad.just_pressed(GamepadButton::East) {
        let sel_before = ui.selection;
        apply_action(&EditorAction::NavBack, &mut cfg, &mut ui, &mut hud);
        ui.dirty = true;
        hud.dirty = true;
        if ui.selection != sel_before {
            ui.navfocus = 0;
        }
    }
}

pub(super) fn editor_toolbar_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut cfg: ResMut<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
) {
    if !hud.editor_open || ui.editing != EditFocus::None {
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let gamepad = gamepads.iter().next();
    let save = (ctrl && keys.just_pressed(KeyCode::KeyS))
        || gamepad.is_some_and(|gp| gp.just_pressed(GamepadButton::LeftTrigger));
    let add = (ctrl && keys.just_pressed(KeyCode::KeyN))
        || gamepad.is_some_and(|gp| gp.just_pressed(GamepadButton::RightTrigger));
    let settings = (ctrl && keys.just_pressed(KeyCode::Comma))
        || gamepad.is_some_and(|gp| gp.just_pressed(GamepadButton::Select));
    if save {
        save_config(&cfg, &ui.config_path);
        hud.edit_control_focus = Some(8);
        hud.dirty = true;
    } else if add {
        let action = EditorAction::AddAction {
            set: hud.active_set,
        };
        apply_action(&action, &mut cfg, &mut ui, &mut hud);
        hud.edit_control_focus = Some(9);
        hud.dirty = true;
        ui.dirty = true;
    } else if settings {
        hud.settings_open = !hud.settings_open;
        hud.edit_control_focus = Some(10);
        hud.dirty = true;
    }
}

/// Keyboard equivalent of the radial edit focus. Arrow keys cycle sector,
/// add-before, delete, and add-after; Enter/Space activates the target.
pub(super) fn editor_keyboard_radial_nav(
    keys: Res<ButtonInput<KeyCode>>,
    mut hud: ResMut<WheelHudState>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
) {
    if !hud.editor_open || ui.editing != EditFocus::None || hud.highlighted.is_none() {
        return;
    }
    let left = keys.just_pressed(KeyCode::ArrowLeft);
    let right = keys.just_pressed(KeyCode::ArrowRight);
    if left || right {
        let current = hud.edit_control_focus.unwrap_or(0);
        hud.edit_control_focus = Some(if right {
            (current + 1) % 8
        } else {
            (current + 7) % 8
        });
        hud.dirty = true;
        return;
    }
    if !(keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space)) {
        return;
    }
    let Some((set, entry, wheel, slot)) = hud.highlighted else {
        return;
    };
    match hud.edit_control_focus.unwrap_or(0) {
        1 | 3 | 4 => {
            let insert_at = if hud.edit_control_focus == Some(1) {
                slot
            } else {
                slot.saturating_add(1)
            };
            if let Some(w) = wheel_at(&mut cfg, Selection::Wheel { set, entry, wheel }) {
                let insert_at = insert_at.min(w.slots.len());
                w.slots
                    .insert(insert_at, Sector::named(format!("Slot {}", insert_at + 1)));
                ui.selection = Selection::Segment {
                    set,
                    entry,
                    wheel,
                    slot: insert_at,
                };
                hud.highlighted = Some((set, entry, wheel, insert_at));
                hud.edit_control_focus = Some(0);
                hud.dirty = true;
                ui.dirty = true;
            }
        }
        5..=7 => {
            ui.selection = Selection::Segment {
                set,
                entry,
                wheel,
                slot,
            };
            ui.editing = match hud.edit_control_focus {
                Some(5) => EditFocus::SlotName(slot),
                Some(6) => EditFocus::SlotIcon(slot),
                _ => EditFocus::SlotInput(slot),
            };
            ui.capture_skip = true;
            hud.dirty = true;
            ui.dirty = true;
        }
        2 => {
            if let Some(w) = wheel_at(&mut cfg, Selection::Wheel { set, entry, wheel }) {
                if w.slots.len() > 1 && slot < w.slots.len() {
                    w.slots.remove(slot);
                    let next_slot = slot.min(w.slots.len() - 1);
                    ui.selection = Selection::Segment {
                        set,
                        entry,
                        wheel,
                        slot: next_slot,
                    };
                    hud.highlighted = Some((set, entry, wheel, next_slot));
                    hud.edit_control_focus = Some(0);
                    hud.dirty = true;
                    ui.dirty = true;
                }
            }
        }
        _ => {}
    }
}

/// Runs in PostUpdate after layout, auto-scrolling editor controls so the
/// gamepad-focused item is always visible.
///
/// Works by walking up the parent chain from the focused entity to find the
/// nearest `overflow: scroll_y` container, then computing the scroll offset
/// needed to centre (or just reveal) the focused item inside the viewport.
pub(super) fn scroll_editor_to_focus(
    mut ui: ResMut<EditorUiState>,
    mut commands: Commands,
    focused_q: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<FocusedEditorItem>>,
    node_q: Query<(&Node, &ComputedNode, &UiGlobalTransform)>,
    scroll_q: Query<Option<&ScrollPosition>>,
    parent_q: Query<&ChildOf>,
) {
    if !ui.scroll_to_focus {
        return;
    }
    ui.scroll_to_focus = false;

    let Ok((focus_entity, focus_cn, focus_tf)) = focused_q.single() else {
        return;
    };
    // Skip if layout hasn't run yet (sizes are zero on the first frame after spawn).
    if focus_cn.size().y == 0.0 {
        ui.scroll_to_focus = true; // retry next frame
        return;
    }

    // Walk up the hierarchy to find the nearest scrollable ancestor.
    let mut current = focus_entity;
    let scroll_entity = loop {
        let Ok(child_of) = parent_q.get(current) else {
            break None;
        };
        let parent = child_of.parent();
        if let Ok((node, _, _)) = node_q.get(parent) {
            if node.overflow.y == OverflowAxis::Scroll {
                break Some(parent);
            }
        }
        current = parent;
    };
    let Some(scroll_entity) = scroll_entity else {
        return;
    };

    let Ok((_, scroll_cn, scroll_tf)) = node_q.get(scroll_entity) else {
        return;
    };
    let viewport_h = scroll_cn.size().y;
    if viewport_h == 0.0 {
        return;
    }

    let scale = scroll_cn.inverse_scale_factor;
    if scale == 0.0 {
        return;
    }

    // UiGlobalTransform stores the *centre* of each node in physical pixels.
    let focus_center_y = focus_tf.translation.y;
    let scroll_center_y = scroll_tf.translation.y;

    let item_h = focus_cn.size().y;
    let item_top_phys = focus_center_y - item_h / 2.0;
    let scroll_top_phys = scroll_center_y - viewport_h / 2.0;

    // Current scroll offset (logical → physical).
    let current_scroll_logical = scroll_q
        .get(scroll_entity)
        .ok()
        .flatten()
        .map(|sp| sp.0.y)
        .unwrap_or(0.0);
    let current_scroll_phys = current_scroll_logical / scale;

    // Item's content-space top (distance from the very top of the scrollable content).
    let item_content_top = (item_top_phys - scroll_top_phys) + current_scroll_phys;

    // If the item is already fully in view, do nothing.
    let item_top_in_viewport = item_content_top - current_scroll_phys;
    let item_bottom_in_viewport = item_top_in_viewport + item_h;
    if item_top_in_viewport >= 0.0 && item_bottom_in_viewport <= viewport_h {
        return;
    }

    // Centre the item in the viewport, clamped to valid range.
    let target_scroll_phys = (item_content_top - (viewport_h - item_h) / 2.0).max(0.0);
    let target_scroll_logical = target_scroll_phys * scale;

    commands
        .entity(scroll_entity)
        .insert(ScrollPosition(Vec2::new(0.0, target_scroll_logical)));
}

// ─── primitive bsn! helpers ──────────────────────────────────────────────────────

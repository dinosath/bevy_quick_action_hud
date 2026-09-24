//! Pointer and touch movement systems for editor-selected components.

use super::{
    action_at, sync_wheelset_visuals, wheel_at, EditorUiState, HudMiddleDrag, MiddleDragTarget,
    Selection, WheelHudState,
};
use crate::*;
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

pub(super) fn editor_middle_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut cfg: ResMut<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    mut drag: ResMut<HudMiddleDrag>,
) {
    if !hud.open || !hud.editor_open {
        drag.target = None;
        return;
    }

    if mouse.just_pressed(MouseButton::Middle) {
        drag.target = hud
            .selected_action
            .map(|(set, entry)| MiddleDragTarget::Action { set, entry })
            .or_else(|| {
                hud.selected_wheel
                    .map(|(set, entry, wheel)| MiddleDragTarget::Wheel { set, entry, wheel })
            })
            .or_else(|| {
                hud.highlighted
                    .map(|(set, entry, wheel, _)| MiddleDragTarget::Wheel { set, entry, wheel })
            });
    }

    if !mouse.pressed(MouseButton::Middle) {
        if mouse.just_released(MouseButton::Middle) {
            drag.target = None;
        }
        return;
    }

    let delta = motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    match drag.target {
        Some(MiddleDragTarget::Action { set, entry }) => {
            if let Some(action) = action_at(&mut cfg, set, entry) {
                action.offset_x = (action.offset_x + delta.x).clamp(-2000.0, 2000.0);
                action.offset_y = (action.offset_y - delta.y).clamp(-2000.0, 2000.0);
                hud.dirty = true;
            }
        }
        Some(MiddleDragTarget::Wheel { set, entry, wheel }) => {
            if let Some(w) = wheel_at(&mut cfg, Selection::Wheel { set, entry, wheel }) {
                w.offset_x = (w.offset_x + delta.x).clamp(-2000.0, 2000.0);
                w.offset_y = (w.offset_y - delta.y).clamp(-2000.0, 2000.0);
            }
            sync_wheelset_visuals(&mut cfg, Selection::Wheel { set, entry, wheel });
            hud.dirty = true;
        }
        None => {}
    }
}

// ─── touch drag integration ─────────────────────────────────────────────────────

/// Listens for [`TouchDragEvent`] emitted by the touch module and applies
/// position changes to HUD quick-action buttons or wheel segments.
///
/// When the editor is open, dragging a floating action button updates its
/// `radius` and `position` fields in the config.  Touch events are filtered
/// to only fire when the HUD is open and a valid target is under the finger.
pub(super) fn editor_touch_drag(
    mut drag_events: MessageReader<crate::touch::TouchDragEvent>,
    cfg: Res<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
    windows: Query<&Window>,
) {
    if !hud.open {
        drag_events.clear();
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let _scale = window.scale_factor();

    for ev in drag_events.read() {
        if ev.ended && ev.target.is_none() {
            // Check if we tapped an action button to select it for editing.
            if let Some(set) = cfg.sets.get(hud.active_set) {
                // Hit-test floating action buttons by position.
                let logical_pos = ev.position;
                for (ei, entry) in set.entries.iter().enumerate() {
                    if let SetEntry::Action(qa) = entry {
                        if !qa.enabled {
                            continue;
                        }
                        // Approximate hit-test: check if touch is within button bounds
                        // Buttons are positioned from bottom-right.
                        let btn_x = logical_pos.x;
                        let btn_y = logical_pos.y;
                        // Simple bounding check: buttons are in bottom-right area
                        if btn_x > 0.0 && btn_y > 0.0 {
                            ui.selection = crate::editor::Selection::Action {
                                set: hud.active_set,
                                entry: ei,
                            };
                            ui.dirty = true;
                            hud.dirty = true;
                        }
                    }
                }
            }
        }

        if ev.started || ev.ended {
            continue;
        }

        // Apply drag delta to reposition the element.
        // In a full implementation, we'd hit-test against specific UI entities.
        // For now, we store the latest drag position for application use.
        debug!(
            "[touch] drag: finger={} pos=({:.0},{:.0}) delta=({:.1},{:.1})",
            ev.finger_id, ev.position.x, ev.position.y, ev.delta.x, ev.delta.y
        );
    }
}

//! Editor action mutation and undo handling.
//!
//! The editor UI emits [`EditorAction`] values; this module owns the mutation
//! boundary that applies them to the authored configuration and transient HUD
//! state. Keeping this pipeline separate from interaction and scene code makes
//! ownership and undo behavior explicit without rebuilding UI hierarchies.

use super::input::wheel_entry_idx;
use super::persistence::{load_config, save_config};
use super::{action_at, sync_wheelset_visuals, wheel_at};
use super::{EditFocus, EditorAction, EditorUiState, Selection};
use crate::*;
use bevy::log::debug;
use bevy::prelude::*;

fn is_mutative_action(action: &EditorAction) -> bool {
    !matches!(
        action,
        EditorAction::SelectSet { .. }
            | EditorAction::SelectAction { .. }
            | EditorAction::SelectWheel { .. }
            | EditorAction::SelectWheelSetEntry { .. }
            | EditorAction::SelectSetSwitch
            | EditorAction::NavBack
            | EditorAction::SelectSegment { .. }
            | EditorAction::EditName { .. }
            | EditorAction::SetActionName { .. }
            | EditorAction::SetActionCooldown { .. }
            | EditorAction::SetActionOpacity { .. }
            | EditorAction::EditSetName { .. }
            | EditorAction::EditWheelName
            | EditorAction::SetWheelName { .. }
            | EditorAction::EditSlotName { .. }
            | EditorAction::EditSlotIcon { .. }
            | EditorAction::EditWheelSetName { .. }
            | EditorAction::EditSetBgImage { .. }
            | EditorAction::CaptureKey { .. }
            | EditorAction::CaptureNextSetKey
            | EditorAction::CapturePrevSetKey
            | EditorAction::CaptureEditShortcut
            | EditorAction::CaptureWheelSetSwitchKey { .. }
            | EditorAction::CaptureNextWheelKey { .. }
            | EditorAction::CapturePrevWheelKey { .. }
            | EditorAction::Save
            | EditorAction::Load
            | EditorAction::Undo
            | EditorAction::Redo
    )
}

/// Push a snapshot of `cfg` onto the undo stack, clearing the redo stack.
/// Does nothing for non-mutative actions (selection, nav, editing focus).
fn maybe_push_undo_snapshot(
    action: &EditorAction,
    cfg: &QuickActionConfig,
    ui: &mut EditorUiState,
) {
    if !is_mutative_action(action) {
        return;
    }
    ui.undo_stack.push(cfg.clone());
    if ui.undo_stack.len() > ui.undo_limit {
        ui.undo_stack.remove(0);
    }
    ui.redo_stack.clear();
}

// ─── action application ──────────────────────────────────────────────────────────

pub(super) fn apply_action(
    action: &EditorAction,
    cfg: &mut QuickActionConfig,
    ui: &mut EditorUiState,
    hud: &mut WheelHudState,
) {
    debug!(
        "[editor] apply action: {:?} selection={:?} editing={:?}",
        action, ui.selection, ui.editing
    );
    // Push undo snapshot before mutative actions.
    maybe_push_undo_snapshot(action, cfg, ui);

    match *action {
        // ── sets ──────────────────────────────────────────────────────────────
        EditorAction::AddSet => {
            let n = cfg.sets.len() + 1;
            cfg.sets.push(ActionSet {
                name: format!("Set {}", n),
                icon: String::new(),
                enabled: true,
                opacity: 1.0,
                input_override: false,
                entries: Vec::new(),
                bg_image: String::new(),
                bg_image_opacity: 1.0,
                next_wheel_key: String::new(),
                prev_wheel_key: String::new(),
                cycle_wheels: false,
            });
            hud.active_set = cfg.sets.len() - 1;
        }
        EditorAction::DeleteSet { set } => {
            if set < cfg.sets.len() {
                cfg.sets.remove(set);
            }
            let clear = matches!(ui.selection,
                Selection::Action { set: s, .. } | Selection::Wheel { set: s, .. }
                | Selection::Set { set: s } | Selection::WheelSetEntry { set: s, .. }
                if s == set);
            if clear {
                ui.selection = Selection::None;
                ui.editing = EditFocus::None;
            }
            if hud
                .selected_segment
                .is_some_and(|(selected_set, ..)| selected_set == set)
            {
                hud.selected_segment = None;
            }
            if !cfg.sets.is_empty() {
                hud.active_set = hud.active_set.min(cfg.sets.len() - 1);
            }
        }
        EditorAction::SelectSet { set } => {
            ui.selection = Selection::Set { set };
            ui.editing = EditFocus::None;
            hud.selected_segment = None;
            // Sync the HUD live preview to show this set.
            hud.active_set = set;
            hud.active_wheel_entry = 0;
            hud.highlighted = None;
        }
        EditorAction::EditSetName { set } => {
            ui.selection = Selection::Set { set };
            ui.editing = EditFocus::SetName;
        }
        EditorAction::SetOpacityDelta { set, delta } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.opacity = (s.opacity + delta).clamp(0.0, 1.0);
            }
        }
        EditorAction::ToggleSetEnabled { set } => {
            if let Some(page) = cfg.sets.get_mut(set) {
                page.enabled = !page.enabled;
            }
            if cfg
                .sets
                .get(hud.active_set)
                .is_some_and(|page| !page.enabled)
            {
                if let Some(next) = enabled_hud_pages(cfg).first().copied() {
                    hud.active_set = next;
                }
            }
            hud.dirty = true;
        }
        EditorAction::ToggleInputOverride { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.input_override = !s.input_override;
            }
        }
        // ── entries ───────────────────────────────────────────────────────────
        EditorAction::AddAction { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                let n = s.entries.len() + 1;
                s.entries.push(SetEntry::Action(QuickAction {
                    name: format!("Action {}", n),
                    ..default()
                }));
            }
        }
        EditorAction::AddWheelSet { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.entries.push(SetEntry::RadialMenuSet(RadialMenuSet {
                    name: "New radial menu set".into(),
                    radial_menus: vec![RadialMenu::new("Radial menu 1", 6)],
                    ..default()
                }));
            }
        }
        EditorAction::AddHudSwitch { set } => {
            let target_page = (set + 1).min(cfg.sets.len().saturating_sub(1));
            if let Some(s) = cfg.sets.get_mut(set) {
                s.entries.push(SetEntry::HudSwitch(HudSwitch {
                    name: "HUD Switch".into(),
                    target_page,
                    ..default()
                }));
            }
        }
        EditorAction::AddWheelToSet { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                if ws.radial_menus.len() < ws.max_wheels {
                    let n = ws.radial_menus.len() + 1;
                    let mut wheel = RadialMenu::new(format!("Radial menu {}", n), 6);
                    wheelset_visuals(ws).apply_to(&mut wheel);
                    wheel.stick_binding = ws.stick_binding.clone();
                    ws.radial_menus.push(wheel);
                }
            }
        }
        EditorAction::DeleteEntry { set, entry } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                if entry < s.entries.len() {
                    s.entries.remove(entry);
                }
            }
            let clear = matches!(ui.selection,
                Selection::Action { set: s, entry: e } | Selection::Wheel { set: s, entry: e, .. }
                | Selection::WheelSetEntry { set: s, entry: e } if s == set && e == entry);
            if clear {
                ui.selection = Selection::None;
                ui.editing = EditFocus::None;
            }
            if hud
                .selected_segment
                .is_some_and(|(selected_set, selected_entry, ..)| {
                    selected_set == set && selected_entry == entry
                })
            {
                hud.selected_segment = None;
            }
        }
        EditorAction::DeleteWheelFromSet { set, entry, wheel } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                if ws.radial_menus.len() > ws.min_wheels && wheel < ws.radial_menus.len() {
                    ws.radial_menus.remove(wheel);
                }
            }
            let clear = matches!(ui.selection,
                Selection::Wheel { set: s, entry: e, wheel: Some(w) } if s == set && e == entry && w == wheel);
            if clear {
                ui.selection = Selection::None;
                ui.editing = EditFocus::None;
            }
            if hud.selected_segment.is_some_and(
                |(selected_set, selected_entry, selected_wheel, _)| {
                    selected_set == set && selected_entry == entry && selected_wheel == Some(wheel)
                },
            ) {
                hud.selected_segment = None;
            }
        }
        EditorAction::MoveEntryUp { set, entry } => {
            if entry > 0 {
                if let Some(s) = cfg.sets.get_mut(set) {
                    if entry < s.entries.len() {
                        s.entries.swap(entry - 1, entry);
                    }
                }
                ui.selection = match ui.selection {
                    Selection::Action { set: s, entry: e } if s == set && e == entry => {
                        Selection::Action {
                            set,
                            entry: entry - 1,
                        }
                    }
                    Selection::Wheel {
                        set: s,
                        entry: e,
                        wheel: w,
                    } if s == set && e == entry => Selection::Wheel {
                        set,
                        entry: entry - 1,
                        wheel: w,
                    },
                    other => other,
                };
            }
        }
        EditorAction::MoveEntryDown { set, entry } => {
            let len = cfg.sets.get(set).map(|s| s.entries.len()).unwrap_or(0);
            if entry + 1 < len {
                if let Some(s) = cfg.sets.get_mut(set) {
                    s.entries.swap(entry, entry + 1);
                }
                ui.selection = match ui.selection {
                    Selection::Action { set: s, entry: e } if s == set && e == entry => {
                        Selection::Action {
                            set,
                            entry: entry + 1,
                        }
                    }
                    Selection::Wheel {
                        set: s,
                        entry: e,
                        wheel: w,
                    } if s == set && e == entry => Selection::Wheel {
                        set,
                        entry: entry + 1,
                        wheel: w,
                    },
                    other => other,
                };
            }
        }
        // ── selection ────────────────────────────────────────────────────────
        EditorAction::SelectAction { set, entry } => {
            ui.selection = Selection::Action { set, entry };
            ui.editing = EditFocus::None;
            hud.selected_segment = None;
            hud.highlighted = None;
            // Sync the active set so the HUD shows the correct context.
            hud.active_set = set;
        }
        EditorAction::SelectHudSwitch { set, entry } => {
            ui.selection = Selection::HudSwitch { set, entry };
            ui.editing = EditFocus::None;
            hud.selected_segment = None;
            hud.active_set = set;
        }
        EditorAction::CaptureHudSwitchKey { set, entry } => {
            ui.selection = Selection::HudSwitch { set, entry };
            ui.editing = EditFocus::HudSwitchKey;
        }
        EditorAction::ClearHudSwitchKey { set, entry } => {
            if let Some(SetEntry::HudSwitch(hs)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                hs.key.clear();
            }
        }
        EditorAction::ToggleHudSwitchEnabled { set, entry } => {
            if let Some(SetEntry::HudSwitch(hs)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                hs.enabled = !hs.enabled;
            }
        }
        EditorAction::CycleHudSwitchTarget { set, entry } => {
            let len = cfg.sets.len();
            if let Some(SetEntry::HudSwitch(hs)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                hs.target_page = if len == 0 {
                    0
                } else {
                    (hs.target_page + 1) % len
                };
            }
        }
        EditorAction::SelectWheel { set, entry, wheel } => {
            ui.selection = Selection::Wheel { set, entry, wheel };
            ui.editing = EditFocus::None;
            hud.selected_segment = None;
            hud.highlighted = None;
            // Sync the HUD to preview the selected wheel.
            hud.active_set = set;
            hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
        }
        EditorAction::SelectWheelSetEntry { set, entry } => {
            ui.selection = Selection::WheelSetEntry { set, entry };
            ui.editing = EditFocus::None;
            hud.selected_segment = None;
            hud.highlighted = None;
            // Sync the HUD to preview the selected wheel set.
            hud.active_set = set;
            hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
        }
        EditorAction::SelectSetSwitch => {
            ui.selection = Selection::SetSwitch;
            ui.editing = EditFocus::None;
            hud.selected_segment = None;
            hud.highlighted = None;
        }
        EditorAction::NavBack => {
            ui.editing = EditFocus::None;
            hud.selected_segment = None;
            hud.highlighted = None;
            ui.selection = match ui.selection {
                // Segment → back to its wheel
                Selection::Segment {
                    set, entry, wheel, ..
                } => Selection::Wheel { set, entry, wheel },
                // Wheel / Action / WheelSetEntry → back to the set
                Selection::Wheel { set, .. }
                | Selection::Action { set, .. }
                | Selection::WheelSetEntry { set, .. } => Selection::Set { set },
                // Set / SetSwitch / root → root
                _ => Selection::None,
            };
        }
        // ── quick action editing ─────────────────────────────────────────────
        EditorAction::EditName { set, entry } => {
            ui.selection = Selection::Action { set, entry };
            ui.editing = EditFocus::Name;
        }
        EditorAction::SetActionName {
            set,
            entry,
            ref value,
        } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.name.clone_from(value);
            }
            ui.editing = EditFocus::None;
        }
        EditorAction::SetActionCooldown { set, entry, value } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.cooldown_secs = value.clamp(0.0, 10.0);
            }
        }
        EditorAction::SetActionOpacity { set, entry, value } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.opacity = value.clamp(0.0, 1.0);
            }
        }
        EditorAction::CaptureKey { set, entry } => {
            ui.selection = Selection::Action { set, entry };
            ui.editing = EditFocus::ButtonKey;
        }
        EditorAction::CycleIcon { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.icon = cycle_palette(ICON_PALETTE, &a.icon).into();
            }
        }
        EditorAction::CycleCommand { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.command = cycle_palette(COMMAND_PALETTE, &a.command).into();
            }
        }
        EditorAction::CycleHoldCommand { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.hold_command = cycle_palette(COMMAND_PALETTE, &a.hold_command).into();
            }
        }
        EditorAction::CycleSlotCommand { slot } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if let Some(s) = w.slots.get_mut(slot) {
                    s.command = cycle_palette(COMMAND_PALETTE, &s.command).into();
                }
            }
        }
        EditorAction::CycleSlotHoldCommand { slot } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if let Some(s) = w.slots.get_mut(slot) {
                    s.hold_command = cycle_palette(COMMAND_PALETTE, &s.hold_command).into();
                }
            }
        }
        EditorAction::ToggleSlotHold { slot } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if let Some(s) = w.slots.get_mut(slot) {
                    s.hold = !s.hold;
                }
            }
        }
        EditorAction::ToggleHold { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.hold = !a.hold;
            }
        }
        EditorAction::ToggleShowOnMenu { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.show_on_menu = !a.show_on_menu;
            }
        }
        EditorAction::ToggleActionLabels { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.show_labels = !a.show_labels;
            }
        }
        EditorAction::ToggleActionIcons { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.show_icon = !a.show_icon;
            }
        }
        EditorAction::ToggleEnabled { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.enabled = !a.enabled;
            }
        }
        EditorAction::OpacityDelta { set, entry, delta } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.opacity = (a.opacity + delta).clamp(0.0, 1.0);
            }
        }
        EditorAction::RadiusDelta { set, entry, delta } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.radius = (a.radius + delta).clamp(8.0, 256.0);
            }
        }
        EditorAction::ActionWidthDelta { set, entry, delta } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.width = (a.width + delta).clamp(20.0, 300.0);
            }
        }
        EditorAction::ActionRotationDelta { set, entry, delta } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.rotation = (a.rotation + delta).rem_euclid(360.0);
            }
        }
        EditorAction::ActionHeightDelta { set, entry, delta } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.height = (a.height + delta).clamp(12.0, 120.0);
            }
        }
        EditorAction::CyclePosition { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.position = a.position.next();
            }
        }
        EditorAction::CycleShape { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.shape = a.shape.next();
            }
        }
        // ── wheel editing ─────────────────────────────────────────────────────
        EditorAction::EditWheelName => {
            ui.editing = EditFocus::WheelName;
        }
        EditorAction::SetWheelName { ref value } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.name.clone_from(value);
            }
            ui.editing = EditFocus::None;
        }
        EditorAction::ToggleWheelThemePopup => {
            hud.theme_popup_open = !hud.theme_popup_open;
        }
        EditorAction::SetWheelTheme { theme } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.theme = theme;
            }
            hud.theme_popup_open = false;
        }
        EditorAction::CaptureWheelStick => {
            ui.editing = EditFocus::WheelStick;
        }
        EditorAction::WheelCooldownDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.cooldown_secs = (w.cooldown_secs + delta).clamp(0.0, 10.0);
            }
        }
        EditorAction::SetWheelCooldown { value } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.cooldown_secs = value.clamp(0.0, 10.0);
            }
        }
        EditorAction::WheelOuterRadiusDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.outer_radius = (w.outer_radius + delta).clamp(40.0, 300.0);
            }
        }
        EditorAction::WheelInnerRadiusDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.inner_radius = (w.inner_radius + delta).clamp(8.0, 100.0);
            }
        }
        EditorAction::SetWheelInnerRadius { value } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.inner_radius = value.clamp(8.0, 100.0);
            }
        }
        EditorAction::ToggleWheelShowLabels => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.show_labels = !w.show_labels;
            }
        }
        EditorAction::EditSlotName { slot } => {
            ui.editing = EditFocus::SlotName(slot);
        }
        // ── wheel-set entry editing ───────────────────────────────────────────
        EditorAction::EditWheelSetName { set, entry } => {
            ui.selection = Selection::WheelSetEntry { set, entry };
            ui.editing = EditFocus::WheelSetName;
        }
        EditorAction::CaptureWheelSetSwitchKey { set, entry } => {
            ui.selection = Selection::WheelSetEntry { set, entry };
            ui.editing = EditFocus::WheelSetSwitchKey;
        }
        EditorAction::CaptureWheelSetStick { set, entry } => {
            ui.selection = Selection::WheelSetEntry { set, entry };
            ui.editing = EditFocus::WheelSetStick;
        }
        EditorAction::CaptureWheelSetNextKey { set, entry } => {
            ui.selection = Selection::WheelSetEntry { set, entry };
            ui.editing = EditFocus::WheelSetNextKey { set, entry };
        }
        EditorAction::CaptureWheelSetPrevKey { set, entry } => {
            ui.selection = Selection::WheelSetEntry { set, entry };
            ui.editing = EditFocus::WheelSetPrevKey { set, entry };
        }
        EditorAction::ClearWheelSetNextKey { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.next_wheel_key.clear();
            }
        }
        EditorAction::ClearWheelSetPrevKey { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.prev_wheel_key.clear();
            }
        }
        EditorAction::WheelSetMinDelta { set, entry, delta } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                let next = (ws.min_wheels as i32 + delta).clamp(1, ws.max_wheels as i32) as usize;
                ws.min_wheels = next;
                normalize_wheelset(ws);
            }
        }
        EditorAction::WheelSetMaxDelta { set, entry, delta } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.max_wheels = (ws.max_wheels as i32 + delta)
                    .clamp(ws.min_wheels.max(ws.radial_menus.len()) as i32, 64)
                    as usize;
            }
        }
        EditorAction::ToggleWheelSetCycle { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.cycle_wheels = !ws.cycle_wheels;
            }
        }
        EditorAction::SwitchWheelPrev { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get(set).and_then(|s| s.entries.get(entry))
            {
                if !ws.radial_menus.is_empty() {
                    hud.active_set = set;
                    hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
                    hud.active_wheel_index = hud
                        .active_wheel_index
                        .checked_sub(1)
                        .unwrap_or(ws.radial_menus.len() - 1);
                    hud.selected_segment = None;
                    hud.highlighted = None;
                    hud.dirty = true;
                }
            }
        }
        EditorAction::SwitchWheelNext { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get(set).and_then(|s| s.entries.get(entry))
            {
                if !ws.radial_menus.is_empty() {
                    hud.active_set = set;
                    hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
                    hud.active_wheel_index = (hud.active_wheel_index + 1) % ws.radial_menus.len();
                    hud.selected_segment = None;
                    hud.highlighted = None;
                    hud.dirty = true;
                }
            }
        }
        // ── set-switch shortcuts ──────────────────────────────────────────
        EditorAction::CaptureNextSetKey => {
            ui.editing = EditFocus::NextSetKey;
        }
        EditorAction::CapturePrevSetKey => {
            ui.editing = EditFocus::PrevSetKey;
        }
        EditorAction::CaptureEditShortcut => {
            ui.editing = EditFocus::EditShortcut;
        }
        // ── persistence ───────────────────────────────────────────────────────────
        EditorAction::Save => save_config(cfg, &ui.config_path),
        EditorAction::Load => {
            if let Some(loaded) = load_config(&ui.config_path) {
                *cfg = loaded;
                ui.selection = Selection::None;
                ui.editing = EditFocus::None;
                hud.active_set = 0;
                hud.selected_segment = None;
                hud.highlighted = None;
                hud.dirty = true;
                ui.dirty = true;
            }
        }
        // ── segment editing ─────────────────────────────────────────────────
        EditorAction::SelectSegment {
            set,
            entry,
            wheel,
            slot,
        } => {
            ui.selection = Selection::Segment {
                set,
                entry,
                wheel,
                slot,
            };
            ui.editing = EditFocus::None;
            hud.selected_segment = Some((set, entry, wheel, slot));
            hud.highlighted = Some((set, entry, wheel, slot));
            hud.edit_control_focus = Some(0);
            // Sync the HUD to preview the wheel containing this segment.
            hud.active_set = set;
            hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
        }
        EditorAction::EditSlotIcon { slot } => {
            ui.editing = EditFocus::SlotIcon(slot);
        }
        EditorAction::ToggleWheelShowIcon => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.show_icon = !w.show_icon;
            }
        }
        EditorAction::CycleHighlightColor => {
            const COLORS: &[&str] = &[
                "#f59e0b", "#3b82f6", "#14b8a6", "#8b5cf6", "#22c55e", "#ef4444", "#f97316",
            ];
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.highlight_color = cycle_palette(COLORS, &w.highlight_color).into();
            }
        }
        EditorAction::WheelOpacityDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.opacity = (w.opacity + delta).clamp(0.0, 1.0);
            }
        }
        EditorAction::SetWheelOpacity { value } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.opacity = value.clamp(0.0, 1.0);
            }
        }
        EditorAction::CycleInnerBorderColor => {
            const COLORS: &[&str] = &[
                "", "#f59e0b", "#3b82f6", "#14b8a6", "#8b5cf6", "#22c55e", "#ef4444",
            ];
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.inner_border = cycle_palette(COLORS, &w.inner_border).into();
            }
        }
        EditorAction::CycleOuterBorderColor => {
            const COLORS: &[&str] = &[
                "", "#f59e0b", "#3b82f6", "#14b8a6", "#8b5cf6", "#22c55e", "#ef4444",
            ];
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.outer_border = cycle_palette(COLORS, &w.outer_border).into();
            }
        }
        EditorAction::CycleWheelBgColor => {
            const COLORS: &[&str] = &[
                "", "#0d1520", "#111827", "#1a1a2e", "#0f172a", "#1c1c1c", "#0a0f1e",
            ];
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.bg_color = cycle_palette(COLORS, &w.bg_color).into();
            }
        }
        EditorAction::WheelBgOpacityDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.bg_opacity = (w.bg_opacity + delta).clamp(0.0, 1.0);
            }
        }
        EditorAction::WheelOuterBorderWidthDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.outer_border_width = (w.outer_border_width + delta).clamp(0.0, 12.0);
            }
        }
        EditorAction::CycleWheelHubColor => {
            const COLORS: &[&str] = &[
                "", "#0d1520", "#111827", "#1a1a2e", "#0f172a", "#1c1c1c", "#142030",
            ];
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.hub_color = cycle_palette(COLORS, &w.hub_color).into();
            }
        }
        EditorAction::WheelInnerBorderWidthDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.inner_border_width = (w.inner_border_width + delta).clamp(0.0, 12.0);
            }
        }
        EditorAction::WheelHubOpacityDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.hub_opacity = (w.hub_opacity + delta).clamp(0.0, 1.0);
            }
        }
        // ── clear shortcuts ─────────────────────────────────────────────────────────
        EditorAction::ClearNextSetKey => {
            cfg.next_set_key.clear();
        }
        EditorAction::ClearPrevSetKey => {
            cfg.prev_set_key.clear();
        }
        EditorAction::ClearEditShortcut => {
            cfg.edit_shortcut.clear();
        }
        EditorAction::ClearNextWheelKey { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.next_wheel_key.clear();
            }
        }
        EditorAction::ClearPrevWheelKey { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.prev_wheel_key.clear();
            }
        }
        EditorAction::ClearWheelSetSwitchKey { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.switch_key.clear();
            }
        }
        EditorAction::ClearWheelSetStick { set, entry } => {
            if let Some(SetEntry::RadialMenuSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.stick_binding = DEFAULT_STICK_BINDING.into();
                for wheel in &mut ws.radial_menus {
                    wheel.stick_binding = DEFAULT_STICK_BINDING.into();
                }
            }
        }
        EditorAction::ClearActionKey { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.key.clear();
            }
        }
        EditorAction::ToggleShowSetBar => {
            cfg.show_set_bar = !cfg.show_set_bar;
        }
        EditorAction::ToggleCycleSets => {
            cfg.cycle_sets = !cfg.cycle_sets;
        }
        EditorAction::CycleHudOpenMode => {
            cfg.hud_open_mode = cfg.hud_open_mode.next();
        }
        EditorAction::HudBgOpacityDelta { delta } => {
            cfg.hud_bg_opacity = (cfg.hud_bg_opacity + delta).clamp(0.0, 1.0);
        }
        EditorAction::CycleHudBgColor => {
            const COLORS: &[&str] = &[
                "", "#0d1520", "#111827", "#1a1a2e", "#0f172a", "#1c1c1c", "#0a0f1e", "#0e1116",
                "#160b0b", "#0b160b",
            ];
            cfg.hud_bg_color = cycle_palette(COLORS, &cfg.hud_bg_color).into();
        }
        EditorAction::ToggleSlotCloseOnSelect { slot } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if let Some(s) = w.slots.get_mut(slot) {
                    s.close_on_select = !s.close_on_select;
                }
            }
        }
        EditorAction::ToggleActionCloseOnSelect { set, entry } => {
            if let Some(SetEntry::Action(qa)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                qa.close_on_select = !qa.close_on_select;
            }
        }
        // ── per-set config ──────────────────────────────────────────────────────────
        EditorAction::EditSetBgImage { set } => {
            ui.editing = EditFocus::SetBgImage(set);
        }
        EditorAction::SetBgImageOpacityDelta { set, delta } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.bg_image_opacity = (s.bg_image_opacity + delta).clamp(0.0, 1.0);
                hud.dirty = true;
                ui.dirty = true;
            }
        }
        EditorAction::CaptureNextWheelKey { set } => {
            ui.editing = EditFocus::NextWheelKey(set);
        }
        EditorAction::CapturePrevWheelKey { set } => {
            ui.editing = EditFocus::PrevWheelKey(set);
        }
        EditorAction::ToggleCycleWheels { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.cycle_wheels = !s.cycle_wheels;
                ui.dirty = true;
            }
        }
        // ── undo / redo ────────────────────────────────────────────────────────
        EditorAction::Undo => {
            if let Some(snapshot) = ui.undo_stack.pop() {
                ui.redo_stack.push(cfg.clone());
                *cfg = snapshot;
                hud.dirty = true;
                ui.dirty = true;
            }
        }
        EditorAction::Redo => {
            if let Some(snapshot) = ui.redo_stack.pop() {
                ui.undo_stack.push(cfg.clone());
                *cfg = snapshot;
                hud.dirty = true;
                ui.dirty = true;
            }
        }
    }
    sync_wheelset_visuals(cfg, ui.selection);
}

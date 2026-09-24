//! Keyboard, text, and gamepad input capture for the editor.

use super::*;
use super::{action_at, sync_wheelset_visuals, wheel_at};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;

fn focused_name<'a>(cfg: &'a mut QuickActionConfig, ui: &EditorUiState) -> Option<&'a mut String> {
    match ui.editing {
        EditFocus::Name => match ui.selection {
            Selection::Action { set, entry } => action_at(cfg, set, entry).map(|a| &mut a.name),
            _ => None,
        },
        EditFocus::SetName => match ui.selection {
            Selection::Set { set } => cfg.sets.get_mut(set).map(|s| &mut s.name),
            _ => None,
        },
        EditFocus::WheelName => wheel_at(cfg, ui.selection).map(|w| &mut w.name),
        EditFocus::SlotName(i) => {
            wheel_at(cfg, ui.selection).and_then(move |w| w.slots.get_mut(i).map(|s| &mut s.name))
        }
        EditFocus::SlotIcon(i) => {
            wheel_at(cfg, ui.selection).and_then(move |w| w.slots.get_mut(i).map(|s| &mut s.icon))
        }
        EditFocus::WheelSetName => match ui.selection {
            Selection::WheelSetEntry { set, entry } => cfg
                .sets
                .get_mut(set)
                .and_then(|s| s.entries.get_mut(entry))
                .and_then(|e| {
                    if let SetEntry::RadialMenuSet(ws) = e {
                        Some(&mut ws.name)
                    } else {
                        None
                    }
                }),
            _ => None,
        },
        EditFocus::SetBgImage(set) => cfg.sets.get_mut(set).map(|s| &mut s.bg_image),
        _ => None,
    }
}

// ─── keyboard input ───────────────────────────────────────────────────────────────

/// Returns the ordinal position of `entry` among Wheel/RadialMenuSetState entries in the set.
/// Used to sync `hud.active_wheel_entry` when the editor selects a wheel.
pub(super) fn wheel_entry_idx(cfg: &QuickActionConfig, set: usize, entry: usize) -> usize {
    let Some(s) = cfg.sets.get(set) else {
        return 0;
    };
    s.entries[..entry.min(s.entries.len())]
        .iter()
        .filter(|e| matches!(e, SetEntry::Wheel(_) | SetEntry::RadialMenuSet(_)))
        .count()
}

pub(super) fn editor_text_input(
    mut messages: MessageReader<KeyboardInput>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
) {
    if !matches!(
        ui.editing,
        EditFocus::Name
            | EditFocus::SetName
            | EditFocus::WheelName
            | EditFocus::SlotName(_)
            | EditFocus::SlotIcon(_)
            | EditFocus::WheelSetName
    ) {
        messages.clear();
        return;
    }

    let mut changed = false;
    let mut stop = false;
    for ev in messages.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        match &ev.logical_key {
            Key::Enter | Key::Escape => {
                stop = true;
                changed = true;
            }
            Key::Backspace => {
                if let Some(n) = focused_name(&mut cfg, &ui) {
                    n.pop();
                    changed = true;
                }
            }
            Key::Space => {
                if let Some(n) = focused_name(&mut cfg, &ui) {
                    if n.chars().count() < 24 {
                        n.push(' ');
                        changed = true;
                    }
                }
            }
            Key::Character(s) => {
                if let Some(n) = focused_name(&mut cfg, &ui) {
                    if n.chars().count() < 24 {
                        n.push_str(s);
                        changed = true;
                    }
                }
            }
            _ => {}
        }
    }
    if stop {
        ui.editing = EditFocus::None;
    }
    if changed {
        sync_wheelset_visuals(&mut cfg, ui.selection);
        ui.dirty = true;
        hud.dirty = true;
    }
}

pub(super) fn editor_capture_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
) {
    // This flag is scoped to one Update frame. The gamepad capture system runs
    // immediately after this one and may set it when no keyboard key matched.
    ui.capture_consumed = false;
    let focus = ui.editing;
    if !matches!(
        focus,
        EditFocus::Key
            | EditFocus::NextSetKey
            | EditFocus::PrevSetKey
            | EditFocus::WheelSetSwitchKey
            | EditFocus::EditShortcut
            | EditFocus::NextWheelKey(_)
            | EditFocus::PrevWheelKey(_)
            | EditFocus::WheelSetNextKey { .. }
            | EditFocus::WheelSetPrevKey { .. }
    ) {
        return;
    }

    for key in keys.get_just_pressed() {
        if is_modifier(*key) {
            continue;
        }
        if *key != KeyCode::Escape {
            let label = key_label(*key);
            match focus {
                EditFocus::Key => {
                    if let Selection::Action { set, entry } = ui.selection {
                        if let Some(a) = action_at(&mut cfg, set, entry) {
                            a.key = label;
                        }
                    }
                }
                EditFocus::WheelSetSwitchKey => {
                    if let Selection::WheelSetEntry { set, entry } = ui.selection {
                        if let Some(SetEntry::RadialMenuSet(ws)) =
                            cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                        {
                            ws.switch_key = label;
                        }
                    }
                }
                EditFocus::NextSetKey => cfg.next_set_key = label,
                EditFocus::PrevSetKey => cfg.prev_set_key = label,
                EditFocus::EditShortcut => cfg.edit_shortcut = label,
                EditFocus::NextWheelKey(set) => {
                    if let Some(s) = cfg.sets.get_mut(set) {
                        s.next_wheel_key = label;
                    }
                }
                EditFocus::PrevWheelKey(set) => {
                    if let Some(s) = cfg.sets.get_mut(set) {
                        s.prev_wheel_key = label;
                    }
                }
                EditFocus::WheelSetNextKey { set, entry } => {
                    if let Some(SetEntry::RadialMenuSet(ws)) =
                        cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                    {
                        ws.next_wheel_key = label;
                    }
                }
                EditFocus::WheelSetPrevKey { set, entry } => {
                    if let Some(SetEntry::RadialMenuSet(ws)) =
                        cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                    {
                        ws.prev_wheel_key = label;
                    }
                }
                _ => {}
            }
        }
        ui.capture_consumed = true;
        ui.editing = EditFocus::None;
        ui.dirty = true;
        return;
    }
}

pub(super) fn editor_capture_gamepad(
    gamepads: Query<&Gamepad>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
) {
    let focus = ui.editing;
    if !matches!(
        focus,
        EditFocus::Key
            | EditFocus::ButtonKey
            | EditFocus::NextSetKey
            | EditFocus::PrevSetKey
            | EditFocus::EditShortcut
            | EditFocus::WheelSetSwitchKey
            | EditFocus::WheelSetStick
            | EditFocus::WheelStick
            | EditFocus::NextWheelKey(_)
            | EditFocus::PrevWheelKey(_)
    ) {
        return;
    }
    // Skip the first frame after entering capture mode so that the South
    // button that activated the field is not immediately captured as input.
    if ui.capture_skip {
        ui.capture_skip = false;
        return;
    }

    // Radial-menu stick capture is intentionally axis-only. Buttons, d-pad,
    // triggers, and keyboard keys must never bind the stick selector.
    if matches!(ui.editing, EditFocus::WheelStick | EditFocus::WheelSetStick) {
        for gamepad in &gamepads {
            let left = GamepadAxis::LeftStickX;
            let right = GamepadAxis::RightStickX;
            let left_strength = gamepad
                .get(left)
                .unwrap_or(0.0)
                .abs()
                .max(gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0).abs());
            let right_strength = gamepad
                .get(right)
                .unwrap_or(0.0)
                .abs()
                .max(gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0).abs());
            let binding = if left_strength >= 0.6 && left_strength >= right_strength {
                Some("GP:LeftStick")
            } else if right_strength >= 0.6 {
                Some("GP:RightStick")
            } else {
                None
            };
            if let Some(binding) = binding {
                if ui.editing == EditFocus::WheelStick {
                    if let Some(w) = wheel_at(&mut cfg, ui.selection) {
                        w.stick_binding = binding.into();
                    }
                    sync_wheelset_visuals(&mut cfg, ui.selection);
                } else if let Selection::WheelSetEntry { set, entry } = ui.selection {
                    if let Some(SetEntry::RadialMenuSet(ws)) =
                        cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                    {
                        ws.stick_binding = binding.into();
                        for wheel in &mut ws.wheels {
                            wheel.stick_binding = ws.stick_binding.clone();
                        }
                    }
                }
                ui.editing = EditFocus::None;
                ui.capture_consumed = true;
                ui.dirty = true;
                hud.dirty = true;
                return;
            }
        }
        return;
    }
    const BUTTONS: &[GamepadButton] = &[
        GamepadButton::South,
        GamepadButton::East,
        GamepadButton::North,
        GamepadButton::West,
        GamepadButton::LeftTrigger,
        GamepadButton::RightTrigger,
        GamepadButton::LeftTrigger2,
        GamepadButton::RightTrigger2,
        GamepadButton::Start,
        GamepadButton::Select,
        GamepadButton::LeftThumb,
        GamepadButton::RightThumb,
        GamepadButton::DPadUp,
        GamepadButton::DPadDown,
        GamepadButton::DPadLeft,
        GamepadButton::DPadRight,
    ];
    for gamepad in &gamepads {
        for &btn in BUTTONS {
            if gamepad.just_pressed(btn) {
                let label = gamepad_btn_label(btn);
                let gp = format!("GP:{}", label);
                match focus {
                    EditFocus::Key | EditFocus::ButtonKey => {
                        if let Selection::Action { set, entry } = ui.selection {
                            if let Some(a) = action_at(&mut cfg, set, entry) {
                                a.key = gp;
                            }
                        }
                    }
                    EditFocus::WheelSetSwitchKey => {
                        if let Selection::WheelSetEntry { set, entry } = ui.selection {
                            if let Some(SetEntry::RadialMenuSet(ws)) =
                                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                            {
                                ws.switch_key = gp;
                            }
                        }
                    }
                    EditFocus::NextSetKey => cfg.next_set_key = gp,
                    EditFocus::PrevSetKey => cfg.prev_set_key = gp,
                    EditFocus::EditShortcut => cfg.edit_shortcut = gp,
                    EditFocus::NextWheelKey(set) => {
                        if let Some(s) = cfg.sets.get_mut(set) {
                            s.next_wheel_key = gp;
                        }
                    }
                    EditFocus::PrevWheelKey(set) => {
                        if let Some(s) = cfg.sets.get_mut(set) {
                            s.prev_wheel_key = gp;
                        }
                    }
                    EditFocus::WheelSetNextKey { set, entry } => {
                        if let Some(SetEntry::RadialMenuSet(ws)) =
                            cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                        {
                            ws.next_wheel_key = gp;
                        }
                    }
                    EditFocus::WheelSetPrevKey { set, entry } => {
                        if let Some(SetEntry::RadialMenuSet(ws)) =
                            cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                        {
                            ws.prev_wheel_key = gp;
                        }
                    }
                    _ => {}
                }
                ui.editing = EditFocus::None;
                ui.capture_consumed = true;
                ui.dirty = true;
                return;
            }
        }
    }
}

fn key_label(k: KeyCode) -> String {
    let dbg = format!("{:?}", k);
    let s = dbg.strip_prefix("Key").unwrap_or(&dbg);
    s.strip_prefix("Digit").unwrap_or(s).to_string()
}

fn gamepad_btn_label(btn: GamepadButton) -> String {
    match btn {
        GamepadButton::South => "A".into(),
        GamepadButton::East => "B".into(),
        GamepadButton::North => "Y".into(),
        GamepadButton::West => "X".into(),
        GamepadButton::LeftTrigger => "LB".into(),
        GamepadButton::RightTrigger => "RB".into(),
        GamepadButton::LeftTrigger2 => "LT".into(),
        GamepadButton::RightTrigger2 => "RT".into(),
        GamepadButton::Start => "Start".into(),
        GamepadButton::Select => "Select".into(),
        GamepadButton::LeftThumb => "LS".into(),
        GamepadButton::RightThumb => "RS".into(),
        GamepadButton::DPadUp => "DUp".into(),
        GamepadButton::DPadDown => "DDown".into(),
        GamepadButton::DPadLeft => "DLeft".into(),
        GamepadButton::DPadRight => "DRight".into(),
        _ => format!("{:?}", btn),
    }
}

fn is_modifier(k: KeyCode) -> bool {
    matches!(
        k,
        KeyCode::ShiftLeft
            | KeyCode::ShiftRight
            | KeyCode::ControlLeft
            | KeyCode::ControlRight
            | KeyCode::AltLeft
            | KeyCode::AltRight
            | KeyCode::SuperLeft
            | KeyCode::SuperRight
    )
}

/// Watches for the configured edit shortcut and toggles the editor sidebar.
// ─── shortcut helper ────────────────────────────────────────────────────────────
/// Returns `true` if `shortcut` was just pressed this frame.
/// Shortcut format: plain key label (e.g. `"Tab"`) or `"GP:{label}"` for gamepad.
pub(super) fn shortcut_just_pressed(
    shortcut: &str,
    keys: &ButtonInput<KeyCode>,
    gamepads: &Query<&Gamepad>,
) -> bool {
    if shortcut.is_empty() {
        return false;
    }
    if let Some(btn_name) = shortcut.strip_prefix("GP:") {
        const BTNS: &[GamepadButton] = &[
            GamepadButton::South,
            GamepadButton::East,
            GamepadButton::North,
            GamepadButton::West,
            GamepadButton::LeftTrigger,
            GamepadButton::RightTrigger,
            GamepadButton::LeftTrigger2,
            GamepadButton::RightTrigger2,
            GamepadButton::Start,
            GamepadButton::Select,
            GamepadButton::LeftThumb,
            GamepadButton::RightThumb,
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
        ];
        for &btn in BTNS {
            if gamepad_btn_label(btn) == btn_name && gamepads.iter().any(|gp| gp.just_pressed(btn))
            {
                return true;
            }
        }
        false
    } else {
        keys.get_just_pressed()
            .any(|&k| !is_modifier(k) && key_label(k) == shortcut)
    }
}

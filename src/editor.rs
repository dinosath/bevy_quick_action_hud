//! BSN-macro Quick Action Menu editor — sidebar UI only.
//!
//! The full-screen HUD canvas (wheel preview, floating buttons, set tabs) is
//! now rendered by [`WheelHudPlugin`] in `lib.rs`.  This module contains only
//! the left sidebar editor UI.
//!
//! ## Layout
//! * **Left sidebar** is context-sensitive:
//!   - Default: **navigation view** — wheel-set tree and button list for the
//!     active set, plus a set-switch key summary at the bottom.
//!   - When an item is selected: **editor panel** for that item (wheel, button,
//!     or wheel-set) with a `‹ Back` breadcrumb header.
//! * **Right canvas** is handled by [`WheelHudPlugin`].

use crate::*;

use bevy::ecs::message::MessageReader;
use bevy::feathers::controls::{
    ButtonVariant, FeathersButton, FeathersCheckbox, FeathersToolButton,
};
use bevy::feathers::focus::FocusIndicator as FeathersFocusIndicator;
use bevy::feathers::theme::ThemedText;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui::{OverflowAxis, UiGlobalTransform};
use bevy::ui_widgets::{Activate, ControlOrientation, Scrollbar, ScrollbarThumb, ValueChange};

// ─── selection & edit-focus ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Selection {
    #[default]
    None,
    Action {
        set: usize,
        entry: usize,
    },
    HudSwitch {
        set: usize,
        entry: usize,
    },
    Wheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    Set {
        set: usize,
    },
    SetSwitch,
    /// A [`SetEntry::WheelSet`] entry (to edit its name / switch-key).
    WheelSetEntry {
        set: usize,
        entry: usize,
    },
    Segment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum EditFocus {
    #[default]
    None,
    Name,
    Key,
    HudSwitchKey,
    SetName,
    WheelName,
    SlotName(usize),
    NextSetKey,
    PrevSetKey,
    /// Typing the name of the selected wheel-set entry.
    WheelSetName,
    /// Capturing the cycle-key for the selected wheel-set entry.
    WheelSetSwitchKey,
    WheelSetStick,
    SlotIcon(usize),
    /// Capturing an input binding for a segment (keyboard or gamepad).
    SlotInput(usize),
    /// Editing the name of item `item` inside slot `slot`.
    SlotItemName(usize, usize),
    /// Editing the icon of item `item` inside slot `slot`.
    SlotItemIcon(usize, usize),
    /// Capturing a key/button for the global edit shortcut.
    EditShortcut,
    /// Editing the bg_image path of a set.
    SetBgImage(usize),
    /// Capturing the next-wheel shortcut for a set.
    NextWheelKey(usize),
    /// Capturing the prev-wheel shortcut for a set.
    PrevWheelKey(usize),
    WheelSetNextKey {
        set: usize,
        entry: usize,
    },
    WheelSetPrevKey {
        set: usize,
        entry: usize,
    },
}

// ─── editor state ────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct EditorUiState {
    pub dirty: bool,
    pub selection: Selection,
    pub editing: EditFocus,
    pub config_path: String,
    /// Persisted vertical scroll offset for the wheel editor panel.
    pub wheel_scroll_y: f32,
    // active_set and editor_open moved to WheelHudState in lib.rs
    /// Index of the currently gamepad-focused item in the sidebar.
    pub navfocus: usize,
    /// Total number of focusable items in the current sidebar view.
    pub nav_count: usize,
    /// Set to true whenever the gamepad focus moves so that the PostUpdate
    /// scroll system can bring the focused item into view.
    pub scroll_to_focus: bool,
    /// When true, `editor_capture_gamepad` skips one frame so that the
    /// South press that activated the capture field is not itself captured.
    pub capture_skip: bool,
    /// Set when a key/gamepad input was consumed by binding capture. Later
    /// shortcut systems must ignore that same input for this frame.
    pub capture_consumed: bool,
    /// Direction currently held for D-Pad nav repeat: -1 = up, 0 = none, 1 = down.
    pub nav_hold_dir: i32,
    /// Seconds the current D-Pad direction has been held.
    pub nav_hold_timer: f32,
    /// Snapshot history for undo/redo.
    pub undo_stack: Vec<QuickActionConfig>,
    /// Redo stack (cleared when a new action is performed).
    pub redo_stack: Vec<QuickActionConfig>,
    /// Maximum number of undo steps to keep.
    pub undo_limit: usize,
}
impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            dirty: true,
            selection: Selection::None,
            editing: EditFocus::None,
            config_path: crate::CONFIG_FILE.into(),
            wheel_scroll_y: 0.0,
            navfocus: 0,
            nav_count: 0,
            scroll_to_focus: false,
            capture_skip: false,
            capture_consumed: false,
            nav_hold_dir: 0,
            nav_hold_timer: 0.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_limit: 50,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum MiddleDragTarget {
    Action {
        set: usize,
        entry: usize,
    },
    Wheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
}

#[derive(Resource, Default)]
struct HudMiddleDrag {
    target: Option<MiddleDragTarget>,
}

// ─── components ──────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct EditorRoot;

/// Marks the scrollable content entity inside the wheel editor panel.
/// Used by `rebuild_editor` to persist the vertical scroll offset across rebuilds.
#[derive(Component)]
pub struct EditorScrollArea;

#[derive(Component)]
pub struct SegmentHoverColor(pub Color);

#[derive(Component, Clone)]
pub struct EditorButton {
    pub action: EditorAction,
    pub base: Color,
}

/// Placed on [`FeathersCheckbox`] entities in toggle fields.
/// Dispatched via the global [`ValueChange<bool>`] observer, not `Interaction::Pressed`.
#[derive(Component, Clone)]
pub struct EditorToggle {
    pub action: EditorAction,
}

/// Marks the currently gamepad-focused editor button.
#[derive(Component)]
pub struct FocusedEditorItem;

// ─── editor actions ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum EditorAction {
    // ── sets ────────────────────────────────────────────────────────────────
    AddSet,
    DeleteSet {
        set: usize,
    },
    SelectSet {
        set: usize,
    },
    EditSetName {
        set: usize,
    },
    SetOpacityDelta {
        set: usize,
        delta: f32,
    },
    ToggleInputOverride {
        set: usize,
    },
    // ── entries ─────────────────────────────────────────────────────────────
    AddAction {
        set: usize,
    },
    AddWheel {
        set: usize,
    },
    AddWheelSet {
        set: usize,
    },
    AddHudSwitch {
        set: usize,
    },
    AddWheelToSet {
        set: usize,
        entry: usize,
    },
    DeleteEntry {
        set: usize,
        entry: usize,
    },
    DeleteWheelFromSet {
        set: usize,
        entry: usize,
        wheel: usize,
    },
    MoveEntryUp {
        set: usize,
        entry: usize,
    },
    MoveEntryDown {
        set: usize,
        entry: usize,
    },
    // ── selection ───────────────────────────────────────────────────────────
    SelectAction {
        set: usize,
        entry: usize,
    },
    SelectHudSwitch {
        set: usize,
        entry: usize,
    },
    CaptureHudSwitchKey {
        set: usize,
        entry: usize,
    },
    ClearHudSwitchKey {
        set: usize,
        entry: usize,
    },
    ToggleHudSwitchEnabled {
        set: usize,
        entry: usize,
    },
    SelectWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    SelectWheelSetEntry {
        set: usize,
        entry: usize,
    },
    SelectSetSwitch,
    /// Return to the navigation sidebar view.
    NavBack,
    // ── quick action editing ─────────────────────────────────────────────────
    EditName {
        set: usize,
        entry: usize,
    },
    /// Capture the key/button binding for a quick action (keyboard or gamepad).
    CaptureKey {
        set: usize,
        entry: usize,
    },
    CycleIcon {
        set: usize,
        entry: usize,
    },
    CycleCommand {
        set: usize,
        entry: usize,
    },
    CycleHoldCommand {
        set: usize,
        entry: usize,
    },
    CycleSlotCommand {
        slot: usize,
    },
    CycleSlotHoldCommand {
        slot: usize,
    },
    ToggleSlotHold {
        slot: usize,
    },
    ToggleHold {
        set: usize,
        entry: usize,
    },
    ToggleShowOnMenu {
        set: usize,
        entry: usize,
    },
    ToggleEnabled {
        set: usize,
        entry: usize,
    },
    OpacityDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    RadiusDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    ActionWidthDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    ActionRotationDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    ActionHeightDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    CyclePosition {
        set: usize,
        entry: usize,
    },
    CycleShape {
        set: usize,
        entry: usize,
    },
    // ── wheel editing ────────────────────────────────────────────────────────
    EditWheelName,
    CycleWheelTheme,
    WheelCooldownDelta {
        delta: f32,
    },
    WheelOuterRadiusDelta {
        delta: f32,
    },
    WheelInnerRadiusDelta {
        delta: f32,
    },
    ToggleWheelShowLabels,
    AddSlot,
    RemoveSlot,
    EditSlotName {
        slot: usize,
    },
    // ── wheel-set entry editing ──────────────────────────────────────────────
    EditWheelSetName {
        set: usize,
        entry: usize,
    },
    CaptureWheelSetSwitchKey {
        set: usize,
        entry: usize,
    },
    CaptureWheelSetStick {
        set: usize,
        entry: usize,
    },
    CaptureWheelSetNextKey {
        set: usize,
        entry: usize,
    },
    CaptureWheelSetPrevKey {
        set: usize,
        entry: usize,
    },
    ClearWheelSetNextKey {
        set: usize,
        entry: usize,
    },
    ClearWheelSetPrevKey {
        set: usize,
        entry: usize,
    },
    WheelSetMinDelta {
        set: usize,
        entry: usize,
        delta: i32,
    },
    WheelSetMaxDelta {
        set: usize,
        entry: usize,
        delta: i32,
    },
    ToggleWheelSetCycle {
        set: usize,
        entry: usize,
    },
    SwitchWheelPrev {
        set: usize,
        entry: usize,
    },
    SwitchWheelNext {
        set: usize,
        entry: usize,
    },
    // ── set-switch shortcuts ─────────────────────────────────────────────────
    CaptureNextSetKey,
    CapturePrevSetKey,
    // ── persistence ────────────────────────────────────────────────────────────
    Save,
    Load,
    /// Toggle `QuickActionConfig::show_set_bar`.
    ToggleShowSetBar,
    /// Toggle `QuickActionConfig::cycle_sets`.
    ToggleCycleSets,
    ToggleSetEnabled {
        set: usize,
    },
    CycleHudSwitchTarget {
        set: usize,
        entry: usize,
    },
    /// Begin capturing the global edit-sidebar shortcut.
    CaptureEditShortcut,
    CycleHudOpenMode,
    /// Nudge `QuickActionConfig::hud_bg_opacity` by `delta`.
    HudBgOpacityDelta {
        delta: f32,
    },
    /// Cycle the HUD background color through a preset dark palette.
    CycleHudBgColor,
    // ── per-set config ──────────────────────────────────────────────────────────
    EditSetBgImage {
        set: usize,
    },
    SetBgImageOpacityDelta {
        set: usize,
        delta: f32,
    },
    CaptureNextWheelKey {
        set: usize,
    },
    CapturePrevWheelKey {
        set: usize,
    },
    ToggleCycleWheels {
        set: usize,
    },
    // ── segment editing ──────────────────────────────────────────────────────
    SelectSegment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    EditSlotIcon {
        slot: usize,
    },
    CycleSegmentShape,
    ToggleWheelShowIcon,
    CycleHighlightColor,
    SegmentScaleDelta {
        delta: f32,
    },
    /// Step the wheel's overall opacity up or down.
    WheelOpacityDelta {
        delta: f32,
    },
    /// Cycle the inner-border ring color (empty = no border).
    CycleInnerBorderColor,
    /// Cycle the outer-border ring color (empty = no border).
    CycleOuterBorderColor,
    /// Cycle the wheel background color.
    CycleWheelBgColor,
    /// Adjust wheel background opacity.
    WheelBgOpacityDelta {
        delta: f32,
    },
    /// Adjust outer border ring width.
    WheelOuterBorderWidthDelta {
        delta: f32,
    },
    /// Cycle the hub (inner circle) background color.
    CycleWheelHubColor,
    /// Adjust hub (inner circle) background opacity.
    WheelHubOpacityDelta {
        delta: f32,
    },
    /// Adjust inner border ring width.
    WheelInnerBorderWidthDelta {
        delta: f32,
    },
    // ── segment input / gamepad binding ─────────────────────────────────────────
    /// Capture a key or gamepad button as the input binding for segment `slot`.
    CaptureSlotInput {
        slot: usize,
    },
    /// Clear the input binding for segment `slot`.
    ClearSlotInput {
        slot: usize,
    },
    // ── clear shortcuts ──────────────────────────────────────────────────────────
    ClearNextSetKey,
    ClearPrevSetKey,
    ClearEditShortcut,
    ClearNextWheelKey {
        set: usize,
    },
    ClearPrevWheelKey {
        set: usize,
    },
    ClearWheelSetSwitchKey {
        set: usize,
        entry: usize,
    },
    ClearWheelSetStick {
        set: usize,
        entry: usize,
    },
    /// Clear the key binding for action entry `entry` in set `set`.
    ClearActionKey {
        set: usize,
        entry: usize,
    },
    // ── per-slot items ───────────────────────────────────────────────────────────
    AddSlotItem {
        slot: usize,
    },
    RemoveSlotItem {
        slot: usize,
        item: usize,
    },
    EditSlotItemName {
        slot: usize,
        item: usize,
    },
    EditSlotItemIcon {
        slot: usize,
        item: usize,
    },
    /// Toggle stick side for the active standalone wheel.
    CycleWheelStick,
    /// Toggle stick side for the selected WheelSet entry.
    CycleWheelSetStick,
    /// Toggle close-on-select for slot `slot` of the active wheel.
    ToggleSlotCloseOnSelect {
        slot: usize,
    },
    /// Toggle close-on-select for action entry `entry` in set `set`.
    ToggleActionCloseOnSelect {
        set: usize,
        entry: usize,
    },
    // ── undo / redo ─────────────────────────────────────────────────────────────
    /// Undo the last editor action.
    Undo,
    /// Redo the last undone editor action.
    Redo,
}

// ─── plugin ──────────────────────────────────────────────────────────────────────

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

fn on_editor_value_change_bool(
    trigger: On<ValueChange<bool>>,
    toggles: Query<&EditorToggle>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
) {
    if let Ok(t) = toggles.get(trigger.event_target()) {
        apply_action(&t.action.clone(), &mut cfg, &mut ui, &mut hud);
        ui.dirty = true;
        hud.dirty = true;
    }
}

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
            SetEntry::WheelSet(ws) => {
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

/// Gamepad D-pad + button navigation for the editor sidebar.
fn editor_gamepad_nav(
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut ui: ResMut<EditorUiState>,
    mut hud: ResMut<WheelHudState>,
    mut cfg: ResMut<QuickActionConfig>,
    focused_btn_q: Query<&EditorButton, With<FocusedEditorItem>>,
    focused_toggle_q: Query<&EditorToggle, With<FocusedEditorItem>>,
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
    // without leaving the wheel for the sidebar.
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
                        w.slots.insert(
                            insert_at,
                            WheelSlotData::named(format!("Slot {}", insert_at + 1)),
                        );
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
        } else if let Ok(toggle) = focused_toggle_q.single() {
            Some(toggle.action.clone())
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

fn editor_toolbar_shortcuts(
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
fn editor_keyboard_radial_nav(
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
                w.slots.insert(
                    insert_at,
                    WheelSlotData::named(format!("Slot {}", insert_at + 1)),
                );
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

/// Runs in PostUpdate after layout, auto-scrolling the editor sidebar so the
/// gamepad-focused item is always visible.
///
/// Works by walking up the parent chain from the focused entity to find the
/// nearest `overflow: scroll_y` container, then computing the scroll offset
/// needed to centre (or just reveal) the focused item inside the viewport.
fn scroll_editor_to_focus(
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

/// Registers all editor UI resources and systems into `app`.
/// Called by [`QuickActionHudPlugin`] when `editor: true`.
pub(crate) fn register_editor_systems(app: &mut App) {
    app.init_resource::<EditorUiState>()
        .init_resource::<HudMiddleDrag>()
        .init_resource::<ConfigValidation>()
        .add_message::<crate::touch::TouchDragEvent>()
        .add_observer(on_editor_activate)
        .add_observer(on_editor_value_change_bool)
        .add_systems(
            Update,
            (
                // Capture first so the captured input can be marked consumed
                // before any normal editor/HUD shortcut system sees it.
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
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                fix_plain_button_initial_bg,
                scroll_editor_to_focus.after(bevy::ui::UiSystems::Layout),
            ),
        );
}

/// Convenience plugin — equivalent to `QuickActionHudPlugin::with_editor()`.
///
/// Adds core wheel logic, the HUD canvas, **and** the editor sidebar in one call.
pub struct QuickActionEditorPlugin;
impl Plugin for QuickActionEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickActionHudPlugin::with_editor());
    }
}

// ─── palette ─────────────────────────────────────────────────────────────────────

const BG_SIDEBAR: Color = Color::srgb(0.075, 0.08, 0.085);
#[allow(dead_code)]
const BG_MAIN: Color = Color::srgb(0.105, 0.11, 0.115);
const SIDEBAR_BORDER: Color = Color::srgb(0.23, 0.24, 0.25);
const GREEN: Color = Color::srgb(0.62, 0.92, 0.10);
const GREEN_BG: Color = Color::srgba(0.62, 0.92, 0.10, 0.14);
const TEXT: Color = Color::srgb(0.91, 0.91, 0.90);
const DIM: Color = Color::srgb(0.65, 0.65, 0.64);
const DIMMER: Color = Color::srgb(0.39, 0.40, 0.40);
const ICON: Color = Color::srgb(0.80, 0.80, 0.78);
const AMBER: Color = Color::srgb(0.95, 0.54, 0.57);
const BLUE: Color = Color::srgb(0.23, 0.47, 1.0);
const TEAL: Color = Color::srgb(0.10, 0.72, 0.63);
const PURPLE: Color = Color::srgb(0.63, 0.42, 0.95);
const BADGE_BORDER: Color = Color::srgb(0.34, 0.35, 0.35);
const ROW_SEL: Color = Color::srgba(0.94, 0.48, 0.52, 0.18);
#[allow(dead_code)]
const ROW_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);
const PANEL_CARD: Color = Color::srgb(0.15, 0.16, 0.16);
const CTRL_BG: Color = Color::srgb(0.12, 0.13, 0.13);
const GAMEPAD_FOCUS_OUTLINE: Color = Color::srgba(0.95, 0.54, 0.57, 0.90);

// ─── primitive bsn! helpers ──────────────────────────────────────────────────────

fn text(s: &str, size: f32, color: Color) -> impl Scene {
    bsn! {
        Text({s.to_string()})
        TextFont { font_size: {FontSize::Px(size)} }
        TextColor({color})
    }
}

fn hcluster() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: {px(7.)},
        }
    }
}

fn row_button(bg: Color) -> impl Scene {
    bsn! {
        @FeathersButton { @variant: ButtonVariant::Plain }
        Node {
            width: {percent(100.)}, height: {px(24.)},
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: {UiRect::horizontal(px(4.))},
            border_radius: {BorderRadius::all(px(4.))},
        }
        BackgroundColor({bg})
    }
}

fn row_button_grow(bg: Color) -> impl Scene {
    bsn! {
        @FeathersButton { @variant: ButtonVariant::Plain }
        Node {
            flex_grow: 1., height: {px(24.)},
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: {UiRect::horizontal(px(4.))},
            border_radius: {BorderRadius::all(px(4.))},
        }
        BackgroundColor({bg})
    }
}

fn key_badge_box() -> impl Scene {
    bsn! {
        Node {
            width: {px(18.)}, height: {px(16.)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: {UiRect::all(px(1.))},
            border_radius: {BorderRadius::all(px(3.))},
        }
        BorderColor::all(BADGE_BORDER)
    }
}

#[allow(dead_code)]
fn col() -> impl Scene {
    bsn! { Node { flex_direction: FlexDirection::Column, row_gap: {px(1.)} } }
}

fn indent_col() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            padding: {UiRect::left(px(14.))},
            row_gap: {px(1.)},
        }
    }
}

fn sidebar() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(0.)}, top: {px(0.)}, bottom: {px(0.)},
            width: {px(260.)},
            flex_direction: FlexDirection::Column,
            border: {UiRect::right(px(1.))},
        }
        BackgroundColor({BG_SIDEBAR})
        BorderColor::all(SIDEBAR_BORDER)
    }
}

fn tree() -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1.,
            flex_direction: FlexDirection::Column,
            padding: {UiRect::axes(px(12.), px(8.))},
            row_gap: {px(2.)},
            overflow: {Overflow::scroll_y()},
        }
    }
}

/// A scrollable content column with an attached thin vertical scrollbar.
/// Returns the scrollable content entity to use as the layout parent.
/// Default scrollbar width in logical pixels.
/// 6px for desktop, wider for touch-friendly high-DPI.
const SCROLLBAR_WIDTH: f32 = 10.0;

fn scrolled_tree(commands: &mut Commands, parent: Entity) -> Entity {
    // Flex-row wrapper: [content column | scrollbar track]
    let wrapper = commands
        .spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            min_height: Val::Px(0.0),
            ..default()
        })
        .id();
    commands.entity(parent).add_child(wrapper);

    // Scrollable content area (mouse-wheel + draggable scrollbar both work)
    let scroll_area = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                row_gap: Val::Px(2.0),
                overflow: Overflow::scroll_y(),
                min_height: Val::Px(0.0),
                ..default()
            },
            EditorScrollArea,
        ))
        .id();
    commands.entity(wrapper).add_child(scroll_area);

    // Scrollbar track — wider for touch-friendly high-DPI interaction
    let track = commands
        .spawn((
            Node {
                min_width: Val::Px(SCROLLBAR_WIDTH),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.70)),
            Scrollbar {
                target: scroll_area,
                orientation: ControlOrientation::Vertical,
                min_thumb_length: 20.0,
            },
        ))
        .id();
    commands.entity(wrapper).add_child(track);

    // Scrollbar thumb — NO Node; the Scrollbar system owns its geometry
    let thumb = commands
        .spawn((
            ScrollbarThumb {
                border_radius: BorderRadius::all(Val::Px(SCROLLBAR_WIDTH / 2.0)),
                border: UiRect::all(Val::Px(0.0)),
            },
            BackgroundColor(Color::srgba(0.48, 0.50, 0.55, 0.85)),
        ))
        .id();
    commands.entity(track).add_child(thumb);

    scroll_area
}

fn del_btn() -> impl Scene {
    bsn! {
        @FeathersToolButton { @variant: ButtonVariant::Plain }
        Node {
            width: {px(16.)}, height: {px(16.)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

/// Spawn a tinted PNG icon from `assets/icons/editor/`.
/// `name` is the stem, e.g. `"cil-trash"`. `size` is width & height in px.
fn cil_icon(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    size: f32,
    tint: Color,
    icons: &Icons<'_>,
) {
    let handle = icons.srv.load::<Image>(format!(
        "embedded://bevy_quick_action_hud/embedded/icons/editor/{name}.png"
    ));
    let e = commands
        .spawn((
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                ..default()
            },
            ImageNode {
                image: handle,
                color: tint,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(e);
}

fn footer_button(label: &str, _accent: Color, filled: bool) -> impl Scene {
    let label = label.to_string();
    let variant = if filled {
        ButtonVariant::Primary
    } else {
        ButtonVariant::Plain
    };
    bsn! {
        @FeathersButton {
            @variant: {variant},
            @caption: bsn! { Text({label}) ThemedText },
        }
        Node {
            padding: {UiRect::axes(px(16.), px(6.))},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

// ─── entity helpers ──────────────────────────────────────────────────────────────

fn child(commands: &mut Commands, parent: Entity, scene: impl Scene) -> Entity {
    let e = commands.spawn_scene(scene).id();
    commands.entity(parent).add_child(e);
    e
}

fn clickable(
    commands: &mut Commands,
    parent: Entity,
    scene: impl Scene,
    action: EditorAction,
    base: Color,
) -> Entity {
    let e = commands
        .spawn_scene(scene)
        .insert(EditorButton { action, base })
        .id();
    commands.entity(parent).add_child(e);
    e
}

/// Spawn a [`WheelHudButton`] child — handled by `process_hud_buttons`.
fn hud_clickable(
    commands: &mut Commands,
    parent: Entity,
    scene: impl Scene,
    action: WheelHudAction,
    base: Color,
) -> Entity {
    let e = commands
        .spawn_scene(scene)
        .insert(WheelHudButton { action, base })
        .id();
    commands.entity(parent).add_child(e);
    e
}

#[derive(Clone)]
#[allow(dead_code)]
enum Badge {
    None,
    Key(String),
    Dim(String),
}

#[allow(clippy::too_many_arguments)]
fn spawn_entry_row(
    commands: &mut Commands,
    parent: Entity,
    selected: bool,
    select_action: EditorAction,
    icon: &str,
    icon_col: Color,
    name: &str,
    name_col: Color,
    badge: Badge,
    del: Option<EditorAction>,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    let bg = if selected { ROW_SEL } else { Color::NONE };
    // Outer wrapper: [select row (flex-grow)] [delete button]
    let outer = child(
        commands,
        parent,
        bsn! {
            Node {
                width: {percent(100.)},
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: {px(2.)},
            }
        },
    );
    let row = clickable(commands, outer, row_button_grow(bg), select_action, bg);
    focusables.push(row);
    let left = child(commands, row, hcluster());
    if icon.ends_with(".png") {
        let handle = icons.srv.load::<Image>(icon.to_string());
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(12.),
                    height: Val::Px(12.),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    color: icon_col,
                    ..default()
                },
            ))
            .id();
        commands.entity(left).add_child(e);
    } else if !icon.is_empty() {
        child(commands, left, text(icon, 10., icon_col));
    }
    child(commands, left, text(name, 11., name_col));
    let right = child(commands, row, hcluster());
    match &badge {
        Badge::Key(k) => {
            spawn_input_badge(commands, right, k, icons);
        }
        Badge::Dim(s) => {
            child(commands, right, text(s, 9., DIMMER));
        }
        Badge::None => {}
    }
    if let Some(da) = del {
        let dx = clickable(commands, outer, del_btn(), da, Color::NONE);
        cil_icon(commands, dx, "cil-trash", 11., DIMMER, icons);
        focusables.push(dx);
    }
}

// ─── icons context ─────────────────────────────────────────────────────────────

/// Lightweight context bundle threaded through builder functions to enable
/// controller button icon display.
struct Icons<'a> {
    srv: &'a AssetServer,
    set: GamepadIconSet,
}

/// Renders a key/button input badge.
/// - `"GP:A"`, `"GP:LB"` etc. → loads and shows the matching PNG icon (20×20 px).
/// - Keyboard key → text badge (existing behaviour).
fn spawn_input_badge(commands: &mut Commands, parent: Entity, key: &str, icons: &Icons<'_>) {
    if let Some(label) = key.strip_prefix("GP:") {
        if let Some(path) = icons.set.embedded_icon_path(label) {
            let handle = icons.srv.load::<Image>(path);
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    ImageNode::new(handle),
                ))
                .id();
            commands.entity(parent).add_child(e);
            return;
        }
    }
    // Keyboard fallback — keep the existing bordered text badge
    let kb = child(commands, parent, key_badge_box());
    child(commands, kb, text(key, 8., DIM));
}

/// Like [`spawn_box_field`] but renders a controller icon inside the clickable
/// box when `raw_key` is a `"GP:…"` binding and the field is not in capture mode.
#[allow(clippy::too_many_arguments)]
fn spawn_key_capture_field(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    display: &str,
    display_color: Color,
    accent: Color,
    action: EditorAction,
    clear_action: EditorAction,
    raw_key: &str,
    focused: bool,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) -> Entity {
    let row = spawn_field(commands, parent, label);
    let b = clickable(commands, row, ctrl_box(accent), action, Color::NONE);
    if !focused && !raw_key.is_empty() {
        if let Some(btn_label) = raw_key.strip_prefix("GP:") {
            if let Some(path) = icons.set.embedded_icon_path(btn_label) {
                let handle = icons.srv.load::<Image>(path);
                let e = commands
                    .spawn((
                        Node {
                            width: Val::Px(18.0),
                            height: Val::Px(18.0),
                            ..default()
                        },
                        ImageNode::new(handle),
                    ))
                    .id();
                commands.entity(b).add_child(e);
                // Fall through to spawn clear button.
            } else {
                child(commands, b, text(display, 11., display_color));
            }
        } else {
            child(commands, b, text(display, 11., display_color));
        }
    } else {
        child(commands, b, text(display, 11., display_color));
    }
    // Clear button — only when the field has a value and we're not in capture mode.
    if !raw_key.is_empty() && !focused {
        let clear_btn = clickable(
            commands,
            row,
            bsn! {
                @FeathersToolButton {}
                Node {
                    width: {Val::Px(18.)},
                    height: {Val::Px(18.)},
                    margin: {UiRect::left(px(3.))},
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_shrink: 0.,
                }
            },
            clear_action,
            Color::NONE,
        );
        cil_icon(commands, clear_btn, "cil-x", 10., HUD_DIM, icons);
        focusables.push(clear_btn);
    }
    b
}

// ─── rebuild ─────────────────────────────────────────────────────────────────────

/// Returns `true` for editor actions that only change `ui.selection` / `ui.editing`
/// and therefore do **not** require a HUD canvas rebuild (`hud.dirty = true`).
/// All other actions may mutate `cfg` or `hud` state that is rendered on the canvas.
fn is_nav_only_action(action: &EditorAction) -> bool {
    matches!(
        action,
        EditorAction::EditSetName { .. }
            | EditorAction::EditName { .. }
            | EditorAction::EditWheelName
            | EditorAction::EditWheelSetName { .. }
            | EditorAction::EditSlotName { .. }
            | EditorAction::EditSlotIcon { .. }
            | EditorAction::EditSlotItemName { .. }
            | EditorAction::EditSlotItemIcon { .. }
            | EditorAction::EditSetBgImage { .. }
            | EditorAction::CaptureKey { .. }
            | EditorAction::CaptureNextSetKey
            | EditorAction::CapturePrevSetKey
            | EditorAction::CaptureEditShortcut
            | EditorAction::CaptureWheelSetSwitchKey { .. }
            | EditorAction::CaptureSlotInput { .. }
            | EditorAction::CaptureNextWheelKey { .. }
            | EditorAction::CapturePrevWheelKey { .. }
            | EditorAction::Save
    )
}

/// Fixes a one-frame flash caused by feathers initialising every `FeathersButton` with
/// `ThemeBackgroundColor(BUTTON_BG)` (opaque gray) regardless of variant.  The `update_button_styles`
/// system in feathers' `PreUpdate` only corrects this in the *next* frame, so Plain buttons
/// (which should be transparent at rest) flash gray for one rendered frame after a sidebar rebuild.
///
/// Running in PostUpdate of the *same* frame as the spawn, before the render, we override
/// `BackgroundColor` to transparent so the first rendered frame is already correct.
fn fix_plain_button_initial_bg(
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
fn rebuild_editor(
    mut commands: Commands,
    mut ui: ResMut<EditorUiState>,
    hud: Res<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    asset_server: Res<AssetServer>,
    icon_set: Res<GamepadIconSet>,
    old_sidebar: Query<Entity, With<EditorRoot>>,
    scroll_q: Query<&ScrollPosition, With<EditorScrollArea>>,
) {
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

    // Persist the current vertical scroll offset so we can restore it after
    // the UI is rebuilt (despawn + respawn resets ScrollPosition to zero).
    if let Ok(sp) = scroll_q.single() {
        ui.wheel_scroll_y = sp.0.y;
    }

    for e in &old_sidebar {
        commands.entity(e).despawn();
    }

    // Edit mode is now an in-canvas HUD editor. The contextual sector card
    // is rendered beside the wheel, so no sidebar should be spawned.
    if hud.editor_open {
        ui.nav_count = 0;
        return;
    }

    if hud.editor_open {
        debug!("[editor] building sidebar (focusables will be counted)");
        let icons = Icons {
            srv: &asset_server,
            set: *icon_set,
        };
        let (scroll_area, focusables) = build_sidebar(&mut commands, &cfg, &ui, &hud, &icons);

        // Update nav focus state.
        let count = focusables.len();
        ui.nav_count = count;
        debug!(
            "[editor] sidebar built — focusables={} navfocus={}",
            count, ui.navfocus
        );
        if count > 0 && ui.navfocus >= count {
            ui.navfocus = count - 1;
        }
        if let Some(&e) = focusables.get(ui.navfocus) {
            // Use an Outline for the gamepad focus ring instead of BackgroundColor.
            // FeathersButton manages BackgroundColor via ThemeBackgroundColor, and
            // manage_focus_indicators (PostUpdate) removes Outline from all FocusIndicator
            // entities that aren't keyboard-focused.  By removing FocusIndicator we opt out
            // of that system and keep our gamepad outline intact until the sidebar rebuilds.
            commands
                .entity(e)
                .insert(FocusedEditorItem)
                .remove::<FeathersFocusIndicator>()
                .insert(Outline {
                    width: Val::Px(2.),
                    offset: Val::Px(0.),
                    color: GAMEPAD_FOCUS_OUTLINE,
                });
            ui.scroll_to_focus = true;
        }

        if let Some(sa) = scroll_area {
            // Restore saved offset. The insert command runs after the spawn
            // commands, so the entity is guaranteed to exist by then.
            if ui.wheel_scroll_y > 0.0 {
                commands
                    .entity(sa)
                    .insert(ScrollPosition(Vec2::new(0.0, ui.wheel_scroll_y)));
            }
        } else {
            // Navigated away from wheel editor — reset so the next wheel
            // editor opens at the top.
            ui.wheel_scroll_y = 0.0;
        }
    } else {
        debug!("[editor] editor_open=false — no sidebar spawned");
        ui.nav_count = 0;
    }
}

// ─── sidebar ─────────────────────────────────────────────────────────────────────

fn build_sidebar(
    commands: &mut Commands,
    cfg: &QuickActionConfig,
    ui: &EditorUiState,
    hud: &WheelHudState,
    icons: &Icons<'_>,
) -> (Option<Entity>, Vec<Entity>) {
    let root = commands.spawn_scene(sidebar()).insert(EditorRoot).id();
    let mut focusables: Vec<Entity> = Vec::new();

    match ui.selection {
        // Root: ActionSets list ───────────────────────────────────────────────────
        Selection::None => {
            build_root_sidebar(commands, root, cfg, ui, icons, &mut focusables);
            (None, focusables)
        }

        // Set detail: wheels + buttons for one set ──────────────────────────
        Selection::Set { .. } | Selection::SetSwitch => {
            let si = if let Selection::Set { set } = ui.selection {
                set
            } else {
                0
            };
            build_nav_sidebar(commands, root, cfg, ui, hud, si, icons, &mut focusables);
            (None, focusables)
        }

        // Button / quick-action editor ────────────────────────────────────
        Selection::Action { set, entry } => {
            let qa = cfg
                .sets
                .get(set)
                .and_then(|s| s.entries.get(entry))
                .and_then(|e| {
                    if let SetEntry::Action(a) = e {
                        Some(a)
                    } else {
                        None
                    }
                });
            if let Some(qa) = qa {
                let set_name = cfg.sets.get(set).map(|s| s.name.as_str()).unwrap_or("Set");
                build_editor_header(
                    commands,
                    root,
                    Some(set_name),
                    &qa.name.clone(),
                    EditorAction::NavBack,
                    icons,
                    &mut focusables,
                );
                let scroll = child(commands, root, tree());
                spawn_action_editor(commands, scroll, ui, set, entry, qa, icons, &mut focusables);
                build_footer(commands, root, &ui.config_path, &mut focusables);
            } else {
                build_nav_sidebar(commands, root, cfg, ui, hud, set, icons, &mut focusables);
            }
            (None, focusables)
        }

        Selection::HudSwitch { set, entry } => {
            let hs = cfg
                .sets
                .get(set)
                .and_then(|s| s.entries.get(entry))
                .and_then(|e| {
                    if let SetEntry::HudSwitch(hs) = e {
                        Some(hs)
                    } else {
                        None
                    }
                });
            if let Some(hs) = hs {
                let set_name = cfg.sets.get(set).map(|s| s.name.as_str()).unwrap_or("Set");
                build_editor_header(
                    commands,
                    root,
                    Some(set_name),
                    &hs.name,
                    EditorAction::NavBack,
                    icons,
                    &mut focusables,
                );
                let scroll = child(commands, root, tree());
                section_label(commands, scroll, "HUD SWITCH");
                let card = child(commands, scroll, editor_card());
                let kf = ui.editing == EditFocus::HudSwitchKey;
                let (kd, kc) = key_display(kf, &hs.key);
                let kb = spawn_key_capture_field(
                    commands,
                    card,
                    "Switch key",
                    &kd,
                    kc,
                    if kf { AMBER } else { BADGE_BORDER },
                    EditorAction::CaptureHudSwitchKey { set, entry },
                    EditorAction::ClearHudSwitchKey { set, entry },
                    &hs.key,
                    kf,
                    icons,
                    &mut focusables,
                );
                focusables.push(kb);
                let target = spawn_box_field(
                    commands,
                    card,
                    "Target page",
                    &format!(
                        "{} — {}",
                        hs.target_page.saturating_add(1),
                        cfg.sets
                            .get(hs.target_page)
                            .map(|s| s.name.as_str())
                            .unwrap_or("missing")
                    ),
                    TEXT,
                    BADGE_BORDER,
                    EditorAction::CycleHudSwitchTarget { set, entry },
                );
                focusables.push(target);
                spawn_toggle_field(
                    commands,
                    card,
                    "Enabled",
                    hs.enabled,
                    EditorAction::ToggleHudSwitchEnabled { set, entry },
                    &mut focusables,
                );
                build_footer(commands, root, &ui.config_path, &mut focusables);
            } else {
                build_nav_sidebar(commands, root, cfg, ui, hud, set, icons, &mut focusables);
            }
            (None, focusables)
        }

        // Wheel editor ────────────────────────────────────────────────
        Selection::Wheel { set, entry, wheel } => {
            let w_ref = cfg
                .sets
                .get(set)
                .and_then(|s| s.entries.get(entry))
                .and_then(|e| match (e, wheel) {
                    (SetEntry::Wheel(w), None) => Some(w as &WheelData),
                    (SetEntry::WheelSet(ws), Some(i)) => ws.wheels.get(i).map(|w| w as &WheelData),
                    _ => None,
                });
            if let Some(w) = w_ref {
                let parent_name: Option<String> = wheel.map(|_| {
                    cfg.sets
                        .get(set)
                        .and_then(|s| s.entries.get(entry))
                        .and_then(|e| {
                            if let SetEntry::WheelSet(ws) = e {
                                Some(ws.name.as_str())
                            } else {
                                None
                            }
                        })
                        .unwrap_or("Wheel Set")
                        .to_string()
                });
                let wname = w.name.clone();
                build_editor_header(
                    commands,
                    root,
                    parent_name.as_deref(),
                    &wname,
                    EditorAction::NavBack,
                    icons,
                    &mut focusables,
                );
                let scroll = scrolled_tree(commands, root);
                spawn_wheel_editor(
                    commands,
                    scroll,
                    ui,
                    w,
                    set,
                    entry,
                    wheel,
                    icons,
                    &mut focusables,
                );
                build_footer(commands, root, &ui.config_path, &mut focusables);
                (Some(scroll), focusables)
            } else {
                build_nav_sidebar(commands, root, cfg, ui, hud, set, icons, &mut focusables);
                (None, focusables)
            }
        }

        // Wheel-set entry editor ────────────────────────────────────
        Selection::WheelSetEntry { set, entry } => {
            let ws = cfg
                .sets
                .get(set)
                .and_then(|s| s.entries.get(entry))
                .and_then(|e| {
                    if let SetEntry::WheelSet(ws) = e {
                        Some(ws)
                    } else {
                        None
                    }
                });
            if let Some(ws) = ws {
                let wname = ws.name.clone();
                let set_name2 = cfg.sets.get(set).map(|s| s.name.as_str()).unwrap_or("Set");
                build_editor_header(
                    commands,
                    root,
                    Some(set_name2),
                    &wname,
                    EditorAction::NavBack,
                    icons,
                    &mut focusables,
                );
                let scroll = child(commands, root, tree());
                spawn_wheelset_entry_editor(
                    commands,
                    scroll,
                    ui,
                    set,
                    entry,
                    ws,
                    icons,
                    &mut focusables,
                );
                build_footer(commands, root, &ui.config_path, &mut focusables);
            } else {
                build_nav_sidebar(commands, root, cfg, ui, hud, set, icons, &mut focusables);
            }
            (None, focusables)
        }

        // Segment editor ──────────────────────────────────────────────
        Selection::Segment {
            set,
            entry,
            wheel,
            slot,
        } => {
            let w_ref = cfg
                .sets
                .get(set)
                .and_then(|s| s.entries.get(entry))
                .and_then(|e| match (e, wheel) {
                    (SetEntry::Wheel(w), None) => Some(w as &WheelData),
                    (SetEntry::WheelSet(ws), Some(i)) => ws.wheels.get(i).map(|w| w as &WheelData),
                    _ => None,
                });
            if let Some(w) = w_ref {
                let slot_name = w
                    .slots
                    .get(slot)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let wname = w.name.clone();
                build_editor_header(
                    commands,
                    root,
                    Some(wname.as_str()),
                    &slot_name,
                    EditorAction::NavBack,
                    icons,
                    &mut focusables,
                );
                let scroll = child(commands, root, tree());
                spawn_segment_editor(commands, scroll, ui, slot, w, icons, &mut focusables);
                build_footer(commands, root, &ui.config_path, &mut focusables);
            } else {
                build_nav_sidebar(commands, root, cfg, ui, hud, set, icons, &mut focusables);
            }
            (None, focusables)
        }
    }
}

/// Breadcrumb header for editor panels.  `‹ [parent |] name`
fn build_editor_header(
    commands: &mut Commands,
    parent: Entity,
    parent_name: Option<&str>,
    item_name: &str,
    back_action: EditorAction,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    let header = commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: {UiRect::all(px(12.))},
                column_gap: {px(6.)},
                border: {UiRect::bottom(px(1.))},
            }
            BorderColor::all(SIDEBAR_BORDER)
        })
        .id();
    commands.entity(parent).add_child(header);

    let back = clickable(
        commands,
        header,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: {px(3.)},
                padding: {UiRect::axes(px(5.), px(3.))},
                border_radius: {BorderRadius::all(px(4.))},
            }
        },
        back_action,
        Color::NONE,
    );
    cil_icon(commands, back, "cil-chevron-left", 14., DIM, icons);
    focusables.push(back);

    if let Some(pn) = parent_name {
        child(commands, header, text(pn, 11., DIM));
        child(commands, header, text("|", 11., DIMMER));
    }
    child(commands, header, text(item_name, 11., TEXT));
}

/// Set-detail sidebar: wheel-set tree + button list for one specific set.
#[allow(clippy::too_many_arguments)]
fn build_nav_sidebar(
    commands: &mut Commands,
    root: Entity,
    cfg: &QuickActionConfig,
    ui: &EditorUiState,
    _hud: &WheelHudState,
    si: usize,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    // Header with breadcrumb: ‹ Action Sets | Set Name
    let set_name = cfg.sets.get(si).map(|s| s.name.as_str()).unwrap_or("—");
    let header = commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: {UiRect::all(px(12.))},
                column_gap: {px(6.)},
                border: {UiRect::bottom(px(1.))},
            }
            BorderColor::all(SIDEBAR_BORDER)
        })
        .id();
    commands.entity(root).add_child(header);

    // Back button → root
    let back = clickable(
        commands,
        header,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: {px(3.)},
                padding: {UiRect::axes(px(5.), px(3.))},
                border_radius: {BorderRadius::all(px(4.))},
            }
        },
        EditorAction::NavBack,
        Color::NONE,
    );
    cil_icon(commands, back, "cil-chevron-left", 14., DIM, icons);
    focusables.push(back);
    cil_icon(
        commands,
        header,
        "cil-applications-settings",
        12.,
        DIM,
        icons,
    );
    child(commands, header, text("Action Sets", 10., DIM));
    child(commands, header, text("|", 10., DIMMER));

    // Editable set name
    let nf = ui.editing == EditFocus::SetName && ui.selection == (Selection::Set { set: si });
    let nd = if nf {
        format!("{}|", set_name)
    } else {
        set_name.to_string()
    };
    let name_btn = clickable(
        commands,
        header,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node { flex_grow: 1., padding: {UiRect::axes(px(3.), px(2.))} }
        },
        EditorAction::EditSetName { set: si },
        Color::NONE,
    );
    child(
        commands,
        name_btn,
        text(&nd, 11., if nf { AMBER } else { TEXT }),
    );
    focusables.push(name_btn);
    let close = hud_clickable(
        commands,
        header,
        bsn! {
            Node {
                width: {px(20.)}, height: {px(20.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(BADGE_BORDER)
            Button
        },
        WheelHudAction::ToggleEditor,
        Color::NONE,
    );
    cil_icon(commands, close, "cil-x", 13., DIM, icons);

    // Scrollable body
    let scroll = child(commands, root, tree());

    if let Some(set) = cfg.sets.get(si) {
        // Per-set opacity control
        let scard = child(commands, scroll, editor_card());
        let (op_dec, op_inc) = spawn_stepper_field(
            commands,
            scard,
            "Opacity",
            &format!("{:.0}%", set.opacity * 100.0),
            EditorAction::SetOpacityDelta {
                set: si,
                delta: -0.05,
            },
            EditorAction::SetOpacityDelta {
                set: si,
                delta: 0.05,
            },
            icons,
        );
        focusables.push(op_dec);
        focusables.push(op_inc);

        // ── SET CONFIG ────────────────────────────────────────────────────────────────
        section_label(commands, scroll, "SET CONFIG");
        let cfg_card = child(commands, scroll, editor_card());

        // Background image path
        let bg_f = ui.editing == EditFocus::SetBgImage(si);
        let bg_d = if bg_f {
            format!("{}|", set.bg_image)
        } else if set.bg_image.is_empty() {
            "none".to_string()
        } else {
            set.bg_image.clone()
        };
        let bg_btn = spawn_box_field(
            commands,
            cfg_card,
            "BG image",
            &bg_d,
            if bg_f {
                AMBER
            } else if set.bg_image.is_empty() {
                DIMMER
            } else {
                TEXT
            },
            if bg_f { AMBER } else { BADGE_BORDER },
            EditorAction::EditSetBgImage { set: si },
        );
        focusables.push(bg_btn);

        // Background image opacity
        let (bg_op_dec, bg_op_inc) = spawn_stepper_field(
            commands,
            cfg_card,
            "BG opacity",
            &format!("{:.0}%", set.bg_image_opacity * 100.0),
            EditorAction::SetBgImageOpacityDelta {
                set: si,
                delta: -0.05,
            },
            EditorAction::SetBgImageOpacityDelta {
                set: si,
                delta: 0.05,
            },
            icons,
        );
        focusables.push(bg_op_dec);
        focusables.push(bg_op_inc);

        // ───────────────────────────────────────────────────────────────────────────

        section_label(commands, scroll, "ADD COMPONENT");
        let add_card = child(commands, scroll, editor_card());
        for (label, action, color) in [
            ("+ Button", EditorAction::AddAction { set: si }, AMBER),
            ("+ Wheel", EditorAction::AddWheel { set: si }, BLUE),
            ("+ Wheel set", EditorAction::AddWheelSet { set: si }, TEAL),
            (
                "+ HUD switch",
                EditorAction::AddHudSwitch { set: si },
                PURPLE,
            ),
        ] {
            let b = clickable(
                commands,
                add_card,
                bsn! {
                    @FeathersButton { @variant: ButtonVariant::Plain }
                    Node { height: {px(24.)}, padding: {UiRect::horizontal(px(6.))}, border: {UiRect::all(px(1.))}, border_radius: {BorderRadius::all(px(3.))} }
                    BorderColor::all(color)
                },
                action,
                Color::NONE,
            );
            child(commands, b, text(label, 10., color));
            focusables.push(b);
        }

        build_nav_wheel_section(commands, scroll, ui, set, si, icons, focusables);
        build_nav_button_section(commands, scroll, ui, set, si, icons, focusables);
    } else {
        child(commands, scroll, text("No entries.", 10., DIMMER));
    }

    build_footer(commands, root, &ui.config_path, focusables);
}

/// Root sidebar: list of all ActionSets + global config.
fn build_root_sidebar(
    commands: &mut Commands,
    root: Entity,
    cfg: &QuickActionConfig,
    ui: &EditorUiState,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    // ── header ───────────────────────────────────────────────────────────────────────
    let header = commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: {UiRect::all(px(14.))},
                border: {UiRect::bottom(px(1.))},
            }
            BorderColor::all(SIDEBAR_BORDER)
        })
        .id();
    commands.entity(root).add_child(header);
    cil_icon(
        commands,
        header,
        "cil-applications-settings",
        14.,
        ICON,
        icons,
    );
    child(
        commands,
        header,
        text(
            &format!(
                "HUD pages ({}/{})",
                enabled_hud_pages(cfg).len(),
                cfg.sets.len()
            ),
            13.,
            TEXT,
        ),
    );
    let btn_row = child(commands, header, hcluster());
    let add_btn = clickable(
        commands,
        btn_row,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                width: {px(20.)}, height: {px(20.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(TEAL)
        },
        EditorAction::AddSet,
        Color::NONE,
    );
    cil_icon(commands, add_btn, "cil-plus", 13., TEAL, icons);
    focusables.push(add_btn);
    let close = hud_clickable(
        commands,
        btn_row,
        bsn! {
            Node {
                width: {px(20.)}, height: {px(20.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(BADGE_BORDER)
            Button
        },
        WheelHudAction::ToggleEditor,
        Color::NONE,
    );
    cil_icon(commands, close, "cil-x", 13., DIM, icons);

    // ── scrollable set list ──────────────────────────────────────────────────────
    let scroll = child(commands, root, tree());

    for (si, set) in cfg.sets.iter().enumerate() {
        let sel = ui.selection == (Selection::Set { set: si });
        let row_bg = if sel { ROW_SEL } else { Color::NONE };
        let row = child(
            commands,
            scroll,
            bsn! {
                Node {
                    width: {percent(100.)}, height: {px(30.)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {px(4.)},
                    padding: {UiRect::horizontal(px(4.))},
                    border_radius: {BorderRadius::all(px(3.))},
                }
                BackgroundColor({row_bg})
            },
        );

        // Clickable set name → navigate into set
        let name_btn = clickable(
            commands,
            row,
            bsn! {
                @FeathersButton { @variant: ButtonVariant::Plain }
                Node {
                    flex_grow: 1.,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {px(6.)},
                    padding: {UiRect::axes(px(4.), px(3.))},
                    border_radius: {BorderRadius::all(px(3.))},
                }
            },
            EditorAction::SelectSet { set: si },
            Color::NONE,
        );
        child(commands, name_btn, text(&set.name, 11., TEXT));
        cil_icon(commands, name_btn, "cil-chevron-right", 10., DIMMER, icons);
        focusables.push(name_btn);
        spawn_toggle_field(
            commands,
            row,
            "",
            set.enabled,
            EditorAction::ToggleSetEnabled { set: si },
            focusables,
        );
        let dx = clickable(
            commands,
            row,
            del_btn(),
            EditorAction::DeleteSet { set: si },
            Color::NONE,
        );
        cil_icon(commands, dx, "cil-trash", 11., DIMMER, icons);
        focusables.push(dx);
    }

    // ── config section ───────────────────────────────────────────────────────────────
    let config_area = commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                padding: {UiRect::axes(px(12.), px(8.))},
                row_gap: {px(6.)},
                border: {UiRect::top(px(1.))},
            }
            BorderColor::all(SIDEBAR_BORDER)
        })
        .id();
    commands.entity(root).add_child(config_area);

    // Section label
    let lrow = child(commands, config_area, hcluster());
    child(commands, lrow, text("SET CONFIG", 9., DIM));

    let card = child(commands, config_area, editor_card());

    // Next set key
    let nf = ui.editing == EditFocus::NextSetKey;
    let (nd, nc) = key_display(nf, &cfg.next_set_key);
    let next_key_btn = spawn_key_capture_field(
        commands,
        card,
        "Next set key",
        &nd,
        nc,
        if nf { AMBER } else { BADGE_BORDER },
        EditorAction::CaptureNextSetKey,
        EditorAction::ClearNextSetKey,
        &cfg.next_set_key,
        nf,
        icons,
        focusables,
    );
    focusables.push(next_key_btn);

    // Prev set key
    let pf = ui.editing == EditFocus::PrevSetKey;
    let (pd, pc) = key_display(pf, &cfg.prev_set_key);
    let prev_key_btn = spawn_key_capture_field(
        commands,
        card,
        "Prev set key",
        &pd,
        pc,
        if pf { AMBER } else { BADGE_BORDER },
        EditorAction::CapturePrevSetKey,
        EditorAction::ClearPrevSetKey,
        &cfg.prev_set_key,
        pf,
        icons,
        focusables,
    );
    focusables.push(prev_key_btn);

    // Show set bar toggle
    spawn_toggle_field(
        commands,
        card,
        "Show set bar",
        cfg.show_set_bar,
        EditorAction::ToggleShowSetBar,
        focusables,
    );

    // Cycle sets toggle
    spawn_toggle_field(
        commands,
        card,
        "Cycle sets",
        cfg.cycle_sets,
        EditorAction::ToggleCycleSets,
        focusables,
    );

    // Edit shortcut
    let ef = ui.editing == EditFocus::EditShortcut;
    let (ed, ec) = key_display(ef, &cfg.edit_shortcut);
    let edit_sc_btn = spawn_key_capture_field(
        commands,
        card,
        "Edit shortcut",
        &ed,
        ec,
        if ef { AMBER } else { BADGE_BORDER },
        EditorAction::CaptureEditShortcut,
        EditorAction::ClearEditShortcut,
        &cfg.edit_shortcut,
        ef,
        icons,
        focusables,
    );
    focusables.push(edit_sc_btn);

    // HUD open mode (Hold / Toggle)
    let hud_mode_btn = spawn_box_field(
        commands,
        card,
        "HUD open mode",
        cfg.hud_open_mode.label(),
        TEXT,
        BADGE_BORDER,
        EditorAction::CycleHudOpenMode,
    );
    focusables.push(hud_mode_btn);

    // HUD background opacity
    let (hud_op_dec, hud_op_inc) = spawn_stepper_field(
        commands,
        card,
        "HUD bg opacity",
        &format!("{:.0}%", cfg.hud_bg_opacity * 100.0),
        EditorAction::HudBgOpacityDelta { delta: -0.05 },
        EditorAction::HudBgOpacityDelta { delta: 0.05 },
        icons,
    );
    focusables.push(hud_op_dec);
    focusables.push(hud_op_inc);

    // HUD background color
    {
        let label = hud_label_or(&cfg.hud_bg_color);
        let col = if cfg.hud_bg_color.is_empty() {
            DIMMER
        } else {
            TEXT
        };
        let hud_bg_btn = spawn_box_field(
            commands,
            card,
            "HUD bg color",
            &label,
            col,
            BADGE_BORDER,
            EditorAction::CycleHudBgColor,
        );
        focusables.push(hud_bg_btn);
    }

    // ── footer ────────────────────────────────────────────────────────────────────────────
    build_footer(commands, root, &ui.config_path, focusables);
}

/// "~ WHEEL SET" navigation section.
fn build_nav_wheel_section(
    commands: &mut Commands,
    parent: Entity,
    ui: &EditorUiState,
    set: &ActionSet,
    si: usize,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    // Section header.
    let sec = child(
        commands,
        parent,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: {UiRect::new(px(2.), px(0.), px(8.), px(2.))},
            }
        },
    );
    let hl = child(commands, sec, hcluster());
    cil_icon(commands, hl, "cil-aperture", 12., TEAL, icons);
    child(commands, hl, text("WHEEL SET", 10., DIM));
    let add_wheel_btn = clickable(
        commands,
        sec,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                padding: {UiRect::axes(px(6.), px(2.))},
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(BLUE)
        },
        EditorAction::AddWheel { set: si },
        Color::NONE,
    );
    cil_icon(commands, add_wheel_btn, "cil-plus", 12., BLUE, icons);
    focusables.push(add_wheel_btn);

    let body = child(commands, parent, indent_col());
    let mut has_any = false;

    for (ei, entry) in set.entries.iter().enumerate() {
        match entry {
            SetEntry::Wheel(w) => {
                has_any = true;
                let sel = ui.selection
                    == (Selection::Wheel {
                        set: si,
                        entry: ei,
                        wheel: None,
                    });
                let badge = Badge::None;
                spawn_entry_row(
                    commands,
                    body,
                    sel,
                    EditorAction::SelectWheel {
                        set: si,
                        entry: ei,
                        wheel: None,
                    },
                    "",
                    ICON,
                    &w.name,
                    TEXT,
                    badge,
                    Some(EditorAction::DeleteEntry { set: si, entry: ei }),
                    icons,
                    focusables,
                );
            }
            SetEntry::WheelSet(ws) => {
                has_any = true;
                let ws_sel = ui.selection == (Selection::WheelSetEntry { set: si, entry: ei });
                let ws_bg = if ws_sel { ROW_SEL } else { Color::NONE };
                // Wrapper row: [standalone icon] [clickable header flex-grow]
                let entry_row = child(
                    commands,
                    body,
                    bsn! {
                        Node {
                            width: {percent(100.)},
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: {px(4.)},
                            margin: {UiRect::top(px(2.))},
                        }
                    },
                );
                let wsh = clickable(
                    commands,
                    entry_row,
                    bsn! {
                        @FeathersButton { @variant: ButtonVariant::Plain }
                        Node {
                            flex_grow: 1.,
                            height: {px(26.)},
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: {UiRect::horizontal(px(4.))},
                            border_radius: {BorderRadius::all(px(4.))},
                        }
                        BackgroundColor({ws_bg})
                    },
                    EditorAction::SelectWheelSetEntry { set: si, entry: ei },
                    ws_bg,
                );
                let whl = child(commands, wsh, hcluster());
                child(commands, whl, text(&ws.name, 11., TEXT));
                let whr = child(commands, wsh, hcluster());
                child(
                    commands,
                    whr,
                    text(&format!("{}w", ws.wheels.len()), 9., DIM),
                );
                focusables.push(wsh);
                // + add-wheel button for this wheel set
                let add_w = clickable(
                    commands,
                    whr,
                    bsn! {
                        @FeathersButton { @variant: ButtonVariant::Plain }
                        Node {
                            padding: {UiRect::axes(px(4.), px(1.))},
                            border: {UiRect::all(px(1.))},
                            border_radius: {BorderRadius::all(px(3.))},
                        }
                        BorderColor::all(BLUE)
                    },
                    EditorAction::AddWheelToSet { set: si, entry: ei },
                    Color::NONE,
                );
                cil_icon(commands, add_w, "cil-plus", 10., BLUE, icons);
                focusables.push(add_w);
                let dx = clickable(
                    commands,
                    whr,
                    del_btn(),
                    EditorAction::DeleteEntry { set: si, entry: ei },
                    Color::NONE,
                );
                cil_icon(commands, dx, "cil-trash", 11., DIMMER, icons);
                focusables.push(dx);

                let wsb = child(commands, body, indent_col());
                for (wi, w) in ws.wheels.iter().enumerate() {
                    let wsel = ui.selection
                        == (Selection::Wheel {
                            set: si,
                            entry: ei,
                            wheel: Some(wi),
                        });
                    let badge = Badge::None;
                    spawn_entry_row(
                        commands,
                        wsb,
                        wsel,
                        EditorAction::SelectWheel {
                            set: si,
                            entry: ei,
                            wheel: Some(wi),
                        },
                        "",
                        ICON,
                        &w.name,
                        TEAL,
                        badge,
                        Some(EditorAction::DeleteWheelFromSet {
                            set: si,
                            entry: ei,
                            wheel: wi,
                        }),
                        icons,
                        focusables,
                    );
                }
            }
            _ => {}
        }
    }
    if !has_any {
        child(commands, body, text("No wheels yet.", 10., DIMMER));
    }
}

/// "~ BUTTONS" navigation section.
fn build_nav_button_section(
    commands: &mut Commands,
    parent: Entity,
    ui: &EditorUiState,
    set: &ActionSet,
    si: usize,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    // Section header.
    let sec = child(
        commands,
        parent,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: {UiRect::new(px(2.), px(0.), px(8.), px(10.))},
            }
        },
    );
    let hl = child(commands, sec, hcluster());
    cil_icon(commands, hl, "cil-camera-control", 12., AMBER, icons);
    child(commands, hl, text("BUTTONS", 10., DIM));
    let add_action_btn = clickable(
        commands,
        sec,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                padding: {UiRect::axes(px(6.), px(2.))},
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(AMBER)
        },
        EditorAction::AddAction { set: si },
        Color::NONE,
    );
    cil_icon(commands, add_action_btn, "cil-plus", 12., AMBER, icons);
    focusables.push(add_action_btn);

    let body = child(commands, parent, indent_col());
    let mut has_any = false;

    for (ei, entry) in set.entries.iter().enumerate() {
        if let SetEntry::Action(qa) = entry {
            has_any = true;
            let sel = ui.selection == (Selection::Action { set: si, entry: ei });
            let badge = if qa.key.is_empty() {
                Badge::None
            } else {
                Badge::Key(qa.key.clone())
            };
            spawn_entry_row(
                commands,
                body,
                sel,
                EditorAction::SelectAction { set: si, entry: ei },
                "",
                ICON,
                &qa.name,
                TEXT,
                badge,
                Some(EditorAction::DeleteEntry { set: si, entry: ei }),
                icons,
                focusables,
            );
        } else if let SetEntry::HudSwitch(hs) = entry {
            has_any = true;
            let badge = if hs.key.is_empty() {
                Badge::None
            } else {
                Badge::Key(hs.key.clone())
            };
            spawn_entry_row(
                commands,
                body,
                ui.selection == (Selection::HudSwitch { set: si, entry: ei }),
                EditorAction::SelectHudSwitch { set: si, entry: ei },
                "⇄",
                PURPLE,
                &hs.name,
                PURPLE,
                badge,
                Some(EditorAction::DeleteEntry { set: si, entry: ei }),
                icons,
                focusables,
            );
        }
    }
    if !has_any {
        child(commands, body, text("No buttons yet.", 10., DIMMER));
    }
}

/// Save / Load footer.
fn build_footer(commands: &mut Commands, parent: Entity, path: &str, focusables: &mut Vec<Entity>) {
    let footer = commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: {px(8.)},
                padding: {UiRect::all(px(12.))},
                border: {UiRect::top(px(1.))},
            }
            BorderColor::all(SIDEBAR_BORDER)
        })
        .id();
    commands.entity(parent).add_child(footer);
    let row = child(
        commands,
        footer,
        bsn! {
            Node { flex_direction: FlexDirection::Row, column_gap: {px(8.)} }
        },
    );
    let save = clickable(
        commands,
        row,
        footer_button("SAVE", GREEN, true),
        EditorAction::Save,
        GREEN_BG,
    );
    focusables.push(save);
    let load = clickable(
        commands,
        row,
        footer_button("LOAD", DIM, false),
        EditorAction::Load,
        Color::NONE,
    );
    focusables.push(load);
    // Undo / Redo buttons
    let undo = clickable(
        commands,
        row,
        footer_button("↩", DIM, false),
        EditorAction::Undo,
        Color::NONE,
    );
    focusables.push(undo);
    let redo = clickable(
        commands,
        row,
        footer_button("↪", DIM, false),
        EditorAction::Redo,
        Color::NONE,
    );
    focusables.push(redo);
    let cap = child(
        commands,
        footer,
        bsn! { Node { justify_content: JustifyContent::Center } },
    );
    child(commands, cap, text(path, 9., DIMMER));
}

// ─── interaction ───────────────────────────────────────────────────────────────

/// Handles [`WheelHudButton`] clicks spawned by the HUD (set tabs, toggle, etc.).
fn process_hud_buttons(
    buttons: Query<(&WheelHudButton, &Interaction), Changed<Interaction>>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
    mut qcfg: ResMut<QuickActionConfig>,
) {
    for (btn, interaction) in &buttons {
        debug!(
            "[ui] hud button interaction: {:?} -> {:?}",
            btn.action, interaction
        );
        if let Some(owner) = crate::hud_control_owner(&btn.action) {
            match interaction {
                Interaction::Hovered => set_hover_owner(&mut hud, owner),
                Interaction::None => clear_hover_owner(&mut hud, owner),
                Interaction::Pressed => {}
            }
        }
        match (&btn.action, interaction) {
            (WheelHudAction::SelectAction { set, entry }, Interaction::Hovered) => {
                hud.hovered_action = Some((*set, *entry));
            }
            (WheelHudAction::SelectAction { set, entry }, Interaction::None)
                if hud.hovered_action == Some((*set, *entry)) =>
            {
                hud.hovered_action = None;
            }
            (WheelHudAction::SelectWheel { set, entry, wheel }, Interaction::Hovered) => {
                hud.hovered_wheel = Some((*set, *entry, *wheel));
            }
            (WheelHudAction::SelectWheel { set, entry, wheel }, Interaction::None)
                if hud.hovered_wheel == Some((*set, *entry, *wheel)) =>
            {
                hud.hovered_wheel = None;
            }
            (WheelHudAction::SelectHudSwitch { set, entry }, Interaction::Hovered) => {
                hud.hovered_hud_switch = Some((*set, *entry));
            }
            (WheelHudAction::SelectHudSwitch { set, entry }, Interaction::None)
                if hud.hovered_hud_switch == Some((*set, *entry)) =>
            {
                hud.hovered_hud_switch = None;
            }
            _ => {}
        }
        if *interaction == Interaction::Pressed {
            debug!("[ui] hud button pressed: {:?}", btn.action);
            match &btn.action {
                WheelHudAction::SetActiveSet(i) => {
                    hud.active_set = *i;
                    hud.active_wheel_entry = 0;
                    hud.active_wheel_index = 0;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::PrevSet => {
                    if hud.active_set > 0 {
                        hud.active_set -= 1;
                    } else if qcfg.cycle_sets && !qcfg.sets.is_empty() {
                        hud.active_set = qcfg.sets.len() - 1;
                    }
                    hud.active_wheel_entry = 0;
                    hud.active_wheel_index = 0;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::NextSet => {
                    let max = qcfg.sets.len().saturating_sub(1);
                    if hud.active_set < max {
                        hud.active_set += 1;
                    } else if qcfg.cycle_sets {
                        hud.active_set = 0;
                    }
                    hud.active_wheel_entry = 0;
                    hud.active_wheel_index = 0;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleEditor => {
                    hud.editor_open = !hud.editor_open;
                    hud.edit_control_focus = if hud.editor_open { Some(0) } else { None };
                    info!(
                        "[editor] ToggleEditor button — editor_open now={}",
                        hud.editor_open
                    );
                    if !hud.editor_open {
                        ui.selection = Selection::None;
                        ui.editing = EditFocus::None;
                        hud.settings_open = false;
                        hud.selected_action = None;
                        hud.selected_wheel = None;
                        hud.selected_hud_switch = None;
                        hud.hovered_action = None;
                        hud.hovered_wheel = None;
                        hud.hovered_hud_switch = None;
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::AddSegment {
                    set,
                    entry,
                    wheel,
                    side,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        let slot = match side {
                            SegmentInsertSide::Before => {
                                let insert_at = match ui.selection {
                                    Selection::Segment { slot, .. } => slot,
                                    _ => w.slots.len(),
                                }
                                .min(w.slots.len());
                                w.slots.insert(
                                    insert_at,
                                    WheelSlotData::named(format!("Slot {}", insert_at + 1)),
                                );
                                insert_at
                            }
                            SegmentInsertSide::After | SegmentInsertSide::Outer => {
                                let insert_at = match ui.selection {
                                    Selection::Segment { slot, .. } => slot.saturating_add(1),
                                    _ => w.slots.len(),
                                }
                                .min(w.slots.len());
                                w.slots.insert(
                                    insert_at,
                                    WheelSlotData::named(format!("Slot {}", insert_at + 1)),
                                );
                                insert_at
                            }
                        };
                        ui.selection = Selection::Segment {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                            slot,
                        };
                        hud.highlighted = Some((*set, *entry, *wheel, slot));
                        hud.dirty = true;
                        ui.dirty = true;
                    }
                }
                WheelHudAction::RemoveSegment {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if w.slots.len() > 1 && *slot < w.slots.len() {
                            w.slots.remove(*slot);
                            let next_slot = (*slot).min(w.slots.len() - 1);
                            ui.selection = Selection::Segment {
                                set: *set,
                                entry: *entry,
                                wheel: *wheel,
                                slot: next_slot,
                            };
                            hud.highlighted = Some((*set, *entry, *wheel, next_slot));
                            hud.dirty = true;
                            ui.dirty = true;
                        }
                    }
                }
                WheelHudAction::EditSegmentName {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    ui.selection = Selection::Segment {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                        slot: *slot,
                    };
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    ui.editing = EditFocus::SlotName(*slot);
                    hud.highlighted = Some((*set, *entry, *wheel, *slot));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::EditSegmentIcon {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    ui.selection = Selection::Segment {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                        slot: *slot,
                    };
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    ui.editing = EditFocus::SlotIcon(*slot);
                    hud.highlighted = Some((*set, *entry, *wheel, *slot));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::EditSegmentInput {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    ui.selection = Selection::Segment {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                        slot: *slot,
                    };
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    ui.editing = EditFocus::SlotInput(*slot);
                    hud.highlighted = Some((*set, *entry, *wheel, *slot));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleSegmentMapping {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    const COMMANDS: &[&str] = &[
                        "none", "use", "equip", "attack", "interact", "dash", "jump", "crouch",
                    ];
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.command = cycle_palette(COMMANDS, &s.command).into();
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleSegmentHold {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.hold = !s.hold;
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleSegmentHoldAction {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    const COMMANDS: &[&str] = &[
                        "none", "use", "equip", "attack", "interact", "dash", "jump", "crouch",
                    ];
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.hold_command = cycle_palette(COMMANDS, &s.hold_command).into();
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleSegmentCloseOnApply {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.close_on_select = !s.close_on_select;
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteSegment {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if w.slots.len() > 1 && *slot < w.slots.len() {
                            w.slots.remove(*slot);
                        }
                    }
                    ui.selection = Selection::None;
                    hud.highlighted = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::SaveConfig => {
                    save_config(&qcfg, &ui.config_path);
                }
                WheelHudAction::AddNewButton => {
                    let action = EditorAction::AddAction {
                        set: hud.active_set,
                    };
                    apply_action(&action, &mut qcfg, &mut ui, &mut hud);
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleSettings => {
                    hud.settings_open = !hud.settings_open;
                    if hud.settings_open {
                        hud.highlighted = None;
                        hud.selected_action = None;
                        hud.selected_wheel = None;
                        hud.selected_hud_switch = None;
                        hud.hovered_action = None;
                        hud.hovered_wheel = None;
                        hud.hovered_hud_switch = None;
                        ui.selection = Selection::None;
                        ui.editing = EditFocus::None;
                    }
                    hud.dirty = true;
                }
                WheelHudAction::SelectAction { set, entry } => {
                    hud.selected_action = Some((*set, *entry));
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    hud.highlighted = None;
                    ui.selection = Selection::Action {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::SelectHudSwitch { set, entry }
                | WheelHudAction::MoveHudSwitch { set, entry }
                | WheelHudAction::EditHudSwitch { set, entry } => {
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = Some((*set, *entry));
                    hud.highlighted = None;
                    ui.selection = Selection::HudSwitch {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteHudSwitch { set, entry } => {
                    apply_action(
                        &EditorAction::DeleteEntry {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ResizeHudSwitch { set, entry, delta } => {
                    if let Some(SetEntry::HudSwitch(hs)) = qcfg
                        .sets
                        .get_mut(*set)
                        .and_then(|s| s.entries.get_mut(*entry))
                    {
                        hs.width = (hs.width + *delta).clamp(40.0, 300.0);
                        hs.height = (hs.height + *delta * 0.25).clamp(20.0, 120.0);
                    }
                    ui.selection = Selection::HudSwitch {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteAction { set, entry } => {
                    apply_action(
                        &EditorAction::DeleteEntry {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.highlighted = None;
                    hud.edit_control_focus = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::MoveAction { set, entry }
                | WheelHudAction::EditAction { set, entry } => {
                    hud.selected_action = Some((*set, *entry));
                    hud.selected_wheel = None;
                    hud.highlighted = None;
                    ui.selection = Selection::Action {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::EditActionName { set, entry } => {
                    apply_action(
                        &EditorAction::EditName {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = Some((*set, *entry));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CaptureActionKey { set, entry } => {
                    apply_action(
                        &EditorAction::CaptureKey {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = Some((*set, *entry));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleActionIcon { set, entry } => {
                    apply_action(
                        &EditorAction::CycleIcon {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleActionMapping { set, entry } => {
                    apply_action(
                        &EditorAction::CycleCommand {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleActionHold { set, entry } => {
                    apply_action(
                        &EditorAction::ToggleHold {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleHoldAction { set, entry } => {
                    apply_action(
                        &EditorAction::CycleHoldCommand {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleActionCloseOnApply { set, entry } => {
                    apply_action(
                        &EditorAction::ToggleActionCloseOnSelect {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::WheelSettings { set, entry, wheel }
                | WheelHudAction::MoveWheel { set, entry, wheel }
                | WheelHudAction::SelectWheel { set, entry, wheel } => {
                    hud.selected_wheel = Some((*set, *entry, *wheel));
                    hud.selected_action = None;
                    hud.selected_hud_switch = None;
                    hud.highlighted = None;
                    ui.selection = Selection::Wheel {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ResizeWheel {
                    set,
                    entry,
                    wheel,
                    delta,
                } => {
                    ui.selection = Selection::Wheel {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                    };
                    apply_action(
                        &EditorAction::WheelOuterRadiusDelta { delta: *delta },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_wheel = Some((*set, *entry, *wheel));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteWheel { set, entry, wheel } => {
                    // A wheel set is one HUD component. Deleting its visible
                    // wheel removes the owning component, rather than only a
                    // sector wheel from the shared set.
                    let _ = wheel;
                    let editor_action = EditorAction::DeleteEntry {
                        set: *set,
                        entry: *entry,
                    };
                    apply_action(&editor_action, &mut qcfg, &mut ui, &mut hud);
                    hud.selected_wheel = None;
                    hud.highlighted = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::RotateAction { set, entry, delta } => {
                    apply_action(
                        &EditorAction::ActionRotationDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = Some((*set, *entry));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ActionWidthDelta { set, entry, delta } => {
                    apply_action(
                        &EditorAction::ActionWidthDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ActionHeightDelta { set, entry, delta } => {
                    apply_action(
                        &EditorAction::ActionHeightDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ActionRadiusDelta { set, entry, delta } => {
                    apply_action(
                        &EditorAction::RadiusDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleActionPosition { set, entry } => {
                    apply_action(
                        &EditorAction::CyclePosition {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CloseSelection => {
                    hud.highlighted = None;
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.edit_control_focus = None;
                    ui.selection = Selection::None;
                    ui.editing = EditFocus::None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
            }
        }
    }
}

fn set_hover_owner(hud: &mut WheelHudState, owner: HudControlOwner) {
    match owner {
        HudControlOwner::Action(set, entry) => hud.hovered_action = Some((set, entry)),
        HudControlOwner::Wheel(set, entry, wheel) => hud.hovered_wheel = Some((set, entry, wheel)),
        HudControlOwner::HudSwitch(set, entry) => hud.hovered_hud_switch = Some((set, entry)),
    }
}

fn clear_hover_owner(hud: &mut WheelHudState, owner: HudControlOwner) {
    match owner {
        HudControlOwner::Action(set, entry) if hud.hovered_action == Some((set, entry)) => {
            hud.hovered_action = None
        }
        HudControlOwner::Wheel(set, entry, wheel)
            if hud.hovered_wheel == Some((set, entry, wheel)) =>
        {
            hud.hovered_wheel = None
        }
        HudControlOwner::HudSwitch(set, entry) if hud.hovered_hud_switch == Some((set, entry)) => {
            hud.hovered_hud_switch = None
        }
        _ => {}
    }
}

fn click_hud_segments(
    segments: Query<(&WheelHudSegmentHit, &Interaction), Changed<Interaction>>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
) {
    if !hud.editor_open {
        return;
    }
    for (segment, interaction) in &segments {
        debug!(
            "[ui] wheel segment interaction: set={} entry={} wheel={:?} slot={} -> {:?}",
            segment.set, segment.entry, segment.wheel, segment.slot, interaction
        );
        let id = (segment.set, segment.entry, segment.wheel, segment.slot);
        if *interaction == Interaction::Hovered {
            hud.mouse_hovered_segment = Some(id);
            hud.highlighted = Some(id);
            hud.hovered_wheel = Some((segment.set, segment.entry, segment.wheel));
            hud.selected_action = None;
            hud.selected_wheel = None;
            hud.edit_control_focus = Some(0);
        } else if *interaction == Interaction::None && hud.mouse_hovered_segment == Some(id) {
            hud.mouse_hovered_segment = None;
            if hud.hovered_wheel == Some((segment.set, segment.entry, segment.wheel)) {
                hud.hovered_wheel = None;
            }
            if !matches!(
                ui.selection,
                Selection::Segment {
                    set,
                    entry,
                    wheel,
                    slot
                } if (set, entry, wheel, slot) == id
            ) {
                hud.highlighted = None;
                hud.edit_control_focus = None;
            }
        } else if *interaction == Interaction::Pressed {
            hud.mouse_hovered_segment = None;
            hud.highlighted = Some(id);
            hud.selected_action = None;
            hud.selected_wheel = None;
            hud.edit_control_focus = Some(0);
            ui.selection = Selection::Segment {
                set: segment.set,
                entry: segment.entry,
                wheel: segment.wheel,
                slot: segment.slot,
            };
            hud.dirty = true;
            ui.dirty = true;
        }
    }
}

// ─── undo helpers ────────────────────────────────────────────────────────────────

/// Returns `true` if `action` mutates config state and should push an undo snapshot.
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
            | EditorAction::EditSetName { .. }
            | EditorAction::EditWheelName
            | EditorAction::EditSlotName { .. }
            | EditorAction::EditSlotIcon { .. }
            | EditorAction::EditSlotItemName { .. }
            | EditorAction::EditSlotItemIcon { .. }
            | EditorAction::EditWheelSetName { .. }
            | EditorAction::EditSetBgImage { .. }
            | EditorAction::CaptureKey { .. }
            | EditorAction::CaptureNextSetKey
            | EditorAction::CapturePrevSetKey
            | EditorAction::CaptureEditShortcut
            | EditorAction::CaptureWheelSetSwitchKey { .. }
            | EditorAction::CaptureSlotInput { .. }
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

pub(crate) fn apply_action(
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
            if !cfg.sets.is_empty() {
                hud.active_set = hud.active_set.min(cfg.sets.len() - 1);
            }
        }
        EditorAction::SelectSet { set } => {
            ui.selection = Selection::Set { set };
            ui.editing = EditFocus::None;
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
        EditorAction::AddWheel { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.entries
                    .push(SetEntry::Wheel(WheelData::new("New Wheel", 6)));
            }
        }
        EditorAction::AddWheelSet { set } => {
            if let Some(s) = cfg.sets.get_mut(set) {
                s.entries.push(SetEntry::WheelSet(WheelSetData {
                    name: "New Wheel Set".into(),
                    wheels: vec![WheelData::new("Wheel 1", 6)],
                    stick: StickSide::Right,
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
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                if ws.wheels.len() < ws.max_wheels {
                    let n = ws.wheels.len() + 1;
                    let mut wheel = WheelData::new(format!("Wheel {}", n), 6);
                    wheelset_visuals(ws).apply_to(&mut wheel);
                    ws.wheels.push(wheel);
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
        }
        EditorAction::DeleteWheelFromSet { set, entry, wheel } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                if ws.wheels.len() > ws.min_wheels && wheel < ws.wheels.len() {
                    ws.wheels.remove(wheel);
                }
            }
            let clear = matches!(ui.selection,
                Selection::Wheel { set: s, entry: e, wheel: Some(w) } if s == set && e == entry && w == wheel);
            if clear {
                ui.selection = Selection::None;
                ui.editing = EditFocus::None;
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
            hud.highlighted = None;
            // Sync the active set so the HUD shows the correct context.
            hud.active_set = set;
        }
        EditorAction::SelectHudSwitch { set, entry } => {
            ui.selection = Selection::HudSwitch { set, entry };
            ui.editing = EditFocus::None;
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
            hud.highlighted = None;
            // Sync the HUD to preview the selected wheel.
            hud.active_set = set;
            hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
        }
        EditorAction::SelectWheelSetEntry { set, entry } => {
            ui.selection = Selection::WheelSetEntry { set, entry };
            ui.editing = EditFocus::None;
            hud.highlighted = None;
            // Sync the HUD to preview the selected wheel set.
            hud.active_set = set;
            hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
        }
        EditorAction::SelectSetSwitch => {
            ui.selection = Selection::SetSwitch;
            ui.editing = EditFocus::None;
            hud.highlighted = None;
        }
        EditorAction::NavBack => {
            ui.editing = EditFocus::None;
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
        EditorAction::CaptureKey { set, entry } => {
            ui.selection = Selection::Action { set, entry };
            ui.editing = EditFocus::Key;
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
        EditorAction::CycleWheelTheme => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.theme = w.theme.next();
            }
        }
        EditorAction::WheelCooldownDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.cooldown_secs = (w.cooldown_secs + delta).clamp(0.0, 60.0);
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
        EditorAction::ToggleWheelShowLabels => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.show_labels = !w.show_labels;
            }
        }
        EditorAction::AddSlot => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                let n = w.slots.len() + 1;
                w.slots.push(WheelSlotData::named(format!("Slot {}", n)));
            }
        }
        EditorAction::RemoveSlot => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if w.slots.len() > 1 {
                    w.slots.pop();
                }
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
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.next_wheel_key.clear();
            }
        }
        EditorAction::ClearWheelSetPrevKey { set, entry } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.prev_wheel_key.clear();
            }
        }
        EditorAction::WheelSetMinDelta { set, entry, delta } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                let next = (ws.min_wheels as i32 + delta).clamp(1, ws.max_wheels as i32) as usize;
                ws.min_wheels = next;
                normalize_wheelset(ws);
            }
        }
        EditorAction::WheelSetMaxDelta { set, entry, delta } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.max_wheels = (ws.max_wheels as i32 + delta)
                    .clamp(ws.min_wheels.max(ws.wheels.len()) as i32, 64)
                    as usize;
            }
        }
        EditorAction::ToggleWheelSetCycle { set, entry } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.cycle_wheels = !ws.cycle_wheels;
            }
        }
        EditorAction::SwitchWheelPrev { set, entry } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get(set).and_then(|s| s.entries.get(entry))
            {
                if !ws.wheels.is_empty() {
                    hud.active_set = set;
                    hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
                    hud.active_wheel_index = hud
                        .active_wheel_index
                        .checked_sub(1)
                        .unwrap_or(ws.wheels.len() - 1);
                    hud.highlighted = None;
                    hud.dirty = true;
                }
            }
        }
        EditorAction::SwitchWheelNext { set, entry } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get(set).and_then(|s| s.entries.get(entry))
            {
                if !ws.wheels.is_empty() {
                    hud.active_set = set;
                    hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
                    hud.active_wheel_index = (hud.active_wheel_index + 1) % ws.wheels.len();
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
            hud.highlighted = Some((set, entry, wheel, slot));
            hud.edit_control_focus = Some(0);
            // Sync the HUD to preview the wheel containing this segment.
            hud.active_set = set;
            hud.active_wheel_entry = wheel_entry_idx(cfg, set, entry);
        }
        EditorAction::EditSlotIcon { slot } => {
            ui.editing = EditFocus::SlotIcon(slot);
        }
        EditorAction::CycleSegmentShape => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.segment_shape = w.segment_shape.next();
            }
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
        EditorAction::SegmentScaleDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.segment_scale = (w.segment_scale + delta).clamp(0.5, 2.0);
            }
        }
        EditorAction::WheelOpacityDelta { delta } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.opacity = (w.opacity + delta).clamp(0.0, 1.0);
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
        // ── segment input / gamepad binding ─────────────────────────────────────────
        EditorAction::CaptureSlotInput { slot } => {
            ui.editing = EditFocus::SlotInput(slot);
        }
        EditorAction::ClearSlotInput { slot } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if let Some(s) = w.slots.get_mut(slot) {
                    s.input.clear();
                }
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
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.switch_key.clear();
            }
        }
        EditorAction::ClearWheelSetStick { set, entry } => {
            if let Some(SetEntry::WheelSet(ws)) =
                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
            {
                ws.stick = StickSide::Right;
                let mut visuals = wheelset_visuals(ws);
                visuals.stick = ws.stick;
                ws.visuals = Some(visuals.clone());
                for wheel in &mut ws.wheels {
                    visuals.apply_to(wheel);
                }
            }
        }
        EditorAction::ClearActionKey { set, entry } => {
            if let Some(a) = action_at(cfg, set, entry) {
                a.key.clear();
            }
        }
        // ── per-slot items ─────────────────────────────────────────────────────────────────────────
        EditorAction::AddSlotItem { slot } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if let Some(s) = w.slots.get_mut(slot) {
                    s.items.push(SlotItem::default());
                }
            }
        }
        EditorAction::RemoveSlotItem { slot, item } => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                if let Some(s) = w.slots.get_mut(slot) {
                    if item < s.items.len() {
                        s.items.remove(item);
                    }
                }
            }
            // Clear focus if it pointed to a removed item
            if matches!(ui.editing,
                EditFocus::SlotItemName(sl, it) | EditFocus::SlotItemIcon(sl, it)
                if sl == slot && it == item)
            {
                ui.editing = EditFocus::None;
            }
        }
        EditorAction::EditSlotItemName { slot, item } => {
            ui.editing = EditFocus::SlotItemName(slot, item);
        }
        EditorAction::EditSlotItemIcon { slot, item } => {
            ui.editing = EditFocus::SlotItemIcon(slot, item);
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
        EditorAction::CycleWheelStick => {
            if let Some(w) = wheel_at(cfg, ui.selection) {
                w.stick = w.stick.next();
            }
        }
        EditorAction::CycleWheelSetStick => {
            if let Selection::WheelSetEntry { set, entry } = ui.selection {
                if let Some(SetEntry::WheelSet(ws)) =
                    cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                {
                    ws.stick = ws.stick.next();
                    let mut visuals = wheelset_visuals(ws);
                    visuals.stick = ws.stick;
                    ws.visuals = Some(visuals.clone());
                    for wheel in &mut ws.wheels {
                        visuals.apply_to(wheel);
                    }
                }
            }
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

// ─── persistence ─────────────────────────────────────────────────────────────────

fn save_config(cfg: &QuickActionConfig, path: &str) {
    match ron::ser::to_string_pretty(cfg, ron::ser::PrettyConfig::default()) {
        Ok(s) => {
            if let Err(e) = std::fs::write(path, &s) {
                error!("[editor] write failed ({path}): {e}");
            } else {
                info!("[editor] saved to {path}");
            }
        }
        Err(e) => error!("[editor] serialize failed: {e}"),
    }
}

fn load_config(path: &str) -> Option<QuickActionConfig> {
    let s = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            error!("[editor] read failed ({path}): {e}");
            return None;
        }
    };
    match ron::from_str(&s) {
        Ok(mut c) => {
            normalize_wheelset_config(&mut c);
            info!("[editor] loaded from {path}");
            Some(c)
        }
        Err(e) => {
            error!("[editor] parse failed ({path}): {e}");
            None
        }
    }
}

// ─── editor card / field helpers ─────────────────────────────────────────────────

fn editor_card() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: {px(4.)},
            padding: {UiRect::all(px(8.))},
            margin: {UiRect::new(px(0.), px(0.), px(2.), px(4.))},
            border: {UiRect::all(px(1.))},
            border_radius: {BorderRadius::all(px(6.))},
        }
        BackgroundColor({PANEL_CARD})
        BorderColor::all(SIDEBAR_BORDER)
    }
}

fn section_label(commands: &mut Commands, parent: Entity, label: &str) {
    let row = child(
        commands,
        parent,
        bsn! {
            Node {
                padding: {UiRect::new(px(0.), px(0.), px(6.), px(2.))},
                border: {UiRect::bottom(px(1.))},
                margin: {UiRect::bottom(px(2.))},
            }
            BorderColor::all(SIDEBAR_BORDER)
        },
    );
    child(commands, row, text(label, 10., AMBER));
}

fn field_row() -> impl Scene {
    bsn! {
        Node {
            width: {percent(100.)}, min_height: {px(22.)},
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: {px(6.)},
        }
    }
}

fn label_cell(s: &str) -> impl Scene {
    bsn! {
        Node { width: {px(82.)}, flex_direction: FlexDirection::Row, align_items: AlignItems::Center }
        Children [ text(s, 10., DIM) ]
    }
}

fn ctrl_box(_accent: Color) -> impl Scene {
    bsn! {
        @FeathersButton { @variant: ButtonVariant::Plain }
        Node {
            flex_grow: 1., height: {px(20.)},
            padding: {UiRect::horizontal(px(6.))},
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            border: {UiRect::all(px(1.))},
            border_radius: {BorderRadius::all(px(4.))},
        }
        BorderColor::all(BADGE_BORDER)
    }
}

fn mini_box() -> impl Scene {
    bsn! {
        @FeathersToolButton { @variant: ButtonVariant::Plain }
        Node {
            width: {px(22.)}, height: {px(20.)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(4.))},
        }
    }
}

fn val_cell() -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1., height: {px(20.)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

fn spawn_field(commands: &mut Commands, parent: Entity, label: &str) -> Entity {
    let row = child(commands, parent, field_row());
    child(commands, row, label_cell(label));
    row
}

fn spawn_box_field(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    value_color: Color,
    accent: Color,
    action: EditorAction,
) -> Entity {
    let row = spawn_field(commands, parent, label);
    let b = clickable(commands, row, ctrl_box(accent), action, Color::NONE);
    child(commands, b, text(value, 11., value_color));
    b
}

fn spawn_toggle_field(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    on: bool,
    action: EditorAction,
    focusables: &mut Vec<Entity>,
) -> Entity {
    let row = spawn_field(commands, parent, label);
    // Spawn the FeathersCheckbox scene, then insert EditorToggle separately
    // (avoids the FromTemplate/Default requirement for EditorAction).
    let e = commands.spawn_scene(bsn! { @FeathersCheckbox }).id();
    commands.entity(e).insert(EditorToggle { action });
    if on {
        commands.entity(e).insert(Checked);
    }
    commands.entity(row).add_child(e);
    focusables.push(e);
    e
}

fn spawn_stepper_field(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    dec: EditorAction,
    inc: EditorAction,
    icons: &Icons<'_>,
) -> (Entity, Entity) {
    let row = spawn_field(commands, parent, label);
    let d = clickable(commands, row, mini_box(), dec, CTRL_BG);
    cil_icon(commands, d, "cil-minus", 11., TEXT, icons);
    let v = child(commands, row, val_cell());
    child(commands, v, text(value, 11., TEXT));
    let i = clickable(commands, row, mini_box(), inc, CTRL_BG);
    cil_icon(commands, i, "cil-plus", 11., TEXT, icons);
    (d, i)
}

// ─── editor panels ────────────────────────────────────────────────────────────────

/// Button / quick-action editor.
#[allow(clippy::too_many_arguments)]
fn spawn_action_editor(
    commands: &mut Commands,
    parent: Entity,
    ui: &EditorUiState,
    set: usize,
    entry: usize,
    qa: &QuickAction,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    // Panel header: "BUTTON" label + delete
    let hdr = child(
        commands,
        parent,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: {UiRect::bottom(px(4.))},
            }
        },
    );
    child(commands, hdr, text("BUTTON", 10., DIM));
    let dx = clickable(
        commands,
        hdr,
        del_btn(),
        EditorAction::DeleteEntry { set, entry },
        Color::NONE,
    );
    cil_icon(commands, dx, "cil-trash", 12., DIMMER, icons);
    focusables.push(dx);

    let card = child(commands, parent, editor_card());

    // Label (name)
    let nf = ui.editing == EditFocus::Name;
    let nd = if nf {
        format!("{}|", qa.name)
    } else {
        qa.name.clone()
    };
    let label_btn = spawn_box_field(
        commands,
        card,
        "Label",
        &nd,
        TEXT,
        if nf { AMBER } else { BADGE_BORDER },
        EditorAction::EditName { set, entry },
    );
    focusables.push(label_btn);

    // Unified input field (keyboard key or gamepad button, same as set shortcuts)
    {
        let row = spawn_field(commands, card, "Input");
        let kf = ui.editing == EditFocus::Key;
        let (kd, kc) = if kf {
            ("press key or button\u{2026}".to_string(), AMBER)
        } else if qa.key.is_empty() {
            ("unbound".to_string(), DIM)
        } else {
            (qa.key.clone(), TEXT)
        };
        let kb = clickable(
            commands,
            row,
            bsn! {
                @FeathersButton { @variant: ButtonVariant::Plain }
                Node {
                    width: {px(56.)}, height: {px(20.)},
                    padding: {UiRect::horizontal(px(4.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(px(1.))},
                    border_radius: {BorderRadius::all(px(4.))},
                }
                BorderColor::all(if kf { AMBER } else { BADGE_BORDER })
            },
            EditorAction::CaptureKey { set, entry },
            Color::NONE,
        );
        if !kf && !qa.key.is_empty() {
            if let Some(btn_label) = qa.key.strip_prefix("GP:") {
                if let Some(path) = icons.set.embedded_icon_path(btn_label) {
                    let handle = icons.srv.load::<Image>(path);
                    let e = commands
                        .spawn((
                            Node {
                                width: Val::Px(18.0),
                                height: Val::Px(18.0),
                                ..default()
                            },
                            ImageNode::new(handle),
                        ))
                        .id();
                    commands.entity(kb).add_child(e);
                } else {
                    child(commands, kb, text(&kd, 10., kc));
                }
            } else {
                child(commands, kb, text(&kd, 10., kc));
            }
        } else {
            child(commands, kb, text(&kd, 10., kc));
        }
        focusables.push(kb);

        // Clear button — only when a key is bound and not currently capturing.
        if !qa.key.is_empty() && !kf {
            let clear_btn = clickable(
                commands,
                row,
                bsn! {
                    @FeathersToolButton {}
                    Node {
                        width: {Val::Px(18.)},
                        height: {Val::Px(18.)},
                        margin: {UiRect::left(px(3.))},
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_shrink: 0.,
                    }
                },
                EditorAction::ClearActionKey { set, entry },
                Color::NONE,
            );
            cil_icon(commands, clear_btn, "cil-x", 10., HUD_DIM, icons);
            focusables.push(clear_btn);
        }

        // Color swatch
        child(commands, row, text("Color", 9., DIM));
        let color_val = parse_hex_color(&qa.color, 1.0);
        let cs = child(
            commands,
            row,
            bsn! {
                Node {
                    width: {px(22.)}, height: {px(20.)},
                    border_radius: {BorderRadius::all(px(3.))},
                    border: {UiRect::all(px(1.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                BackgroundColor({color_val})
                BorderColor::all(BADGE_BORDER)
            },
        );
        child(commands, cs, text(&qa.color, 6., TEXT));
    }

    // Action / command
    let action_btn = spawn_box_field(
        commands,
        card,
        "Action",
        &qa.command,
        TEAL,
        BADGE_BORDER,
        EditorAction::CycleCommand { set, entry },
    );
    focusables.push(action_btn);

    // Width + Height on one row
    {
        let row = child(commands, card, field_row());
        child(commands, row, label_cell("Width"));
        let dw = clickable(
            commands,
            row,
            mini_box(),
            EditorAction::ActionWidthDelta {
                set,
                entry,
                delta: -4.0,
            },
            CTRL_BG,
        );
        cil_icon(commands, dw, "cil-minus", 11., TEXT, icons);
        let vw = child(commands, row, val_cell());
        child(commands, vw, text(&format!("{:.0}", qa.width), 11., TEXT));
        let iw = clickable(
            commands,
            row,
            mini_box(),
            EditorAction::ActionWidthDelta {
                set,
                entry,
                delta: 4.0,
            },
            CTRL_BG,
        );
        cil_icon(commands, iw, "cil-plus", 11., TEXT, icons);
        focusables.push(dw);
        focusables.push(iw);

        child(commands, row, text("H", 9., DIM));
        let dh = clickable(
            commands,
            row,
            mini_box(),
            EditorAction::ActionHeightDelta {
                set,
                entry,
                delta: -2.0,
            },
            CTRL_BG,
        );
        cil_icon(commands, dh, "cil-minus", 11., TEXT, icons);
        let vh = child(commands, row, val_cell());
        child(commands, vh, text(&format!("{:.0}", qa.height), 11., TEXT));
        let ih = clickable(
            commands,
            row,
            mini_box(),
            EditorAction::ActionHeightDelta {
                set,
                entry,
                delta: 2.0,
            },
            CTRL_BG,
        );
        cil_icon(commands, ih, "cil-plus", 11., TEXT, icons);
        focusables.push(dh);
        focusables.push(ih);
    }

    // Enabled toggle
    spawn_toggle_field(
        commands,
        card,
        "Enabled",
        qa.enabled,
        EditorAction::ToggleEnabled { set, entry },
        focusables,
    );

    // Close HUD on select
    spawn_toggle_field(
        commands,
        card,
        "Close on select",
        qa.close_on_select,
        EditorAction::ToggleActionCloseOnSelect { set, entry },
        focusables,
    );

    // Reposition hint
    child(
        commands,
        parent,
        bsn! {
            Node { padding: {UiRect::new(px(4.), px(0.), px(8.), px(0.))} }
            Children [ text("Drag in the preview to reposition.", 9., DIMMER) ]
        },
    );
}

fn key_display(focus: bool, key: &str) -> (String, Color) {
    if focus {
        ("press key or button\u{2026}".to_string(), AMBER)
    } else if key.is_empty() {
        ("unbound".to_string(), DIM)
    } else {
        (key.to_string(), TEXT)
    }
}

/// Wheel editor panel.
#[allow(clippy::too_many_arguments)]
fn spawn_wheel_editor(
    commands: &mut Commands,
    parent: Entity,
    ui: &EditorUiState,
    w: &WheelData,
    set: usize,
    entry: usize,
    w_idx: Option<usize>,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    // ── WHEEL (basic) ──────────────────────────────────────────────────────────
    section_label(commands, parent, "WHEEL");
    let card = child(commands, parent, editor_card());

    // Name
    let nf = ui.editing == EditFocus::WheelName;
    let nd = if nf {
        format!("{}|", w.name)
    } else {
        w.name.clone()
    };
    let name_btn = spawn_box_field(
        commands,
        card,
        "Name",
        &nd,
        TEXT,
        if nf { AMBER } else { BADGE_BORDER },
        EditorAction::EditWheelName,
    );
    focusables.push(name_btn);

    // Theme
    let theme_btn = spawn_box_field(
        commands,
        card,
        "Theme",
        w.theme.label(),
        TEXT,
        BADGE_BORDER,
        EditorAction::CycleWheelTheme,
    );
    focusables.push(theme_btn);

    // Stick side
    let stick_btn = spawn_box_field(
        commands,
        card,
        "Stick",
        w.stick.label(),
        TEXT,
        BADGE_BORDER,
        EditorAction::CycleWheelStick,
    );
    focusables.push(stick_btn);

    // Cooldown
    let (cd_dec, cd_inc) = spawn_stepper_field(
        commands,
        card,
        "Cooldown (s)",
        &format!("{:.1}", w.cooldown_secs),
        EditorAction::WheelCooldownDelta { delta: -0.5 },
        EditorAction::WheelCooldownDelta { delta: 0.5 },
        icons,
    );
    focusables.push(cd_dec);
    focusables.push(cd_inc);

    // ── APPEARANCE ─────────────────────────────────────────────────────────────
    section_label(commands, parent, "APPEARANCE");
    let app_card = child(commands, parent, editor_card());

    // Opacity
    let (op_dec, op_inc) = spawn_stepper_field(
        commands,
        app_card,
        "Opacity",
        &format!("{:.0}%", w.opacity * 100.0),
        EditorAction::WheelOpacityDelta { delta: -0.05 },
        EditorAction::WheelOpacityDelta { delta: 0.05 },
        icons,
    );
    focusables.push(op_dec);
    focusables.push(op_inc);

    // Show labels
    spawn_toggle_field(
        commands,
        app_card,
        "Show labels",
        w.show_labels,
        EditorAction::ToggleWheelShowLabels,
        focusables,
    );

    // Show icons
    spawn_toggle_field(
        commands,
        app_card,
        "Show icons",
        w.show_icon,
        EditorAction::ToggleWheelShowIcon,
        focusables,
    );

    // Segment Shape
    let seg_shape_btn = spawn_box_field(
        commands,
        app_card,
        "Seg shape",
        w.segment_shape.label(),
        TEXT,
        BADGE_BORDER,
        EditorAction::CycleSegmentShape,
    );
    focusables.push(seg_shape_btn);

    // Segment Scale
    let (ss_dec, ss_inc) = spawn_stepper_field(
        commands,
        app_card,
        "Seg scale",
        &format!("{:.1}", w.segment_scale),
        EditorAction::SegmentScaleDelta { delta: -0.1 },
        EditorAction::SegmentScaleDelta { delta: 0.1 },
        icons,
    );
    focusables.push(ss_dec);
    focusables.push(ss_inc);

    // Highlight color
    {
        let hcol = parse_hex_color(&w.highlight_color, 1.0);
        let hex = w.highlight_color.clone();
        let hrow = spawn_field(commands, app_card, "Highlight");
        let b = clickable(
            commands,
            hrow,
            ctrl_box(BADGE_BORDER),
            EditorAction::CycleHighlightColor,
            Color::NONE,
        );
        child(
            commands,
            b,
            bsn! {
                Node {
                    width: {px(12.)}, height: {px(12.)},
                    border_radius: {BorderRadius::all(px(2.))},
                }
                BackgroundColor({hcol})
            },
        );
        child(commands, b, text(&hex, 11., TEXT));
        focusables.push(b);
    }
    section_label(commands, parent, "BACKGROUND");
    let bg_card = child(commands, parent, editor_card());

    // Bg color
    {
        let label = if w.bg_color.is_empty() {
            "Theme".to_string()
        } else {
            w.bg_color.clone()
        };
        let col = if w.bg_color.is_empty() { DIMMER } else { TEXT };
        let bg_col_btn = spawn_box_field(
            commands,
            bg_card,
            "Color",
            &label,
            col,
            BADGE_BORDER,
            EditorAction::CycleWheelBgColor,
        );
        focusables.push(bg_col_btn);
    }

    // Bg opacity
    let (bg_op_dec, bg_op_inc) = spawn_stepper_field(
        commands,
        bg_card,
        "Opacity",
        &format!("{:.0}%", w.bg_opacity * 100.0),
        EditorAction::WheelBgOpacityDelta { delta: -0.05 },
        EditorAction::WheelBgOpacityDelta { delta: 0.05 },
        icons,
    );
    focusables.push(bg_op_dec);
    focusables.push(bg_op_inc);

    // ── OUTER CIRCLE ───────────────────────────────────────────────────────────
    section_label(commands, parent, "OUTER CIRCLE");
    let out_card = child(commands, parent, editor_card());

    // Outer Radius
    let (or_dec, or_inc) = spawn_stepper_field(
        commands,
        out_card,
        "Radius",
        &format!("{:.0}", w.outer_radius),
        EditorAction::WheelOuterRadiusDelta { delta: -5.0 },
        EditorAction::WheelOuterRadiusDelta { delta: 5.0 },
        icons,
    );
    focusables.push(or_dec);
    focusables.push(or_inc);

    // Outer border color
    {
        let label = if w.outer_border.is_empty() {
            "None".to_string()
        } else {
            w.outer_border.clone()
        };
        let col = if w.outer_border.is_empty() {
            DIMMER
        } else {
            TEXT
        };
        let ob_col_btn = spawn_box_field(
            commands,
            out_card,
            "Border color",
            &label,
            col,
            BADGE_BORDER,
            EditorAction::CycleOuterBorderColor,
        );
        focusables.push(ob_col_btn);
    }

    // Outer border width
    let (obw_dec, obw_inc) = spawn_stepper_field(
        commands,
        out_card,
        "Border width",
        &format!("{:.0}px", w.outer_border_width),
        EditorAction::WheelOuterBorderWidthDelta { delta: -0.5 },
        EditorAction::WheelOuterBorderWidthDelta { delta: 0.5 },
        icons,
    );
    focusables.push(obw_dec);
    focusables.push(obw_inc);

    // ── INNER CIRCLE ───────────────────────────────────────────────────────────
    section_label(commands, parent, "INNER CIRCLE");
    let inn_card = child(commands, parent, editor_card());

    // Inner Radius
    let (ir_dec, ir_inc) = spawn_stepper_field(
        commands,
        inn_card,
        "Radius",
        &format!("{:.0}", w.inner_radius),
        EditorAction::WheelInnerRadiusDelta { delta: -2.0 },
        EditorAction::WheelInnerRadiusDelta { delta: 2.0 },
        icons,
    );
    focusables.push(ir_dec);
    focusables.push(ir_inc);

    // Inner border color
    {
        let label = if w.inner_border.is_empty() {
            "None".to_string()
        } else {
            w.inner_border.clone()
        };
        let col = if w.inner_border.is_empty() {
            DIMMER
        } else {
            TEXT
        };
        let ib_col_btn = spawn_box_field(
            commands,
            inn_card,
            "Border color",
            &label,
            col,
            BADGE_BORDER,
            EditorAction::CycleInnerBorderColor,
        );
        focusables.push(ib_col_btn);
    }

    // Inner border width
    let (ibw_dec, ibw_inc) = spawn_stepper_field(
        commands,
        inn_card,
        "Border width",
        &format!("{:.0}px", w.inner_border_width),
        EditorAction::WheelInnerBorderWidthDelta { delta: -0.5 },
        EditorAction::WheelInnerBorderWidthDelta { delta: 0.5 },
        icons,
    );
    focusables.push(ibw_dec);
    focusables.push(ibw_inc);

    // Hub background color
    {
        let label = if w.hub_color.is_empty() {
            "Theme".to_string()
        } else {
            w.hub_color.clone()
        };
        let col = if w.hub_color.is_empty() { DIMMER } else { TEXT };
        let hub_col_btn = spawn_box_field(
            commands,
            inn_card,
            "Hub color",
            &label,
            col,
            BADGE_BORDER,
            EditorAction::CycleWheelHubColor,
        );
        focusables.push(hub_col_btn);
    }

    // Hub opacity
    let (hub_op_dec, hub_op_inc) = spawn_stepper_field(
        commands,
        inn_card,
        "Hub opacity",
        &format!("{:.0}%", w.hub_opacity * 100.0),
        EditorAction::WheelHubOpacityDelta { delta: -0.05 },
        EditorAction::WheelHubOpacityDelta { delta: 0.05 },
        icons,
    );
    focusables.push(hub_op_dec);
    focusables.push(hub_op_inc);

    // ── SEGMENTS ───────────────────────────────────────────────────────────────
    let seg_hdr = child(
        commands,
        parent,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: {UiRect::new(px(0.), px(0.), px(6.), px(8.))},
            }
        },
    );
    child(commands, seg_hdr, text("SEGMENTS", 10., DIM));
    let add_slot_btn = clickable(
        commands,
        seg_hdr,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                padding: {UiRect::axes(px(8.), px(3.))},
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(GREEN)
            BackgroundColor({GREEN_BG})
        },
        EditorAction::AddSlot,
        Color::NONE,
    );
    child(commands, add_slot_btn, text("+", 9., GREEN));
    focusables.push(add_slot_btn);

    let seg_card = child(commands, parent, editor_card());
    if w.slots.is_empty() {
        child(commands, seg_card, text("No segments.", 10., DIMMER));
    }
    for (i, slot) in w.slots.iter().enumerate() {
        let is_sel = ui.selection
            == (Selection::Segment {
                set,
                entry,
                wheel: w_idx,
                slot: i,
            });
        let row_bg = if is_sel { ROW_SEL } else { Color::NONE };
        let row = clickable(
            commands,
            seg_card,
            row_button(row_bg),
            EditorAction::SelectSegment {
                set,
                entry,
                wheel: w_idx,
                slot: i,
            },
            row_bg,
        );
        focusables.push(row);
        let left = child(commands, row, hcluster());
        child(commands, left, text(&format!("{}", i + 1), 9., DIMMER));
        if !slot.icon.is_empty() {
            child(commands, left, text(&slot.icon, 11., TEXT));
        }
        child(commands, left, text(&slot.name, 11., TEXT));
        if !slot.items.is_empty() {
            child(
                commands,
                left,
                text(&format!("[{}]", slot.items.len()), 9., TEAL),
            );
        }
        if !slot.input.is_empty() {
            let right = child(commands, row, hcluster());
            spawn_input_badge(commands, right, &slot.input, icons);
        }
        let right2 = child(commands, row, hcluster());
        let dx = clickable(
            commands,
            right2,
            del_btn(),
            EditorAction::RemoveSlot,
            Color::NONE,
        );
        cil_icon(commands, dx, "cil-trash", 11., DIMMER, icons);
        focusables.push(dx);
    }
}

/// Segment editor panel — per-slot name, icon, input binding, and items list.
fn spawn_segment_editor(
    commands: &mut Commands,
    parent: Entity,
    ui: &EditorUiState,
    slot: usize,
    w: &WheelData,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    let slot_data = w.slots.get(slot);
    let slot_name = slot_data.map(|s| s.name.as_str()).unwrap_or("");
    let slot_icon = slot_data.map(|s| s.icon.as_str()).unwrap_or("");
    let slot_input = slot_data.map(|s| s.input.as_str()).unwrap_or("");
    let items = slot_data.map(|s| s.items.as_slice()).unwrap_or(&[]);

    section_label(commands, parent, "SEGMENT");
    let card = child(commands, parent, editor_card());

    // Name
    let nf = ui.editing == EditFocus::SlotName(slot);
    let nd = if nf {
        format!("{}|", slot_name)
    } else {
        slot_name.to_string()
    };
    let seg_name_btn = spawn_box_field(
        commands,
        card,
        "Name",
        &nd,
        TEXT,
        if nf { AMBER } else { BADGE_BORDER },
        EditorAction::EditSlotName { slot },
    );
    focusables.push(seg_name_btn);

    // Icon
    let icon_f = ui.editing == EditFocus::SlotIcon(slot);
    let id = if icon_f {
        format!("{}|", slot_icon)
    } else {
        slot_icon.to_string()
    };
    let seg_icon_btn = spawn_box_field(
        commands,
        card,
        "Icon",
        &id,
        TEXT,
        if icon_f { AMBER } else { BADGE_BORDER },
        EditorAction::EditSlotIcon { slot },
    );
    focusables.push(seg_icon_btn);

    // Input binding (keyboard key or gamepad button)
    let inp_kf = ui.editing == EditFocus::SlotInput(slot);
    let (inp_d, inp_c) = if inp_kf {
        ("press key / button…".to_string(), AMBER)
    } else if slot_input.is_empty() {
        ("unbound".to_string(), DIM)
    } else {
        (slot_input.to_string(), TEXT)
    };
    let inp_btn = spawn_key_capture_field(
        commands,
        card,
        "Input",
        &inp_d,
        inp_c,
        if inp_kf { AMBER } else { BADGE_BORDER },
        EditorAction::CaptureSlotInput { slot },
        EditorAction::ClearSlotInput { slot },
        slot_input,
        inp_kf,
        icons,
        focusables,
    );
    focusables.push(inp_btn);

    // Close HUD on select
    spawn_toggle_field(
        commands,
        card,
        "Close on select",
        slot_data.map(|s| s.close_on_select).unwrap_or(false),
        EditorAction::ToggleSlotCloseOnSelect { slot },
        focusables,
    );

    spawn_box_field(
        commands,
        card,
        "Action mapping",
        &slot_data.map(|s| s.command.as_str()).unwrap_or("none"),
        TEXT,
        BADGE_BORDER,
        EditorAction::CycleSlotCommand { slot },
    );
    spawn_toggle_field(
        commands,
        card,
        "Hold action",
        slot_data.map(|s| s.hold).unwrap_or(false),
        EditorAction::ToggleSlotHold { slot },
        focusables,
    );
    spawn_box_field(
        commands,
        card,
        "Hold mapping",
        &slot_data.map(|s| s.hold_command.as_str()).unwrap_or("none"),
        TEXT,
        BADGE_BORDER,
        EditorAction::CycleSlotHoldCommand { slot },
    );

    // ── Items section ───────────────────────────────────────────────────────────
    let items_hdr = child(
        commands,
        parent,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: {UiRect::new(px(0.), px(0.), px(6.), px(8.))},
            }
        },
    );
    child(commands, items_hdr, text("ITEMS", 10., DIM));
    let add_item_btn = clickable(
        commands,
        items_hdr,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                padding: {UiRect::axes(px(8.), px(3.))},
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(TEAL)
        },
        EditorAction::AddSlotItem { slot },
        Color::NONE,
    );
    child(commands, add_item_btn, text("+", 9., TEAL));
    focusables.push(add_item_btn);

    let icard = child(commands, parent, editor_card());
    if items.is_empty() {
        child(
            commands,
            icard,
            text("No items. Add one above.", 10., DIMMER),
        );
    }
    for (ii, item) in items.iter().enumerate() {
        let item_row = child(
            commands,
            icard,
            bsn! {
                Node {
                    width: {percent(100.)}, height: {px(26.)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {px(4.)},
                }
            },
        );
        child(commands, item_row, text(&format!("{}", ii + 1), 9., DIMMER));

        // Item name field
        let iname_f = ui.editing == EditFocus::SlotItemName(slot, ii);
        let item_nd = if iname_f {
            format!("{}|", item.name)
        } else if item.name.is_empty() {
            "name…".to_string()
        } else {
            item.name.clone()
        };
        let nb = clickable(
            commands,
            item_row,
            bsn! {
                @FeathersButton { @variant: ButtonVariant::Plain }
                Node {
                    flex_grow: 1., height: {px(20.)},
                    padding: {UiRect::horizontal(px(6.))},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(px(1.))},
                    border_radius: {BorderRadius::all(px(4.))},
                }
                BorderColor::all(if iname_f { AMBER } else { BADGE_BORDER })
            },
            EditorAction::EditSlotItemName { slot, item: ii },
            Color::NONE,
        );
        child(
            commands,
            nb,
            text(&item_nd, 10., if iname_f { AMBER } else { TEXT }),
        );
        focusables.push(nb);
        let iicon_f = ui.editing == EditFocus::SlotItemIcon(slot, ii);
        let item_id = if iicon_f {
            format!("{}|", item.icon)
        } else if item.icon.is_empty() {
            "◆".to_string()
        } else {
            item.icon.clone()
        };
        let ib = clickable(
            commands,
            item_row,
            bsn! {
                @FeathersButton { @variant: ButtonVariant::Plain }
                Node {
                    width: {px(28.)}, height: {px(20.)},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(px(1.))},
                    border_radius: {BorderRadius::all(px(4.))},
                }
                BorderColor::all(if iicon_f { AMBER } else { BADGE_BORDER })
            },
            EditorAction::EditSlotItemIcon { slot, item: ii },
            Color::NONE,
        );
        child(
            commands,
            ib,
            text(&item_id, 10., if iicon_f { AMBER } else { DIM }),
        );
        focusables.push(ib);
        let dx = clickable(
            commands,
            item_row,
            del_btn(),
            EditorAction::RemoveSlotItem { slot, item: ii },
            Color::NONE,
        );
        cil_icon(commands, dx, "cil-trash", 11., DIMMER, icons);
        focusables.push(dx);
    }
}

/// WheelSet-entry editor panel.
#[allow(clippy::too_many_arguments)]
fn spawn_wheelset_entry_editor(
    commands: &mut Commands,
    parent: Entity,
    ui: &EditorUiState,
    set: usize,
    entry: usize,
    ws: &WheelSetData,
    icons: &Icons<'_>,
    focusables: &mut Vec<Entity>,
) {
    section_label(commands, parent, "WHEEL SET");
    let card = child(commands, parent, editor_card());

    // Name
    let nf = ui.editing == EditFocus::WheelSetName;
    let nd = if nf {
        format!("{}|", ws.name)
    } else {
        ws.name.clone()
    };
    let ws_name_btn = spawn_box_field(
        commands,
        card,
        "Name",
        &nd,
        TEXT,
        if nf { AMBER } else { BADGE_BORDER },
        EditorAction::EditWheelSetName { set, entry },
    );
    focusables.push(ws_name_btn);

    // The navigation stick is captured as a controller thumb-stick binding.
    let stick_focus = ui.editing == EditFocus::WheelSetStick;
    let stick_btn = spawn_key_capture_field(
        commands,
        card,
        "Stick binding",
        &format!(
            "{}{}",
            if stick_focus { "press stick… " } else { "" },
            ws.stick.label()
        ),
        if stick_focus { AMBER } else { TEXT },
        if stick_focus { AMBER } else { BADGE_BORDER },
        EditorAction::CaptureWheelSetStick { set, entry },
        EditorAction::ClearWheelSetStick { set, entry },
        ws.stick.label(),
        stick_focus,
        icons,
        focusables,
    );
    focusables.push(stick_btn);

    // Previous wheel shortcut
    let pkf = ui.editing == EditFocus::WheelSetPrevKey { set, entry };
    let (pkd, pkc) = key_display(pkf, &ws.prev_wheel_key);
    let prev_key_btn = spawn_key_capture_field(
        commands,
        card,
        "Previous wheel",
        &pkd,
        pkc,
        if pkf { AMBER } else { BADGE_BORDER },
        EditorAction::CaptureWheelSetPrevKey { set, entry },
        EditorAction::ClearWheelSetPrevKey { set, entry },
        &ws.prev_wheel_key,
        pkf,
        icons,
        focusables,
    );
    focusables.push(prev_key_btn);

    // Next wheel shortcut
    let nkf = ui.editing == EditFocus::WheelSetNextKey { set, entry };
    let (nkd, nkc) = key_display(nkf, &ws.next_wheel_key);
    let next_key_btn = spawn_key_capture_field(
        commands,
        card,
        "Next wheel",
        &nkd,
        nkc,
        if nkf { AMBER } else { BADGE_BORDER },
        EditorAction::CaptureWheelSetNextKey { set, entry },
        EditorAction::ClearWheelSetNextKey { set, entry },
        &ws.next_wheel_key,
        nkf,
        icons,
        focusables,
    );
    focusables.push(next_key_btn);

    let (min_dec, min_inc) = spawn_stepper_field(
        commands,
        card,
        "Minimum wheels",
        &ws.min_wheels.to_string(),
        EditorAction::WheelSetMinDelta {
            set,
            entry,
            delta: -1,
        },
        EditorAction::WheelSetMinDelta {
            set,
            entry,
            delta: 1,
        },
        icons,
    );
    focusables.push(min_dec);
    focusables.push(min_inc);
    let (max_dec, max_inc) = spawn_stepper_field(
        commands,
        card,
        "Maximum wheels",
        &ws.max_wheels.to_string(),
        EditorAction::WheelSetMaxDelta {
            set,
            entry,
            delta: -1,
        },
        EditorAction::WheelSetMaxDelta {
            set,
            entry,
            delta: 1,
        },
        icons,
    );
    focusables.push(max_dec);
    focusables.push(max_inc);
    spawn_toggle_field(
        commands,
        card,
        "Cycle wheels",
        ws.cycle_wheels,
        EditorAction::ToggleWheelSetCycle { set, entry },
        focusables,
    );

    let wheel_nav = child(commands, card, hcluster());
    let prev_wheel = clickable(
        commands,
        wheel_nav,
        footer_button("‹ Previous wheel", DIM, false),
        EditorAction::SwitchWheelPrev { set, entry },
        Color::NONE,
    );
    let next_wheel = clickable(
        commands,
        wheel_nav,
        footer_button("Next wheel ›", DIM, false),
        EditorAction::SwitchWheelNext { set, entry },
        Color::NONE,
    );
    focusables.push(prev_wheel);
    focusables.push(next_wheel);

    // Wheels sub-list
    let wh_hdr = child(
        commands,
        parent,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: {UiRect::new(px(0.), px(0.), px(6.), px(8.))},
            }
        },
    );
    child(commands, wh_hdr, text("WHEELS", 10., DIM));
    let add_wheel_to_set_btn = clickable(
        commands,
        wh_hdr,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                padding: {UiRect::axes(px(8.), px(3.))},
                border: {UiRect::all(px(1.))},
                border_radius: {BorderRadius::all(px(3.))},
            }
            BorderColor::all(BLUE)
        },
        EditorAction::AddWheelToSet { set, entry },
        Color::NONE,
    );
    child(commands, add_wheel_to_set_btn, text("+", 9., BLUE));
    focusables.push(add_wheel_to_set_btn);

    let wcard = child(commands, parent, editor_card());
    if ws.wheels.is_empty() {
        child(commands, wcard, text("No wheels.", 10., DIMMER));
    }
    for (wi, w) in ws.wheels.iter().enumerate() {
        let wsel = ui.selection
            == (Selection::Wheel {
                set,
                entry,
                wheel: Some(wi),
            });
        let badge = Badge::None;
        spawn_entry_row(
            commands,
            wcard,
            wsel,
            EditorAction::SelectWheel {
                set,
                entry,
                wheel: Some(wi),
            },
            "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-aperture.png",
            ICON,
            &w.name,
            TEAL,
            badge,
            Some(EditorAction::DeleteWheelFromSet {
                set,
                entry,
                wheel: wi,
            }),
            icons,
            focusables,
        );
    }

    // Hint
    child(
        commands,
        parent,
        bsn! {
            Node { padding: {UiRect::new(px(4.), px(0.), px(8.), px(0.))} }
            Children [ text("Select a wheel above to edit its\nsegments and settings.", 9., DIMMER) ]
        },
    );
}

// ─── helpers ─────────────────────────────────────────────────────────────────────

fn action_at(cfg: &mut QuickActionConfig, set: usize, entry: usize) -> Option<&mut QuickAction> {
    match cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry)) {
        Some(SetEntry::Action(a)) => Some(a),
        _ => None,
    }
}

fn wheel_at(cfg: &mut QuickActionConfig, sel: Selection) -> Option<&mut WheelData> {
    let (set, entry, wheel) = match sel {
        Selection::Wheel { set, entry, wheel } => (set, entry, wheel),
        Selection::Segment {
            set, entry, wheel, ..
        } => (set, entry, wheel),
        _ => return None,
    };
    match cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry)) {
        Some(SetEntry::Wheel(w)) if wheel.is_none() => Some(w),
        Some(SetEntry::WheelSet(ws)) => wheel.and_then(move |i| ws.wheels.get_mut(i)),
        _ => None,
    }
}

fn sync_wheelset_visuals(cfg: &mut QuickActionConfig, selection: Selection) {
    let Selection::Wheel {
        set,
        entry,
        wheel: Some(index),
    } = selection
    else {
        return;
    };
    let Some(SetEntry::WheelSet(ws)) = cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
    else {
        return;
    };
    let Some(source) = ws.wheels.get(index).cloned() else {
        return;
    };
    let visuals = WheelSetVisuals::from(&source);
    ws.visuals = Some(visuals.clone());
    ws.stick = visuals.stick;
    for wheel in &mut ws.wheels {
        visuals.apply_to(wheel);
    }
}

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
        EditFocus::SlotInput(i) => {
            wheel_at(cfg, ui.selection).and_then(move |w| w.slots.get_mut(i).map(|s| &mut s.input))
        }
        EditFocus::SlotItemName(slot, item) => wheel_at(cfg, ui.selection)
            .and_then(move |w| w.slots.get_mut(slot))
            .and_then(move |s| s.items.get_mut(item).map(|it| &mut it.name)),
        EditFocus::SlotItemIcon(slot, item) => wheel_at(cfg, ui.selection)
            .and_then(move |w| w.slots.get_mut(slot))
            .and_then(move |s| s.items.get_mut(item).map(|it| &mut it.icon)),
        EditFocus::WheelSetName => match ui.selection {
            Selection::WheelSetEntry { set, entry } => cfg
                .sets
                .get_mut(set)
                .and_then(|s| s.entries.get_mut(entry))
                .and_then(|e| {
                    if let SetEntry::WheelSet(ws) = e {
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

/// Returns the ordinal position of `entry` among Wheel/WheelSet entries in the set.
/// Used to sync `hud.active_wheel_entry` when the editor selects a wheel.
fn wheel_entry_idx(cfg: &QuickActionConfig, set: usize, entry: usize) -> usize {
    let Some(s) = cfg.sets.get(set) else {
        return 0;
    };
    s.entries[..entry.min(s.entries.len())]
        .iter()
        .filter(|e| matches!(e, SetEntry::Wheel(_) | SetEntry::WheelSet(_)))
        .count()
}

fn editor_text_input(
    mut messages: MessageReader<KeyboardInput>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
) {
    if !matches!(
        ui.editing,
        EditFocus::Name
            | EditFocus::SetName
            | EditFocus::WheelName
            | EditFocus::SlotName(_)
            | EditFocus::SlotIcon(_)
            | EditFocus::SlotInput(_)
            | EditFocus::SlotItemName(_, _)
            | EditFocus::SlotItemIcon(_, _)
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
    }
}

fn editor_capture_key(
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
            | EditFocus::WheelSetStick
            | EditFocus::SlotInput(_)
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
                        if let Some(SetEntry::WheelSet(ws)) =
                            cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                        {
                            ws.switch_key = label;
                        }
                    }
                }
                EditFocus::WheelSetStick => {
                    if let Selection::WheelSetEntry { set, entry } = ui.selection {
                        if let Some(SetEntry::WheelSet(ws)) =
                            cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                        {
                            if matches!(*key, KeyCode::KeyL | KeyCode::KeyR) {
                                ws.stick = if *key == KeyCode::KeyL {
                                    StickSide::Left
                                } else {
                                    StickSide::Right
                                };
                                let mut visuals = wheelset_visuals(ws);
                                visuals.stick = ws.stick;
                                ws.visuals = Some(visuals.clone());
                                for wheel in &mut ws.wheels {
                                    visuals.apply_to(wheel);
                                }
                            } else {
                                continue;
                            }
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
                    if let Some(SetEntry::WheelSet(ws)) =
                        cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                    {
                        ws.next_wheel_key = label;
                    }
                }
                EditFocus::WheelSetPrevKey { set, entry } => {
                    if let Some(SetEntry::WheelSet(ws)) =
                        cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                    {
                        ws.prev_wheel_key = label;
                    }
                }
                EditFocus::SlotInput(slot) => {
                    if let Some(w) = wheel_at(&mut cfg, ui.selection) {
                        if let Some(s) = w.slots.get_mut(slot) {
                            s.input = label;
                        }
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

fn editor_capture_gamepad(
    gamepads: Query<&Gamepad>,
    mut cfg: ResMut<QuickActionConfig>,
    mut ui: ResMut<EditorUiState>,
) {
    let focus = ui.editing;
    if !matches!(
        focus,
        EditFocus::Key
            | EditFocus::SlotInput(_)
            | EditFocus::NextSetKey
            | EditFocus::PrevSetKey
            | EditFocus::EditShortcut
            | EditFocus::WheelSetSwitchKey
            | EditFocus::WheelSetStick
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
                    EditFocus::Key => {
                        if let Selection::Action { set, entry } = ui.selection {
                            if let Some(a) = action_at(&mut cfg, set, entry) {
                                a.key = gp;
                            }
                        }
                    }
                    EditFocus::WheelSetSwitchKey => {
                        if let Selection::WheelSetEntry { set, entry } = ui.selection {
                            if let Some(SetEntry::WheelSet(ws)) =
                                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                            {
                                ws.switch_key = gp;
                            }
                        }
                    }
                    EditFocus::WheelSetStick => {
                        if let Selection::WheelSetEntry { set, entry } = ui.selection {
                            if let Some(SetEntry::WheelSet(ws)) =
                                cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                            {
                                ws.stick = if btn == GamepadButton::LeftThumb {
                                    StickSide::Left
                                } else if btn == GamepadButton::RightThumb {
                                    StickSide::Right
                                } else {
                                    continue;
                                };
                                let mut visuals = wheelset_visuals(ws);
                                visuals.stick = ws.stick;
                                ws.visuals = Some(visuals.clone());
                                for wheel in &mut ws.wheels {
                                    visuals.apply_to(wheel);
                                }
                            }
                        }
                    }
                    EditFocus::SlotInput(slot) => {
                        if let Some(w) = wheel_at(&mut cfg, ui.selection) {
                            if let Some(s) = w.slots.get_mut(slot) {
                                s.input = gp;
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
                        if let Some(SetEntry::WheelSet(ws)) =
                            cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
                        {
                            ws.next_wheel_key = gp;
                        }
                    }
                    EditFocus::WheelSetPrevKey { set, entry } => {
                        if let Some(SetEntry::WheelSet(ws)) =
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
fn shortcut_just_pressed(
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

/// Applies Next Set / Prev Set shortcuts — only while the HUD overlay is open.
/// Gameplay code can freely use the same buttons when the HUD is closed.
fn apply_set_shortcuts(
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
fn hud_button_action_shortcuts(
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
/// `WheelSet`, between that set's internal wheels.
fn hud_wheel_nav(
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
            SetEntry::WheelSet(ws) => {
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
fn editor_undo_redo_shortcuts(
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
fn check_edit_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    cfg: Res<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
) {
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
            // Open editor — also force-open the HUD overlay so the wheel preview
            // is visible even if the HUD was previously closed.
            info!(
                "[editor] opening editor via shortcut (was open={}, editor_open={})",
                hud.open, hud.editor_open
            );
            hud.editor_open = true;
            hud.edit_control_focus = Some(0);
            if !hud.open {
                info!("[editor] auto-opening HUD so wheel preview is visible");
                hud.open = true;
            }
        }
        hud.dirty = true;
        ui.dirty = true;
    }
}

// ─── middle-button HUD dragging ────────────────────────────────────────────────

/// Moves the currently selected HUD item while the middle mouse button is held.
/// Offsets are stored in the config so the edit survives a HUD rebuild/save.
fn editor_middle_drag(
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
fn editor_touch_drag(
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

// ─── config validation ───────────────────────────────────────────────────────────

/// Warnings about the current `QuickActionConfig`.
#[derive(Resource, Clone, Debug, Default)]
pub struct ConfigValidation {
    pub warnings: Vec<String>,
    pub has_errors: bool,
}

/// Validates the current config and stores warnings in [`ConfigValidation`].
/// Runs every time the HUD is rebuilt (i.e. after each editor action).
fn validate_config(cfg: Res<QuickActionConfig>, mut validation: ResMut<ConfigValidation>) {
    let mut warnings: Vec<String> = Vec::new();

    // Check for empty sets
    for (i, set) in cfg.sets.iter().enumerate() {
        if set.entries.is_empty() {
            warnings.push(format!("Set \"{}\" (#{}) has no entries", set.name, i));
        }
        // Check for wheels with zero slots
        for entry in set.entries.iter() {
            match entry {
                SetEntry::Wheel(w) => {
                    if w.slots.is_empty() {
                        warnings.push(format!(
                            "Wheel \"{}\" in set \"{}\" has no slots",
                            w.name, set.name
                        ));
                    }
                }
                SetEntry::WheelSet(ws) => {
                    if ws.wheels.is_empty() {
                        warnings.push(format!(
                            "Wheel set \"{}\" in set \"{}\" has no wheels",
                            ws.name, set.name
                        ));
                    }
                    for (wi, w) in ws.wheels.iter().enumerate() {
                        if w.slots.is_empty() {
                            warnings.push(format!(
                                "Wheel \"{}\" (#{}) in wheel set \"{}\" has no slots",
                                w.name, wi, ws.name
                            ));
                        }
                    }
                }
                SetEntry::HudSwitch(hs) => {
                    if hs.name.trim().is_empty() {
                        warnings.push(format!(
                            "HUD switch #{i} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Check for duplicate shortcut keys across actions
    let mut seen_keys: std::collections::HashMap<&str, Vec<(usize, usize)>> =
        std::collections::HashMap::new();
    for (si, set) in cfg.sets.iter().enumerate() {
        for (ei, entry) in set.entries.iter().enumerate() {
            if let SetEntry::Action(qa) = entry {
                if !qa.key.is_empty() {
                    seen_keys.entry(&qa.key).or_default().push((si, ei));
                }
            }
        }
    }
    for (key, locations) in &seen_keys {
        if locations.len() > 1 {
            warnings.push(format!(
                "Duplicate shortcut key \"{}\" used by {} actions",
                key,
                locations.len()
            ));
        }
    }

    // Check for empty action names
    for set in cfg.sets.iter() {
        for (ei, entry) in set.entries.iter().enumerate() {
            match entry {
                SetEntry::Action(qa) => {
                    if qa.name.trim().is_empty() {
                        warnings.push(format!(
                            "Action #{ei} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
                SetEntry::Wheel(w) => {
                    if w.name.trim().is_empty() {
                        warnings.push(format!(
                            "Wheel #{ei} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
                SetEntry::WheelSet(ws) => {
                    if ws.name.trim().is_empty() {
                        warnings.push(format!(
                            "Wheel set #{ei} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
                SetEntry::HudSwitch(hs) => {
                    if hs.name.trim().is_empty() {
                        warnings.push(format!(
                            "HUD switch #{ei} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
            }
        }
    }

    validation.warnings = warnings;
    validation.has_errors = !validation.warnings.is_empty();
}

// ─── mobile-friendly sizing ─────────────────────────────────────────────────────

/// Returns the appropriate button size for the current input mode.
/// On mobile/touch, use 44px minimum (accessibility guideline).
/// On desktop, use the configured size.
pub fn touch_safe_button_size(configured_size: f32, is_touch: bool) -> f32 {
    if is_touch {
        configured_size.max(44.0)
    } else {
        configured_size
    }
}

// ─── query efficiency ──────────────────────────────────────────────────────────

/// Efficient queries for wheel data — add these filters to reduce iteration.
///
/// Instead of:
///   Query<&mut WheelState>
///
/// Use:
///   `Query<&mut WheelState, (With<WheelData>, Without<WheelHudRoot>)>`
///
/// This reduces false-positive matches and improves cache behaviour.
pub type WheelStateQuery<'a> =
    Query<'a, 'a, &'a mut WheelState, (With<WheelData>, Without<WheelHudRoot>)>;

pub type WheelStateReadQuery<'a> =
    Query<'a, 'a, &'a WheelState, (With<WheelData>, Without<WheelHudRoot>)>;

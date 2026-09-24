//! Actions dispatched by the in-canvas editor.

use crate::WheelTheme;

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
    /// Clear the current editor selection.
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
    ToggleWheelThemePopup,
    SetWheelTheme {
        theme: WheelTheme,
    },
    CaptureWheelStick,
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
    /// Begin capturing the global edit shortcut.
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
    ToggleWheelShowIcon,
    CycleHighlightColor,
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
    /// Toggle stick side for the active standalone wheel.
    CycleWheelStick,
    /// Toggle stick side for the selected RadialMenuSetState entry.
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

//! HUD presentation intents emitted by clickable overlay controls.

/// Actions emitted by interactive HUD presentation controls.
#[derive(Clone, Debug)]
pub enum WheelHudAction {
    SetActiveSet(usize),
    PrevSet,
    NextSet,
    ToggleEditor,
    /// Add a segment to the wheel currently shown in the HUD editor.
    AddSegment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        side: SegmentInsertSide,
    },
    /// Remove the selected segment from the wheel currently shown in the HUD editor.
    RemoveSegment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    EditSegmentName {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    EditSegmentIcon {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    EditSegmentInput {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    CycleSegmentMapping {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    ToggleSegmentHold {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    CycleSegmentHoldAction {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    ToggleSegmentCloseOnApply {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    DeleteSegment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    SaveConfig,
    AddNewButton,
    ToggleSettings,
    SelectAction {
        set: usize,
        entry: usize,
    },
    DeleteAction {
        set: usize,
        entry: usize,
    },
    MoveAction {
        set: usize,
        entry: usize,
    },
    RotateAction {
        set: usize,
        entry: usize,
        delta: f32,
    },
    EditAction {
        set: usize,
        entry: usize,
    },
    SelectHudSwitch {
        set: usize,
        entry: usize,
    },
    DeleteHudSwitch {
        set: usize,
        entry: usize,
    },
    MoveHudSwitch {
        set: usize,
        entry: usize,
    },
    ResizeHudSwitch {
        set: usize,
        entry: usize,
        delta: f32,
    },
    EditHudSwitch {
        set: usize,
        entry: usize,
    },
    EditActionName {
        set: usize,
        entry: usize,
    },
    CaptureActionKey {
        set: usize,
        entry: usize,
    },
    CycleActionIcon {
        set: usize,
        entry: usize,
    },
    CycleActionMapping {
        set: usize,
        entry: usize,
    },
    ToggleActionHold {
        set: usize,
        entry: usize,
    },
    CycleHoldAction {
        set: usize,
        entry: usize,
    },
    ToggleActionCloseOnApply {
        set: usize,
        entry: usize,
    },
    ActionWidthDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    ActionHeightDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    ActionRadiusDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    CycleActionPosition {
        set: usize,
        entry: usize,
    },
    WheelSettings {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    MoveWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    DeleteWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    ResizeWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        delta: f32,
    },
    SelectWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    CloseSelection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentInsertSide {
    Before,
    After,
    Outer,
}

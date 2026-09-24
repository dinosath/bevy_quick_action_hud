//! Editor selection, focus, and transient interaction state.

use crate::QuickActionConfig;
use bevy::prelude::*;

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
    WheelSetName,
    WheelSetSwitchKey,
    WheelSetStick,
    SlotIcon(usize),
    SlotInput(usize),
    SlotItemName(usize, usize),
    SlotItemIcon(usize, usize),
    EditShortcut,
    SetBgImage(usize),
    NextWheelKey(usize),
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

#[derive(Resource)]
pub struct EditorUiState {
    pub dirty: bool,
    pub selection: Selection,
    pub editing: EditFocus,
    pub config_path: String,
    pub wheel_scroll_y: f32,
    pub navfocus: usize,
    pub nav_count: usize,
    pub scroll_to_focus: bool,
    pub capture_skip: bool,
    pub capture_consumed: bool,
    pub nav_hold_dir: i32,
    pub nav_hold_timer: f32,
    pub undo_stack: Vec<QuickActionConfig>,
    pub redo_stack: Vec<QuickActionConfig>,
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
pub(super) enum MiddleDragTarget {
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
pub(super) struct HudMiddleDrag {
    pub(super) target: Option<MiddleDragTarget>,
}

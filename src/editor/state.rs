//! Editor selection, focus, and transient interaction state.

use crate::QuickActionConfig;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
/// What the editor currently owns or highlights.
pub enum Selection {
    /// No editor item is selected.
    #[default]
    None,
    /// A floating action button.
    Action {
        /// Page index.
        set: usize,
        /// Entry index within the page.
        entry: usize,
    },
    /// A HUD page switch control.
    HudSwitch {
        /// Page index.
        set: usize,
        /// Entry index within the page.
        entry: usize,
    },
    /// A radial menu, optionally inside a radial-menu set.
    Wheel {
        /// Page index.
        set: usize,
        /// Entry index within the page.
        entry: usize,
        /// Wheel index within a set.
        wheel: Option<usize>,
    },
    /// A HUD page.
    Set {
        /// Page index.
        set: usize,
    },
    /// The page-switch configuration.
    SetSwitch,
    /// A radial-menu set entry.
    WheelSetEntry {
        /// Page index.
        set: usize,
        /// Entry index within the page.
        entry: usize,
    },
    /// A sector within a radial menu.
    Segment {
        /// Page index.
        set: usize,
        /// Entry index within the page.
        entry: usize,
        /// Wheel index within a set.
        wheel: Option<usize>,
        /// Sector index within the wheel.
        slot: usize,
    },
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
/// Text, binding, or navigation field currently receiving input.
pub enum EditFocus {
    /// No text or binding capture is active.
    #[default]
    None,
    /// Generic name input.
    Name,
    /// Legacy generic binding input.
    Key,
    /// Button binding input.
    ButtonKey,
    /// HUD switch binding input.
    HudSwitchKey,
    /// Page name input.
    SetName,
    /// Radial menu name input.
    WheelName,
    /// Radial menu stick capture.
    WheelStick,
    /// Sector name input for the indexed sector.
    SlotName(usize),
    /// Previous-page shortcut capture.
    NextSetKey,
    /// Next-page shortcut capture.
    PrevSetKey,
    /// Wheel-set name input.
    WheelSetName,
    /// Wheel-set switch shortcut capture.
    WheelSetSwitchKey,
    /// Wheel-set stick capture.
    WheelSetStick,
    /// Sector icon input for the indexed sector.
    SlotIcon(usize),
    /// Editor shortcut capture.
    EditShortcut,
    /// Page background image input.
    SetBgImage(usize),
    /// Next-wheel shortcut capture for a standalone wheel.
    NextWheelKey(usize),
    /// Previous-wheel shortcut capture for a standalone wheel.
    PrevWheelKey(usize),
    /// Next-wheel shortcut capture for a wheel-set entry.
    WheelSetNextKey {
        /// Page index.
        set: usize,
        /// Entry index.
        entry: usize,
    },
    /// Previous-wheel shortcut capture for a wheel-set entry.
    WheelSetPrevKey {
        /// Page index.
        set: usize,
        /// Entry index.
        entry: usize,
    },
}

#[derive(Resource)]
/// Retained editor interaction state and undo history.
pub struct EditorUiState {
    /// Whether editor-owned retained UI needs rebuilding.
    pub dirty: bool,
    /// Current editor selection.
    pub selection: Selection,
    /// Active text or input-capture target.
    pub editing: EditFocus,
    /// Persistence path for the authored configuration.
    pub config_path: String,
    /// Vertical scroll offset of the editor list.
    pub wheel_scroll_y: f32,
    /// Keyboard/gamepad navigation focus index.
    pub navfocus: usize,
    /// Number of currently navigable controls.
    pub nav_count: usize,
    /// Whether the next frame should scroll to the focused control.
    pub scroll_to_focus: bool,
    /// Suppresses the first input event after capture begins.
    pub capture_skip: bool,
    /// Whether the current capture event has been consumed.
    pub capture_consumed: bool,
    /// Current held radial-navigation direction.
    pub nav_hold_dir: i32,
    /// Timer used by repeated radial navigation.
    pub nav_hold_timer: f32,
    /// Undo snapshots, newest last.
    pub undo_stack: Vec<QuickActionConfig>,
    /// Redo snapshots, newest last.
    pub redo_stack: Vec<QuickActionConfig>,
    /// Maximum number of undo snapshots retained.
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
    /// Floating action being dragged.
    Action {
        /// Page index.
        set: usize,
        /// Entry index.
        entry: usize,
    },
    /// Radial menu being dragged.
    Wheel {
        /// Page index.
        set: usize,
        /// Entry index.
        entry: usize,
        /// Wheel index within a set.
        wheel: Option<usize>,
    },
}

#[derive(Resource, Default)]
pub(super) struct HudMiddleDrag {
    pub(super) target: Option<MiddleDragTarget>,
}

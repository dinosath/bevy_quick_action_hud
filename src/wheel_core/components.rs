//! Entity-local state for the core wheel runtime.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{CastingMode, TimeMode, WheelToggleMode};

/// Marks a slice within a wheel menu.
#[derive(Component, Clone)]
pub struct SectorEntity {
    /// Index of this slice (0-based).
    pub index: usize,
}

/// Optional content for a wheel slice.
#[derive(Component, Clone, Default)]
pub struct SectorContent {
    /// Label text for this slice.
    pub label: Option<String>,
    /// Icon path/identifier for this slice.
    pub icon: Option<String>,
}

/// Current input state of a wheel menu.
#[derive(Component, Default, Clone)]
pub struct RadialMenuState {
    /// Current input direction (normalized).
    pub dir: Vec2,
    /// Currently hovered slice index.
    pub hovered: Option<usize>,
    /// Whether the wheel was open (any slice hovered) last frame.
    pub open: bool,
}

/// Full runtime configuration for a wheel menu.
#[derive(Component, Clone)]
pub struct RadialMenuConfig {
    pub time_mode: TimeMode,
    pub casting_mode: CastingMode,
    pub toggle_mode: WheelToggleMode,
    pub auto_snap: bool,
    pub block_gameplay_input: bool,
}

impl Default for RadialMenuConfig {
    fn default() -> Self {
        Self {
            time_mode: TimeMode::Normal,
            casting_mode: CastingMode::Vanilla,
            toggle_mode: WheelToggleMode::Hold,
            auto_snap: true,
            block_gameplay_input: false,
        }
    }
}

/// Runtime progress for hold-to-activate casting.
#[derive(Component, Default, Clone)]
pub struct RadialMenuHoldState {
    pub progress: f32,
    pub holding: bool,
}

/// Runtime active-wheel state for a wheel set.
#[derive(Component, Clone)]
pub struct RadialMenuSetState {
    pub active: usize,
    pub count: usize,
    pub prev_button: GamepadButton,
    pub next_button: GamepadButton,
}

impl Default for RadialMenuSetState {
    fn default() -> Self {
        Self {
            active: 0,
            count: 1,
            prev_button: GamepadButton::LeftTrigger,
            next_button: GamepadButton::RightTrigger,
        }
    }
}

/// Runtime item-count data for a slice.
#[derive(Component, Clone, Default)]
pub struct SectorCount {
    pub current: u32,
    pub max: u32,
    pub low_threshold: u32,
    pub low_notified: bool,
}

/// Runtime edit-mode state for a wheel.
#[derive(Component, Default, Clone)]
pub struct RadialMenuEditMode {
    pub active: bool,
    pub toggle_button: Option<GamepadButton>,
}

/// Visual configuration / skin for a wheel.
#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RadialMenuStyle {
    pub base_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub selected_color: [f32; 4],
    pub text_color: [f32; 4],
    pub skin: String,
}

impl Default for RadialMenuStyle {
    fn default() -> Self {
        Self {
            base_color: [0.08, 0.12, 0.18, 0.85],
            hover_color: [0.2, 0.5, 0.9, 0.95],
            selected_color: [0.1, 0.7, 0.4, 0.9],
            text_color: [0.85, 0.88, 0.92, 1.0],
            skin: "default".into(),
        }
    }
}

impl RadialMenuStyle {
    pub fn base(&self) -> Color {
        Color::srgba(
            self.base_color[0],
            self.base_color[1],
            self.base_color[2],
            self.base_color[3],
        )
    }

    pub fn hover(&self) -> Color {
        Color::srgba(
            self.hover_color[0],
            self.hover_color[1],
            self.hover_color[2],
            self.hover_color[3],
        )
    }

    pub fn selected(&self) -> Color {
        Color::srgba(
            self.selected_color[0],
            self.selected_color[1],
            self.selected_color[2],
            self.selected_color[3],
        )
    }

    pub fn text(&self) -> Color {
        Color::srgba(
            self.text_color[0],
            self.text_color[1],
            self.text_color[2],
            self.text_color[3],
        )
    }
}

/// Sound asset paths associated with wheel lifecycle actions.
#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct RadialMenuAudio {
    pub open: Option<String>,
    pub hover: Option<String>,
    pub select: Option<String>,
    pub submenu: Option<String>,
}

/// Runtime parent/child links for nested submenu wheels.
#[derive(Component, Clone, Default)]
pub struct RadialMenuHierarchy {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

// Legacy runtime names remain aliases, so there is one definition and one
// implementation for each concept.
pub type WheelSlice = SectorEntity;
pub type WheelSliceContent = SectorContent;
pub type WheelState = RadialMenuState;
pub type WheelMenuConfig = RadialMenuConfig;
pub type WheelHoldState = RadialMenuHoldState;
pub type WheelSet = RadialMenuSetState;
pub type WheelSliceCount = SectorCount;
pub type WheelEditMode = RadialMenuEditMode;
pub type WheelStyle = RadialMenuStyle;
pub type WheelAudio = RadialMenuAudio;
pub type WheelHierarchy = RadialMenuHierarchy;

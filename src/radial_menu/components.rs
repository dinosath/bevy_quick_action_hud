//! Entity-local runtime state for the core radial-menu input pipeline.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// How an action is confirmed and triggered from a radial menu.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub enum CastingMode {
    /// Standard: press the confirm button while hovering a sector.
    #[default]
    Vanilla,
    /// Release the stick from a hovered sector to select it.
    ReleaseToUse,
    /// Dwell on a sector for `duration` seconds to trigger it.
    HoldToActivate {
        /// Required hold duration in seconds.
        duration: f32,
    },
    /// Activate immediately when a new sector is hovered.
    Direct,
}

/// How a radial menu opens and closes.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub enum RadialMenuToggleMode {
    /// Press the hotkey once to open; press again to close.
    Toggle,
    /// Hold the key or button to keep open; release to close.
    #[default]
    Hold,
    /// Short press toggles; long press uses hold behavior.
    Hybrid {
        /// Threshold separating a short press from a hold.
        hold_threshold_secs: f32,
    },
}

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
    /// Policy used to confirm a hovered sector.
    pub casting_mode: CastingMode,
    /// Policy used to open and close the menu.
    pub toggle_mode: RadialMenuToggleMode,
    /// Whether directional input is snapped toward a sector center.
    pub auto_snap: bool,
    /// Whether the menu consumes gameplay input while open.
    pub block_gameplay_input: bool,
}

impl Default for RadialMenuConfig {
    fn default() -> Self {
        Self {
            casting_mode: CastingMode::Vanilla,
            toggle_mode: RadialMenuToggleMode::Hold,
            auto_snap: true,
            block_gameplay_input: false,
        }
    }
}

/// Runtime progress for hold-to-activate casting.
#[derive(Component, Default, Clone)]
pub struct RadialMenuHoldState {
    /// Normalized hold progress from zero to one.
    pub progress: f32,
    /// Whether the activation input is currently held.
    pub holding: bool,
}

/// Runtime item-count data for a slice.
#[derive(Component, Clone, Default)]
pub struct SectorCount {
    /// Current quantity represented by the sector.
    pub current: u32,
    /// Maximum quantity represented by the sector.
    pub max: u32,
    /// Quantity at or below which a low-count message is emitted.
    pub low_threshold: u32,
    /// Prevents repeated low-count notifications until reset.
    pub low_notified: bool,
}

/// Runtime edit-mode state for a wheel.
#[derive(Component, Default, Clone)]
pub struct RadialMenuEditMode {
    /// Whether editor interactions are enabled for this menu.
    pub active: bool,
    /// Optional gamepad button used to toggle edit mode.
    pub toggle_button: Option<GamepadButton>,
}

/// Visual configuration / skin for a wheel.
#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RadialMenuStyle {
    /// Normal sector color in RGBA component values.
    pub base_color: [f32; 4],
    /// Hovered sector color.
    pub hover_color: [f32; 4],
    /// Selected sector color.
    pub selected_color: [f32; 4],
    /// Text color.
    pub text_color: [f32; 4],
    /// Application-defined skin identifier.
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
    /// Converts the base color into a Bevy color.
    pub fn base(&self) -> Color {
        rgba(self.base_color)
    }

    /// Converts the hover color into a Bevy color.
    pub fn hover(&self) -> Color {
        rgba(self.hover_color)
    }

    /// Converts the selected color into a Bevy color.
    pub fn selected(&self) -> Color {
        rgba(self.selected_color)
    }

    /// Converts the text color into a Bevy color.
    pub fn text(&self) -> Color {
        rgba(self.text_color)
    }
}

fn rgba([red, green, blue, alpha]: [f32; 4]) -> Color {
    Color::srgba(red, green, blue, alpha)
}

/// Sound asset paths associated with wheel lifecycle actions.
#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct RadialMenuAudio {
    /// Sound played when the menu opens.
    pub open: Option<String>,
    /// Sound played when the hovered sector changes.
    pub hover: Option<String>,
    /// Sound played when a sector is selected.
    pub select: Option<String>,
    /// Sound played when a submenu is opened.
    pub submenu: Option<String>,
}

/// Runtime parent/child links for nested submenu wheels.
#[derive(Component, Clone, Default)]
pub struct RadialMenuHierarchy {
    /// Optional parent menu entity.
    pub parent: Option<Entity>,
    /// Child submenu entities owned by this menu.
    pub children: Vec<Entity>,
}

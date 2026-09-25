//! Authored configuration for a floating HUD button.

use serde::{Deserialize, Serialize};

use crate::page::HudComponent;
use crate::serde_defaults::default_true;

/// Coordinate interpretation for a floating button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum PositionMode {
    #[default]
    /// Position relative to the HUD action layout.
    Relative,
    /// Position using absolute HUD coordinates.
    Absolute,
}
impl PositionMode {
    /// Returns the human-readable editor label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Relative => "Relative",
            Self::Absolute => "Absolute",
        }
    }
    /// Returns the next placement mode.
    pub fn next(self) -> Self {
        match self {
            Self::Relative => Self::Absolute,
            Self::Absolute => Self::Relative,
        }
    }
}

/// Shape used by a floating HUD button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum ActionShape {
    #[default]
    /// Rounded rectangle.
    Rounded,
    /// Circle-like button.
    Round,
    /// Square button.
    Square,
    /// Diamond-shaped button.
    Diamond,
}
impl ActionShape {
    /// Returns the human-readable editor label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Rounded => "Rounded",
            Self::Round => "Round",
            Self::Square => "Square",
            Self::Diamond => "Diamond",
        }
    }
    /// Returns the next button shape.
    pub fn next(self) -> Self {
        match self {
            Self::Rounded => Self::Round,
            Self::Round => Self::Square,
            Self::Square => Self::Diamond,
            Self::Diamond => Self::Rounded,
        }
    }
}

pub(crate) fn default_button_color() -> String {
    "#3b82f6".into()
}
pub(crate) fn default_button_width() -> f32 {
    80.0
}
pub(crate) fn default_button_height() -> f32 {
    28.0
}
fn default_hold_command() -> String {
    "none".into()
}

/// A key-bound floating HUD button.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct QuickAction {
    /// Display name shown on the button.
    pub name: String,
    /// Optional explanatory text for this HUD button.
    #[serde(default)]
    pub description: String,
    /// Keyboard key or gamepad button ("GP:\u{2026}" prefix) that triggers this action.
    pub key: String,
    /// Optional icon asset path or symbolic icon.
    pub icon: String,
    /// Action mapping invoked by the button.
    pub command: String,
    /// Command/mapping used while the input is held.
    #[serde(default = "default_hold_command")]
    pub hold_command: String,
    /// Whether holding the binding invokes `hold_command`.
    pub hold: bool,
    /// Whether the button is visible while the HUD is open.
    pub show_on_menu: bool,
    /// Minimum time between activations while the HUD is open.
    #[serde(default)]
    pub cooldown_secs: f32,
    #[serde(default = "default_true")]
    /// Whether the button label is rendered.
    pub show_labels: bool,
    #[serde(default = "default_true")]
    /// Whether the button icon is rendered.
    pub show_icon: bool,
    /// Button opacity.
    pub opacity: f32,
    /// Position interpretation used by the HUD layout.
    pub position: PositionMode,
    /// Radial distance from the action anchor.
    pub radius: f32,
    /// Horizontal editor offset from the HUD's default action area.
    #[serde(default)]
    pub offset_x: f32,
    /// Vertical editor offset from the HUD's default action area.
    #[serde(default)]
    pub offset_y: f32,
    /// Rotation of the floating button in degrees.
    #[serde(default)]
    pub rotation: f32,
    /// Button shape.
    pub shape: ActionShape,
    #[serde(default = "default_button_color")]
    /// Button fill color.
    pub color: String,
    #[serde(default = "default_button_width")]
    /// Button width in logical pixels.
    pub width: f32,
    #[serde(default = "default_button_height")]
    /// Button height in logical pixels.
    pub height: f32,
    #[serde(default = "default_true")]
    /// Whether the button is active.
    pub enabled: bool,
    /// Close the HUD overlay when this action's shortcut is pressed.
    #[serde(default = "default_true")]
    pub close_on_select: bool,
}
impl Default for QuickAction {
    fn default() -> Self {
        Self {
            name: "Action".into(),
            description: String::new(),
            key: String::new(),
            icon: "◆".into(),
            command: "none".into(),
            hold_command: "none".into(),
            hold: false,
            show_on_menu: true,
            cooldown_secs: 0.0,
            show_labels: true,
            show_icon: true,
            opacity: 1.0,
            position: PositionMode::Relative,
            radius: 48.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            shape: ActionShape::Rounded,
            color: default_button_color(),
            width: default_button_width(),
            height: default_button_height(),
            enabled: true,
            close_on_select: true,
        }
    }
}

impl HudComponent for QuickAction {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        self.enabled
    }
}

/// Compatibility alias for [`QuickAction`] as a HUD button.
pub type HudButton = QuickAction;

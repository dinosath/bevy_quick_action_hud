//! Authored configuration for a page-switch control.

use serde::{Deserialize, Serialize};

use crate::button::config::{default_button_height, default_button_width};
use crate::page::HudComponent;

/// A visible HUD component that switches to another enabled HUD page.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct HudSwitch {
    /// Display name shown on the switch.
    pub name: String,
    /// Binding that activates the switch.
    pub key: String,
    /// Index of the page activated by this switch.
    pub target_page: usize,
    /// Whether the switch participates in the HUD.
    pub enabled: bool,
    #[serde(default)]
    /// Horizontal switch offset.
    pub offset_x: f32,
    #[serde(default)]
    /// Vertical switch offset.
    pub offset_y: f32,
    #[serde(default = "default_button_width")]
    /// Switch width in logical pixels.
    pub width: f32,
    #[serde(default = "default_button_height")]
    /// Switch height in logical pixels.
    pub height: f32,
}
impl Default for HudSwitch {
    fn default() -> Self {
        Self {
            name: "HUD Switch".into(),
            key: String::new(),
            target_page: 0,
            enabled: true,
            offset_x: 0.0,
            offset_y: 0.0,
            width: 100.0,
            height: 28.0,
        }
    }
}

impl HudComponent for HudSwitch {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        self.enabled
    }
}

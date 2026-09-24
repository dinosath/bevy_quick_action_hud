//! Entity-local state for the core wheel runtime.

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
    HoldToActivate { duration: f32 },
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
    Hybrid { hold_threshold_secs: f32 },
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
    pub casting_mode: CastingMode,
    pub toggle_mode: RadialMenuToggleMode,
    pub auto_snap: bool,
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
    pub progress: f32,
    pub holding: bool,
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

#[cfg(test)]
mod tests {
    use super::RadialMenuToggleMode;

    #[test]
    fn toggle_mode_default_is_hold() {
        assert_eq!(RadialMenuToggleMode::default(), RadialMenuToggleMode::Hold);
    }
}

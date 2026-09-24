//! Runtime state owned by a radial-menu set.

use bevy::prelude::*;

/// Runtime active-menu state for a radial-menu set.
#[derive(Component, Clone)]
pub struct RadialMenuSetState {
    /// Index of the currently active radial menu.
    pub active: usize,
    /// Number of radial menus in the set.
    pub count: usize,
    /// Button used to select the previous radial menu.
    pub prev_button: GamepadButton,
    /// Button used to select the next radial menu.
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

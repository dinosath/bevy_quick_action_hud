//! Runtime state owned by a radial-menu set.

use bevy::prelude::*;

/// Runtime active-menu state for a radial-menu set.
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

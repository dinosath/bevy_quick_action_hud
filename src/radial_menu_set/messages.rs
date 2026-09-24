//! Messages emitted while switching between menus in a radial-menu set.

use bevy::prelude::*;

#[derive(Message, Clone)]
/// Emitted when the active wheel in a set changes.
pub struct WheelSwitched {
    /// Previously active radial-menu index.
    pub previous: usize,
    /// Newly active radial-menu index.
    pub current: usize,
    /// Entity that owns the radial-menu set.
    pub menu_entity: Entity,
}

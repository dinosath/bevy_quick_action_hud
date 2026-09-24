//! Messages emitted while switching between menus in a radial-menu set.

use bevy::prelude::*;

#[derive(Message, Clone)]
pub struct WheelSwitched {
    pub previous: usize,
    pub current: usize,
    pub menu_entity: Entity,
}

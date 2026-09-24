//! Buffered messages emitted by the core wheel runtime.

use bevy::prelude::*;

#[derive(Message, Clone)]
pub struct WheelMenuSelected {
    pub index: usize,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelMenuHoverChanged {
    pub previous: Option<usize>,
    pub current: Option<usize>,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelActionResolved {
    pub input: super::input::InputAction,
    pub action: super::input::WheelAction,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelSwitched {
    pub previous: usize,
    pub current: usize,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelMenuHoldProgress {
    pub index: usize,
    pub progress: f32,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelMenuHoldActivated {
    pub index: usize,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelMenuLowCount {
    pub index: usize,
    pub current: u32,
    pub threshold: u32,
    pub slice_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelEditModeChanged {
    pub active: bool,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelSliceReorder {
    pub from_index: usize,
    pub to_index: usize,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelOpened {
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelClosed {
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct SlotSelected {
    pub slot_index: usize,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct ActionTriggered {
    pub slot_index: usize,
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
pub struct WheelSlotItemChanged {
    pub slot_index: usize,
    pub previous_item: usize,
    pub current_item: usize,
    pub menu_entity: Entity,
}

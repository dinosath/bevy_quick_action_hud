//! Contextual input bindings owned by the core wheel feature.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum InputAction {
    PrimaryConfirm,
    Secondary,
    ButtonX,
    ButtonY,
    CycleNext,
    CyclePrev,
    Custom(u32),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum WheelAction {
    UseSlot,
    UseItem(usize),
    CycleItem { forward: bool },
    OpenSubmenu,
    Named(String),
}

#[derive(Component, Clone, Default, Serialize, Deserialize)]
pub struct WheelInputOverride {
    pub bindings: HashMap<InputAction, WheelAction>,
    pub priority: u8,
}

#[derive(Component, Clone, Copy)]
pub struct ActiveSlotContext {
    pub slot_entity: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct WheelSliceLink {
    pub menu: Entity,
}

#[derive(Resource, Clone, Default)]
pub struct GlobalBindings {
    pub bindings: HashMap<InputAction, WheelAction>,
}

pub fn resolve_input(
    input: InputAction,
    active_slot: Option<&WheelInputOverride>,
    wheel: Option<&WheelInputOverride>,
    global: &GlobalBindings,
) -> Option<WheelAction> {
    active_slot
        .and_then(|slot| slot.bindings.get(&input))
        .or_else(|| wheel.and_then(|wheel| wheel.bindings.get(&input)))
        .or_else(|| global.bindings.get(&input))
        .cloned()
}

pub(crate) const DEFAULT_BUTTON_MAP: &[(GamepadButton, InputAction)] = &[
    (GamepadButton::South, InputAction::PrimaryConfirm),
    (GamepadButton::East, InputAction::Secondary),
    (GamepadButton::West, InputAction::ButtonX),
    (GamepadButton::North, InputAction::ButtonY),
    (GamepadButton::RightThumb, InputAction::CycleNext),
    (GamepadButton::LeftThumb, InputAction::CyclePrev),
];

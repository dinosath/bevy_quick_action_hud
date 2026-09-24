//! Contextual input bindings owned by the core wheel feature.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
/// Logical inputs that can be overridden at slot, wheel, or global scope.
pub enum InputAction {
    /// Primary confirmation input.
    PrimaryConfirm,
    /// Secondary/cancel input.
    Secondary,
    /// Logical X action.
    ButtonX,
    /// Logical Y action.
    ButtonY,
    /// Advance to the next menu.
    CycleNext,
    /// Return to the previous menu.
    CyclePrev,
    /// Application-defined logical input identifier.
    Custom(u32),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// Actions resolved from logical wheel inputs.
pub enum WheelAction {
    /// Activate the currently selected sector.
    UseSlot,
    /// Activate a concrete item index.
    UseItem(usize),
    /// Move through items in the selected sector.
    CycleItem {
        /// Whether to move toward the next item.
        forward: bool,
    },
    /// Open a child radial menu.
    OpenSubmenu,
    /// Invoke an application-defined named action.
    Named(String),
}

#[derive(Component, Clone, Default, Serialize, Deserialize)]
/// Input overrides attached to a slot or wheel entity.
pub struct WheelInputOverride {
    /// Bindings that take precedence over lower-priority scopes.
    pub bindings: HashMap<InputAction, WheelAction>,
    /// Relative priority used when multiple overrides are available.
    pub priority: u8,
}

#[derive(Component, Clone, Copy)]
/// Runtime link to the currently active slot entity.
pub struct ActiveSlotContext {
    /// Entity representing the currently active sector.
    pub slot_entity: Entity,
}

#[derive(Component, Clone, Copy)]
/// Runtime link from a sector entity to its owning menu.
pub struct WheelSliceLink {
    /// Entity of the radial menu that owns the sector.
    pub menu: Entity,
}

#[derive(Resource, Clone, Default)]
/// Fallback logical bindings used when no local override matches.
pub struct GlobalBindings {
    /// Application-wide fallback bindings.
    pub bindings: HashMap<InputAction, WheelAction>,
}

/// Resolves an input using slot, wheel, then global precedence.
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

//! Buffered messages emitted by the core wheel runtime.

use bevy::prelude::*;

#[derive(Message, Clone)]
/// Emitted when a sector is selected.
pub struct WheelMenuSelected {
    /// Selected sector index.
    pub index: usize,
    /// Entity that owns the radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted when the hovered sector changes.
pub struct WheelMenuHoverChanged {
    /// Sector index that was previously hovered.
    pub previous: Option<usize>,
    /// Sector index that is now hovered.
    pub current: Option<usize>,
    /// Entity that owns the radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted after a logical input resolves to a wheel action.
pub struct WheelActionResolved {
    /// Logical input that was resolved.
    pub input: super::input::InputAction,
    /// Action selected by the binding resolver.
    pub action: super::input::WheelAction,
    /// Entity that owns the radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted while a hold-to-activate sector is charging.
pub struct WheelMenuHoldProgress {
    /// Sector being held.
    pub index: usize,
    /// Hold progress from zero to one.
    pub progress: f32,
    /// Entity that owns the radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted when hold-to-activate reaches its threshold.
pub struct WheelMenuHoldActivated {
    /// Sector whose hold action activated.
    pub index: usize,
    /// Entity that owns the radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted when a sector count falls to its configured low threshold.
pub struct WheelMenuLowCount {
    /// Sector whose count crossed the low threshold.
    pub index: usize,
    /// Current sector count.
    pub current: u32,
    /// Configured low-count threshold.
    pub threshold: u32,
    /// Entity representing the affected sector.
    pub slice_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted when a menu enters or leaves edit mode.
pub struct WheelEditModeChanged {
    /// New edit-mode state.
    pub active: bool,
    /// Entity that owns the radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted when a sector changes position in the menu.
pub struct WheelSliceReorder {
    /// Original sector index.
    pub from_index: usize,
    /// New sector index.
    pub to_index: usize,
    /// Entity that owns the radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted when a radial menu opens.
pub struct WheelOpened {
    /// Entity that owns the opened radial menu.
    pub menu_entity: Entity,
}
#[derive(Message, Clone)]
/// Emitted when a radial menu closes.
pub struct WheelClosed {
    /// Entity that owns the closed radial menu.
    pub menu_entity: Entity,
}

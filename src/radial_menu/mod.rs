//! Core radial-menu feature: authored data, geometry, style, the BSN widget,
//! and the runtime input pipeline.

pub(crate) mod components;
pub(crate) mod geometry;
pub(crate) mod input;
pub(crate) mod messages;
pub(crate) mod model;
pub(crate) mod style;
pub(crate) mod systems;
#[cfg(test)]
mod tests;
pub(crate) mod widget;

pub use components::{
    CastingMode, RadialMenuAudio, RadialMenuConfig, RadialMenuEditMode, RadialMenuHierarchy,
    RadialMenuHoldState, RadialMenuState, RadialMenuStyle, RadialMenuToggleMode, SectorContent,
    SectorCount, SectorEntity,
};
pub use geometry::{sector_anchor, slice_angles, slice_center, SectorAnchor};
pub(crate) use input::DEFAULT_BUTTON_MAP;
pub use input::{
    resolve_input, ActiveSlotContext, GlobalBindings, InputAction, WheelAction, WheelInputOverride,
    WheelSliceLink,
};
pub use model::{
    RadialMenu, RadialMenuGeometry, Sector, WheelTheme, DEFAULT_STICK_BINDING, MIN_SECTORS,
};
pub use systems::*;

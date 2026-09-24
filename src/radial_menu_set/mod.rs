//! Radial-menu set feature: grouped menus, shared visuals, and switching state.

pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod editor;
pub(crate) mod messages;
pub(crate) mod systems;

pub use components::RadialMenuSetState;
pub use config::{normalize_wheelset, wheelset_visuals, RadialMenuSet, RadialMenuSetVisuals};
pub use systems::update_wheel_set;

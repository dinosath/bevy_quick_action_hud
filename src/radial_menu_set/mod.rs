//! Radial-menu set feature: grouped menus, shared visuals, and switching state.

pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod editor;
pub(crate) mod messages;
mod navigation;
pub(crate) mod systems;
mod view;

use bevy::prelude::*;

pub use components::RadialMenuSetState;
pub use config::{normalize_wheelset, wheelset_visuals, RadialMenuSet, RadialMenuSetVisuals};
pub(crate) use navigation::hud_stick_nav;
pub use systems::update_wheel_set;
pub use view::RadialMenuSets;
pub(crate) use view::{keyed_highlight, sector_in, RadialMenuRef};

/// Registers the observer that fills [`RadialMenuSets`] slots.
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(view::fill_radial_menu_sets);
    crate::hud::slot::register::<RadialMenuSets>(app);
}

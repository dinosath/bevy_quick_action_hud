//! Page-switch feature: a HUD control that activates another page.

pub(crate) mod config;
mod view;

use bevy::prelude::*;

pub use config::HudSwitch;
pub use view::PageSwitches;

/// Registers the observer that fills [`PageSwitches`] slots.
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(view::fill_page_switches);
}

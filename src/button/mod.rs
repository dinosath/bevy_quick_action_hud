//! Floating HUD button feature: authored data and its page slot.

pub(crate) mod config;
mod view;

use bevy::prelude::*;

pub use config::{ActionShape, HudButton, PositionMode, QuickAction};
pub(crate) use view::tick_dry_run_flash;
pub use view::Buttons;

/// Registers the observer that fills [`Buttons`] slots.
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(view::fill_buttons);
    crate::hud::slot::register::<Buttons>(app);
}

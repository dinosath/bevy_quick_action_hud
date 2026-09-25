//! HUD page feature: authored page data, the page scene, and page tabs.

pub(crate) mod config;
mod view;

use bevy::prelude::*;

pub use config::{
    count_radial_menu_sets, enabled_hud_pages, ActionSet, HudComponent, HudPage, SetEntry,
};
pub use view::PageTabs;
pub(crate) use view::{no_page, page};

/// Registers the observers that fill the page slots.
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(view::fill_page_tabs)
        .add_observer(view::fill_page_background);
}

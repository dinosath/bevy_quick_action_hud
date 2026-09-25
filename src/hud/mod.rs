//! HUD feature: the document root, runtime state, and the HUD scene.
//!
//! The HUD only declares which components it contains. Each component is a
//! slot whose owning feature module fills it in when it is added.

use bevy::prelude::*;
use bevy::scene::prelude::Scene;

use crate::editor::EditMode;
use crate::page::{self, PageTabs};

pub(crate) mod actions;
pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod gamepad;
pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod theme;

pub use actions::{SegmentInsertSide, WheelHudAction};
pub(crate) use components::control_owner as hud_control_owner;
pub use components::{
    HudContextControl, HudControlOwner, HudRadialMenu, WheelHudButton, WheelHudRoot,
};
pub use config::*;
pub(crate) use gamepad::detect_gamepad_icon_set;
pub use gamepad::GamepadIconSet;
pub(crate) use state::HudView;
pub use state::{HudSegmentSelected, WheelHudState};
pub(crate) use systems::{button_feedback as hud_button_feedback, rebuild_hud};
pub use theme::*;

/// The HUD: the active page, the page tabs, and the edit-mode controls on top.
pub(crate) fn hud(active_page: Option<usize>) -> impl Scene {
    let page: Box<dyn Scene> = match active_page {
        Some(index) => Box::new(page::page(index)),
        None => Box::new(page::no_page()),
    };
    bsn! {
        WheelHudRoot
        Children [ @{page} -- PageTabs -- EditMode ]
    }
}

/// Plugin that registers the retained HUD canvas and runtime navigation.
pub struct WheelHudPlugin;
impl Plugin for WheelHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::QuickActionHudPlugin::default());
    }
}

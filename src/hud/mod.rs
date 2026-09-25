//! HUD feature: the document root, runtime state, and the HUD scene.
//!
//! The HUD only declares which components it contains. Each component is a
//! slot whose owning feature module fills it in when it is added.

use bevy::prelude::*;
use bevy::scene::prelude::{Scene, SceneList};
use bevy::scene::EntityScene;

use crate::editor::EditMode;
use crate::page::{self, PageTabs};

pub(crate) mod actions;
pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod gamepad;
pub(crate) mod slot;
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
pub(crate) use systems::{button_feedback as hud_button_feedback, rebuild_hud, show_hud};
pub use theme::*;

/// The HUD: every enabled page (only the active one is shown), the page tabs,
/// and the edit-mode controls on top.
pub(crate) fn hud(pages: &[usize]) -> impl Scene {
    let pages: Vec<Box<dyn SceneList>> = if pages.is_empty() {
        vec![Box::new(EntityScene(page::no_page()))]
    } else {
        pages
            .iter()
            .map(|&index| Box::new(EntityScene(page::page(index))) as Box<dyn SceneList>)
            .collect()
    };
    bsn! {
        WheelHudRoot
        Children [ {pages} -- PageTabs -- EditMode ]
    }
}

/// Plugin that registers the retained HUD canvas and runtime navigation.
pub struct WheelHudPlugin;
impl Plugin for WheelHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::QuickActionHudPlugin::default());
    }
}

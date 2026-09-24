//! HUD presentation split by hierarchy ownership.
//!
//! Static layout is authored with BSN fragments. Procedural radial geometry and
//! runtime input remain in their focused ECS boundaries.

mod buttons;
mod canvas;
mod navigation;
mod primitives;
mod radial_menu;
mod runtime;

pub(crate) use canvas::{hud_action_field, hud_action_stepper};
pub use primitives::parse_hex_color;
pub(crate) use primitives::{hud_child, hud_clickable, hud_label_or, hud_text};
pub use runtime::WheelHudPlugin;
pub(crate) use runtime::{hud_stick_nav, rebuild_hud};

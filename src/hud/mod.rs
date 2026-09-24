//! HUD feature modules.

pub(crate) mod actions;
pub(crate) mod bsn;
pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod gamepad;
pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod theme;
pub(crate) mod ui;

pub use actions::{SegmentInsertSide, WheelHudAction};
pub(crate) use components::control_owner as hud_control_owner;
pub use components::{
    HudContextControl, HudControlOwner, WheelHudButton, WheelHudRoot, WheelHudSegmentHit,
};
pub use config::*;
pub(crate) use gamepad::detect_gamepad_icon_set;
pub use gamepad::GamepadIconSet;
pub use state::{HudSegmentSelected, WedgeMaterial, WedgeParams, WheelHudState};
pub(crate) use systems::{
    button_feedback as hud_button_feedback, context_visibility as hud_context_visibility,
    tick_dry_run_flash as tick_hud_dry_run_flash,
};
pub use theme::*;
pub use ui::parse_hex_color;
pub use ui::WheelHudPlugin;
pub(crate) use ui::{hud_stick_nav, rebuild_hud};

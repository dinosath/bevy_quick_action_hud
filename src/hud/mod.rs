//! HUD feature modules.

pub(crate) mod actions;
pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod gamepad;
pub(crate) mod render;
pub(crate) mod state;
pub(crate) mod theme;

pub use actions::{SegmentInsertSide, WheelHudAction};
pub use components::{
    HudContextControl, HudControlOwner, WheelHudButton, WheelHudRoot, WheelHudSegmentHit,
};
pub use config::*;
pub(crate) use gamepad::detect_gamepad_icon_set;
pub use gamepad::GamepadIconSet;
pub use render::*;
pub(crate) use render::{
    hud_button_feedback, hud_context_visibility, hud_control_owner, hud_stick_nav, rebuild_hud,
    tick_hud_dry_run_flash,
};
pub use state::{HudSegmentSelected, WedgeMaterial, WedgeParams, WheelHudState};
pub use theme::*;

//! Runtime state and rendering types for the HUD feature.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{GamepadIconSet, QuickActionConfig};

/// Read-only resources shared by every HUD component view.
#[derive(SystemParam)]
pub(crate) struct HudView<'w> {
    pub(crate) cfg: Res<'w, QuickActionConfig>,
    pub(crate) hud: Res<'w, WheelHudState>,
    pub(crate) asset_server: Res<'w, AssetServer>,
    pub(crate) icon_set: Res<'w, GamepadIconSet>,
}

impl HudView<'_> {
    /// Loads the controller glyph for a `GP:` binding, if the binding has one.
    pub(crate) fn gamepad_glyph(&self, binding: &str) -> Option<Handle<Image>> {
        let label = binding.strip_prefix("GP:")?;
        let path = self.icon_set.embedded_icon_path(label)?;
        Some(self.asset_server.load(path))
    }
}

#[derive(Resource)]
/// Runtime state for HUD visibility and selection.
///
/// The HUD follows changes automatically: config edits respawn it, and each
/// component slot respawns when the part of this state it shows changes.
pub struct WheelHudState {
    /// Whether the runtime HUD is open.
    pub open: bool,
    /// Active page index.
    pub active_set: usize,
    /// Whether editor affordances and inspectors are enabled.
    pub editor_open: bool,
    /// Transient sector hover/highlight identity.
    pub highlighted: Option<(usize, usize, Option<usize>, usize)>,
    /// Sector selected in edit mode. Unlike `highlighted`, this is not driven
    /// by pointer hover and owns the sector inspector window.
    pub selected_segment: Option<(usize, usize, Option<usize>, usize)>,
    /// Active page entry containing a wheel.
    pub active_wheel_entry: usize,
    /// Action entry currently shown with a dry-run flash.
    pub flash_action_entry: Option<usize>,
    /// Remaining dry-run flash duration.
    pub flash_action_ttl: f32,
    /// Focused editor-control index.
    pub edit_control_focus: Option<usize>,
    /// Whether the global HUD settings window is open.
    pub settings_open: bool,
    /// Whether a wheel theme popup is open.
    pub theme_popup_open: bool,
    /// Sector currently under the pointer.
    pub mouse_hovered_segment: Option<(usize, usize, Option<usize>, usize)>,
    /// Selected floating action identity.
    pub selected_action: Option<(usize, usize)>,
    /// Selected wheel identity.
    pub selected_wheel: Option<(usize, usize, Option<usize>)>,
    /// Selected page-switch identity.
    pub selected_hud_switch: Option<(usize, usize)>,
    /// Hovered floating action identity.
    pub hovered_action: Option<(usize, usize)>,
    /// Hovered wheel identity.
    pub hovered_wheel: Option<(usize, usize, Option<usize>)>,
    /// Hovered page-switch identity.
    pub hovered_hud_switch: Option<(usize, usize)>,
    /// Active wheel index within the current wheel set.
    pub active_wheel_index: usize,
}

impl Default for WheelHudState {
    fn default() -> Self {
        Self {
            open: false,
            active_set: 0,
            editor_open: false,
            highlighted: None,
            selected_segment: None,
            active_wheel_entry: 0,
            flash_action_entry: None,
            flash_action_ttl: 0.0,
            edit_control_focus: None,
            settings_open: false,
            theme_popup_open: false,
            mouse_hovered_segment: None,
            selected_action: None,
            selected_wheel: None,
            selected_hud_switch: None,
            hovered_action: None,
            hovered_wheel: None,
            hovered_hud_switch: None,
            active_wheel_index: 0,
        }
    }
}

#[derive(Message, Clone, Debug)]
/// Message emitted when a HUD sector is explicitly selected.
pub struct HudSegmentSelected {
    /// Page containing the selected sector.
    pub set: usize,
    /// Entry containing the selected radial menu.
    pub entry: usize,
    /// Wheel index within a wheel set, if applicable.
    pub wheel: Option<usize>,
    /// Selected sector index.
    pub slot: usize,
}

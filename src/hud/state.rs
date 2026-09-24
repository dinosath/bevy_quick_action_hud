//! Runtime state and rendering types for the HUD feature.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

#[derive(Clone, ShaderType)]
/// Uniform values consumed by the procedural radial wedge shader.
pub struct WedgeParams {
    /// Fill color in shader space.
    pub color: Vec4,
    /// Border color in shader space.
    pub border_color: Vec4,
    /// Inner radius of the procedural wedge.
    pub inner_r: f32,
    /// Outer radius of the procedural wedge.
    pub outer_r: f32,
    /// Start angle in radians.
    pub angle_start: f32,
    /// End angle in radians.
    pub angle_end: f32,
    /// Width of the rendered edge.
    pub edge_width: f32,
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
/// UI material used to render a procedural pie sector.
pub struct WedgeMaterial {
    #[uniform(0)]
    /// Uniform parameters consumed by the wedge shader.
    pub params: WedgeParams,
}

impl UiMaterial for WedgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_quick_action_hud/embedded/shaders/wedge.wgsl".into()
    }
}

#[derive(Resource)]
/// Runtime state for HUD visibility, selection, and retained rebuilding.
pub struct WheelHudState {
    /// Whether the retained HUD tree must be rebuilt.
    pub dirty: bool,
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
            dirty: true,
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

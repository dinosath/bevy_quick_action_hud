//! Runtime state and rendering types for the HUD feature.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

#[derive(Clone, ShaderType)]
pub struct WedgeParams {
    pub color: Vec4,
    pub border_color: Vec4,
    pub inner_r: f32,
    pub outer_r: f32,
    pub angle_start: f32,
    pub angle_end: f32,
    pub edge_width: f32,
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct WedgeMaterial {
    #[uniform(0)]
    pub params: WedgeParams,
}

impl UiMaterial for WedgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_quick_action_hud/embedded/shaders/wedge.wgsl".into()
    }
}

#[derive(Resource)]
pub struct WheelHudState {
    pub dirty: bool,
    pub open: bool,
    pub active_set: usize,
    pub editor_open: bool,
    pub highlighted: Option<(usize, usize, Option<usize>, usize)>,
    pub active_wheel_entry: usize,
    pub flash_action_entry: Option<usize>,
    pub flash_action_ttl: f32,
    pub edit_control_focus: Option<usize>,
    pub settings_open: bool,
    pub mouse_hovered_segment: Option<(usize, usize, Option<usize>, usize)>,
    pub selected_action: Option<(usize, usize)>,
    pub selected_wheel: Option<(usize, usize, Option<usize>)>,
    pub selected_hud_switch: Option<(usize, usize)>,
    pub hovered_action: Option<(usize, usize)>,
    pub hovered_wheel: Option<(usize, usize, Option<usize>)>,
    pub hovered_hud_switch: Option<(usize, usize)>,
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
            active_wheel_entry: 0,
            flash_action_entry: None,
            flash_action_ttl: 0.0,
            edit_control_focus: None,
            settings_open: false,
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
pub struct HudSegmentSelected {
    pub set: usize,
    pub entry: usize,
    pub wheel: Option<usize>,
    pub slot: usize,
}

//! Authored configuration and shared presentation for a radial-menu set.

use crate::radial_menu::{RadialMenu, SegmentShape, StickSide, WheelTheme};
use serde::{Deserialize, Serialize};

fn default_wheelset_min() -> usize {
    1
}
fn default_wheelset_max() -> usize {
    8
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenuSetVisuals {
    pub offset_x: f32,
    pub offset_y: f32,
    pub rotation: f32,
    pub theme: WheelTheme,
    pub outer_radius: f32,
    pub inner_radius: f32,
    pub show_labels: bool,
    pub segment_shape: SegmentShape,
    pub show_icon: bool,
    pub highlight_color: String,
    pub segment_scale: f32,
    pub opacity: f32,
    pub inner_border: String,
    pub outer_border: String,
    pub outer_border_width: f32,
    pub inner_border_width: f32,
    pub bg_color: String,
    pub bg_opacity: f32,
    pub hub_color: String,
    pub hub_opacity: f32,
    pub deadzone: f32,
    pub gap: f32,
    pub arc_span: f32,
    pub arc_offset: f32,
    pub overlap: bool,
    pub stick: StickSide,
}
impl Default for RadialMenuSetVisuals {
    fn default() -> Self {
        Self::from(&RadialMenu::default())
    }
}
impl From<&RadialMenu> for RadialMenuSetVisuals {
    fn from(w: &RadialMenu) -> Self {
        Self {
            offset_x: w.offset_x,
            offset_y: w.offset_y,
            rotation: w.rotation,
            theme: w.theme,
            outer_radius: w.outer_radius,
            inner_radius: w.inner_radius,
            show_labels: w.show_labels,
            segment_shape: w.segment_shape,
            show_icon: w.show_icon,
            highlight_color: w.highlight_color.clone(),
            segment_scale: w.segment_scale,
            opacity: w.opacity,
            inner_border: w.inner_border.clone(),
            outer_border: w.outer_border.clone(),
            outer_border_width: w.outer_border_width,
            inner_border_width: w.inner_border_width,
            bg_color: w.bg_color.clone(),
            bg_opacity: w.bg_opacity,
            hub_color: w.hub_color.clone(),
            hub_opacity: w.hub_opacity,
            deadzone: w.deadzone,
            gap: w.gap,
            arc_span: w.arc_span,
            arc_offset: w.arc_offset,
            overlap: w.overlap,
            stick: w.stick,
        }
    }
}
impl RadialMenuSetVisuals {
    pub fn apply_to(&self, w: &mut RadialMenu) {
        w.offset_x = self.offset_x;
        w.offset_y = self.offset_y;
        w.rotation = self.rotation;
        w.theme = self.theme;
        w.outer_radius = self.outer_radius;
        w.inner_radius = self.inner_radius;
        w.show_labels = self.show_labels;
        w.segment_shape = self.segment_shape;
        w.show_icon = self.show_icon;
        w.highlight_color = self.highlight_color.clone();
        w.segment_scale = self.segment_scale;
        w.opacity = self.opacity;
        w.inner_border = self.inner_border.clone();
        w.outer_border = self.outer_border.clone();
        w.outer_border_width = self.outer_border_width;
        w.inner_border_width = self.inner_border_width;
        w.bg_color = self.bg_color.clone();
        w.bg_opacity = self.bg_opacity;
        w.hub_color = self.hub_color.clone();
        w.hub_opacity = self.hub_opacity;
        w.deadzone = self.deadzone;
        w.gap = self.gap;
        w.arc_span = self.arc_span;
        w.arc_offset = self.arc_offset;
        w.overlap = self.overlap;
        w.stick = self.stick;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenuSet {
    pub name: String,
    pub wheels: Vec<RadialMenu>,
    pub visuals: Option<RadialMenuSetVisuals>,
    #[serde(default = "default_wheelset_min")]
    pub min_wheels: usize,
    #[serde(default = "default_wheelset_max")]
    pub max_wheels: usize,
    pub prev_wheel_key: String,
    pub next_wheel_key: String,
    pub cycle_wheels: bool,
    pub switch_key: String,
    pub stick: StickSide,
}
impl Default for RadialMenuSet {
    fn default() -> Self {
        Self {
            name: "Radial menu set".into(),
            wheels: vec![RadialMenu::default()],
            visuals: Some(RadialMenuSetVisuals::default()),
            min_wheels: 1,
            max_wheels: 8,
            prev_wheel_key: String::new(),
            next_wheel_key: String::new(),
            cycle_wheels: false,
            switch_key: String::new(),
            stick: StickSide::Right,
        }
    }
}

pub fn wheelset_visuals(ws: &RadialMenuSet) -> RadialMenuSetVisuals {
    ws.visuals
        .clone()
        .or_else(|| ws.wheels.first().map(RadialMenuSetVisuals::from))
        .unwrap_or_default()
}
pub fn normalize_wheelset(ws: &mut RadialMenuSet) {
    let visuals = wheelset_visuals(ws);
    ws.visuals = Some(visuals.clone());
    ws.min_wheels = ws.min_wheels.max(1);
    ws.max_wheels = ws.max_wheels.max(ws.min_wheels).max(ws.wheels.len());
    while ws.wheels.len() < ws.min_wheels {
        let mut wheel = RadialMenu::new(format!("Radial menu {}", ws.wheels.len() + 1), 6);
        visuals.apply_to(&mut wheel);
        ws.wheels.push(wheel);
    }
    for wheel in &mut ws.wheels {
        visuals.apply_to(wheel);
    }
}

//! Authored configuration and shared presentation for a radial-menu set.

use crate::radial_menu::{RadialMenu, SegmentShape, WheelTheme, DEFAULT_STICK_BINDING};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
/// Presentation values shared by every radial menu in a set.
pub struct RadialMenuSetVisuals {
    /// Shared horizontal offset.
    pub offset_x: f32,
    /// Shared vertical offset.
    pub offset_y: f32,
    /// Shared rotation in degrees.
    pub rotation: f32,
    /// Shared theme preset.
    pub theme: WheelTheme,
    /// Shared outer radius.
    pub outer_radius: f32,
    /// Shared inner radius.
    pub inner_radius: f32,
    /// Shared angular gap between sectors.
    pub gap: f32,
    /// Shared total angular span in radians.
    pub arc_span: f32,
    /// Shared angular offset in radians.
    pub arc_offset: f32,
    /// Whether all wheels show sector labels.
    pub show_labels: bool,
    /// Shared sector shape.
    pub segment_shape: SegmentShape,
    /// Whether all wheels show sector icons.
    pub show_icon: bool,
    /// Shared highlight color.
    pub highlight_color: String,
    /// Shared sector scale.
    pub segment_scale: f32,
    /// Shared menu opacity.
    pub opacity: f32,
    /// Shared inner border color.
    pub inner_border: String,
    /// Shared outer border color.
    pub outer_border: String,
    /// Shared outer border width.
    pub outer_border_width: f32,
    /// Shared inner border width.
    pub inner_border_width: f32,
    /// Shared background color.
    pub bg_color: String,
    /// Shared background opacity.
    pub bg_opacity: f32,
    /// Shared hub color.
    pub hub_color: String,
    /// Shared hub opacity.
    pub hub_opacity: f32,
    /// Shared analog deadzone.
    pub deadzone: f32,
    /// Whether sector bounds may overlap.
    pub overlap: bool,
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
            gap: w.gap,
            arc_span: w.arc_span,
            arc_offset: w.arc_offset,
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
            overlap: w.overlap,
        }
    }
}
impl RadialMenuSetVisuals {
    /// Applies the shared presentation values to one wheel.
    pub fn apply_to(&self, w: &mut RadialMenu) {
        w.offset_x = self.offset_x;
        w.offset_y = self.offset_y;
        w.rotation = self.rotation;
        w.theme = self.theme;
        w.outer_radius = self.outer_radius;
        w.inner_radius = self.inner_radius;
        w.gap = self.gap;
        w.arc_span = self.arc_span;
        w.arc_offset = self.arc_offset;
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
        w.overlap = self.overlap;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
/// Group of radial menus sharing presentation and navigation settings.
pub struct RadialMenuSet {
    /// Display name of the wheel set.
    pub name: String,
    /// Ordered radial menus in this set.
    pub wheels: Vec<RadialMenu>,
    /// Shared visual values applied to every wheel.
    pub visuals: Option<RadialMenuSetVisuals>,
    /// Minimum number of wheels allowed in the set.
    pub min_wheels: usize,
    /// Maximum number of wheels allowed in the set.
    pub max_wheels: usize,
    /// Shortcut for selecting the previous wheel.
    pub prev_wheel_key: String,
    /// Shortcut for selecting the next wheel.
    pub next_wheel_key: String,
    /// Whether next/previous navigation wraps around.
    pub cycle_wheels: bool,
    /// Shortcut associated with this set's wheel switch control.
    pub switch_key: String,
    /// Thumbstick binding used by the set's radial menus.
    pub stick_binding: String,
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
            stick_binding: DEFAULT_STICK_BINDING.into(),
        }
    }
}

/// Returns the effective shared presentation values for a wheel set.
///
/// Legacy configurations without an explicit `visuals` block inherit values
/// from their first wheel.
pub fn wheelset_visuals(ws: &RadialMenuSet) -> RadialMenuSetVisuals {
    ws.visuals
        .clone()
        .or_else(|| ws.wheels.first().map(RadialMenuSetVisuals::from))
        .unwrap_or_default()
}
/// Repairs wheel-count bounds and reapplies shared visuals to every wheel.
pub fn normalize_wheelset(ws: &mut RadialMenuSet) {
    let visuals = wheelset_visuals(ws);
    ws.visuals = Some(visuals.clone());
    ws.min_wheels = ws.min_wheels.max(1);
    ws.max_wheels = ws.max_wheels.max(ws.min_wheels).max(ws.wheels.len());
    while ws.wheels.len() < ws.min_wheels {
        let mut wheel = RadialMenu::new(format!("Radial menu {}", ws.wheels.len() + 1), 6);
        visuals.apply_to(&mut wheel);
        wheel.stick_binding = ws.stick_binding.clone();
        ws.wheels.push(wheel);
    }
    for wheel in &mut ws.wheels {
        visuals.apply_to(wheel);
        wheel.stick_binding = ws.stick_binding.clone();
    }
}

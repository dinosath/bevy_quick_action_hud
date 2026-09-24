//! Authored radial-menu configuration and reusable radial-menu data.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}
fn default_action_command() -> String {
    "none".into()
}
fn default_hold_command() -> String {
    "none".into()
}
fn default_outer_radius() -> f32 {
    270.0
}
fn default_inner_radius() -> f32 {
    130.0
}
fn full_opacity() -> f32 {
    1.0
}
fn default_highlight_color() -> String {
    "#ef8b92".into()
}
fn default_segment_scale() -> f32 {
    1.0
}
fn default_border_width() -> f32 {
    2.0
}
fn default_deadzone() -> f32 {
    0.3
}
fn default_gap() -> f32 {
    0.012
}
fn default_arc_span() -> f32 {
    std::f32::consts::TAU
}
fn default_arc_offset() -> f32 {
    std::f32::consts::FRAC_PI_6
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum WheelTheme {
    #[default]
    Dark,
    Light,
}
impl WheelTheme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum SegmentShape {
    #[default]
    Rounded,
    Square,
    Circle,
    Wedge,
    Pie,
}
impl SegmentShape {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rounded => "Rounded",
            Self::Square => "Square",
            Self::Circle => "Circle",
            Self::Wedge => "Wedge",
            Self::Pie => "Pie",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Rounded => Self::Square,
            Self::Square => Self::Circle,
            Self::Circle => Self::Wedge,
            Self::Wedge => Self::Pie,
            Self::Pie => Self::Rounded,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Sector {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub input: String,
    #[serde(default = "default_action_command")]
    pub command: String,
    #[serde(default = "default_hold_command")]
    pub hold_command: String,
    pub hold: bool,
    #[serde(default = "default_true")]
    pub close_on_select: bool,
}
impl Default for Sector {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            icon: String::new(),
            input: String::new(),
            command: "none".into(),
            hold_command: "none".into(),
            hold: false,
            close_on_select: true,
        }
    }
}
impl Sector {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenu {
    pub name: String,
    pub cooldown_secs: f32,
    pub slots: Vec<Sector>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub rotation: f32,
    pub theme: WheelTheme,
    #[serde(default = "default_outer_radius")]
    pub outer_radius: f32,
    #[serde(default = "default_inner_radius")]
    pub inner_radius: f32,
    #[serde(default = "default_true")]
    pub show_labels: bool,
    pub segment_shape: SegmentShape,
    #[serde(default = "default_true")]
    pub show_icon: bool,
    #[serde(default = "default_highlight_color")]
    pub highlight_color: String,
    #[serde(default = "default_segment_scale")]
    pub segment_scale: f32,
    #[serde(default = "full_opacity")]
    pub opacity: f32,
    pub inner_border: String,
    pub outer_border: String,
    #[serde(default = "default_border_width")]
    pub outer_border_width: f32,
    #[serde(default = "default_border_width")]
    pub inner_border_width: f32,
    pub bg_color: String,
    #[serde(default = "full_opacity")]
    pub bg_opacity: f32,
    pub hub_color: String,
    #[serde(default = "full_opacity")]
    pub hub_opacity: f32,
    #[serde(default = "default_deadzone")]
    pub deadzone: f32,
    #[serde(default = "default_gap")]
    pub gap: f32,
    #[serde(default = "default_arc_span")]
    pub arc_span: f32,
    #[serde(default = "default_arc_offset")]
    pub arc_offset: f32,
    pub overlap: bool,
    pub stick: StickSide,
}
impl Default for RadialMenu {
    fn default() -> Self {
        Self {
            name: "Radial menu".into(),
            cooldown_secs: 6.0,
            slots: vec![Sector::named("Slot 1")],
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            theme: WheelTheme::Dark,
            outer_radius: default_outer_radius(),
            inner_radius: default_inner_radius(),
            show_labels: false,
            segment_shape: SegmentShape::Pie,
            show_icon: true,
            highlight_color: default_highlight_color(),
            segment_scale: 1.0,
            opacity: 1.0,
            inner_border: String::new(),
            outer_border: String::new(),
            outer_border_width: 2.0,
            inner_border_width: 2.0,
            bg_color: String::new(),
            bg_opacity: 1.0,
            hub_color: String::new(),
            hub_opacity: 1.0,
            deadzone: default_deadzone(),
            gap: default_gap(),
            arc_span: default_arc_span(),
            arc_offset: default_arc_offset(),
            overlap: false,
            stick: StickSide::Right,
        }
    }
}
impl RadialMenu {
    pub fn new(name: impl Into<String>, n: usize) -> Self {
        Self {
            name: name.into(),
            slots: (0..n.max(1))
                .map(|i| Sector::named(format!("Slot {}", i + 1)))
                .collect(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StickSide {
    #[default]
    Right,
    Left,
}
impl StickSide {
    pub fn label(self) -> &'static str {
        match self {
            Self::Right => "R Stick",
            Self::Left => "L Stick",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
}

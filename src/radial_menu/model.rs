//! Authored radial-menu data: menus, sectors, themes, and default geometry.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Default binding used to navigate a radial menu.
pub const DEFAULT_STICK_BINDING: &str = "GP:RightStick";

/// Minimum number of sectors in a radial menu.
pub const MIN_SECTORS: usize = 2;

/// Built-in visual themes for radial menus.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum WheelTheme {
    /// Dark, high-contrast wheel presentation.
    #[default]
    Dark,
    /// Light wheel presentation.
    Light,
}

impl WheelTheme {
    /// Returns the stable, lowercase identifier used by the editor and logs.
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    /// Returns the next theme in the built-in theme cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

/// Geometry options shared by radial-menu layout and sector hit testing.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenuGeometry {
    /// Outer radius in logical pixels.
    pub outer_radius: f32,
    /// Inner hub radius in logical pixels.
    pub inner_radius: f32,
    /// Angular gap between adjacent sectors in radians.
    pub gap: f32,
    /// Total angular span of the menu in radians.
    pub arc_span: f32,
    /// Angular offset of the first sector in radians.
    pub arc_offset: f32,
}

impl Default for RadialMenuGeometry {
    fn default() -> Self {
        Self {
            outer_radius: 270.0,
            inner_radius: 145.0,
            gap: 0.012,
            arc_span: std::f32::consts::TAU,
            // With four default sectors this centers the first sector at
            // twelve o'clock.
            arc_offset: std::f32::consts::FRAC_PI_4,
        }
    }
}

/// Authored content for one radial-menu sector.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Sector {
    /// Display name shown inside the sector.
    pub name: String,
    /// Optional descriptive text for an inspector or tooltip.
    pub description: String,
    /// Optional icon asset path or icon identifier.
    pub icon: String,
    /// Gameplay command or action mapping invoked on selection.
    pub command: String,
    /// Alternate mapping invoked while the input remains held.
    pub hold_command: String,
    /// Whether the sector has a distinct hold action.
    pub hold: bool,
    /// Whether selecting this sector closes the HUD.
    pub close_on_select: bool,
}

impl Default for Sector {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            icon: String::new(),
            command: "none".into(),
            hold_command: "none".into(),
            hold: false,
            close_on_select: true,
        }
    }
}

impl Sector {
    /// Creates a sector with the supplied name and all other defaults.
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// Authored configuration for one radial menu and its sectors.
#[derive(Component, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenu {
    /// Display name of this radial menu.
    pub name: String,
    /// Minimum time between activations, in seconds.
    pub cooldown_secs: f32,
    /// Ordered sectors around the menu.
    pub slots: Vec<Sector>,
    /// Horizontal offset from the menu anchor.
    pub offset_x: f32,
    /// Vertical offset from the menu anchor.
    pub offset_y: f32,
    /// Rotation in degrees applied to the menu.
    pub rotation: f32,
    /// Color/theme preset used by the menu.
    pub theme: WheelTheme,
    /// Outer radius in logical pixels.
    pub outer_radius: f32,
    /// Inner hub radius in logical pixels.
    pub inner_radius: f32,
    /// Angular gap between adjacent sectors in radians.
    pub gap: f32,
    /// Total angular span of the menu in radians.
    pub arc_span: f32,
    /// Angular offset of the first sector in radians.
    pub arc_offset: f32,
    /// Whether sector labels are rendered.
    pub show_labels: bool,
    /// Whether sector icons are rendered.
    pub show_icon: bool,
    /// Color used to identify the hovered or selected sector.
    pub highlight_color: String,
    /// Scale applied to sector panels.
    pub segment_scale: f32,
    /// Overall menu opacity.
    pub opacity: f32,
    /// Inner ring border color.
    pub inner_border: String,
    /// Outer ring border color.
    pub outer_border: String,
    /// Outer ring border width.
    pub outer_border_width: f32,
    /// Inner ring border width.
    pub inner_border_width: f32,
    /// Background color of the menu disc.
    pub bg_color: String,
    /// Background opacity.
    pub bg_opacity: f32,
    /// Center hub color.
    pub hub_color: String,
    /// Hub opacity.
    pub hub_opacity: f32,
    /// Input deadzone below which no sector is selected.
    pub deadzone: f32,
    /// Whether sectors may overlap their nominal bounds.
    pub overlap: bool,
    /// Thumbstick binding used to navigate the menu.
    pub stick_binding: String,
}

impl Default for RadialMenu {
    fn default() -> Self {
        let geometry = RadialMenuGeometry::default();
        Self {
            name: "Radial menu".into(),
            cooldown_secs: 6.0,
            slots: vec![
                Sector::named("Slot 1"),
                Sector::named("Slot 2"),
                Sector::named("Slot 3"),
                Sector::named("Slot 4"),
            ],
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            theme: WheelTheme::Dark,
            outer_radius: geometry.outer_radius,
            inner_radius: geometry.inner_radius,
            show_labels: false,
            show_icon: true,
            highlight_color: "#a8e9ec".into(),
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
            gap: geometry.gap,
            arc_span: geometry.arc_span,
            arc_offset: geometry.arc_offset,
            deadzone: 0.3,
            overlap: false,
            stick_binding: DEFAULT_STICK_BINDING.into(),
        }
    }
}

impl RadialMenu {
    /// Creates a menu with at least [`MIN_SECTORS`] named sectors.
    pub fn new(name: impl Into<String>, n: usize) -> Self {
        Self {
            name: name.into(),
            slots: (0..n.max(MIN_SECTORS))
                .map(|i| Sector::named(format!("Slot {}", i + 1)))
                .collect(),
            ..Default::default()
        }
    }

    /// Ensures that authored data satisfies the minimum radial-menu size.
    pub fn ensure_minimum_sectors(&mut self) {
        while self.slots.len() < MIN_SECTORS {
            self.slots
                .push(Sector::named(format!("Slot {}", self.slots.len() + 1)));
        }
    }
}

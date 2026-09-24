//! Entity-local state for the core wheel runtime.

use bevy::prelude::*;
use bevy::scene::prelude::Scene;
use serde::{Deserialize, Serialize};

/// Default binding used to navigate a radial menu.
pub const DEFAULT_STICK_BINDING: &str = "GP:RightStick";

/// How an action is confirmed and triggered from a radial menu.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub enum CastingMode {
    /// Standard: press the confirm button while hovering a sector.
    #[default]
    Vanilla,
    /// Release the stick from a hovered sector to select it.
    ReleaseToUse,
    /// Dwell on a sector for `duration` seconds to trigger it.
    HoldToActivate {
        /// Required hold duration in seconds.
        duration: f32,
    },
    /// Activate immediately when a new sector is hovered.
    Direct,
}

/// How a radial menu opens and closes.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub enum RadialMenuToggleMode {
    /// Press the hotkey once to open; press again to close.
    Toggle,
    /// Hold the key or button to keep open; release to close.
    #[default]
    Hold,
    /// Short press toggles; long press uses hold behavior.
    Hybrid {
        /// Threshold separating a short press from a hold.
        hold_threshold_secs: f32,
    },
}

/// Marks a slice within a wheel menu.
#[derive(Component, Clone)]
pub struct SectorEntity {
    /// Index of this slice (0-based).
    pub index: usize,
}

/// Optional content for a wheel slice.
#[derive(Component, Clone, Default)]
pub struct SectorContent {
    /// Label text for this slice.
    pub label: Option<String>,
    /// Icon path/identifier for this slice.
    pub icon: Option<String>,
}

/// Current input state of a wheel menu.
#[derive(Component, Default, Clone)]
pub struct RadialMenuState {
    /// Current input direction (normalized).
    pub dir: Vec2,
    /// Currently hovered slice index.
    pub hovered: Option<usize>,
    /// Whether the wheel was open (any slice hovered) last frame.
    pub open: bool,
}

/// Full runtime configuration for a wheel menu.
#[derive(Component, Clone)]
pub struct RadialMenuConfig {
    /// Policy used to confirm a hovered sector.
    pub casting_mode: CastingMode,
    /// Policy used to open and close the menu.
    pub toggle_mode: RadialMenuToggleMode,
    /// Whether directional input is snapped toward a sector center.
    pub auto_snap: bool,
    /// Whether the menu consumes gameplay input while open.
    pub block_gameplay_input: bool,
}

impl Default for RadialMenuConfig {
    fn default() -> Self {
        Self {
            casting_mode: CastingMode::Vanilla,
            toggle_mode: RadialMenuToggleMode::Hold,
            auto_snap: true,
            block_gameplay_input: false,
        }
    }
}

/// Runtime progress for hold-to-activate casting.
#[derive(Component, Default, Clone)]
pub struct RadialMenuHoldState {
    /// Normalized hold progress from zero to one.
    pub progress: f32,
    /// Whether the activation input is currently held.
    pub holding: bool,
}

/// Runtime item-count data for a slice.
#[derive(Component, Clone, Default)]
pub struct SectorCount {
    /// Current quantity represented by the sector.
    pub current: u32,
    /// Maximum quantity represented by the sector.
    pub max: u32,
    /// Quantity at or below which a low-count message is emitted.
    pub low_threshold: u32,
    /// Prevents repeated low-count notifications until reset.
    pub low_notified: bool,
}

/// Runtime edit-mode state for a wheel.
#[derive(Component, Default, Clone)]
pub struct RadialMenuEditMode {
    /// Whether editor interactions are enabled for this menu.
    pub active: bool,
    /// Optional gamepad button used to toggle edit mode.
    pub toggle_button: Option<GamepadButton>,
}

/// Visual configuration / skin for a wheel.
#[derive(Component, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RadialMenuStyle {
    /// Normal sector color in RGBA component values.
    pub base_color: [f32; 4],
    /// Hovered sector color.
    pub hover_color: [f32; 4],
    /// Selected sector color.
    pub selected_color: [f32; 4],
    /// Text color.
    pub text_color: [f32; 4],
    /// Application-defined skin identifier.
    pub skin: String,
}

impl Default for RadialMenuStyle {
    fn default() -> Self {
        Self {
            base_color: [0.08, 0.12, 0.18, 0.85],
            hover_color: [0.2, 0.5, 0.9, 0.95],
            selected_color: [0.1, 0.7, 0.4, 0.9],
            text_color: [0.85, 0.88, 0.92, 1.0],
            skin: "default".into(),
        }
    }
}

impl RadialMenuStyle {
    /// Converts the base color into a Bevy color.
    pub fn base(&self) -> Color {
        Color::srgba(
            self.base_color[0],
            self.base_color[1],
            self.base_color[2],
            self.base_color[3],
        )
    }

    /// Converts the hover color into a Bevy color.
    pub fn hover(&self) -> Color {
        Color::srgba(
            self.hover_color[0],
            self.hover_color[1],
            self.hover_color[2],
            self.hover_color[3],
        )
    }

    /// Converts the selected color into a Bevy color.
    pub fn selected(&self) -> Color {
        Color::srgba(
            self.selected_color[0],
            self.selected_color[1],
            self.selected_color[2],
            self.selected_color[3],
        )
    }

    /// Converts the text color into a Bevy color.
    pub fn text(&self) -> Color {
        Color::srgba(
            self.text_color[0],
            self.text_color[1],
            self.text_color[2],
            self.text_color[3],
        )
    }
}

/// Sound asset paths associated with wheel lifecycle actions.
#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct RadialMenuAudio {
    /// Sound played when the menu opens.
    pub open: Option<String>,
    /// Sound played when the hovered sector changes.
    pub hover: Option<String>,
    /// Sound played when a sector is selected.
    pub select: Option<String>,
    /// Sound played when a submenu is opened.
    pub submenu: Option<String>,
}

/// Runtime parent/child links for nested submenu wheels.
#[derive(Component, Clone, Default)]
pub struct RadialMenuHierarchy {
    /// Optional parent menu entity.
    pub parent: Option<Entity>,
    /// Child submenu entities owned by this menu.
    pub children: Vec<Entity>,
}

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
            inner_radius: 130.0,
            gap: 0.012,
            arc_span: std::f32::consts::TAU,
            arc_offset: std::f32::consts::FRAC_PI_6,
        }
    }
}

/// Geometry options for rendering radial-menu sectors.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum SegmentShape {
    /// Rounded rectangular sector panel.
    #[default]
    Rounded,
    /// Square-cornered sector panel.
    Square,
    /// Circular sector panel.
    Circle,
    /// Tapered sector panel.
    Wedge,
    /// Procedural pie-slice sector.
    Pie,
}

impl SegmentShape {
    /// Returns the human-readable editor label for this shape.
    pub fn label(self) -> &'static str {
        match self {
            Self::Rounded => "Rounded",
            Self::Square => "Square",
            Self::Circle => "Circle",
            Self::Wedge => "Wedge",
            Self::Pie => "Pie",
        }
    }

    /// Returns the next shape in the built-in shape cycle.
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
    /// Geometry used to render each sector.
    pub segment_shape: SegmentShape,
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
            slots: vec![Sector::named("Slot 1")],
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            theme: WheelTheme::Dark,
            outer_radius: geometry.outer_radius,
            inner_radius: geometry.inner_radius,
            show_labels: false,
            segment_shape: SegmentShape::Pie,
            show_icon: true,
            highlight_color: "#ef8b92".into(),
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
    /// Creates a menu with at least one named sector.
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

/// Calculates the start and end angles of a sector, including its gap.
pub fn slice_angles(menu: &RadialMenu, index: usize) -> (f32, f32) {
    let sector_count = menu.slots.len().max(1);
    let sector_angle = menu.arc_span / sector_count as f32;
    let half_gap = if menu.overlap { 0.0 } else { menu.gap / 2.0 };
    let start = menu.arc_offset + index as f32 * sector_angle + half_gap;
    let end = menu.arc_offset + (index + 1) as f32 * sector_angle - half_gap;
    (start, end)
}

/// Returns the center position of a sector in logical pixels.
pub fn slice_center(menu: &RadialMenu, index: usize) -> Vec2 {
    let (start, end) = slice_angles(menu, index);
    let center_angle = (start + end) / 2.0;
    let center_radius = (menu.inner_radius + menu.outer_radius) / 2.0;
    Vec2::new(
        center_angle.cos() * center_radius,
        center_angle.sin() * center_radius,
    )
}

/// Returns a zero-size hub [`Node`] used as the positioning origin for
/// sectors and radial-menu rings.
pub fn wheel_hub() -> impl Scene {
    bsn! {
        Node { width: { Val::Px(0.) }, height: { Val::Px(0.) } }
    }
}

/// Returns a circular background disc that fills the radial menu area.
pub fn wheel_bg_disc(outer_radius: f32, color: Color) -> impl Scene {
    let radius = outer_radius + 4.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(-radius) },
            top: { Val::Px(-radius) },
            width: { Val::Px(radius * 2.0) },
            height: { Val::Px(radius * 2.0) },
            border_radius: { BorderRadius::all(Val::Px(radius)) },
        }
        BackgroundColor({ color })
    }
}

/// Returns the outer border ring for a radial menu.
pub fn wheel_outer_ring(outer_radius: f32, color: Color, border_width: f32) -> impl Scene {
    let radius = outer_radius + 1.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(-radius) },
            top: { Val::Px(-radius) },
            width: { Val::Px(radius * 2.0) },
            height: { Val::Px(radius * 2.0) },
            border_radius: { BorderRadius::all(Val::Px(radius)) },
            border: { UiRect::all(Val::Px(border_width)) },
        }
        BackgroundColor({ Color::NONE })
        BorderColor::all(color)
    }
}

/// Returns the inner ring surrounding the radial-menu hub.
pub fn wheel_center_ring(
    radius: f32,
    background: Color,
    ring_color: Color,
    ring_width: f32,
) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(-radius) },
            top: { Val::Px(-radius) },
            width: { Val::Px(radius * 2.0) },
            height: { Val::Px(radius * 2.0) },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: { BorderRadius::all(Val::Px(radius)) },
            border: { UiRect::all(Val::Px(ring_width)) },
        }
        BackgroundColor({ background })
        BorderColor::all(ring_color)
    }
}

/// Returns a text scene used to display a sector label.
pub fn wheel_slice_label(text: String, font_size: f32, color: Color) -> impl Scene {
    bsn! {
        Text({ text })
        TextFont { font_size: { FontSize::Px(font_size) } }
        TextColor({ color })
    }
}

#[cfg(test)]
mod tests {
    use super::RadialMenuToggleMode;

    #[test]
    fn toggle_mode_default_is_hold() {
        assert_eq!(RadialMenuToggleMode::default(), RadialMenuToggleMode::Hold);
    }
}

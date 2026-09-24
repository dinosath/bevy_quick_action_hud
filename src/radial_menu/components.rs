//! Entity-local state for the core wheel runtime.

use bevy::prelude::*;
use bevy::scene::prelude::{Scene, SceneList};
use serde::{Deserialize, Serialize};

/// Default binding used to navigate a radial menu.
pub const DEFAULT_STICK_BINDING: &str = "GP:RightStick";

/// Minimum number of sectors in a radial menu.
pub const MIN_SECTORS: usize = 2;

/// Extra radial extent applied to the highlighted sector.
///
/// The reference menu keeps the unselected sectors on the base outer ring,
/// while the selected sector projects beyond that ring.  The inner radius is
/// intentionally unchanged so its inner edge remains the hub's circular arc.
pub const HIGHLIGHTED_SECTOR_OUTSET: f32 = 38.0;

/// Amount by which the hub overlaps sector geometry to guarantee a clean
/// circular inner edge despite the native-strip approximation.
pub const RADIAL_MENU_HUB_OVERLAP: f32 = 4.0;

/// Extra radius of the circular background behind the base outer ring.
pub const RADIAL_MENU_BACKGROUND_BLEED: f32 = 4.0;

/// Extra radius of the retained outer-ring node beyond the authored radius.
pub const RADIAL_MENU_OUTER_RING_OUTSET: f32 = 1.0;

/// RGB fallback for the radial-menu background surface.
pub const RADIAL_MENU_BACKGROUND_RGB: [f32; 3] = [0.096, 0.118, 0.157];

/// RGBA fallback for the base outer-ring stroke.
pub const RADIAL_MENU_OUTER_BORDER_RGBA: [f32; 4] = [0.38, 0.39, 0.39, 0.90];

/// RGBA fill for an unselected sector.
pub const RADIAL_MENU_SECTOR_RGBA: [f32; 4] = [0.10, 0.11, 0.12, 1.0];

/// RGBA stroke for an unselected sector divider.
pub const RADIAL_MENU_DIVIDER_RGBA: [f32; 4] = [0.30, 0.32, 0.33, 0.85];

/// RGBA fallback for the hub border.
pub const RADIAL_MENU_HUB_BORDER_RGBA: [f32; 4] = [0.34, 0.35, 0.35, 1.0];

/// RGBA color for visible sector labels.
pub const RADIAL_MENU_LABEL_RGBA: [f32; 4] = [0.91, 0.91, 0.89, 1.0];

/// Fraction of the sector thickness used for label sizing.
pub const RADIAL_MENU_LABEL_SIZE_RATIO: f32 = 0.18;

/// Minimum and maximum sector-label font sizes in logical pixels.
pub const RADIAL_MENU_LABEL_SIZE_RANGE: (f32, f32) = (9.0, 13.0);

/// Fraction of the sector panel height used for its icon badge.
pub const RADIAL_MENU_ICON_SIZE_RATIO: f32 = 0.42;

/// Minimum and maximum icon badge dimensions in logical pixels.
pub const RADIAL_MENU_ICON_SIZE_RANGE: (f32, f32) = (24.0, 44.0);

/// Padding applied around sector labels and icon badges.
pub const RADIAL_MENU_SECTOR_CONTENT_PADDING: f32 = 6.0;

/// Width of the selected-sector native UI outline.
pub const RADIAL_MENU_SELECTED_OUTLINE_WIDTH: f32 = 2.0;

/// Width of sector divider strokes.
pub const RADIAL_MENU_DIVIDER_WIDTH: f32 = 1.5;

/// Weight used when blending the configured highlight into a selected sector.
pub const RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT: f32 = 0.34;

/// Default fallback width for both the base outer and hub border strokes.
pub const RADIAL_MENU_DEFAULT_BORDER_WIDTH: f32 = 1.0;

/// Number of native UI strips used to approximate one circular sector edge.
///
/// The count keeps the reference arc smooth at normal HUD scale without a
/// custom material or shader.  Sector scenes are rebuilt only on HUD changes,
/// not on pointer movement.
const RADIAL_MENU_SECTOR_STRIPS: usize = 256;

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

/// Returns the radial extent used by a sector in the retained UI scene.
///
/// Normal sectors terminate at the authored outer radius.  The highlighted
/// sector extends beyond that base ring as measured from the reference UI.
pub(crate) fn rendered_sector_outer_radius(menu: &RadialMenu, highlighted: bool) -> f32 {
    menu.outer_radius * menu.segment_scale
        + if highlighted {
            HIGHLIGHTED_SECTOR_OUTSET
        } else {
            0.0
        }
}

/// Returns the hub radius that clips the sector panels into an inner arc.
pub(crate) fn radial_menu_hub_radius(menu: &RadialMenu) -> f32 {
    (menu.inner_radius - RADIAL_MENU_HUB_OVERLAP).max(8.0)
}

/// Returns the full radial panel height required for a sector to reach the hub.
///
/// The hub clips this panel into an annular sector.  It must not be shortened
/// to the visible annulus thickness because that creates a flat inner edge.
pub(crate) fn radial_menu_sector_panel_height(outer_radius: f32) -> f32 {
    outer_radius.max(24.0)
}

/// Blends an overlay color into a sector fill without changing its opacity.
pub(crate) fn blend_radial_menu_colors(base: Color, overlay: Color, weight: f32) -> Color {
    let base = base.to_srgba();
    let overlay = overlay.to_srgba();
    let weight = weight.clamp(0.0, 1.0);
    Color::srgb(
        base.red * (1.0 - weight) + overlay.red * weight,
        base.green * (1.0 - weight) + overlay.green * weight,
        base.blue * (1.0 - weight) + overlay.blue * weight,
    )
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

/// Resolves `angle` in menu-local radians to the sector it occupies.
///
/// This is the input counterpart of [`slice_angles`]: it observes the authored
/// arc span and inter-sector gaps, returning `None` for a direction outside the
/// menu or inside a visible gap.
pub(crate) fn sector_index_at_angle(menu: &RadialMenu, angle: f32) -> Option<usize> {
    let sector_count = menu.slots.len();
    let arc_span = menu.arc_span.clamp(0.0, std::f32::consts::TAU);
    if sector_count == 0 || arc_span <= f32::EPSILON {
        return None;
    }

    let relative = (angle - menu.arc_offset).rem_euclid(std::f32::consts::TAU);
    if relative >= arc_span {
        return None;
    }

    let sector_span = arc_span / sector_count as f32;
    let within_sector = relative.rem_euclid(sector_span);
    let half_gap = if menu.overlap {
        0.0
    } else {
        (menu.gap * 0.5).min(sector_span * 0.5)
    };
    if within_sector < half_gap || within_sector > sector_span - half_gap {
        return None;
    }

    Some((relative / sector_span).floor() as usize)
}

/// Declares the flex-layout anchor used for a centered radial menu.
pub(crate) fn radial_menu_anchor() -> impl Scene {
    bsn! {
        Node {
            width: { Val::Percent(100.) },
            height: { Val::Percent(100.) },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

/// Declares the zero-size positioning origin inside a centered menu anchor.
pub(crate) fn radial_menu_hub() -> impl Scene {
    bsn! {
        Node {
            width: { Val::Px(0.) },
            height: { Val::Px(0.) },
        }
    }
}

/// Declares the circular background surface for a radial menu.
pub(crate) fn radial_menu_background(outer_radius: f32, color: Color) -> impl Scene {
    let radius = outer_radius + RADIAL_MENU_BACKGROUND_BLEED;
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

/// Declares the outer border ring for a radial menu.
pub(crate) fn radial_menu_outer_ring(
    outer_radius: f32,
    color: Color,
    border_width: f32,
) -> impl Scene {
    let radius = outer_radius + RADIAL_MENU_OUTER_RING_OUTSET;
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

/// Declares the inner ring surrounding a radial-menu hub.
pub(crate) fn radial_menu_center_ring(
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

/// Declares one native Bevy UI annular sector panel.
///
/// Sectors use the radial shape shared by every menu: native UI strips
/// constrained to the menu's outer circle. This keeps rendering in BSN and
/// avoids a custom material or shader.
pub(crate) fn radial_menu_sector(
    center: Vec2,
    width: f32,
    height: f32,
    outer_radius: f32,
    sector_span: f32,
    color: Color,
    outline: Option<Color>,
    rotation: f32,
) -> impl Scene {
    let strip_height = height / RADIAL_MENU_SECTOR_STRIPS as f32;
    let mut children: Vec<Box<dyn SceneList>> = Vec::with_capacity(RADIAL_MENU_SECTOR_STRIPS);
    for index in 0..RADIAL_MENU_SECTOR_STRIPS {
        let radial_distance = outer_radius - (index as f32 + 0.5) * strip_height;
        let strip_width = sector_strip_width(radial_distance, outer_radius, sector_span);
        let left = (width - strip_width) * 0.5;
        let top = index as f32 * strip_height;
        children.push(Box::new(bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: { Val::Px(left) },
                top: { Val::Px(top) },
                width: { Val::Px(strip_width) },
                height: { Val::Px(strip_height + 2.0) },
            }
            BackgroundColor({ outline.unwrap_or(color) })
        }) as Box<dyn SceneList>);
    }

    if outline.is_some() {
        // Keep the panel's radial edge at the hub, but pull the fill back by
        // two pixels at the outer edge.  This is a native-UI outline: the
        // underlying sector remains visible as the selected sector's curved
        // outer border and its two radial edges.
        let outline_width = RADIAL_MENU_SELECTED_OUTLINE_WIDTH;
        let fill_radius = (outer_radius - outline_width).max(0.0);
        let fill_height = (height - outline_width).max(0.0);
        let fill_strip_height = fill_height / RADIAL_MENU_SECTOR_STRIPS as f32;
        for index in 0..RADIAL_MENU_SECTOR_STRIPS {
            let radial_distance = fill_radius - (index as f32 + 0.5) * fill_strip_height;
            let strip_width = sector_strip_width(radial_distance, fill_radius, sector_span);
            let left = (width - strip_width) * 0.5;
            let top = outline_width + index as f32 * fill_strip_height;
            children.push(Box::new(bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: { Val::Px(left) },
                    top: { Val::Px(top) },
                    width: { Val::Px(strip_width) },
                    height: { Val::Px(fill_strip_height + 2.0) },
                }
                BackgroundColor({ color })
            }) as Box<dyn SceneList>);
        }
    }

    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left:   { Val::Px(center.x - width / 2.0) },
            top:    { Val::Px(-center.y - height / 2.0) },
            width:  { Val::Px(width) },
            height: { Val::Px(height) },
        }
        // `Transform` is for world-space entities. Native Bevy UI resolves
        // `UiTransform` during layout, so this is what actually rotates the
        // tapered strips around their sector centre.
        UiTransform::from_rotation(Rot2::radians(rotation))
        Children [{ children }]
    }
}

/// Declares a straight divider between two adjacent annular sectors.
pub(crate) fn radial_menu_divider(
    angle: f32,
    inner_radius: f32,
    outer_radius: f32,
    color: Color,
    width: f32,
) -> impl Scene {
    let length = (outer_radius - inner_radius).max(0.0);
    let center_radius = (inner_radius + outer_radius) * 0.5;
    let center = Vec2::new(angle.cos() * center_radius, angle.sin() * center_radius);
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(center.x - width * 0.5) },
            top: { Val::Px(-center.y - length * 0.5) },
            width: { Val::Px(width) },
            height: { Val::Px(length) },
        }
        BackgroundColor({ color })
        UiTransform::from_rotation(Rot2::radians(std::f32::consts::FRAC_PI_2 - angle))
    }
}

/// Returns one horizontal strip width for a native annular-sector approximation.
///
/// The radial boundary limits the width to the sector's two radial edges, while
/// the circular boundary prevents the strip from extending beyond the menu's
/// outer ring. The hub covers the corresponding inner-circle edge.
fn sector_strip_width(radial_distance: f32, outer_radius: f32, sector_span: f32) -> f32 {
    let radial_half_width = radial_distance * (sector_span * 0.5).tan().abs();
    let circle_half_width = (outer_radius.powi(2) - radial_distance.powi(2))
        .max(0.0)
        .sqrt();
    2.0 * radial_half_width.min(circle_half_width)
}

/// Declares the upright content layer inside a rotated radial-menu sector.
pub(crate) fn radial_menu_sector_content(rotation: f32, outward_offset: f32) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(0.) },
            top: { Val::Px(0.) },
            width: { Val::Percent(100.) },
            height: { Val::Percent(100.) },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            padding: { UiRect::all(Val::Px(RADIAL_MENU_SECTOR_CONTENT_PADDING)) },
        }
        // A sector panel runs beneath the hub so the hub creates its circular
        // inner edge.  Offset its content back into the visible annulus and
        // counter-rotate it so labels and icons remain upright.
        UiTransform {
            translation: { Val2::px(0., -outward_offset) },
            scale: { Vec2::ONE },
            rotation: { Rot2::radians(-rotation) },
        }
    }
}

/// Declares the text presentation for one sector label.
pub(crate) fn radial_menu_label(text: String, font_size: f32, color: Color) -> impl Scene {
    bsn! {
        Text({ text })
        TextFont { font_size: { FontSize::Px(font_size) } }
        TextColor({ color })
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Color, Vec2};

    use super::{
        blend_radial_menu_colors, radial_menu_hub_radius, radial_menu_sector_panel_height,
        rendered_sector_outer_radius, sector_index_at_angle, sector_strip_width, RadialMenu,
        RadialMenuToggleMode, WheelTheme, HIGHLIGHTED_SECTOR_OUTSET, RADIAL_MENU_BACKGROUND_BLEED,
        RADIAL_MENU_BACKGROUND_RGB, RADIAL_MENU_DEFAULT_BORDER_WIDTH, RADIAL_MENU_DIVIDER_RGBA,
        RADIAL_MENU_DIVIDER_WIDTH, RADIAL_MENU_HUB_BORDER_RGBA, RADIAL_MENU_HUB_OVERLAP,
        RADIAL_MENU_ICON_SIZE_RANGE, RADIAL_MENU_ICON_SIZE_RATIO, RADIAL_MENU_LABEL_RGBA,
        RADIAL_MENU_LABEL_SIZE_RANGE, RADIAL_MENU_LABEL_SIZE_RATIO, RADIAL_MENU_OUTER_BORDER_RGBA,
        RADIAL_MENU_OUTER_RING_OUTSET, RADIAL_MENU_SECTOR_CONTENT_PADDING, RADIAL_MENU_SECTOR_RGBA,
        RADIAL_MENU_SECTOR_STRIPS, RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT,
        RADIAL_MENU_SELECTED_OUTLINE_WIDTH,
    };

    fn assert_color_eq(actual: Color, expected: [f32; 4]) {
        let actual = actual.to_srgba();
        let actual = [actual.red, actual.green, actual.blue, actual.alpha];
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-5,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    #[test]
    fn toggle_mode_default_is_hold() {
        assert_eq!(RadialMenuToggleMode::default(), RadialMenuToggleMode::Hold);
    }

    #[test]
    fn sector_resolution_excludes_configured_gaps() {
        let mut menu = RadialMenu::new("Test", 4);
        menu.arc_offset = 0.0;
        menu.gap = 0.2;

        assert_eq!(sector_index_at_angle(&menu, 0.4), Some(0));
        assert_eq!(sector_index_at_angle(&menu, 1.58), None);
    }

    #[test]
    fn sector_resolution_respects_partial_arcs() {
        let mut menu = RadialMenu::new("Test", 2);
        menu.arc_offset = 0.0;
        menu.arc_span = std::f32::consts::PI;
        menu.gap = 0.0;

        assert_eq!(sector_index_at_angle(&menu, 0.5), Some(0));
        assert_eq!(sector_index_at_angle(&menu, 2.0), Some(1));
        assert_eq!(sector_index_at_angle(&menu, 4.0), None);
    }

    #[test]
    fn sector_rendering_stays_inside_the_outer_circle() {
        let outer_radius = 270.0;
        let sector_span = std::f32::consts::TAU / 6.0;
        for row in 0..RADIAL_MENU_SECTOR_STRIPS {
            let strip_height = outer_radius / RADIAL_MENU_SECTOR_STRIPS as f32;
            let radius = outer_radius - (row as f32 + 0.5) * strip_height;
            let width = sector_strip_width(radius, outer_radius, sector_span);
            let half_width = width * 0.5;
            assert!(radius.powi(2) + half_width.powi(2) <= outer_radius.powi(2) + 0.01);
        }
    }

    #[test]
    fn sector_rendering_opens_toward_the_outer_ring() {
        let outer_radius = 270.0;
        let span = std::f32::consts::TAU / 6.0;
        let inner_width = sector_strip_width(132.0, outer_radius, span);
        let middle_width = sector_strip_width(200.0, outer_radius, span);

        assert!(middle_width > inner_width);
    }

    #[test]
    fn default_menu_is_centered_without_a_user_translation() {
        let menu = RadialMenu::default();

        assert_eq!(menu.offset_x, 0.0);
        assert_eq!(menu.offset_y, 0.0);
    }

    #[test]
    fn default_four_sector_menu_starts_at_twelve_oclock() {
        let menu = RadialMenu::default();
        let (start, end) = super::slice_angles(&menu, 0);

        assert!(((start + end) * 0.5 - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn default_layout_matches_measured_reference_image() {
        let menu = RadialMenu::default();
        let center_radius = (menu.inner_radius + menu.outer_radius) * 0.5;
        let expected_centers = [
            Vec2::new(0.0, center_radius),
            Vec2::new(-center_radius, 0.0),
            Vec2::new(0.0, -center_radius),
            Vec2::new(center_radius, 0.0),
        ];

        assert_eq!(menu.slots.len(), 4);
        assert_eq!(menu.outer_radius, 270.0);
        assert_eq!(menu.inner_radius, 145.0);
        assert_eq!(menu.highlight_color, "#a8e9ec");
        assert_eq!(HIGHLIGHTED_SECTOR_OUTSET, 38.0);
        assert_eq!(rendered_sector_outer_radius(&menu, false), 270.0);
        assert_eq!(rendered_sector_outer_radius(&menu, true), 308.0);
        assert_eq!(radial_menu_hub_radius(&menu), 141.0);

        // Measurements from image.png's menu crop: the four-sector base ring
        // has a 270 px radius, the selected top sector reaches 308 px, and
        // the circular hub masks panels at roughly 141 px.  The first sector
        // spans 45°..135°, giving these two selected outer endpoints.
        let (start, end) = super::slice_angles(&menu, 0);
        assert!((start - (std::f32::consts::FRAC_PI_4 + menu.gap * 0.5)).abs() < 1e-5);
        assert!((end - (std::f32::consts::FRAC_PI_4 * 3.0 - menu.gap * 0.5)).abs() < 1e-5);
        let selected_radius = rendered_sector_outer_radius(&menu, true);
        let left_endpoint = Vec2::from_angle(end) * selected_radius;
        let right_endpoint = Vec2::from_angle(start) * selected_radius;
        assert!(left_endpoint.distance(Vec2::new(-216.5, 219.1)) < 0.2);
        assert!(right_endpoint.distance(Vec2::new(216.5, 219.1)) < 0.2);

        // Panels run from the selected outer edge to the hub.  This guards
        // against reintroducing the old `outer_radius - inner_radius` panel
        // height, which produced a flat inner trapezoid instead of the
        // reference's concave circular boundary.
        let panel_height = radial_menu_sector_panel_height(selected_radius);
        let final_strip_radius = selected_radius
            - (RADIAL_MENU_SECTOR_STRIPS as f32 - 0.5)
                * (panel_height / RADIAL_MENU_SECTOR_STRIPS as f32);
        assert!(final_strip_radius < radial_menu_hub_radius(&menu));
        for (index, expected) in expected_centers.into_iter().enumerate() {
            assert!(super::slice_center(&menu, index).distance(expected) < 1e-4);
        }
    }

    #[test]
    fn default_reference_presentation_locks_colors_spacing_and_control_style() {
        let menu = RadialMenu::default();

        // Authored default state: the reference is a centered, dark, icon-led
        // four-sector menu.  Empty border/color fields deliberately select
        // the fallback presentation values asserted below.
        assert_eq!(menu.theme, WheelTheme::Dark);
        assert!(!menu.show_labels);
        assert!(menu.show_icon);
        assert_eq!(menu.opacity, 1.0);
        assert_eq!(menu.bg_opacity, 1.0);
        assert_eq!(menu.hub_opacity, 1.0);
        assert!(!menu.overlap);
        assert_eq!(menu.gap, 0.012);
        assert_eq!(menu.rotation, 0.0);
        assert!(menu.bg_color.is_empty());
        assert!(menu.hub_color.is_empty());
        assert!(menu.inner_border.is_empty());
        assert!(menu.outer_border.is_empty());
        assert_eq!(menu.outer_border_width, 2.0);
        assert_eq!(menu.inner_border_width, 2.0);

        // Exact fallback palette used by the BSN scene.
        assert_color_eq(
            Color::srgba(
                RADIAL_MENU_BACKGROUND_RGB[0],
                RADIAL_MENU_BACKGROUND_RGB[1],
                RADIAL_MENU_BACKGROUND_RGB[2],
                menu.bg_opacity,
            ),
            [0.096, 0.118, 0.157, 1.0],
        );
        assert_color_eq(
            Color::srgba(
                RADIAL_MENU_OUTER_BORDER_RGBA[0],
                RADIAL_MENU_OUTER_BORDER_RGBA[1],
                RADIAL_MENU_OUTER_BORDER_RGBA[2],
                RADIAL_MENU_OUTER_BORDER_RGBA[3],
            ),
            [0.38, 0.39, 0.39, 0.90],
        );
        assert_color_eq(
            Color::srgba(
                RADIAL_MENU_SECTOR_RGBA[0],
                RADIAL_MENU_SECTOR_RGBA[1],
                RADIAL_MENU_SECTOR_RGBA[2],
                RADIAL_MENU_SECTOR_RGBA[3],
            ),
            [0.10, 0.11, 0.12, 1.0],
        );
        assert_color_eq(
            Color::srgba(
                RADIAL_MENU_DIVIDER_RGBA[0],
                RADIAL_MENU_DIVIDER_RGBA[1],
                RADIAL_MENU_DIVIDER_RGBA[2],
                RADIAL_MENU_DIVIDER_RGBA[3],
            ),
            [0.30, 0.32, 0.33, 0.85],
        );
        assert_color_eq(
            Color::srgba(
                RADIAL_MENU_HUB_BORDER_RGBA[0],
                RADIAL_MENU_HUB_BORDER_RGBA[1],
                RADIAL_MENU_HUB_BORDER_RGBA[2],
                RADIAL_MENU_HUB_BORDER_RGBA[3],
            ),
            [0.34, 0.35, 0.35, 1.0],
        );
        assert_color_eq(
            Color::srgba(
                RADIAL_MENU_LABEL_RGBA[0],
                RADIAL_MENU_LABEL_RGBA[1],
                RADIAL_MENU_LABEL_RGBA[2],
                RADIAL_MENU_LABEL_RGBA[3],
            ),
            [0.91, 0.91, 0.89, 1.0],
        );

        // #a8e9ec is the selected outline and, blended at 34%, produces the
        // exact selected fill emitted by the retained scene.
        let selected = blend_radial_menu_colors(
            Color::srgba(
                RADIAL_MENU_SECTOR_RGBA[0],
                RADIAL_MENU_SECTOR_RGBA[1],
                RADIAL_MENU_SECTOR_RGBA[2],
                RADIAL_MENU_SECTOR_RGBA[3],
            ),
            Color::srgb(168.0 / 255.0, 233.0 / 255.0, 236.0 / 255.0),
            RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT,
        );
        assert_color_eq(selected, [0.29, 0.383_266_7, 0.393_866_7, 1.0]);

        // Structural styling and spacing of the native BSN sector scene.
        assert_eq!(RADIAL_MENU_BACKGROUND_BLEED, 4.0);
        assert_eq!(RADIAL_MENU_OUTER_RING_OUTSET, 1.0);
        assert_eq!(RADIAL_MENU_DEFAULT_BORDER_WIDTH, 1.0);
        assert_eq!(RADIAL_MENU_DIVIDER_WIDTH, 1.5);
        assert_eq!(RADIAL_MENU_SELECTED_OUTLINE_WIDTH, 2.0);
        assert_eq!(RADIAL_MENU_HUB_OVERLAP, 4.0);
        assert_eq!(RADIAL_MENU_SECTOR_CONTENT_PADDING, 6.0);
        assert_eq!(RADIAL_MENU_SECTOR_STRIPS, 256);

        let thickness = menu.outer_radius - menu.inner_radius;
        let label_size = (thickness * RADIAL_MENU_LABEL_SIZE_RATIO).clamp(
            RADIAL_MENU_LABEL_SIZE_RANGE.0,
            RADIAL_MENU_LABEL_SIZE_RANGE.1,
        );
        let icon_size = (rendered_sector_outer_radius(&menu, false) * RADIAL_MENU_ICON_SIZE_RATIO)
            .clamp(RADIAL_MENU_ICON_SIZE_RANGE.0, RADIAL_MENU_ICON_SIZE_RANGE.1);
        assert_eq!(label_size, 13.0);
        assert_eq!(icon_size, 44.0);
    }
}

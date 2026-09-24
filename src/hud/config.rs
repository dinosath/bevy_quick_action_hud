//! Serializable HUD document model: pages, buttons, switches, and defaults.

use crate::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Placement reference for a floating quick-action button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
/// Coordinate interpretation for a floating button.
pub enum PositionMode {
    #[default]
    /// Position relative to the HUD action layout.
    Relative,
    /// Position using absolute HUD coordinates.
    Absolute,
}
impl PositionMode {
    /// Returns the human-readable editor label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Relative => "Relative",
            Self::Absolute => "Absolute",
        }
    }
    /// Returns the next placement mode.
    pub fn next(self) -> Self {
        match self {
            Self::Relative => Self::Absolute,
            Self::Absolute => Self::Relative,
        }
    }
}

/// Shape of a quick-action HUD button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
/// Shape used by a floating HUD button.
pub enum ActionShape {
    #[default]
    /// Rounded rectangle.
    Rounded,
    /// Circle-like button.
    Round,
    /// Square button.
    Square,
    /// Diamond-shaped button.
    Diamond,
}
impl ActionShape {
    /// Returns the human-readable editor label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Rounded => "Rounded",
            Self::Round => "Round",
            Self::Square => "Square",
            Self::Diamond => "Diamond",
        }
    }
    /// Returns the next button shape.
    pub fn next(self) -> Self {
        match self {
            Self::Rounded => Self::Round,
            Self::Round => Self::Square,
            Self::Square => Self::Diamond,
            Self::Diamond => Self::Rounded,
        }
    }
}

// ── palette helpers ─────────────────────────────────────────────────────────────
/// Built-in symbolic icons used by the editor.
pub const ICON_PALETTE: &[&str] = &["◆", "●", "★", "▲", "✦", "✚", "◈", "○", "◐", "✱"];
/// Built-in action command identifiers used by the editor.
pub const COMMAND_PALETTE: &[&str] = &[
    "none", "attack", "heal", "block", "dash", "reload", "interact", "jump", "crouch", "sprint",
];
/// Returns the next value in a cyclic string palette.
pub fn cycle_palette<'a>(list: &[&'a str], current: &str) -> &'a str {
    let idx = list.iter().position(|s| *s == current).unwrap_or(0);
    list[(idx + 1) % list.len()]
}

// ── serde defaults ──────────────────────────────────────────────────────────────
fn _default_true() -> bool {
    true
}
pub(crate) fn _default_action_color() -> String {
    "#3b82f6".into()
}
fn _default_action_width() -> f32 {
    80.0
}
fn _default_action_height() -> f32 {
    28.0
}
fn _default_hold_command() -> String {
    "none".into()
}
fn _default_action_command() -> String {
    "none".into()
}
fn _full_opacity() -> f32 {
    1.0
}
fn _default_highlight_color() -> String {
    "#a8e9ec".into()
}
fn _default_segment_scale() -> f32 {
    1.0
}
fn _default_border_width() -> f32 {
    2.0
}
fn _default_deadzone() -> f32 {
    0.3
}
fn _default_gap() -> f32 {
    0.012
}
fn _default_arc_span() -> f32 {
    std::f32::consts::TAU
}
fn _default_arc_offset() -> f32 {
    std::f32::consts::FRAC_PI_6
}
fn _default_wheelset_min() -> usize {
    1
}
fn _default_wheelset_max() -> usize {
    8
}

// ── quick action ─────────────────────────────────────────────────────────────────

/// A key-bound floating HUD button.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct QuickAction {
    /// Display name shown on the button.
    pub name: String,
    /// Optional explanatory text for this HUD button.
    #[serde(default)]
    pub description: String,
    /// Keyboard key or gamepad button ("GP:\u{2026}" prefix) that triggers this action.
    pub key: String,
    /// Optional icon asset path or symbolic icon.
    pub icon: String,
    /// Action mapping invoked by the button.
    pub command: String,
    /// Command/mapping used while the input is held.
    #[serde(default = "_default_hold_command")]
    pub hold_command: String,
    /// Whether holding the binding invokes `hold_command`.
    pub hold: bool,
    /// Whether the button is visible while the HUD is open.
    pub show_on_menu: bool,
    /// Minimum time between activations while the HUD is open.
    #[serde(default)]
    pub cooldown_secs: f32,
    #[serde(default = "_default_true")]
    /// Whether the button label is rendered.
    pub show_labels: bool,
    #[serde(default = "_default_true")]
    /// Whether the button icon is rendered.
    pub show_icon: bool,
    /// Button opacity.
    pub opacity: f32,
    /// Position interpretation used by the HUD layout.
    pub position: PositionMode,
    /// Radial distance from the action anchor.
    pub radius: f32,
    /// Horizontal editor offset from the HUD's default action area.
    #[serde(default)]
    pub offset_x: f32,
    /// Vertical editor offset from the HUD's default action area.
    #[serde(default)]
    pub offset_y: f32,
    /// Rotation of the floating button in degrees.
    #[serde(default)]
    pub rotation: f32,
    /// Button shape.
    pub shape: ActionShape,
    #[serde(default = "_default_action_color")]
    /// Button fill color.
    pub color: String,
    #[serde(default = "_default_action_width")]
    /// Button width in logical pixels.
    pub width: f32,
    #[serde(default = "_default_action_height")]
    /// Button height in logical pixels.
    pub height: f32,
    #[serde(default = "_default_true")]
    /// Whether the button is active.
    pub enabled: bool,
    /// Close the HUD overlay when this action's shortcut is pressed.
    #[serde(default = "_default_true")]
    pub close_on_select: bool,
}
impl Default for QuickAction {
    fn default() -> Self {
        Self {
            name: "Action".into(),
            description: String::new(),
            key: String::new(),
            icon: "◆".into(),
            command: "none".into(),
            hold_command: "none".into(),
            hold: false,
            show_on_menu: true,
            cooldown_secs: 0.0,
            show_labels: true,
            show_icon: true,
            opacity: 1.0,
            position: PositionMode::Relative,
            radius: 48.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            shape: ActionShape::Rounded,
            color: _default_action_color(),
            width: _default_action_width(),
            height: _default_action_height(),
            enabled: true,
            close_on_select: true,
        }
    }
}

/// A visible HUD component that switches to another enabled HUD page.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct HudSwitch {
    /// Display name shown on the switch.
    pub name: String,
    /// Binding that activates the switch.
    pub key: String,
    /// Index of the page activated by this switch.
    pub target_page: usize,
    /// Whether the switch participates in the HUD.
    pub enabled: bool,
    #[serde(default)]
    /// Horizontal switch offset.
    pub offset_x: f32,
    #[serde(default)]
    /// Vertical switch offset.
    pub offset_y: f32,
    #[serde(default = "_default_action_width")]
    /// Switch width in logical pixels.
    pub width: f32,
    #[serde(default = "_default_action_height")]
    /// Switch height in logical pixels.
    pub height: f32,
}
impl Default for HudSwitch {
    fn default() -> Self {
        Self {
            name: "HUD Switch".into(),
            key: String::new(),
            target_page: 0,
            enabled: true,
            offset_x: 0.0,
            offset_y: 0.0,
            width: 100.0,
            height: 28.0,
        }
    }
}

/// Common editor contract for components that can appear on a HUD canvas.
pub trait HudComponent {
    /// Returns the display name used by editor lists.
    fn hud_name(&self) -> &str;
    /// Returns whether the component should be rendered.
    fn hud_enabled(&self) -> bool;
}
impl HudComponent for QuickAction {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        self.enabled
    }
}
impl HudComponent for RadialMenuSet {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        true
    }
}
impl HudComponent for HudSwitch {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        self.enabled
    }
}
/// Ensures the document has a page and normalizes each radial-menu set.
pub fn normalize_wheelset_config(cfg: &mut QuickActionConfig) {
    // A HUD document always contains at least one page, including after
    // loading an empty hand-authored or legacy RON document.
    if cfg.sets.is_empty() {
        cfg.sets.push(ActionSet::default());
    }
    for set in &mut cfg.sets {
        for entry in &mut set.entries {
            if let SetEntry::RadialMenuSet(ws) = entry {
                normalize_wheelset(ws);
            }
        }
    }
}

/// One entry inside an [`ActionSet`].
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SetEntry {
    /// A floating quick-action button.
    Action(QuickAction),
    /// A set containing one or more radial menus.
    #[serde(alias = "WheelSet")]
    RadialMenuSet(RadialMenuSet),
    /// A component that changes the active HUD page.
    HudSwitch(HudSwitch),
}

/// A named HUD page that holds buttons, radial-menu sets, and page switches.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ActionSet {
    /// Page name shown in the HUD page selector.
    pub name: String,
    /// Optional page icon path or symbolic icon identifier.
    #[serde(default)]
    pub icon: String,
    /// Whether this HUD page participates in the page switcher.
    #[serde(default = "_default_true")]
    pub enabled: bool,
    #[serde(default = "_full_opacity")]
    /// Page opacity.
    pub opacity: f32,
    #[serde(default)]
    /// Whether page-local input bindings override global bindings.
    pub input_override: bool,
    /// Components contained by this page.
    pub entries: Vec<SetEntry>,
    #[serde(default)]
    /// Optional page background image path.
    pub bg_image: String,
    #[serde(default = "_full_opacity")]
    /// Background image opacity.
    pub bg_image_opacity: f32,
    #[serde(default)]
    /// Shortcut for the next wheel on this page.
    pub next_wheel_key: String,
    #[serde(default)]
    /// Shortcut for the previous wheel on this page.
    pub prev_wheel_key: String,
    #[serde(default)]
    /// Whether page-level wheel navigation wraps.
    pub cycle_wheels: bool,
}
impl Default for ActionSet {
    fn default() -> Self {
        Self {
            name: "Set".into(),
            icon: String::new(),
            enabled: true,
            opacity: 1.0,
            input_override: false,
            entries: Vec::new(),
            bg_image: String::new(),
            bg_image_opacity: 1.0,
            next_wheel_key: String::new(),
            prev_wheel_key: String::new(),
            cycle_wheels: false,
        }
    }
}

/// Canonical page type for the authored HUD document.
pub type HudPage = ActionSet;
/// Compatibility alias for [`QuickAction`] as a HUD button.
pub type HudButton = QuickAction;

/// Returns the number of radial-menu-set entries in a page.
pub fn count_radial_menu_sets(set: &ActionSet) -> usize {
    set.entries
        .iter()
        .filter(|e| matches!(e, SetEntry::RadialMenuSet(_)))
        .count()
}

/// Returns the page indices currently included in HUD page navigation.
pub fn enabled_hud_pages(cfg: &QuickActionConfig) -> Vec<usize> {
    cfg.sets
        .iter()
        .enumerate()
        .filter_map(|(i, page)| page.enabled.then_some(i))
        .collect()
}

/// Whether the HUD overlay opens while a button is held (released = close)
/// or toggles open/closed on each press.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum HudOpenMode {
    /// Hold the trigger to keep the HUD open; releasing closes it.
    Hold,
    /// First press opens the HUD; second press closes it.
    #[default]
    Toggle,
}

impl HudOpenMode {
    /// Returns the human-readable editor label.
    pub fn label(&self) -> &'static str {
        match self {
            HudOpenMode::Hold => "Hold",
            HudOpenMode::Toggle => "Toggle",
        }
    }
    /// Returns the other HUD opening mode.
    pub fn next(&self) -> Self {
        match self {
            HudOpenMode::Hold => HudOpenMode::Toggle,
            HudOpenMode::Toggle => HudOpenMode::Hold,
        }
    }
}

/// The complete editable document (Bevy `Resource`).
#[derive(Resource, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct QuickActionConfig {
    #[serde(default)]
    /// Shortcut for the next page.
    pub next_set_key: String,
    #[serde(default)]
    /// Shortcut for the previous page.
    pub prev_set_key: String,
    /// Show the ActionSet tab bar in the HUD overlay.
    #[serde(default = "_default_true")]
    pub show_set_bar: bool,
    /// Pressing Next on the last set wraps to the first (cycle), otherwise stops.
    #[serde(default)]
    pub cycle_sets: bool,
    /// Key or gamepad button ("GP:…" prefix) that opens/closes the editor sidebar.
    #[serde(default)]
    pub edit_shortcut: String,
    /// Whether the HUD trigger button is a hold (release = close) or a toggle.
    #[serde(default)]
    pub hud_open_mode: HudOpenMode,
    /// Opacity of the full-screen HUD background overlay (0.0 = invisible, 1.0 = opaque).
    #[serde(default = "_full_opacity")]
    pub hud_bg_opacity: f32,
    /// Hex tint color for the HUD background overlay (e.g. "#0d1520"); empty = default dark.
    #[serde(default)]
    pub hud_bg_color: String,
    /// Ordered pages in the HUD document.
    pub sets: Vec<ActionSet>,
}

impl Default for QuickActionConfig {
    fn default() -> Self {
        let mut combat_wheel = RadialMenu::new("Combat radial menu", 4);
        combat_wheel.slots = vec![
            Sector {
                name: "Open map".into(),
                icon: "⌖".into(),
                ..default()
            },
            Sector {
                name: "Attack".into(),
                icon: "⚒".into(),
                ..default()
            },
            Sector {
                name: "Block".into(),
                icon: "✹".into(),
                ..default()
            },
            Sector {
                name: "Heal".into(),
                icon: "♧".into(),
                ..default()
            },
        ];
        Self {
            next_set_key: "Tab".into(),
            prev_set_key: "Q".into(),
            show_set_bar: true,
            cycle_sets: false,
            edit_shortcut: "GP:Start".into(),
            hud_open_mode: HudOpenMode::Toggle,
            hud_bg_opacity: 1.0,
            hud_bg_color: String::new(),
            sets: vec![
                ActionSet {
                    name: "Combat".into(),
                    icon: String::new(),
                    enabled: true,
                    opacity: 1.0,
                    input_override: false,
                    entries: vec![
                        SetEntry::RadialMenuSet(RadialMenuSet {
                            name: "Combat radial menu set".into(),
                            radial_menus: vec![combat_wheel, RadialMenu::new("Radial menu 2", 4)],
                            ..default()
                        }),
                        SetEntry::Action(QuickAction {
                            name: "Interact".into(),
                            key: "E".into(),
                            icon: "◆".into(),
                            command: "interact".into(),
                            color: "#14b8a6".into(),
                            width: 90.0,
                            height: 28.0,
                            ..default()
                        }),
                        SetEntry::Action(QuickAction {
                            name: "Inventory".into(),
                            key: "I".into(),
                            icon: "◈".into(),
                            command: "none".into(),
                            color: "#8b5cf6".into(),
                            width: 80.0,
                            height: 28.0,
                            ..default()
                        }),
                    ],
                    bg_image: String::new(),
                    bg_image_opacity: 1.0,
                    next_wheel_key: String::new(),
                    prev_wheel_key: String::new(),
                    cycle_wheels: false,
                },
                ActionSet {
                    name: "Stealth".into(),
                    icon: String::new(),
                    enabled: true,
                    opacity: 1.0,
                    input_override: false,
                    entries: vec![
                        SetEntry::RadialMenuSet(RadialMenuSet {
                            name: "Stealth radial menus".into(),
                            radial_menus: vec![RadialMenu::new("Stealth radial menu", 4)],
                            ..default()
                        }),
                        SetEntry::Action(QuickAction {
                            name: "Hide".into(),
                            key: "H".into(),
                            icon: "◐".into(),
                            command: "crouch".into(),
                            color: "#6366f1".into(),
                            width: 70.0,
                            height: 28.0,
                            ..default()
                        }),
                    ],
                    bg_image: String::new(),
                    bg_image_opacity: 1.0,
                    next_wheel_key: String::new(),
                    prev_wheel_key: String::new(),
                    cycle_wheels: false,
                },
            ],
        }
    }
}

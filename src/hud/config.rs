//! Serializable HUD document model: pages, buttons, switches, and defaults.

use crate::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Placement reference for a floating quick-action button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum PositionMode {
    #[default]
    Relative,
    Absolute,
}
impl PositionMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Relative => "Relative",
            Self::Absolute => "Absolute",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Relative => Self::Absolute,
            Self::Absolute => Self::Relative,
        }
    }
}

/// Shape of a quick-action HUD button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum ActionShape {
    #[default]
    Rounded,
    Round,
    Square,
    Diamond,
}
impl ActionShape {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rounded => "Rounded",
            Self::Round => "Round",
            Self::Square => "Square",
            Self::Diamond => "Diamond",
        }
    }
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
pub const ICON_PALETTE: &[&str] = &["◆", "●", "★", "▲", "✦", "✚", "◈", "○", "◐", "✱"];
pub const COMMAND_PALETTE: &[&str] = &[
    "none", "attack", "heal", "block", "dash", "reload", "interact", "jump", "crouch", "sprint",
];
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
pub(crate) fn _default_outer_radius() -> f32 {
    270.0
}
pub(crate) fn _default_inner_radius() -> f32 {
    130.0
}
fn _full_opacity() -> f32 {
    1.0
}
fn _default_highlight_color() -> String {
    "#ef8b92".into()
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
    pub name: String,
    /// Optional explanatory text for this HUD button.
    #[serde(default)]
    pub description: String,
    /// Keyboard key or gamepad button ("GP:\u{2026}" prefix) that triggers this action.
    pub key: String,
    pub icon: String,
    pub command: String,
    /// Command/mapping used while the input is held.
    #[serde(default = "_default_hold_command")]
    pub hold_command: String,
    pub hold: bool,
    pub show_on_menu: bool,
    pub opacity: f32,
    pub position: PositionMode,
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
    pub shape: ActionShape,
    #[serde(default = "_default_action_color")]
    pub color: String,
    #[serde(default = "_default_action_width")]
    pub width: f32,
    #[serde(default = "_default_action_height")]
    pub height: f32,
    #[serde(default = "_default_true")]
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
    pub name: String,
    pub key: String,
    pub target_page: usize,
    pub enabled: bool,
    #[serde(default)]
    pub offset_x: f32,
    #[serde(default)]
    pub offset_y: f32,
    #[serde(default = "_default_action_width")]
    pub width: f32,
    #[serde(default = "_default_action_height")]
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
    fn hud_name(&self) -> &str;
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
    Action(QuickAction),
    Wheel(RadialMenu),
    RadialMenuSet(RadialMenuSet),
    HudSwitch(HudSwitch),
}

/// A named context group that holds quick actions and wheels.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ActionSet {
    pub name: String,
    /// Optional page icon path or symbolic icon identifier.
    #[serde(default)]
    pub icon: String,
    /// Whether this HUD page participates in the page switcher.
    #[serde(default = "_default_true")]
    pub enabled: bool,
    #[serde(default = "_full_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub input_override: bool,
    pub entries: Vec<SetEntry>,
    #[serde(default)]
    pub bg_image: String,
    #[serde(default = "_full_opacity")]
    pub bg_image_opacity: f32,
    #[serde(default)]
    pub next_wheel_key: String,
    #[serde(default)]
    pub prev_wheel_key: String,
    #[serde(default)]
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

pub type HudPage = ActionSet;
pub type HudButton = QuickAction;

/// Returns the number of `Wheel` and `RadialMenuSetState` entries in a set.
pub fn count_wheel_entries(set: &ActionSet) -> usize {
    set.entries
        .iter()
        .filter(|e| matches!(e, SetEntry::Wheel(_) | SetEntry::RadialMenuSet(_)))
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
    pub fn label(&self) -> &'static str {
        match self {
            HudOpenMode::Hold => "Hold",
            HudOpenMode::Toggle => "Toggle",
        }
    }
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
    pub next_set_key: String,
    #[serde(default)]
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
    pub sets: Vec<ActionSet>,
}

impl Default for QuickActionConfig {
    fn default() -> Self {
        let mut combat_wheel = RadialMenu::new("Combat radial menu", 6);
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
            Sector {
                name: "Ability".into(),
                icon: "△".into(),
                ..default()
            },
            Sector {
                name: "Sprint".into(),
                icon: "◌".into(),
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
                            wheels: vec![combat_wheel, RadialMenu::new("Radial menu 2", 6)],
                            stick: StickSide::Right,
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
                            wheels: vec![RadialMenu::new("Stealth radial menu", 4)],
                            stick: StickSide::Right,
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

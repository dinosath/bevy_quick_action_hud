//! Serializable HUD document root: global settings and the ordered pages.

use crate::serde_defaults::{default_true, full_opacity};
use crate::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
    #[serde(default = "default_true")]
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
    #[serde(default = "full_opacity")]
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

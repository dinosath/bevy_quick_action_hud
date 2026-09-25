//! Authored configuration for a HUD page and its entries.

use serde::{Deserialize, Serialize};

use crate::serde_defaults::{default_true, full_opacity};
use crate::{HudSwitch, QuickAction, QuickActionConfig, RadialMenuSet};

/// Common editor contract for components that can appear on a HUD page.
pub trait HudComponent {
    /// Returns the display name used by editor lists.
    fn hud_name(&self) -> &str;
    /// Returns whether the component should be rendered.
    fn hud_enabled(&self) -> bool;
}

impl HudComponent for RadialMenuSet {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        true
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
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "full_opacity")]
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
    #[serde(default = "full_opacity")]
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

impl ActionSet {
    /// Returns the `nth` radial-menu set (clamped to the last one) and its entry index.
    pub fn radial_menu_set(&self, nth: usize) -> Option<(usize, &RadialMenuSet)> {
        let sets = || {
            self.entries
                .iter()
                .enumerate()
                .filter_map(|(entry, e)| match e {
                    SetEntry::RadialMenuSet(set) => Some((entry, set)),
                    _ => None,
                })
        };
        let count = sets().count();
        sets().nth(nth.min(count.checked_sub(1)?))
    }

    /// Enabled buttons with their entry indices.
    pub fn buttons(&self) -> impl Iterator<Item = (usize, &QuickAction)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                SetEntry::Action(a) if a.enabled => Some((i, a)),
                _ => None,
            })
    }

    /// Enabled page switches with their entry indices.
    pub fn page_switches(&self) -> impl Iterator<Item = (usize, &HudSwitch)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                SetEntry::HudSwitch(s) if s.enabled => Some((i, s)),
                _ => None,
            })
    }
}

/// Canonical page type for the authored HUD document.
pub type HudPage = ActionSet;

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

//! Configuration validation for authored HUD pages and components.

use crate::*;
use bevy::prelude::*;

/// Warnings about the current `QuickActionConfig`.
#[derive(Resource, Clone, Debug, Default)]
pub struct ConfigValidation {
    /// Human-readable warnings found in the authored configuration.
    pub warnings: Vec<String>,
    /// Whether validation found a blocking error.
    pub has_errors: bool,
}

/// Validates the current config and stores warnings in [`ConfigValidation`].
/// Runs every time the HUD is rebuilt (i.e. after each editor action).
pub(super) fn validate_config(
    cfg: Res<QuickActionConfig>,
    mut validation: ResMut<ConfigValidation>,
) {
    let mut warnings: Vec<String> = Vec::new();

    // Check for empty sets
    for (i, set) in cfg.sets.iter().enumerate() {
        if set.entries.is_empty() {
            warnings.push(format!("Set \"{}\" (#{}) has no entries", set.name, i));
        }
        // Check radial-menu-set and sector cardinality.
        for entry in set.entries.iter() {
            match entry {
                SetEntry::RadialMenuSet(ws) => {
                    if ws.radial_menus.is_empty() {
                        warnings.push(format!(
                            "Radial menu set \"{}\" in page \"{}\" has no radial menus",
                            ws.name, set.name
                        ));
                    }
                    for (wi, w) in ws.radial_menus.iter().enumerate() {
                        if w.slots.len() < 2 {
                            warnings.push(format!(
                                "Radial menu \"{}\" (#{}) in radial menu set \"{}\" must have at least 2 sectors",
                                w.name, wi, ws.name
                            ));
                        }
                    }
                }
                SetEntry::HudSwitch(hs) => {
                    if hs.name.trim().is_empty() {
                        warnings.push(format!(
                            "HUD switch #{i} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Every component shortcut must be unique within its HUD page; identical
    // shortcuts on different pages are allowed.
    for (si, page) in cfg.sets.iter().enumerate() {
        let mut seen_keys: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut record_key = |key: &str| {
            if !key.is_empty() {
                *seen_keys.entry(key.to_owned()).or_default() += 1;
            }
        };
        for entry in &page.entries {
            match entry {
                SetEntry::Action(button) => record_key(&button.key),
                SetEntry::HudSwitch(switch) => record_key(&switch.key),
                SetEntry::RadialMenuSet(wheel_set) => {
                    record_key(&wheel_set.prev_wheel_key);
                    record_key(&wheel_set.next_wheel_key);
                    record_key(&wheel_set.switch_key);
                }
            }
        }
        for (key, count) in seen_keys {
            if count > 1 {
                warnings.push(format!(
                    "Page \"{}\" (#{si}) reuses shortcut \"{key}\" {count} times",
                    page.name
                ));
            }
        }
    }

    if cfg.sets.is_empty() {
        warnings.push("HUD must contain at least one page".into());
    }

    // Check for empty action names
    for set in cfg.sets.iter() {
        for (ei, entry) in set.entries.iter().enumerate() {
            match entry {
                SetEntry::Action(qa) => {
                    if qa.name.trim().is_empty() {
                        warnings.push(format!(
                            "Action #{ei} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
                SetEntry::RadialMenuSet(ws) => {
                    if ws.name.trim().is_empty() {
                        warnings.push(format!(
                            "Radial menu set #{ei} in page \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
                SetEntry::HudSwitch(hs) => {
                    if hs.name.trim().is_empty() {
                        warnings.push(format!(
                            "HUD switch #{ei} in set \"{}\" has an empty name",
                            set.name
                        ));
                    }
                }
            }
        }
    }

    validation.warnings = warnings;
    validation.has_errors = !validation.warnings.is_empty();
}

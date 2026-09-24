//! Editor configuration persistence.

use crate::QuickActionConfig;
use bevy::prelude::*;

pub(super) fn save_config(cfg: &QuickActionConfig, path: &str) {
    match ron::ser::to_string_pretty(cfg, ron::ser::PrettyConfig::default()) {
        Ok(serialized) => {
            if let Err(error) = std::fs::write(path, serialized) {
                error!("[editor] write failed ({path}): {error}");
            } else {
                info!("[editor] saved to {path}");
            }
        }
        Err(error) => error!("[editor] serialize failed: {error}"),
    }
}

pub(super) fn load_config(path: &str) -> Option<QuickActionConfig> {
    let serialized = match std::fs::read_to_string(path) {
        Ok(serialized) => serialized,
        Err(error) => {
            error!("[editor] read failed ({path}): {error}");
            return None;
        }
    };
    match ron::from_str(&serialized) {
        Ok(mut config) => {
            crate::normalize_wheelset_config(&mut config);
            info!("[editor] loaded from {path}");
            Some(config)
        }
        Err(error) => {
            error!("[editor] parse failed ({path}): {error}");
            None
        }
    }
}

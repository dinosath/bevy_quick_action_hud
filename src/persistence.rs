//! Persistence adapters for authored HUD configuration.
//!
//! Runtime ECS state is intentionally not serialized here. This module owns
//! the current RON autoload adapter and is a seam for a future async/platform
//! persistence plugin.

use super::*;

/// Loads the optional user configuration during `PostStartup`.
pub(crate) fn try_autoload_config(mut cfg: ResMut<QuickActionConfig>) {
    match std::fs::read_to_string(CONFIG_FILE) {
        Err(_) => {}
        Ok(s) => match ron::from_str::<QuickActionConfig>(&s) {
            Ok(mut loaded) => {
                normalize_wheelset_config(&mut loaded);
                *cfg = loaded;
                info!("[wheel_menu] config auto-loaded from {CONFIG_FILE}");
            }
            Err(e) => warn!("[wheel_menu] failed to parse {CONFIG_FILE}: {e}"),
        },
    }
}

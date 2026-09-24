//! Editor operations that only concern shared radial-menu-set visuals.

use super::config::{RadialMenuSet, RadialMenuSetVisuals};

/// Copies one wheel's shared presentation settings to every wheel in its set.
pub(crate) fn sync_visuals_from_wheel(set: &mut RadialMenuSet, wheel_index: usize) {
    let Some(source) = set.radial_menus.get(wheel_index).cloned() else {
        return;
    };
    let visuals = RadialMenuSetVisuals::from(&source);
    set.visuals = Some(visuals.clone());
    set.stick_binding = source.stick_binding;
    for wheel in &mut set.radial_menus {
        visuals.apply_to(wheel);
        wheel.stick_binding = set.stick_binding.clone();
    }
}

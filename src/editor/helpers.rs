//! Shared editor data accessors and action classification helpers.

use super::Selection;
use crate::*;

/// Fixes a one-frame flash caused by feathers initialising every `FeathersButton` with
/// `ThemeBackgroundColor(BUTTON_BG)` (opaque gray) regardless of variant.  The `update_button_styles`
/// system in feathers' `PreUpdate` only corrects this in the *next* frame, so Plain buttons
/// (which should be transparent at rest) flash gray for one rendered frame after a sidebar rebuild.
///
/// Running in PostUpdate of the *same* frame as the spawn, before the render, we override
pub(super) fn action_at(
    cfg: &mut QuickActionConfig,
    set: usize,
    entry: usize,
) -> Option<&mut QuickAction> {
    match cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry)) {
        Some(SetEntry::Action(a)) => Some(a),
        _ => None,
    }
}

pub(super) fn wheel_at(cfg: &mut QuickActionConfig, sel: Selection) -> Option<&mut RadialMenu> {
    let (set, entry, wheel) = match sel {
        Selection::Wheel { set, entry, wheel } => (set, entry, wheel),
        Selection::Segment {
            set, entry, wheel, ..
        } => (set, entry, wheel),
        _ => return None,
    };
    match cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry)) {
        Some(SetEntry::RadialMenuSet(ws)) => wheel.and_then(move |i| ws.radial_menus.get_mut(i)),
        _ => None,
    }
}

pub(super) fn sync_wheelset_visuals(cfg: &mut QuickActionConfig, selection: Selection) {
    let Selection::Wheel {
        set,
        entry,
        wheel: Some(index),
    } = selection
    else {
        return;
    };
    let Some(SetEntry::RadialMenuSet(ws)) =
        cfg.sets.get_mut(set).and_then(|s| s.entries.get_mut(entry))
    else {
        return;
    };
    crate::radial_menu_set::editor::sync_visuals_from_wheel(ws, index);
}

/// Returns a minimum button size appropriate for the active input surface.
pub fn touch_safe_button_size(configured_size: f32, is_touch: bool) -> f32 {
    if is_touch {
        configured_size.max(44.0)
    } else {
        configured_size
    }
}

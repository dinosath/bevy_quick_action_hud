//! Thumbstick navigation of the active radial menu while the HUD is open.

use bevy::prelude::*;

use crate::radial_menu::geometry::sector_index_at_angle;
use crate::{HudSegmentSelected, QuickActionConfig, SetEntry, WheelHudState};

/// Highlights the sector under the thumbstick.
///
/// Release-to-use: returning the stick to the dead zone emits
/// [`HudSegmentSelected`] for the previously highlighted sector.
pub(crate) fn hud_stick_nav(
    gamepads: Query<&Gamepad>,
    mut hud: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    mut selected: MessageWriter<HudSegmentSelected>,
) {
    if !hud.open || hud.editor_open {
        return;
    }
    let Some((entry, menu_set)) = cfg
        .sets
        .get(hud.active_set)
        .and_then(|page| page.radial_menu_set(hud.active_wheel_entry))
    else {
        return;
    };
    let wheel = hud
        .active_wheel_index
        .min(menu_set.radial_menu_count().saturating_sub(1));
    let Some(menu) = menu_set.radial_menu(wheel) else {
        return;
    };

    let stick = thumbstick_axes(&menu_set.stick_binding)
        .zip(gamepads.iter().next())
        .map(|((x, y), pad)| Vec2::new(pad.get(x).unwrap_or(0.), pad.get(y).unwrap_or(0.)))
        .unwrap_or(Vec2::ZERO);
    let previous = hud.highlighted;
    let current = if stick.length() < menu.deadzone {
        None
    } else {
        // The menu is rotated clockwise on screen, so undo that rotation in input space.
        let local_angle = stick.y.atan2(stick.x) - menu.rotation.to_radians();
        sector_index_at_angle(menu, local_angle)
            .map(|slot| (hud.active_set, entry, Some(wheel), slot))
    };
    if previous == current {
        return;
    }
    debug!("[hud] stick highlight changed: {previous:?} -> {current:?}");

    if let (Some((set, entry, wheel, slot)), None) = (previous, current) {
        let close_on_select = cfg
            .sets
            .get(set)
            .and_then(|page| page.entries.get(entry))
            .and_then(|entry| match (entry, wheel) {
                (SetEntry::RadialMenuSet(ws), Some(wi)) => ws.radial_menu(wi)?.slots.get(slot),
                _ => None,
            })
            .is_some_and(|sector| sector.close_on_select);
        selected.write(HudSegmentSelected {
            set,
            entry,
            wheel,
            slot,
        });
        if close_on_select {
            hud.open = false;
        }
    }
    hud.highlighted = current;
    hud.dirty = true;
}

fn thumbstick_axes(binding: &str) -> Option<(GamepadAxis, GamepadAxis)> {
    match binding {
        "GP:LeftStick" | "LeftStick" => Some((GamepadAxis::LeftStickX, GamepadAxis::LeftStickY)),
        "GP:RightStick" | "RightStick" | "" => {
            Some((GamepadAxis::RightStickX, GamepadAxis::RightStickY))
        }
        _ => None,
    }
}

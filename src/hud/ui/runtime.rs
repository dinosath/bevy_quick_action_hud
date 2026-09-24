use crate::*;
use bevy::log::debug;
use bevy::prelude::*;

use super::canvas::build_hud_canvas;

/// Plugin that registers the retained HUD canvas and runtime navigation.
pub struct WheelHudPlugin;
impl Plugin for WheelHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickActionHudPlugin::default());
    }
}

/// Updates [`WheelHudState::highlighted`] while the HUD wheel is open.
///
/// Uses **release-to-use**: the slot that was highlighted when the stick
/// returns to the dead-zone is emitted as a [`HudSegmentSelected`] event.
/// The thumbstick binding is read from the active wheel entry.
pub(crate) fn hud_stick_nav(
    gamepads: Query<&Gamepad>,
    mut hud: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    mut select_ev: MessageWriter<HudSegmentSelected>,
) {
    if !hud.open || hud.editor_open {
        return;
    }

    // Locate the active wheel entry (honoring active_wheel_entry).
    let Some(set) = cfg.sets.get(hud.active_set) else {
        return;
    };
    let mut found: Option<(usize, Option<usize>, usize, String)> = None;
    let target = hud.active_wheel_entry;
    let mut wcount = 0usize;
    for (ei, entry) in set.entries.iter().enumerate() {
        let is_wheel = matches!(entry, SetEntry::Wheel(_) | SetEntry::RadialMenuSet(_));
        if !is_wheel {
            continue;
        }
        if wcount != target {
            wcount += 1;
            continue;
        }
        match entry {
            SetEntry::Wheel(w) => {
                found = Some((ei, None, w.slots.len(), w.stick_binding.clone()));
            }
            SetEntry::RadialMenuSet(ws) => {
                let wheel_index = hud
                    .active_wheel_index
                    .min(ws.wheels.len().saturating_sub(1));
                if let Some(w) = ws.wheels.get(wheel_index) {
                    found = Some((
                        ei,
                        Some(wheel_index),
                        w.slots.len(),
                        ws.stick_binding.clone(),
                    ));
                }
            }
            _ => {}
        }
        break;
    }
    let Some((entry_idx, wheel_idx, n_slots, stick_binding)) = found else {
        return;
    };
    if n_slots == 0 {
        return;
    }

    // Read only the axes represented by the configured thumbstick binding.
    let mut stick = Vec2::ZERO;
    if let Some((xa, ya)) = thumbstick_axes(&stick_binding) {
        if let Some(gamepad) = gamepads.iter().next() {
            stick = Vec2::new(
                gamepad.get(xa).unwrap_or(0.0),
                gamepad.get(ya).unwrap_or(0.0),
            );
        }
    }

    const DEADZONE: f32 = 0.2;
    let prev = hud.highlighted;

    let new_highlight = if stick.length() < DEADZONE {
        if hud.editor_open {
            prev
        } else {
            None
        }
    } else {
        // Same angle mapping as RadialMenu::arc_offset default (FRAC_PI_6).
        let a = stick.y.atan2(stick.x);
        let rel = (a - std::f32::consts::FRAC_PI_6).rem_euclid(std::f32::consts::TAU);
        let idx = ((rel / std::f32::consts::TAU) * n_slots as f32).floor() as usize;
        Some((hud.active_set, entry_idx, wheel_idx, idx.min(n_slots - 1)))
    };

    if prev != new_highlight {
        debug!(
            "[hud] stick highlight changed: {:?} -> {:?} editor_open={}",
            prev, new_highlight, hud.editor_open
        );
        // Release-to-use: emit selection when stick returns to dead-zone.
        if let (Some((s, e, w, slot)), None) = (prev, new_highlight) {
            if !hud.editor_open {
                // Normal mode: fire selection and optionally close.
                let slot_close = cfg
                    .sets
                    .get(s)
                    .and_then(|set| set.entries.get(e))
                    .and_then(|entry| match (entry, w) {
                        (SetEntry::Wheel(wd), None) => wd.slots.get(slot),
                        (SetEntry::RadialMenuSet(ws), Some(wi)) => {
                            ws.wheels.get(wi).and_then(|wd| wd.slots.get(slot))
                        }
                        _ => None,
                    })
                    .map(|s| s.close_on_select)
                    .unwrap_or(false);

                select_ev.write(HudSegmentSelected {
                    set: s,
                    entry: e,
                    wheel: w,
                    slot,
                });

                if slot_close {
                    hud.open = false;
                }
            }
            // Dry-run mode: highlight clears visually — no event, no close.
        }
        hud.highlighted = new_highlight;
        if hud.editor_open && new_highlight.is_some() {
            hud.mouse_hovered_segment = None;
            hud.edit_control_focus = Some(0);
        }
        hud.dirty = true;
    }
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

/// Counts down the dry-run flash timer.  When expired it clears the flash entry and
/// triggers a HUD rebuild so the button returns to its normal colour.
pub(crate) fn rebuild_hud(
    mut commands: Commands,
    mut hud: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    asset_server: Res<AssetServer>,
    icon_set: Res<GamepadIconSet>,
    old_hud: Query<Entity, With<WheelHudRoot>>,
    children: Query<&Children>,
    mut wedge_materials: ResMut<Assets<WedgeMaterial>>,
) {
    if !hud.dirty {
        return;
    }
    debug!("[hud] rebuild requested: open={} editor_open={} active_set={} selected_action={:?} hovered_action={:?} selected_wheel={:?} hovered_wheel={:?}",
        hud.open, hud.editor_open, hud.active_set, hud.selected_action, hud.hovered_action,
        hud.selected_wheel, hud.hovered_wheel);
    hud.dirty = false;
    if cfg
        .sets
        .get(hud.active_set)
        .is_none_or(|page| !page.enabled)
    {
        if let Some(page) = enabled_hud_pages(&cfg).first().copied() {
            hud.active_set = page;
        }
    }

    debug!(
        "[hud] rebuild_hud — open={} editor_open={} active_set={} active_wheel_entry={}",
        hud.open, hud.editor_open, hud.active_set, hud.active_wheel_entry
    );

    for e in &old_hud {
        debug!("[hud] recursively despawning HUD root {:?}", e);
        despawn_hud_tree(&mut commands, e, &children);
        commands.entity(e).despawn();
    }

    if !cfg.sets.is_empty() && hud.active_set >= cfg.sets.len() {
        hud.active_set = cfg.sets.len() - 1;
    }

    build_hud_canvas(
        &mut commands,
        &cfg,
        &hud,
        &asset_server,
        *icon_set,
        &mut wedge_materials,
    );
}

fn despawn_hud_tree(commands: &mut Commands, entity: Entity, children: &Query<&Children>) {
    if let Ok(kids) = children.get(entity) {
        for child in kids.iter() {
            despawn_hud_tree(commands, child, children);
            commands.entity(child).despawn();
        }
    }
}

//! Core radial-menu runtime systems.

use crate::*;
use bevy::prelude::*;

use super::geometry::sector_index_at_angle;

/// Determines which slice is hovered and handles per-mode activation:
/// - [`CastingMode::ReleaseToUse`]: fires [`WheelMenuSelected`] when the stick
///   returns to centre.
/// - [`CastingMode::Direct`]: fires [`WheelMenuSelected`] immediately on hover.
pub fn update_wheel_hover(
    mut q: Query<(
        Entity,
        &RadialMenu,
        &mut RadialMenuState,
        Option<&RadialMenuConfig>,
    )>,
    mut hover_ev: MessageWriter<WheelMenuHoverChanged>,
    mut select_ev: MessageWriter<WheelMenuSelected>,
) {
    for (entity, menu, mut state, config) in &mut q {
        let previous = state.hovered;

        if state.dir.length() < menu.deadzone {
            state.hovered = None;
        } else {
            state.hovered = sector_index_at_angle(menu, state.dir.y.atan2(state.dir.x));
        }

        if previous != state.hovered {
            hover_ev.write(WheelMenuHoverChanged {
                previous,
                current: state.hovered,
                menu_entity: entity,
            });

            if let Some(cfg) = config {
                match &cfg.casting_mode {
                    CastingMode::ReleaseToUse => {
                        // Fire when stick returns to centre after hovering.
                        if let (Some(prev_idx), None) = (previous, state.hovered) {
                            select_ev.write(WheelMenuSelected {
                                index: prev_idx,
                                menu_entity: entity,
                            });
                        }
                    }
                    CastingMode::Direct => {
                        // Fire immediately when a new slice is entered.
                        if let Some(idx) = state.hovered {
                            select_ev.write(WheelMenuSelected {
                                index: idx,
                                menu_entity: entity,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Emits [`WheelMenuSelected`] on confirm-button press for [`CastingMode::Vanilla`].
///
/// All other casting modes handle their own activation logic.
pub fn emit_selection(
    gamepads: Query<&Gamepad>,
    q: Query<(Entity, &RadialMenuState, Option<&RadialMenuConfig>), With<RadialMenu>>,
    mut ev: MessageWriter<WheelMenuSelected>,
    hud: Option<Res<WheelHudState>>,
) {
    // `WheelMenuPlugin` is also valid without the HUD canvas, so the HUD
    // state resource is not guaranteed to exist in core-only applications.
    if hud.as_ref().is_some_and(|state| state.editor_open) {
        return;
    }
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            for (entity, state, config) in &q {
                if let Some(cfg) = config {
                    match cfg.casting_mode {
                        CastingMode::Vanilla => {} // fall through
                        _ => continue,             // another mode is active
                    }
                }
                if let Some(i) = state.hovered {
                    ev.write(WheelMenuSelected {
                        index: i,
                        menu_entity: entity,
                    });
                }
            }
        }
    }
}

// ─── additional systems ───────────────────────────────────────────────────────

/// Emits [`WheelOpened`] the first frame a wheel gains a hovered slice, and
/// [`WheelClosed`] the first frame it loses one.  Runs after [`update_wheel_hover`].
pub fn emit_lifecycle(
    mut q: Query<(Entity, &mut RadialMenuState), With<RadialMenu>>,
    mut opened_ev: MessageWriter<WheelOpened>,
    mut closed_ev: MessageWriter<WheelClosed>,
) {
    for (entity, mut state) in &mut q {
        let is_open = state.hovered.is_some();
        if is_open && !state.open {
            state.open = true;
            opened_ev.write(WheelOpened {
                menu_entity: entity,
            });
        } else if !is_open && state.open {
            state.open = false;
            closed_ev.write(WheelClosed {
                menu_entity: entity,
            });
        }
    }
}

/// Tracks dwell time on a hovered slice for [`CastingMode::HoldToActivate`].
/// Emits [`WheelMenuHoldProgress`] each frame and [`WheelMenuHoldActivated`]
/// when `duration` is reached.
pub fn update_wheel_hold(
    time: Res<Time>,
    mut q: Query<(
        Entity,
        &RadialMenuConfig,
        &RadialMenuState,
        &mut RadialMenuHoldState,
    )>,
    mut progress_ev: MessageWriter<WheelMenuHoldProgress>,
    mut activate_ev: MessageWriter<WheelMenuHoldActivated>,
) {
    for (entity, config, state, mut hold) in &mut q {
        let duration = match config.casting_mode {
            CastingMode::HoldToActivate { duration } => duration,
            _ => {
                hold.progress = 0.0;
                hold.holding = false;
                continue;
            }
        };
        match state.hovered {
            Some(index) => {
                hold.holding = true;
                hold.progress = (hold.progress + time.delta_secs() / duration).clamp(0.0, 1.0);
                progress_ev.write(WheelMenuHoldProgress {
                    index,
                    progress: hold.progress,
                    menu_entity: entity,
                });
                if hold.progress >= 1.0 {
                    activate_ev.write(WheelMenuHoldActivated {
                        index,
                        menu_entity: entity,
                    });
                    hold.progress = 0.0;
                }
            }
            None => {
                hold.holding = false;
                hold.progress = 0.0;
            }
        }
    }
}

/// Emits [`WheelMenuLowCount`] once each time a [`SectorCount`] transitions
/// from above to at-or-below its threshold.  The flag resets when the count
/// rises above the threshold again.
pub fn check_low_counts(
    mut q: Query<(Entity, &SectorEntity, &mut SectorCount)>,
    mut ev: MessageWriter<WheelMenuLowCount>,
) {
    for (entity, slice, mut count) in &mut q {
        let is_low = count.max > 0 && count.current <= count.low_threshold;
        if is_low && !count.low_notified {
            count.low_notified = true;
            ev.write(WheelMenuLowCount {
                index: slice.index,
                current: count.current,
                threshold: count.low_threshold,
                slice_entity: entity,
            });
        } else if !is_low {
            count.low_notified = false;
        }
    }
}

/// Toggles edit mode when the configured button is pressed, and emits
/// [`WheelSliceReorder`] events when D-pad Up/Down is pressed while hovering a
/// slice in edit mode.
pub fn update_edit_mode(
    gamepads: Query<&Gamepad>,
    mut q: Query<(
        Entity,
        &RadialMenu,
        &RadialMenuState,
        &mut RadialMenuEditMode,
    )>,
    mut mode_ev: MessageWriter<WheelEditModeChanged>,
    mut reorder_ev: MessageWriter<WheelSliceReorder>,
    hud: Option<Res<WheelHudState>>,
) {
    // `WheelMenuPlugin` is also valid without the HUD canvas, so the HUD
    // state resource is not guaranteed to exist in core-only applications.
    if hud.as_ref().is_some_and(|state| state.editor_open) {
        return;
    }
    for (entity, menu, state, mut edit) in &mut q {
        for gamepad in &gamepads {
            if let Some(btn) = edit.toggle_button {
                if gamepad.just_pressed(btn) {
                    edit.active = !edit.active;
                    mode_ev.write(WheelEditModeChanged {
                        active: edit.active,
                        menu_entity: entity,
                    });
                }
            }
            if edit.active {
                if let Some(hovered) = state.hovered {
                    if gamepad.just_pressed(GamepadButton::DPadUp) && hovered > 0 {
                        reorder_ev.write(WheelSliceReorder {
                            from_index: hovered,
                            to_index: hovered - 1,
                            menu_entity: entity,
                        });
                    }
                    if gamepad.just_pressed(GamepadButton::DPadDown)
                        && hovered + 1 < menu.slots.len().max(1)
                    {
                        reorder_ev.write(WheelSliceReorder {
                            from_index: hovered,
                            to_index: hovered + 1,
                            menu_entity: entity,
                        });
                    }
                }
            }
        }
    }
}

/// Maintains [`ActiveSlotContext`] on each wheel from its currently hovered
/// slice.  Only slices carrying a [`WheelSliceLink`] participate, so the link
/// back to the owning wheel is explicit and query scans stay cheap.
pub fn update_active_slot_context(
    mut commands: Commands,
    wheel_q: Query<(Entity, &RadialMenuState)>,
    slice_q: Query<(Entity, &SectorEntity, &WheelSliceLink)>,
) {
    for (menu, state) in &wheel_q {
        let mut found: Option<Entity> = None;
        if let Some(hovered) = state.hovered {
            for (slice_entity, slice, link) in &slice_q {
                if link.menu == menu && slice.index == hovered {
                    found = Some(slice_entity);
                    break;
                }
            }
        }
        match found {
            Some(slot_entity) => {
                commands
                    .entity(menu)
                    .insert(ActiveSlotContext { slot_entity });
            }
            None => {
                commands.entity(menu).remove::<ActiveSlotContext>();
            }
        }
    }
}

/// Reads gamepad face/thumb buttons, maps them onto [`InputAction`]s, resolves
/// each against the slot → wheel → global override chain, and emits
/// [`WheelActionResolved`].  This implements the contextual input-override
/// system: a hovered slot's bindings take priority over the wheel's, which take
/// priority over [`GlobalBindings`].
#[allow(clippy::type_complexity)]
pub fn resolve_wheel_input(
    gamepads: Query<&Gamepad>,
    global: Res<GlobalBindings>,
    wheel_q: Query<
        (
            Entity,
            Option<&WheelInputOverride>,
            Option<&ActiveSlotContext>,
        ),
        With<RadialMenuState>,
    >,
    slot_q: Query<&WheelInputOverride, Without<RadialMenuState>>,
    mut ev: MessageWriter<WheelActionResolved>,
) {
    for (menu, wheel_override, active_slot) in &wheel_q {
        let slot_override = active_slot.and_then(|ctx| slot_q.get(ctx.slot_entity).ok());
        for gamepad in &gamepads {
            for (button, input) in radial_menu::DEFAULT_BUTTON_MAP {
                if gamepad.just_pressed(*button) {
                    if let Some(action) =
                        resolve_input(*input, slot_override, wheel_override, &global)
                    {
                        ev.write(WheelActionResolved {
                            input: *input,
                            action,
                            menu_entity: menu,
                        });
                    }
                }
            }
        }
    }
}

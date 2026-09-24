//! HUD button and sector interaction systems.

use super::*;

pub(super) fn process_hud_buttons(
    buttons: Query<(&WheelHudButton, &Interaction), Changed<Interaction>>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
    mut qcfg: ResMut<QuickActionConfig>,
) {
    for (btn, interaction) in &buttons {
        debug!(
            "[ui] hud button interaction: {:?} -> {:?}",
            btn.action, interaction
        );
        if let Some(owner) = crate::hud_control_owner(&btn.action) {
            match interaction {
                Interaction::Hovered => set_hover_owner(&mut hud, owner),
                Interaction::None => clear_hover_owner(&mut hud, owner),
                Interaction::Pressed => {}
            }
        }
        match (&btn.action, interaction) {
            (WheelHudAction::SelectAction { set, entry }, Interaction::Hovered) => {
                hud.hovered_action = Some((*set, *entry));
            }
            (WheelHudAction::SelectAction { set, entry }, Interaction::None)
                if hud.hovered_action == Some((*set, *entry)) =>
            {
                hud.hovered_action = None;
            }
            (WheelHudAction::SelectWheel { set, entry, wheel }, Interaction::Hovered) => {
                hud.hovered_wheel = Some((*set, *entry, *wheel));
            }
            (WheelHudAction::SelectWheel { set, entry, wheel }, Interaction::None)
                if hud.hovered_wheel == Some((*set, *entry, *wheel)) =>
            {
                hud.hovered_wheel = None;
            }
            (WheelHudAction::SelectHudSwitch { set, entry }, Interaction::Hovered) => {
                hud.hovered_hud_switch = Some((*set, *entry));
            }
            (WheelHudAction::SelectHudSwitch { set, entry }, Interaction::None)
                if hud.hovered_hud_switch == Some((*set, *entry)) =>
            {
                hud.hovered_hud_switch = None;
            }
            _ => {}
        }
        if *interaction == Interaction::Pressed {
            debug!("[ui] hud button pressed: {:?}", btn.action);
            match &btn.action {
                WheelHudAction::SetActiveSet(i) => {
                    hud.active_set = *i;
                    hud.active_wheel_entry = 0;
                    hud.active_wheel_index = 0;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::PrevSet => {
                    if hud.active_set > 0 {
                        hud.active_set -= 1;
                    } else if qcfg.cycle_sets && !qcfg.sets.is_empty() {
                        hud.active_set = qcfg.sets.len() - 1;
                    }
                    hud.active_wheel_entry = 0;
                    hud.active_wheel_index = 0;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::NextSet => {
                    let max = qcfg.sets.len().saturating_sub(1);
                    if hud.active_set < max {
                        hud.active_set += 1;
                    } else if qcfg.cycle_sets {
                        hud.active_set = 0;
                    }
                    hud.active_wheel_entry = 0;
                    hud.active_wheel_index = 0;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleEditor => {
                    hud.editor_open = !hud.editor_open;
                    hud.edit_control_focus = if hud.editor_open { Some(0) } else { None };
                    info!(
                        "[editor] ToggleEditor button — editor_open now={}",
                        hud.editor_open
                    );
                    if !hud.editor_open {
                        ui.selection = Selection::None;
                        ui.editing = EditFocus::None;
                        hud.settings_open = false;
                        hud.selected_action = None;
                        hud.selected_wheel = None;
                        hud.selected_hud_switch = None;
                        hud.selected_segment = None;
                        hud.hovered_action = None;
                        hud.hovered_wheel = None;
                        hud.hovered_hud_switch = None;
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::AddSegment {
                    set,
                    entry,
                    wheel,
                    side,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        let slot = match side {
                            SegmentInsertSide::Before => {
                                let insert_at = match ui.selection {
                                    Selection::Segment { slot, .. } => slot,
                                    _ => w.slots.len(),
                                }
                                .min(w.slots.len());
                                w.slots.insert(
                                    insert_at,
                                    Sector::named(format!("Slot {}", insert_at + 1)),
                                );
                                insert_at
                            }
                            SegmentInsertSide::After | SegmentInsertSide::Outer => {
                                let insert_at = match ui.selection {
                                    Selection::Segment { slot, .. } => slot.saturating_add(1),
                                    _ => w.slots.len(),
                                }
                                .min(w.slots.len());
                                w.slots.insert(
                                    insert_at,
                                    Sector::named(format!("Slot {}", insert_at + 1)),
                                );
                                insert_at
                            }
                        };
                        ui.selection = Selection::Segment {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                            slot,
                        };
                        hud.selected_segment = Some((*set, *entry, *wheel, slot));
                        hud.highlighted = Some((*set, *entry, *wheel, slot));
                        hud.dirty = true;
                        ui.dirty = true;
                    }
                }
                WheelHudAction::RemoveSegment {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if w.slots.len() > MIN_SECTORS && *slot < w.slots.len() {
                            w.slots.remove(*slot);
                            let next_slot = (*slot).min(w.slots.len() - 1);
                            ui.selection = Selection::Segment {
                                set: *set,
                                entry: *entry,
                                wheel: *wheel,
                                slot: next_slot,
                            };
                            hud.selected_segment = Some((*set, *entry, *wheel, next_slot));
                            hud.highlighted = Some((*set, *entry, *wheel, next_slot));
                            hud.dirty = true;
                            ui.dirty = true;
                        }
                    }
                }
                WheelHudAction::EditSegmentName {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    ui.selection = Selection::Segment {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                        slot: *slot,
                    };
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    hud.selected_segment = Some((*set, *entry, *wheel, *slot));
                    ui.editing = EditFocus::SlotName(*slot);
                    hud.highlighted = Some((*set, *entry, *wheel, *slot));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::EditSegmentIcon {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    ui.selection = Selection::Segment {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                        slot: *slot,
                    };
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    hud.selected_segment = Some((*set, *entry, *wheel, *slot));
                    ui.editing = EditFocus::SlotIcon(*slot);
                    hud.highlighted = Some((*set, *entry, *wheel, *slot));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleSegmentMapping {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    const COMMANDS: &[&str] = &[
                        "none", "use", "equip", "attack", "interact", "dash", "jump", "crouch",
                    ];
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.command = cycle_palette(COMMANDS, &s.command).into();
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleSegmentHold {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.hold = !s.hold;
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleSegmentHoldAction {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    const COMMANDS: &[&str] = &[
                        "none", "use", "equip", "attack", "interact", "dash", "jump", "crouch",
                    ];
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.hold_command = cycle_palette(COMMANDS, &s.hold_command).into();
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleSegmentCloseOnApply {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if let Some(s) = w.slots.get_mut(*slot) {
                            s.close_on_select = !s.close_on_select;
                        }
                    }
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteSegment {
                    set,
                    entry,
                    wheel,
                    slot,
                } => {
                    if let Some(w) = wheel_at(
                        &mut qcfg,
                        Selection::Wheel {
                            set: *set,
                            entry: *entry,
                            wheel: *wheel,
                        },
                    ) {
                        if w.slots.len() > MIN_SECTORS && *slot < w.slots.len() {
                            w.slots.remove(*slot);
                        }
                    }
                    ui.selection = Selection::None;
                    hud.selected_segment = None;
                    hud.highlighted = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::SaveConfig => {
                    save_config(&qcfg, &ui.config_path);
                }
                WheelHudAction::AddNewButton => {
                    let action = EditorAction::AddAction {
                        set: hud.active_set,
                    };
                    apply_action(&action, &mut qcfg, &mut ui, &mut hud);
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleSettings => {
                    hud.settings_open = !hud.settings_open;
                    if hud.settings_open {
                        hud.highlighted = None;
                        hud.selected_action = None;
                        hud.selected_wheel = None;
                        hud.selected_hud_switch = None;
                        hud.hovered_action = None;
                        hud.hovered_wheel = None;
                        hud.hovered_hud_switch = None;
                        ui.selection = Selection::None;
                        ui.editing = EditFocus::None;
                    }
                    hud.dirty = true;
                }
                WheelHudAction::SelectAction { set, entry } => {
                    hud.selected_action = Some((*set, *entry));
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    hud.selected_segment = None;
                    hud.highlighted = None;
                    ui.selection = Selection::Action {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::SelectHudSwitch { set, entry }
                | WheelHudAction::MoveHudSwitch { set, entry }
                | WheelHudAction::EditHudSwitch { set, entry } => {
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_segment = None;
                    hud.selected_hud_switch = Some((*set, *entry));
                    hud.highlighted = None;
                    ui.selection = Selection::HudSwitch {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteHudSwitch { set, entry } => {
                    apply_action(
                        &EditorAction::DeleteEntry {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_hud_switch = None;
                    hud.selected_segment = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ResizeHudSwitch { set, entry, delta } => {
                    if let Some(SetEntry::HudSwitch(hs)) = qcfg
                        .sets
                        .get_mut(*set)
                        .and_then(|s| s.entries.get_mut(*entry))
                    {
                        hs.width = (hs.width + *delta).clamp(40.0, 300.0);
                        hs.height = (hs.height + *delta * 0.25).clamp(20.0, 120.0);
                    }
                    ui.selection = Selection::HudSwitch {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteAction { set, entry } => {
                    apply_action(
                        &EditorAction::DeleteEntry {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_segment = None;
                    hud.highlighted = None;
                    hud.edit_control_focus = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::MoveAction { set, entry }
                | WheelHudAction::EditAction { set, entry } => {
                    hud.selected_action = Some((*set, *entry));
                    hud.selected_wheel = None;
                    hud.highlighted = None;
                    ui.selection = Selection::Action {
                        set: *set,
                        entry: *entry,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::EditActionName { set, entry } => {
                    apply_action(
                        &EditorAction::EditName {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = Some((*set, *entry));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CaptureActionKey { set, entry } => {
                    apply_action(
                        &EditorAction::CaptureKey {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = Some((*set, *entry));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleActionIcon { set, entry } => {
                    apply_action(
                        &EditorAction::CycleIcon {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleActionMapping { set, entry } => {
                    apply_action(
                        &EditorAction::CycleCommand {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleActionHold { set, entry } => {
                    apply_action(
                        &EditorAction::ToggleHold {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleHoldAction { set, entry } => {
                    apply_action(
                        &EditorAction::CycleHoldCommand {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ToggleActionCloseOnApply { set, entry } => {
                    apply_action(
                        &EditorAction::ToggleActionCloseOnSelect {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::WheelSettings { set, entry, wheel }
                | WheelHudAction::MoveWheel { set, entry, wheel }
                | WheelHudAction::SelectWheel { set, entry, wheel } => {
                    hud.selected_wheel = Some((*set, *entry, *wheel));
                    hud.selected_action = None;
                    hud.selected_hud_switch = None;
                    hud.selected_segment = None;
                    hud.highlighted = None;
                    ui.selection = Selection::Wheel {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                    };
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ResizeWheel {
                    set,
                    entry,
                    wheel,
                    delta,
                } => {
                    ui.selection = Selection::Wheel {
                        set: *set,
                        entry: *entry,
                        wheel: *wheel,
                    };
                    apply_action(
                        &EditorAction::WheelOuterRadiusDelta { delta: *delta },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_wheel = Some((*set, *entry, *wheel));
                    hud.selected_segment = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::DeleteWheel { set, entry, wheel } => {
                    // A wheel set is one HUD component. Deleting its visible
                    // wheel removes the owning component, rather than only a
                    // sector wheel from the shared set.
                    let _ = wheel;
                    let editor_action = EditorAction::DeleteEntry {
                        set: *set,
                        entry: *entry,
                    };
                    apply_action(&editor_action, &mut qcfg, &mut ui, &mut hud);
                    hud.selected_wheel = None;
                    hud.selected_segment = None;
                    hud.highlighted = None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::RotateAction { set, entry, delta } => {
                    apply_action(
                        &EditorAction::ActionRotationDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.selected_action = Some((*set, *entry));
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ActionWidthDelta { set, entry, delta } => {
                    apply_action(
                        &EditorAction::ActionWidthDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ActionHeightDelta { set, entry, delta } => {
                    apply_action(
                        &EditorAction::ActionHeightDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::ActionRadiusDelta { set, entry, delta } => {
                    apply_action(
                        &EditorAction::RadiusDelta {
                            set: *set,
                            entry: *entry,
                            delta: *delta,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CycleActionPosition { set, entry } => {
                    apply_action(
                        &EditorAction::CyclePosition {
                            set: *set,
                            entry: *entry,
                        },
                        &mut qcfg,
                        &mut ui,
                        &mut hud,
                    );
                    hud.dirty = true;
                    ui.dirty = true;
                }
                WheelHudAction::CloseSelection => {
                    hud.highlighted = None;
                    hud.selected_action = None;
                    hud.selected_wheel = None;
                    hud.selected_segment = None;
                    hud.edit_control_focus = None;
                    ui.selection = Selection::None;
                    ui.editing = EditFocus::None;
                    hud.dirty = true;
                    ui.dirty = true;
                }
            }
        }
    }
}

fn set_hover_owner(hud: &mut WheelHudState, owner: HudControlOwner) {
    match owner {
        HudControlOwner::Action(set, entry) => hud.hovered_action = Some((set, entry)),
        HudControlOwner::Wheel(set, entry, wheel) => hud.hovered_wheel = Some((set, entry, wheel)),
        HudControlOwner::HudSwitch(set, entry) => hud.hovered_hud_switch = Some((set, entry)),
    }
}

fn clear_hover_owner(hud: &mut WheelHudState, owner: HudControlOwner) {
    match owner {
        HudControlOwner::Action(set, entry) if hud.hovered_action == Some((set, entry)) => {
            hud.hovered_action = None
        }
        HudControlOwner::Wheel(set, entry, wheel)
            if hud.hovered_wheel == Some((set, entry, wheel)) =>
        {
            hud.hovered_wheel = None
        }
        HudControlOwner::HudSwitch(set, entry) if hud.hovered_hud_switch == Some((set, entry)) => {
            hud.hovered_hud_switch = None
        }
        _ => {}
    }
}

pub(super) fn click_hud_segments(
    segments: Query<(&WheelHudSegmentHit, &Interaction), Changed<Interaction>>,
    settings_panels: Query<&Interaction, With<WheelSettingsPanel>>,
    mut hud: ResMut<WheelHudState>,
    mut ui: ResMut<EditorUiState>,
) {
    if !hud.editor_open {
        return;
    }
    // Settings is a modal interaction surface relative to the radial preview.
    // If it received this frame's pointer interaction, never let a sector
    // underneath clear the selected wheel or replace the settings card.
    if settings_panels
        .iter()
        .any(|interaction| matches!(interaction, Interaction::Hovered | Interaction::Pressed))
    {
        return;
    }
    for (segment, interaction) in &segments {
        debug!(
            "[ui] wheel segment interaction: set={} entry={} wheel={:?} slot={} -> {:?}",
            segment.set, segment.entry, segment.wheel, segment.slot, interaction
        );
        let id = (segment.set, segment.entry, segment.wheel, segment.slot);
        if *interaction == Interaction::Hovered {
            hud.mouse_hovered_segment = Some(id);
            hud.highlighted = Some(id);
            hud.hovered_wheel = Some((segment.set, segment.entry, segment.wheel));
            hud.selected_action = None;
            hud.selected_wheel = None;
            hud.edit_control_focus = Some(0);
        } else if *interaction == Interaction::None && hud.mouse_hovered_segment == Some(id) {
            hud.mouse_hovered_segment = None;
            if hud.hovered_wheel == Some((segment.set, segment.entry, segment.wheel)) {
                hud.hovered_wheel = None;
            }
            if !matches!(
                ui.selection,
                Selection::Segment {
                    set,
                    entry,
                    wheel,
                    slot
                } if (set, entry, wheel, slot) == id
            ) {
                hud.highlighted = None;
                hud.edit_control_focus = None;
            }
        } else if *interaction == Interaction::Pressed {
            hud.mouse_hovered_segment = None;
            hud.highlighted = Some(id);
            hud.selected_segment = Some(id);
            hud.selected_action = None;
            hud.selected_wheel = None;
            hud.edit_control_focus = Some(0);
            ui.selection = Selection::Segment {
                set: segment.set,
                entry: segment.entry,
                wheel: segment.wheel,
                slot: segment.slot,
            };
            hud.dirty = true;
            ui.dirty = true;
        }
    }
}

// ─── undo helpers ────────────────────────────────────────────────────────────────

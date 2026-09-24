use crate::*;
use bevy::prelude::*;

use super::primitives::*;

pub(crate) fn build_centered_wheel_hud(
    commands: &mut Commands,
    parent: Entity,
    wheel: &RadialMenu,
    set: usize,
    entry: usize,
    w_idx: Option<usize>,
    highlighted: Option<(usize, usize, Option<usize>, usize)>,
    selected_segment: Option<(usize, usize, Option<usize>, usize)>,
    selected_wheel: Option<(usize, usize, Option<usize>)>,
    hovered_wheel: Option<(usize, usize, Option<usize>)>,
    editor_open: bool,
    edit_control_focus: Option<usize>,
    theme_popup_open: bool,
    wedge_materials: &mut Assets<WedgeMaterial>,
) {
    let n_slices = wheel.slots.len().max(1);
    let hub = hud_child(commands, parent, wheel_hub());
    commands.entity(hub).insert(Node {
        position_type: PositionType::Relative,
        left: Val::Px(wheel.offset_x),
        top: Val::Px(-wheel.offset_y),
        ..default()
    });
    if wheel.rotation != 0.0 {
        commands
            .entity(hub)
            .insert(Transform::from_rotation(Quat::from_rotation_z(
                wheel.rotation.to_radians(),
            )));
    }

    let is_pie = wheel.segment_shape == SegmentShape::Pie;
    if !is_pie {
        let bg_col = if wheel.bg_color.is_empty() {
            Color::srgba(0.096, 0.118, 0.157, wheel.bg_opacity)
        } else {
            parse_hex_color(&wheel.bg_color, wheel.bg_opacity)
        };
        hud_child(commands, hub, wheel_bg_disc(wheel.outer_radius, bg_col));
    }
    let outer_col = if wheel.outer_border.is_empty() {
        Color::srgba(0.38, 0.39, 0.39, 0.90)
    } else {
        parse_hex_color(&wheel.outer_border, 1.0)
    };
    let outer_bw = if wheel.outer_border.is_empty() {
        1.0_f32
    } else {
        wheel.outer_border_width.max(0.0)
    };
    hud_child(
        commands,
        hub,
        wheel_outer_ring(wheel.outer_radius, outer_col, outer_bw),
    );

    let slice_angle = std::f32::consts::TAU / n_slices as f32;
    let base_pw = (2.0 * wheel.outer_radius * (slice_angle / 2.0).sin() * 0.72).max(48.0);
    let base_ph = ((wheel.outer_radius - wheel.inner_radius) * 0.85).max(40.0);
    let panel_w = (base_pw * wheel.segment_scale).max(32.0);
    let panel_h = (base_ph * wheel.segment_scale).max(24.0);
    let min_dim = panel_w.min(panel_h);
    let highlight_col = parse_hex_color(&wheel.highlight_color, 1.0);
    let slice_bg = Color::srgb(0.115, 0.12, 0.12);
    let label_c = Color::srgb(0.91, 0.91, 0.89);
    let label_sz = (panel_h * 0.18).clamp(9.0, 13.0);

    for (i, slot) in wheel.slots.iter().enumerate() {
        if i >= n_slices {
            break;
        }
        let is_sel = highlighted
            .map(|(s, e, w, sl)| s == set && e == entry && w == w_idx && sl == i)
            .unwrap_or(false);
        // The reference keeps the selected sector translucent so the dark
        // wheel surface remains visible beneath the coral tint.
        let seg_color = if is_sel {
            highlight_col.with_alpha(0.38)
        } else {
            slice_bg
        };

        if is_pie {
            let (a0, a1) = slice_angles(wheel, i);
            let mat_handle = wedge_materials.add(WedgeMaterial {
                params: WedgeParams {
                    color: seg_color.to_linear().to_vec4(),
                    border_color: if is_sel {
                        highlight_col.to_linear().to_vec4()
                    } else {
                        Color::srgb(0.30, 0.31, 0.31).to_linear().to_vec4()
                    },
                    inner_r: wheel.inner_radius,
                    outer_r: wheel.outer_radius,
                    angle_start: a0,
                    angle_end: a1,
                    edge_width: if is_sel { 2.0 } else { 0.8 },
                },
            });
            let dia = wheel.outer_radius * 2.0;
            let wedge_e = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(-wheel.outer_radius),
                        top: Val::Px(-wheel.outer_radius),
                        width: Val::Px(dia),
                        height: Val::Px(dia),
                        ..default()
                    },
                    MaterialNode(mat_handle),
                ))
                .id();
            commands.entity(hub).add_child(wedge_e);
            let ctr = slice_center(wheel, i);
            let panel_e = commands
                .spawn_scene(bsn! {
                    Node {
                        position_type: PositionType::Absolute,
                        left:   {Val::Px(ctr.x - panel_w / 2.0)},
                        top:    {Val::Px(-ctr.y - panel_h / 2.0)},
                        width:  {Val::Px(panel_w)}, height: {Val::Px(panel_h)},
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: {UiRect::all(Val::Px(6.))},
                    }
                    BackgroundColor({Color::NONE})
                })
                .insert((
                    WheelHudSegmentHit {
                        set,
                        entry,
                        wheel: w_idx,
                        slot: i,
                    },
                    Interaction::None,
                    Button,
                ))
                .id();
            commands.entity(hub).add_child(panel_e);
            if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    wheel_slice_label(slot.name.to_uppercase(), label_sz, label_c),
                );
            }
            if wheel.show_icon && !slot.icon.is_empty() {
                hud_wheel_icon(
                    commands,
                    panel_e,
                    &slot.icon,
                    (panel_h * 0.42).clamp(24.0, 44.0),
                    i,
                );
            } else if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    bsn! { Node { width: {Val::Px(4.)}, height: {Val::Px(4.)} } },
                );
            }
        } else {
            let seg_br = match wheel.segment_shape {
                SegmentShape::Square => BorderRadius::all(Val::Px(0.0)),
                SegmentShape::Rounded => BorderRadius::all(Val::Px(min_dim * 0.14)),
                SegmentShape::Circle => BorderRadius::all(Val::Px(min_dim * 0.5)),
                SegmentShape::Wedge => BorderRadius {
                    top_left: Val::Px(min_dim * 0.40),
                    top_right: Val::Px(min_dim * 0.40),
                    bottom_left: Val::Px(min_dim * 0.05),
                    bottom_right: Val::Px(min_dim * 0.05),
                },
                SegmentShape::Pie => unreachable!(),
            };
            let ctr = slice_center(wheel, i);
            let panel_e = commands
                .spawn_scene(bsn! {
                    Node {
                        position_type: PositionType::Absolute,
                        left:   {Val::Px(ctr.x - panel_w / 2.0)},
                        top:    {Val::Px(-ctr.y - panel_h / 2.0)},
                        width:  {Val::Px(panel_w)}, height: {Val::Px(panel_h)},
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: {UiRect::all(Val::Px(6.))},
                        border_radius: {seg_br},
                    }
                    BackgroundColor({seg_color})
                })
                .insert((
                    WheelHudSegmentHit {
                        set,
                        entry,
                        wheel: w_idx,
                        slot: i,
                    },
                    Interaction::None,
                    Button,
                ))
                .id();
            commands.entity(hub).add_child(panel_e);
            if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    wheel_slice_label(slot.name.to_uppercase(), label_sz, label_c),
                );
            }
            if wheel.show_icon && !slot.icon.is_empty() {
                hud_wheel_icon(
                    commands,
                    panel_e,
                    &slot.icon,
                    (panel_h * 0.42).clamp(24.0, 44.0),
                    i,
                );
            } else if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    bsn! { Node { width: {Val::Px(4.)}, height: {Val::Px(4.)} } },
                );
            }
        }
    }

    // Centre hub ring.
    let disc_r = (wheel.inner_radius - 4.0).max(8.0);
    let ring_col = if wheel.inner_border.is_empty() {
        Color::srgb(0.34, 0.35, 0.35)
    } else {
        parse_hex_color(&wheel.inner_border, 1.0)
    };
    let hub_bg = if wheel.hub_color.is_empty() {
        Color::srgba(0.10, 0.10, 0.10, wheel.hub_opacity)
    } else {
        parse_hex_color(&wheel.hub_color, wheel.hub_opacity)
    };
    let inner_bw = if wheel.inner_border.is_empty() {
        1.0_f32
    } else {
        wheel.inner_border_width.max(0.0)
    };
    let center = hud_child(
        commands,
        hub,
        wheel_center_ring(disc_r, hub_bg, ring_col, inner_bw),
    );
    if editor_open {
        commands.entity(center).insert((
            WheelHudButton {
                action: WheelHudAction::SelectWheel {
                    set,
                    entry,
                    wheel: w_idx,
                },
                base: hub_bg,
            },
            Button,
            Interaction::None,
        ));
    }

    // Hover remains transient preview state. The sector inspector is owned by
    // explicit selection so moving the pointer cannot close or reopen it.
    let hub_slot = highlighted.and_then(|(hs, he, hw, si)| {
        if hs == set && he == entry && hw == w_idx {
            wheel.slots.get(si)
        } else {
            None
        }
    });
    if let Some(slot) = hub_slot {
        let info_col = hud_child(
            commands,
            center,
            bsn! {
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: {Val::Px(2.)},
                }
            },
        );
        let name_sz = (disc_r * 0.22).clamp(7.0, 10.0);
        if !slot.icon.is_empty() {
            let icon_index = highlighted.map(|(_, _, _, index)| index).unwrap_or(0);
            hud_wheel_icon(
                commands,
                info_col,
                &slot.icon,
                (disc_r * 0.58).clamp(24.0, 42.0),
                icon_index,
            );
        }
        if !slot.name.is_empty() {
            hud_child(commands, info_col, hud_text(&slot.name, name_sz, HUD_DIM));
        }
        if editor_open {
            hud_child(commands, info_col, hud_text("▣  Apply", 9., HUD_DIM));
            hud_child(commands, info_col, hud_text("▣  Back", 9., HUD_DIM));
        }
    }
    if editor_open {
        if let Some((slot, slot_index)) = selected_segment.and_then(|(ss, se, sw, si)| {
            (ss == set && se == entry && sw == w_idx)
                .then(|| wheel.slots.get(si).map(|slot| (slot, si)))
                .flatten()
        }) {
            spawn_segment_editor_card(
                commands,
                hub,
                slot,
                set,
                entry,
                w_idx,
                slot_index,
                wheel.outer_radius,
                edit_control_focus,
            );
        }
    }
    if editor_open
        && highlighted.is_none()
        && (selected_wheel == Some((set, entry, w_idx))
            || hovered_wheel == Some((set, entry, w_idx)))
    {
        spawn_wheel_settings_card(commands, hub, wheel, theme_popup_open);
    }

    // Wheel-level controls are always visible in edit mode so hovering or
    // selecting the wheel exposes the same affordances as action buttons.
    if editor_open {
        let r = wheel.outer_radius + 28.0;
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(0.0, -r),
            WheelHudAction::WheelSettings {
                set,
                entry,
                wheel: w_idx,
            },
            "⚙",
            HUD_TEXT,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(r, 0.0),
            WheelHudAction::DeleteWheel {
                set,
                entry,
                wheel: w_idx,
            },
            "×",
            HUD_AMBER,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(0.0, r),
            WheelHudAction::MoveWheel {
                set,
                entry,
                wheel: w_idx,
            },
            "↕",
            HUD_TEXT,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(-r, 0.0),
            WheelHudAction::ResizeWheel {
                set,
                entry,
                wheel: w_idx,
                delta: 10.0,
            },
            "⌗",
            HUD_TEXT,
            false,
        );
    }

    // In edit mode, place the same compact radial controls used by the
    // reference UI: plus buttons on both sides of the selected sector and a
    // trash button on its inner edge.
    if editor_open {
        if let Some((hs, he, hw, slot)) = highlighted {
            if hs == set && he == entry && hw == w_idx && slot < n_slices {
                let (a0, a1) = slice_angles(wheel, slot);
                let mid = (a0 + a1) * 0.5;
                let p = Vec2::new(
                    a0.cos() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                    a0.sin() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::Before,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(1),
                );
                let p = Vec2::new(
                    mid.cos() * (wheel.inner_radius + 2.0),
                    mid.sin() * (wheel.inner_radius + 2.0),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::RemoveSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        slot,
                    },
                    "×",
                    HUD_TEXT,
                    edit_control_focus == Some(2),
                );
                // The second boundary is the after/right insertion point.
                let p = Vec2::new(
                    a1.cos() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                    a1.sin() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::After,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(3),
                );
                // Outer-side add affordance, matching the reference editor.
                let p = Vec2::new(
                    mid.cos() * (wheel.outer_radius + 2.0),
                    mid.sin() * (wheel.outer_radius + 2.0),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::Outer,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(4),
                );
            }
        }
    }
}

fn spawn_wheel_settings_card(
    commands: &mut Commands,
    parent: Entity,
    wheel: &RadialMenu,
    theme_popup_open: bool,
) {
    editor::components::spawn_wheel_settings_card(commands, parent, wheel, theme_popup_open);
}
fn spawn_segment_editor_card(
    commands: &mut Commands,
    parent: Entity,
    slot: &Sector,
    set: usize,
    entry: usize,
    wheel: Option<usize>,
    slot_index: usize,
    outer_radius: f32,
    edit_control_focus: Option<usize>,
) {
    editor::components::spawn_segment_editor_card(
        commands,
        parent,
        slot,
        set,
        entry,
        wheel,
        slot_index,
        outer_radius,
        edit_control_focus,
    );
}
fn spawn_radial_edit_button(
    commands: &mut Commands,
    parent: Entity,
    position: Vec2,
    action: WheelHudAction,
    label: &str,
    color: Color,
    focused: bool,
) {
    editor::components::spawn_radial_edit_button(
        commands, parent, position, action, label, color, focused,
    );
}

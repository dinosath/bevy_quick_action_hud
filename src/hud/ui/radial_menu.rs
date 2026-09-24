use crate::*;
use bevy::prelude::*;

use super::primitives::*;
use crate::radial_menu::components::{
    blend_radial_menu_colors, radial_menu_anchor, radial_menu_background, radial_menu_center_ring,
    radial_menu_divider, radial_menu_hub, radial_menu_hub_radius, radial_menu_label,
    radial_menu_outer_ring, radial_menu_sector, radial_menu_sector_content,
    radial_menu_sector_panel_height, rendered_sector_outer_radius, RADIAL_MENU_BACKGROUND_RGB,
    RADIAL_MENU_DEFAULT_BORDER_WIDTH, RADIAL_MENU_DIVIDER_RGBA, RADIAL_MENU_DIVIDER_WIDTH,
    RADIAL_MENU_HUB_BORDER_RGBA, RADIAL_MENU_ICON_SIZE_RANGE, RADIAL_MENU_ICON_SIZE_RATIO,
    RADIAL_MENU_LABEL_RGBA, RADIAL_MENU_LABEL_SIZE_RANGE, RADIAL_MENU_LABEL_SIZE_RATIO,
    RADIAL_MENU_OUTER_BORDER_RGBA, RADIAL_MENU_SECTOR_RGBA, RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT,
};

pub(crate) fn build_radial_menu_set_hud(
    commands: &mut Commands,
    parent: Entity,
    menu_set: &RadialMenuSet,
    set: usize,
    entry: usize,
    active_menu: usize,
    highlighted: Option<(usize, usize, Option<usize>, usize)>,
    selected_segment: Option<(usize, usize, Option<usize>, usize)>,
    selected_wheel: Option<(usize, usize, Option<usize>)>,
    hovered_wheel: Option<(usize, usize, Option<usize>)>,
    editor_open: bool,
    edit_control_focus: Option<usize>,
    theme_popup_open: bool,
) -> bool {
    let menu_index = active_menu.min(menu_set.radial_menu_count().saturating_sub(1));
    let Some(menu) = menu_set.radial_menu(menu_index) else {
        return false;
    };
    let mut display_menu = menu.clone();
    wheelset_visuals(menu_set).apply_to(&mut display_menu);
    build_radial_menu_hud(
        commands,
        parent,
        &display_menu,
        set,
        entry,
        Some(menu_index),
        highlighted,
        selected_segment,
        selected_wheel,
        hovered_wheel,
        editor_open,
        edit_control_focus,
        theme_popup_open,
    );
    true
}

#[allow(clippy::too_many_arguments)]
fn build_radial_menu_hud(
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
) {
    let n_slices = wheel.slots.len().max(1);
    let anchor = hud_child(commands, parent, radial_menu_anchor());
    let hub = hud_child(commands, anchor, radial_menu_hub());
    commands.entity(hub).insert(UiTransform {
        translation: Val2::px(wheel.offset_x, -wheel.offset_y),
        rotation: Rot2::degrees(wheel.rotation),
        ..default()
    });

    let bg_col = if wheel.bg_color.is_empty() {
        Color::srgba(
            RADIAL_MENU_BACKGROUND_RGB[0],
            RADIAL_MENU_BACKGROUND_RGB[1],
            RADIAL_MENU_BACKGROUND_RGB[2],
            wheel.bg_opacity,
        )
    } else {
        parse_hex_color(&wheel.bg_color, wheel.bg_opacity)
    };
    hud_child(
        commands,
        hub,
        radial_menu_background(wheel.outer_radius, bg_col),
    );
    let outer_col = if wheel.outer_border.is_empty() {
        Color::srgba(
            RADIAL_MENU_OUTER_BORDER_RGBA[0],
            RADIAL_MENU_OUTER_BORDER_RGBA[1],
            RADIAL_MENU_OUTER_BORDER_RGBA[2],
            RADIAL_MENU_OUTER_BORDER_RGBA[3],
        )
    } else {
        parse_hex_color(&wheel.outer_border, 1.0)
    };
    let outer_bw = if wheel.outer_border.is_empty() {
        RADIAL_MENU_DEFAULT_BORDER_WIDTH
    } else {
        wheel.outer_border_width.max(0.0)
    };
    let highlight_col = parse_hex_color(&wheel.highlight_color, 1.0);
    let slice_bg = Color::srgba(
        RADIAL_MENU_SECTOR_RGBA[0],
        RADIAL_MENU_SECTOR_RGBA[1],
        RADIAL_MENU_SECTOR_RGBA[2],
        RADIAL_MENU_SECTOR_RGBA[3],
    );
    let sector_border = Color::srgba(
        RADIAL_MENU_DIVIDER_RGBA[0],
        RADIAL_MENU_DIVIDER_RGBA[1],
        RADIAL_MENU_DIVIDER_RGBA[2],
        RADIAL_MENU_DIVIDER_RGBA[3],
    );
    let label_c = Color::srgba(
        RADIAL_MENU_LABEL_RGBA[0],
        RADIAL_MENU_LABEL_RGBA[1],
        RADIAL_MENU_LABEL_RGBA[2],
        RADIAL_MENU_LABEL_RGBA[3],
    );
    let mut selected_visual = None;

    for (i, slot) in wheel.slots.iter().enumerate() {
        if i >= n_slices {
            break;
        }
        let is_sel = highlighted
            .map(|(s, e, w, sl)| s == set && e == entry && w == w_idx && sl == i)
            .unwrap_or(false);
        let seg_color = if is_sel {
            blend_radial_menu_colors(
                slice_bg,
                highlight_col,
                RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT,
            )
        } else {
            slice_bg
        };

        let (start, end) = slice_angles(wheel, i);
        let sector_span = (end - start).max(0.0);
        let half_sector_sine = (sector_span * 0.5).sin().abs();
        let sector_rotation = std::f32::consts::FRAC_PI_2 - (start + end) * 0.5;
        let render_outer_radius = rendered_sector_outer_radius(wheel, is_sel);
        // The panel reaches the hub rather than stopping at `inner_radius`.
        // The hub is then the one authoritative circular clip for every
        // sector's inner edge.  Stopping at the inner radius creates the
        // incorrect flat, trapezoidal termination visible in the old probe.
        let panel_h = radial_menu_sector_panel_height(render_outer_radius);
        let center_angle = (start + end) * 0.5;
        let ctr = Vec2::new(
            center_angle.cos() * panel_h * 0.5,
            center_angle.sin() * panel_h * 0.5,
        );
        let label_sz = ((wheel.outer_radius - wheel.inner_radius) * RADIAL_MENU_LABEL_SIZE_RATIO)
            .clamp(
                RADIAL_MENU_LABEL_SIZE_RANGE.0,
                RADIAL_MENU_LABEL_SIZE_RANGE.1,
            );
        let sector_width = (2.0 * render_outer_radius * half_sector_sine).max(32.0);
        let content_rotation = sector_rotation;
        let panel_scene = radial_menu_sector(
            ctr,
            sector_width,
            panel_h,
            render_outer_radius,
            sector_span,
            seg_color,
            is_sel.then_some(highlight_col),
            content_rotation,
        );
        let panel_e = commands
            .spawn_scene(bsn! {
                { panel_scene }
                ChildOf(hub)
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
        if is_sel {
            // Repaint this scene after the shared base ring below.  Keeping
            // the interactive panel here preserves one hit target per sector.
            selected_visual = Some((
                ctr,
                sector_width,
                panel_h,
                render_outer_radius,
                sector_span,
                content_rotation,
            ));
        }
        let content_e = hud_child(
            commands,
            panel_e,
            radial_menu_sector_content(content_rotation, wheel.inner_radius * 0.5),
        );
        if is_sel {
            // The selected surface is repainted after the base outer ring;
            // keep its label/icon above that visual-only repaint.
            commands.entity(content_e).insert(GlobalZIndex(1));
        }
        if wheel.show_labels {
            hud_child(
                commands,
                content_e,
                radial_menu_label(slot.name.to_uppercase(), label_sz, label_c),
            );
        }
        if wheel.show_icon && !slot.icon.is_empty() {
            hud_wheel_icon(
                commands,
                content_e,
                &slot.icon,
                (panel_h * RADIAL_MENU_ICON_SIZE_RATIO)
                    .clamp(RADIAL_MENU_ICON_SIZE_RANGE.0, RADIAL_MENU_ICON_SIZE_RANGE.1),
                i,
            );
        } else if wheel.show_labels {
            hud_child(
                commands,
                content_e,
                bsn! { Node { width: {Val::Px(4.)}, height: {Val::Px(4.)} } },
            );
        }
    }

    // The shared outer ring stays continuous around unselected sectors.  The
    // highlighted sector is then redrawn over its own portion of the ring.
    hud_child(
        commands,
        hub,
        radial_menu_outer_ring(wheel.outer_radius, outer_col, outer_bw),
    );
    if let Some((center, width, height, outer_radius, sector_span, rotation)) = selected_visual {
        hud_child(
            commands,
            hub,
            radial_menu_sector(
                center,
                width,
                height,
                outer_radius,
                sector_span,
                blend_radial_menu_colors(
                    slice_bg,
                    highlight_col,
                    RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT,
                ),
                Some(highlight_col),
                rotation,
            ),
        );
    }

    for i in 0..n_slices {
        let (start, _) = slice_angles(wheel, i);
        let previous = (i + n_slices - 1) % n_slices;
        let selected_boundary = highlighted
            .map(|(s, e, w, slot)| {
                s == set && e == entry && w == w_idx && (slot == i || slot == previous)
            })
            .unwrap_or(false);
        hud_child(
            commands,
            hub,
            radial_menu_divider(
                start,
                wheel.inner_radius,
                rendered_sector_outer_radius(wheel, selected_boundary),
                if selected_boundary {
                    highlight_col
                } else {
                    sector_border
                },
                RADIAL_MENU_DIVIDER_WIDTH,
            ),
        );
    }

    // Centre hub ring.
    let disc_r = radial_menu_hub_radius(wheel);
    let ring_col = if wheel.inner_border.is_empty() {
        Color::srgba(
            RADIAL_MENU_HUB_BORDER_RGBA[0],
            RADIAL_MENU_HUB_BORDER_RGBA[1],
            RADIAL_MENU_HUB_BORDER_RGBA[2],
            RADIAL_MENU_HUB_BORDER_RGBA[3],
        )
    } else {
        parse_hex_color(&wheel.inner_border, 1.0)
    };
    let hub_bg = if wheel.hub_color.is_empty() {
        Color::srgba(0.10, 0.10, 0.10, wheel.hub_opacity)
    } else {
        parse_hex_color(&wheel.hub_color, wheel.hub_opacity)
    };
    let inner_bw = if wheel.inner_border.is_empty() {
        RADIAL_MENU_DEFAULT_BORDER_WIDTH
    } else {
        wheel.inner_border_width.max(0.0)
    };
    let center = hud_child(
        commands,
        hub,
        radial_menu_center_ring(disc_r, hub_bg, ring_col, inner_bw),
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

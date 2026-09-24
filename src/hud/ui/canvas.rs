use crate::*;
use bevy::prelude::*;

use super::{buttons::*, navigation::*, primitives::*, radial_menu::*};

pub(crate) fn build_hud_canvas(
    commands: &mut Commands,
    cfg: &QuickActionConfig,
    hud: &WheelHudState,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
    wedge_materials: &mut Assets<WedgeMaterial>,
) {
    let root = commands
        .spawn_scene(hud_canvas_root())
        .insert(WheelHudRoot)
        .id();
    if !hud.open {
        return;
    }

    commands.entity(root).insert(Visibility::Visible);

    // Apply the user-configured HUD background opacity.
    let hud_bg = if cfg.hud_bg_color.is_empty() {
        HUD_BG.with_alpha(cfg.hud_bg_opacity)
    } else {
        parse_hex_color(&cfg.hud_bg_color, cfg.hud_bg_opacity)
    };
    commands.entity(root).insert(BackgroundColor(hud_bg));

    // Edit toggle button — visible only while the wheel is open, hidden when
    // the editor sidebar is already showing.
    if !hud.editor_open {
        let btn = hud_clickable(
            commands,
            root,
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    top: {Val::Px(14.)}, left: {Val::Px(14.)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                    padding: {UiRect::axes(Val::Px(10.), Val::Px(6.))},
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(5.))},
                }
                BorderColor::all(HUD_BADGE_BORDER)
                BackgroundColor({HUD_PANEL_CARD})
                Button
            },
            WheelHudAction::ToggleEditor,
            HUD_PANEL_CARD,
        );
        // Show the assigned edit shortcut icon (if it's a gamepad button).
        if let Some(lbl) = cfg.edit_shortcut.strip_prefix("GP:") {
            if let Some(path) = icon_set.embedded_icon_path(lbl) {
                let handle = asset_server.load::<Image>(path);
                let icon_e = commands
                    .spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        ImageNode::new(handle),
                    ))
                    .id();
                commands.entity(btn).add_child(icon_e);
            }
        }
        {
            let handle = asset_server.load::<Image>(
                "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-cog.png",
            );
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    ImageNode {
                        image: handle,
                        color: HUD_DIM,
                        ..default()
                    },
                ))
                .id();
            commands.entity(btn).add_child(e);
        }
        hud_child(commands, btn, hud_text("Edit", 10., HUD_DIM));
    }

    // Set tabs at the top centre (only when enabled in config).
    if cfg.show_set_bar {
        build_hud_set_tabs(commands, root, cfg, hud, asset_server, icon_set);
    }
    if hud.editor_open {
        build_hud_editor_toolbar(
            commands,
            root,
            hud.settings_open,
            hud.edit_control_focus,
            &cfg.edit_shortcut,
        );
    }

    if cfg.sets.is_empty() {
        hud_child(
            commands,
            root,
            hud_text("No sets — open the editor to add one.", 12., HUD_DIMMER),
        );
        return;
    }

    let mut active_page = None;
    for (index, set) in cfg.sets.iter().enumerate() {
        if index == hud.active_set && set.enabled {
            active_page = Some(root);
        }
    }

    if let (Some(set), Some(page)) = (cfg.sets.get(hud.active_set), active_page) {
        // Background image for this set, if configured.
        if !set.bg_image.is_empty() {
            let handle = asset_server.load::<Image>(set.bg_image.clone());
            let bg_e = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.),
                        top: Val::Px(0.),
                        right: Val::Px(0.),
                        bottom: Val::Px(0.),
                        ..default()
                    },
                    ImageNode {
                        image: handle,
                        color: Color::WHITE.with_alpha(set.bg_image_opacity),
                        ..default()
                    },
                ))
                .id();
            commands.entity(page).add_child(bg_e);
        }

        // Clamp active_wheel_entry to a valid range.
        let n_wheels = count_wheel_entries(set);
        let target = if n_wheels == 0 {
            0
        } else {
            hud.active_wheel_entry.min(n_wheels - 1)
        };

        // Find the target-th Wheel / RadialMenuSetState entry.
        let mut rendered = false;
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
                    build_centered_wheel_hud(
                        commands,
                        page,
                        w,
                        hud.active_set,
                        ei,
                        None,
                        hud.highlighted,
                        hud.selected_segment,
                        hud.selected_wheel,
                        hud.hovered_wheel,
                        hud.editor_open,
                        hud.edit_control_focus,
                        hud.theme_popup_open,
                        wedge_materials,
                    );
                    rendered = true;
                }
                SetEntry::RadialMenuSet(ws) => {
                    let wheel_index = hud
                        .active_wheel_index
                        .min(ws.wheels.len().saturating_sub(1));
                    if let Some(w) = ws.wheels.get(wheel_index) {
                        let mut display_wheel = w.clone();
                        wheelset_visuals(ws).apply_to(&mut display_wheel);
                        build_centered_wheel_hud(
                            commands,
                            page,
                            &display_wheel,
                            hud.active_set,
                            ei,
                            Some(wheel_index),
                            hud.highlighted,
                            hud.selected_segment,
                            hud.selected_wheel,
                            hud.hovered_wheel,
                            hud.editor_open,
                            hud.edit_control_focus,
                            hud.theme_popup_open,
                            wedge_materials,
                        );
                        rendered = true;
                    }
                }
                _ => {}
            }
            break;
        }
        if !rendered {
            hud_child(
                commands,
                page,
                hud_text("No radial menus in this set.", 11., HUD_DIMMER),
            );
        }
        build_hud_action_buttons(
            commands,
            page,
            hud.active_set,
            set,
            asset_server,
            icon_set,
            hud.flash_action_entry,
            hud.editor_open,
        );
        build_hud_switch_buttons(
            commands,
            page,
            hud.active_set,
            set,
            asset_server,
            hud.editor_open,
        );
        if hud.highlighted.is_none() && hud.selected_wheel.is_none() && hud.hovered_wheel.is_none()
        {
            if let Some((selected_set, selected_entry)) = hud.selected_action {
                if selected_set == hud.active_set {
                    if let Some(SetEntry::Action(action)) = set.entries.get(selected_entry) {
                        build_hud_action_editor_card(
                            commands,
                            root,
                            selected_set,
                            selected_entry,
                            action,
                        );
                    }
                }
            }
        }
    }
}

fn build_hud_action_editor_card(
    commands: &mut Commands,
    parent: Entity,
    set: usize,
    entry: usize,
    action: &QuickAction,
) {
    editor::components::build_hud_action_editor_card(commands, parent, set, entry, action);
}
pub(crate) fn hud_action_field(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    height: f32,
    action: WheelHudAction,
    value_color: Color,
) {
    let field = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                height: {Val::Px(height)},
                padding: {UiRect::horizontal(Val::Px(9.))},
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        action,
        HUD_PANEL_CARD,
    );
    hud_child(commands, field, hud_text(label, 9., HUD_DIM));
    hud_child(commands, field, hud_text(value, 11., value_color));
}

pub(crate) fn hud_action_stepper(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    decrement: WheelHudAction,
    increment: WheelHudAction,
) {
    let row = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                height: {Val::Px(26.)},
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: {Val::Px(4.)},
            }
        },
    );
    hud_child(commands, row, hud_text(label, 10., HUD_TEXT));
    let minus = hud_clickable(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(26.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        decrement,
        HUD_PANEL_CARD,
    );
    hud_child(commands, minus, hud_text("−", 14., HUD_TEXT));
    let value_node = hud_child(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(52.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
        },
    );
    hud_child(commands, value_node, hud_text(value, 10., HUD_DIM));
    let plus = hud_clickable(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(26.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        increment,
        HUD_PANEL_CARD,
    );
    hud_child(commands, plus, hud_text("+", 14., HUD_TEXT));
}

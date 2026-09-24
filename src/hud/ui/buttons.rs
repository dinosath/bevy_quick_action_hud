use crate::*;
use bevy::prelude::*;

use super::primitives::*;

pub(crate) fn build_hud_action_buttons(
    commands: &mut Commands,
    parent: Entity,
    set_index: usize,
    set: &ActionSet,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
    flash_entry: Option<usize>,
    editor_open: bool,
) {
    let btns: Vec<(usize, &QuickAction)> = set
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if let SetEntry::Action(a) = e {
                Some((i, a))
            } else {
                None
            }
        })
        .filter(|(_, a)| a.enabled)
        .collect();
    if btns.is_empty() {
        return;
    }

    // Keep each action on an independent anchor. A flex column would include
    // every button's current height in the layout calculation, so resizing one
    // button would move all buttons above it.
    const ACTION_STACK_STEP: f32 = 36.0;
    let container = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(60.)}, right: {Val::Px(36.)},
                width: {Val::Px(360.)}, height: {Val::Px(520.)},
            }
        })
        .id();
    commands.entity(parent).add_child(container);

    for (stack_index, (entry_idx, qa)) in btns.iter().rev().enumerate() {
        let is_flash = flash_entry == Some(*entry_idx);
        let eff = (set.opacity * qa.opacity).clamp(0.05, 1.0);
        let w = qa.width.max(40.0);
        let h = qa.height.max(20.0);
        let bg = if is_flash {
            Color::srgba(0.38, 0.62, 0.95, 0.90) // bright blue flash
        } else {
            parse_hex_color(&qa.color, eff * 0.85)
        };
        let tc = HUD_TEXT.with_alpha(if is_flash { 1.0 } else { eff });
        let bc = HUD_BADGE_BORDER.with_alpha(eff);

        let row = hud_child(
            commands,
            container,
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    right: {Val::Px(-qa.offset_x)},
                    bottom: {Val::Px(stack_index as f32 * ACTION_STACK_STEP + qa.offset_y)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                }
            },
        );
        if !qa.key.is_empty() {
            // GP: key → show the controller button icon; keyboard key → text badge.
            let mut showed_icon = false;
            if let Some(btn_label) = qa.key.strip_prefix("GP:") {
                if let Some(path) = icon_set.embedded_icon_path(btn_label) {
                    let handle = asset_server.load::<Image>(path);
                    let icon_e = commands
                        .spawn((
                            Node {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                ..default()
                            },
                            ImageNode::new(handle),
                        ))
                        .id();
                    commands.entity(row).add_child(icon_e);
                    showed_icon = true;
                }
            }
            if !showed_icon {
                // Keyboard fallback — bordered text badge.
                let key_disp = qa.key.strip_prefix("GP:").unwrap_or(&qa.key);
                let kb = hud_child(
                    commands,
                    row,
                    bsn! {
                        Node {
                            min_width: {Val::Px(16.)}, height: {Val::Px(16.)},
                            padding: {UiRect::horizontal(Val::Px(3.))},
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: {UiRect::all(Val::Px(1.))},
                            border_radius: {BorderRadius::all(Val::Px(2.))},
                        }
                        BorderColor::all(HUD_BADGE_BORDER)
                    },
                );
                hud_child(commands, kb, hud_text(key_disp, 8., HUD_DIM));
            }
        }
        let button_wrap = hud_child(
            commands,
            row,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    width: {Val::Px(w)}, height: {Val::Px(h)},
                }
            },
        );
        let btn_node = commands
            .spawn_scene(bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: {Val::Px(0.)}, top: {Val::Px(0.)},
                    width: {Val::Px(w)}, height: {Val::Px(h)},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(4.))},
                }
                BackgroundColor({bg})
                BorderColor::all(bc)
            })
            .insert((
                WheelHudButton {
                    action: WheelHudAction::SelectAction {
                        set: set_index,
                        entry: *entry_idx,
                    },
                    base: bg,
                },
                Button,
                Interaction::None,
            ))
            .id();
        if qa.rotation != 0.0 {
            commands
                .entity(btn_node)
                .insert(Transform::from_rotation(Quat::from_rotation_z(
                    qa.rotation.to_radians(),
                )));
        }
        commands.entity(button_wrap).add_child(btn_node);
        hud_child(commands, btn_node, hud_text(&qa.name, 10., tc));
        if editor_open {
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w * 0.5 - 11.),
                Val::Px(-24.),
                asset_server,
                "cil-camera-control",
                WheelHudAction::MoveAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w + 2.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-aperture",
                WheelHudAction::RotateAction {
                    set: set_index,
                    entry: *entry_idx,
                    delta: 15.0,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w * 0.5 - 11.),
                Val::Px(h + 2.),
                asset_server,
                "cil-trash",
                WheelHudAction::DeleteAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_AMBER,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(-24.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-cog",
                WheelHudAction::EditAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_TEXT,
            );
        }
    }
}

/// Floating HUD-switch components share the same contextual editor affordances
/// as quick-action buttons: settings, move, resize, and delete.
pub(crate) fn build_hud_switch_buttons(
    commands: &mut Commands,
    parent: Entity,
    set_index: usize,
    set: &ActionSet,
    asset_server: &AssetServer,
    editor_open: bool,
) {
    let switches: Vec<(usize, &HudSwitch)> = set
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if let SetEntry::HudSwitch(s) = e {
                s.enabled.then_some((i, s))
            } else {
                None
            }
        })
        .collect();
    if switches.is_empty() {
        return;
    }
    let container = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(104.)}, right: {Val::Px(36.)},
                flex_direction: FlexDirection::Column, row_gap: {Val::Px(8.)},
                align_items: AlignItems::FlexEnd,
            }
        })
        .id();
    commands.entity(parent).add_child(container);
    for (entry, switch) in switches.iter().rev() {
        let row = hud_child(
            commands,
            container,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    left: {Val::Px(switch.offset_x)}, top: {Val::Px(-switch.offset_y)},
                    flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                }
            },
        );
        let button = hud_clickable(
            commands,
            row,
            bsn! {
                Node {
                    width: {Val::Px(switch.width.max(40.))}, height: {Val::Px(switch.height.max(20.))},
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))}, border_radius: {BorderRadius::all(Val::Px(4.))},
                }
                BackgroundColor({Color::srgba(0.38, 0.26, 0.62, 0.85)})
                BorderColor::all(HUD_BADGE_BORDER)
                Button
            },
            WheelHudAction::SelectHudSwitch {
                set: set_index,
                entry: *entry,
            },
            HUD_PANEL_CARD,
        );
        hud_child(commands, button, hud_text(&switch.name, 10., HUD_TEXT));
        if editor_open {
            let w = switch.width.max(40.);
            let h = switch.height.max(20.);
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w * 0.5 - 11.),
                Val::Px(-24.),
                asset_server,
                "cil-camera-control",
                WheelHudAction::MoveHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w + 2.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-aperture",
                WheelHudAction::ResizeHudSwitch {
                    set: set_index,
                    entry: *entry,
                    delta: 8.0,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w * 0.5 - 11.),
                Val::Px(h + 2.),
                asset_server,
                "cil-trash",
                WheelHudAction::DeleteHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_AMBER,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(-24.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-cog",
                WheelHudAction::EditHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_TEXT,
            );
        }
    }
}

fn spawn_action_edge_button(
    commands: &mut Commands,
    parent: Entity,
    left: Val,
    top: Val,
    asset_server: &AssetServer,
    icon: &str,
    action: WheelHudAction,
    color: Color,
) {
    let owner = hud_control_owner(&action);
    let button = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {left}, top: {top},
                width: {Val::Px(22.)}, height: {Val::Px(22.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(11.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(if color == HUD_AMBER { HUD_AMBER } else { HUD_BADGE_BORDER })
            Button
        },
        action,
        HUD_PANEL_CARD,
    );
    if let Some(owner) = owner {
        commands
            .entity(button)
            .insert((HudContextControl { owner }, Visibility::Hidden));
    }
    let handle = asset_server.load::<Image>(format!(
        "embedded://bevy_quick_action_hud/embedded/icons/editor/{icon}.png"
    ));
    let icon_node = commands
        .spawn((
            Node {
                width: Val::Px(14.),
                height: Val::Px(14.),
                ..default()
            },
            ImageNode {
                image: handle,
                color,
                ..default()
            },
        ))
        .id();
    commands.entity(button).add_child(icon_node);
}

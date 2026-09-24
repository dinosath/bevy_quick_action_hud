use crate::*;
use bevy::prelude::*;

use super::primitives::*;

pub(crate) fn build_hud_editor_toolbar(
    commands: &mut Commands,
    parent: Entity,
    settings_open: bool,
    edit_focus: Option<usize>,
    edit_shortcut: &str,
) {
    let bar = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(14.)}, left: {Val::Px(14.)},
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: {Val::Px(6.)},
                padding: {UiRect::all(Val::Px(0.))},
            }
            BackgroundColor({Color::NONE})
        },
    );
    let close = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                padding: {UiRect::axes(Val::Px(10.), Val::Px(6.))},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({if edit_focus == Some(11) { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if edit_focus == Some(11) { HUD_TEXT } else { HUD_AMBER })
            Button
        },
        WheelHudAction::ToggleEditor,
        HUD_PANEL_CARD,
    );
    hud_child(
        commands,
        close,
        hud_text(&format!("Close  [{}]", edit_shortcut), 10., HUD_AMBER),
    );
    for (label, shortcut, action, accent, focus) in [
        (
            "Save",
            "LB / Ctrl+S",
            WheelHudAction::SaveConfig,
            HUD_GREEN,
            8,
        ),
        (
            "+ Add new",
            "RB / Ctrl+N",
            WheelHudAction::AddNewButton,
            HUD_TEXT,
            9,
        ),
        (
            if settings_open {
                "Settings ✓"
            } else {
                "Settings"
            },
            "Select / Ctrl+,",
            WheelHudAction::ToggleSettings,
            HUD_TEXT,
            10,
        ),
    ] {
        let button = hud_clickable(
            commands,
            bar,
            bsn! {
                Node {
                    height: {Val::Px(28.)},
                    padding: {UiRect::horizontal(Val::Px(10.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(3.))},
                }
                BackgroundColor({if edit_focus == Some(focus) { HUD_AMBER } else { HUD_PANEL_CARD }})
                BorderColor::all(if edit_focus == Some(focus) {
                    HUD_TEXT
                } else {
                    HUD_BADGE_BORDER
                })
                Button
            },
            action,
            HUD_PANEL_CARD,
        );
        hud_child(
            commands,
            button,
            hud_text(&format!("{label}  [{shortcut}]"), 10., accent),
        );
    }
}

/// Renders a radial wheel preview centred in the HUD.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_hud_set_tabs(
    commands: &mut Commands,
    parent: Entity,
    cfg: &QuickActionConfig,
    hud: &WheelHudState,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
) {
    let bar = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(12.)}, left: {Val::Px(0.)}, right: {Val::Px(0.)},
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
        })
        .id();
    commands.entity(parent).add_child(bar);

    let prev_idx = hud.active_set.saturating_sub(1);
    let larrow = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                width: {Val::Px(28.)}, height: {Val::Px(32.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::left(Val::Px(6.))},
            }
            BorderColor::all(HUD_SIDEBAR_BORDER)
            BackgroundColor({HUD_PANEL_CARD})
            Button
        },
        WheelHudAction::SetActiveSet(prev_idx),
        HUD_PANEL_CARD,
    );
    // Chevron-left PNG icon; gamepad icon overlays it when a button is assigned.
    {
        let handle = asset_server.load::<Image>(
            "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-chevron-left.png",
        );
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    color: HUD_DIM,
                    ..default()
                },
            ))
            .id();
        commands.entity(larrow).add_child(e);
    }
    // Overlay the assigned prev-set icon if it's a gamepad button.
    if let Some(lbl) = cfg.prev_set_key.strip_prefix("GP:") {
        if let Some(path) = icon_set.embedded_icon_path(lbl) {
            let handle = asset_server.load::<Image>(path);
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    ImageNode::new(handle),
                ))
                .id();
            commands.entity(larrow).add_child(e);
        }
    }

    for (i, set) in cfg.sets.iter().enumerate() {
        let active = i == hud.active_set;
        let (bg, tc, bc) = if active {
            (Color::srgba(0.38, 0.62, 0.95, 0.20), HUD_TEXT, HUD_BLUE)
        } else {
            (HUD_PANEL_CARD, HUD_DIM, HUD_SIDEBAR_BORDER)
        };
        let tab = hud_clickable(
            commands,
            bar,
            bsn! {
                Node {
                    padding: {UiRect::axes(Val::Px(14.), Val::Px(7.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                }
                BorderColor::all(bc)
                BackgroundColor({bg})
                Button
            },
            WheelHudAction::SetActiveSet(i),
            bg,
        );
        hud_child(commands, tab, hud_text(&set.name, 11., tc));
    }

    let next_idx = (hud.active_set + 1).min(cfg.sets.len().saturating_sub(1));
    let rarrow = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                width: {Val::Px(28.)}, height: {Val::Px(32.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::right(Val::Px(6.))},
            }
            BorderColor::all(HUD_SIDEBAR_BORDER)
            BackgroundColor({HUD_PANEL_CARD})
            Button
        },
        WheelHudAction::SetActiveSet(next_idx),
        HUD_PANEL_CARD,
    );
    // Chevron-right PNG icon; gamepad icon overlays it when a button is assigned.
    {
        let handle = asset_server.load::<Image>(
            "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-chevron-right.png",
        );
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    color: HUD_DIM,
                    ..default()
                },
            ))
            .id();
        commands.entity(rarrow).add_child(e);
    }
    // Overlay the assigned next-set icon if it's a gamepad button.
    if let Some(lbl) = cfg.next_set_key.strip_prefix("GP:") {
        if let Some(path) = icon_set.embedded_icon_path(lbl) {
            let handle = asset_server.load::<Image>(path);
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    ImageNode::new(handle),
                ))
                .id();
            commands.entity(rarrow).add_child(e);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// WheelHudPlugin
// ─────────────────────────────────────────────────────────────────────────────────

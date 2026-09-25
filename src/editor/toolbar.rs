//! The [`EditMode`] slot: the "Edit" toggle, the editor toolbar, and the
//! inspector of the selected button.

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::components::build_hud_action_editor_card;
use crate::hud::slot::HudSlot;
use crate::radial_menu_set::keyed_highlight;
use crate::widgets::{
    editor_icon_path, hud_clickable, hud_layer, image_icon, spawn_child, text_label,
};
use crate::{
    HudView, SetEntry, WheelHudAction, WheelHudState, HUD_AMBER, HUD_BADGE_BORDER, HUD_DIM,
    HUD_GREEN, HUD_PANEL_CARD, HUD_TEXT,
};

/// Slot holding the editor entry points; drawn above every page component.
#[derive(Component, Default, Clone, Copy)]
#[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
pub struct EditMode;

/// Editor state shown by the toolbar and the button inspector.
type EditModeKey = (Option<usize>, bool, bool, Option<(usize, usize)>);

impl HudSlot for EditMode {
    type Key = Option<EditModeKey>;

    fn key(&self, hud: &WheelHudState) -> Self::Key {
        hud.editor_open.then(|| {
            (
                hud.edit_control_focus.filter(|&focus| focus != 0),
                hud.settings_open,
                keyed_highlight(hud).is_some() || hud.selected_wheel.is_some(),
                hud.selected_action
                    .filter(|(page, _)| *page == hud.active_set),
            )
        })
    }
}

pub(crate) fn fill_edit_mode(add: On<Add<EditMode>>, view: HudView, mut commands: Commands) {
    let slot = add.entity;
    if !view.hud.editor_open {
        spawn_edit_toggle(&mut commands, slot, &view);
        return;
    }
    spawn_editor_toolbar(&mut commands, slot, &view);

    let hud = &view.hud;
    if hud.highlighted.is_some() || hud.selected_wheel.is_some() || hud.hovered_wheel.is_some() {
        return;
    }
    let Some((page, entry)) = hud.selected_action.filter(|(p, _)| *p == hud.active_set) else {
        return;
    };
    if let Some(SetEntry::Action(action)) =
        view.cfg.sets.get(page).and_then(|p| p.entries.get(entry))
    {
        build_hud_action_editor_card(&mut commands, slot, page, entry, action);
    }
}

fn spawn_edit_toggle(commands: &mut Commands, slot: Entity, view: &HudView) {
    let button = hud_clickable(
        commands,
        slot,
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
    if let Some(glyph) = view.gamepad_glyph(&view.cfg.edit_shortcut) {
        commands.spawn((image_icon(glyph, 16., Color::WHITE), ChildOf(button)));
    }
    let cog = view.asset_server.load(editor_icon_path("cil-cog"));
    commands.spawn((image_icon(cog, 14., HUD_DIM), ChildOf(button)));
    spawn_child(commands, button, text_label("Edit", 10., HUD_DIM));
}

fn spawn_editor_toolbar(commands: &mut Commands, slot: Entity, view: &HudView) {
    let focus = view.hud.edit_control_focus;
    let bar = spawn_child(
        commands,
        slot,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(14.)}, left: {Val::Px(14.)},
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: {Val::Px(6.)},
            }
        },
    );
    let close_focused = focus == Some(11);
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
            BackgroundColor({if close_focused { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if close_focused { HUD_TEXT } else { HUD_AMBER })
            Button
        },
        WheelHudAction::ToggleEditor,
        HUD_PANEL_CARD,
    );
    let close_label = format!("Close  [{}]", view.cfg.edit_shortcut);
    spawn_child(commands, close, text_label(&close_label, 10., HUD_AMBER));

    let settings = if view.hud.settings_open {
        "Settings ✓"
    } else {
        "Settings"
    };
    for (name, shortcut, action, accent, index) in [
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
            settings,
            "Select / Ctrl+,",
            WheelHudAction::ToggleSettings,
            HUD_TEXT,
            10,
        ),
    ] {
        let focused = focus == Some(index);
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
                BackgroundColor({if focused { HUD_AMBER } else { HUD_PANEL_CARD }})
                BorderColor::all(if focused { HUD_TEXT } else { HUD_BADGE_BORDER })
                Button
            },
            action,
            HUD_PANEL_CARD,
        );
        let caption = format!("{name}  [{shortcut}]");
        spawn_child(commands, button, text_label(&caption, 10., accent));
    }
}

//! The [`PageSwitches`] slot: every enabled page-switch control on a page.

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::editor::overlays::spawn_edge_affordances;
use crate::hud::slot::HudSlot;
use crate::widgets::{hud_clickable, hud_layer, spawn_child, text_label};
use crate::{HudView, WheelHudAction, WheelHudState, HUD_BADGE_BORDER, HUD_PANEL_CARD, HUD_TEXT};

/// Slot holding the page switches of page `.0`.
#[derive(Component, Default, Clone, Copy)]
#[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
pub struct PageSwitches(pub usize);

impl HudSlot for PageSwitches {
    type Key = bool;

    fn key(&self, hud: &WheelHudState) -> bool {
        hud.editor_open
    }
}

pub(crate) fn fill_page_switches(
    add: On<Add<PageSwitches>>,
    slots: Query<&PageSwitches>,
    view: HudView,
    mut commands: Commands,
) {
    let Ok(&PageSwitches(set)) = slots.get(add.entity) else {
        return;
    };
    let Some(page) = view.cfg.sets.get(set) else {
        return;
    };
    let switches: Vec<_> = page.page_switches().collect();
    if switches.is_empty() {
        return;
    }
    let column = spawn_child(
        &mut commands,
        add.entity,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(104.)}, right: {Val::Px(36.)},
                flex_direction: FlexDirection::Column, row_gap: {Val::Px(8.)},
                align_items: AlignItems::FlexEnd,
            }
        },
    );
    for (entry, switch) in switches.into_iter().rev() {
        let size = Vec2::new(switch.width.max(40.), switch.height.max(20.));
        let row = spawn_child(
            &mut commands,
            column,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    left: {Val::Px(switch.offset_x)}, top: {Val::Px(-switch.offset_y)},
                    flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                }
            },
        );
        let control = hud_clickable(
            &mut commands,
            row,
            bsn! {
                Node {
                    width: {Val::Px(size.x)}, height: {Val::Px(size.y)},
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))}, border_radius: {BorderRadius::all(Val::Px(4.))},
                }
                BackgroundColor({Color::srgba(0.38, 0.26, 0.62, 0.85)})
                BorderColor::all(HUD_BADGE_BORDER)
                Button
            },
            WheelHudAction::SelectHudSwitch { set, entry },
            HUD_PANEL_CARD,
        );
        spawn_child(
            &mut commands,
            control,
            text_label(&switch.name, 10., HUD_TEXT),
        );
        if view.hud.editor_open {
            spawn_edge_affordances(
                &mut commands,
                control,
                size,
                &view.asset_server,
                [
                    WheelHudAction::MoveHudSwitch { set, entry },
                    WheelHudAction::ResizeHudSwitch {
                        set,
                        entry,
                        delta: 8.0,
                    },
                    WheelHudAction::DeleteHudSwitch { set, entry },
                    WheelHudAction::EditHudSwitch { set, entry },
                ],
            );
        }
    }
}

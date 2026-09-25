//! The [`Buttons`] slot: every enabled floating button on a page.

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::editor::overlays::spawn_edge_affordances;
use crate::hud::slot::HudSlot;
use crate::widgets::{hud_layer, image_icon, parse_hex_color, spawn_child, text_label};
use crate::{
    HudView, QuickAction, WheelHudAction, WheelHudButton, WheelHudState, HUD_BADGE_BORDER, HUD_DIM,
    HUD_TEXT,
};

/// Vertical distance between stacked button anchors.
const BUTTON_STACK_STEP: f32 = 36.0;

/// Slot holding the buttons of page `.0`.
#[derive(Component, Default, Clone, Copy)]
#[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
pub struct Buttons(pub usize);

impl HudSlot for Buttons {
    type Key = (bool, Option<usize>);

    fn key(&self, hud: &WheelHudState) -> Self::Key {
        (hud.editor_open, hud.flash_action_entry)
    }
}

pub(crate) fn fill_buttons(
    add: On<Add<Buttons>>,
    slots: Query<&Buttons>,
    view: HudView,
    mut commands: Commands,
) {
    let Ok(&Buttons(page_index)) = slots.get(add.entity) else {
        return;
    };
    let Some(page) = view.cfg.sets.get(page_index) else {
        return;
    };
    let buttons: Vec<_> = page.buttons().collect();
    if buttons.is_empty() {
        return;
    }
    let area = spawn_child(
        &mut commands,
        add.entity,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(60.)}, right: {Val::Px(36.)},
                width: {Val::Px(360.)}, height: {Val::Px(520.)},
            }
            Pickable::IGNORE
        },
    );
    let opacity = page.opacity;
    // Each button keeps an independent anchor so resizing one does not move the others.
    for (stack_index, (entry, button)) in buttons.into_iter().rev().enumerate() {
        let slot = ButtonSlot {
            page_index,
            entry,
            stack_index,
            page_opacity: opacity,
        };
        spawn_button(&mut commands, area, slot, button, &view);
    }
}

struct ButtonSlot {
    page_index: usize,
    entry: usize,
    stack_index: usize,
    page_opacity: f32,
}

fn spawn_button(
    commands: &mut Commands,
    area: Entity,
    slot: ButtonSlot,
    button: &QuickAction,
    view: &HudView,
) {
    let flashing = view.hud.flash_action_entry == Some(slot.entry);
    let opacity = (slot.page_opacity * button.opacity).clamp(0.05, 1.0);
    let size = Vec2::new(button.width.max(40.0), button.height.max(20.0));
    let fill = if flashing {
        Color::srgba(0.38, 0.62, 0.95, 0.90)
    } else {
        parse_hex_color(&button.color, opacity * 0.85)
    };
    let text_color = HUD_TEXT.with_alpha(if flashing { 1.0 } else { opacity });
    let border = HUD_BADGE_BORDER.with_alpha(opacity);

    let row = spawn_child(
        commands,
        area,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                right: {Val::Px(-button.offset_x)},
                bottom: {Val::Px(slot.stack_index as f32 * BUTTON_STACK_STEP + button.offset_y)},
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: {Val::Px(5.)},
            }
        },
    );
    spawn_binding_badge(commands, row, &button.key, view);

    let frame = spawn_child(
        commands,
        row,
        bsn! {
            Node {
                position_type: PositionType::Relative,
                width: {Val::Px(size.x)}, height: {Val::Px(size.y)},
            }
        },
    );
    let (set, entry) = (slot.page_index, slot.entry);
    let rotation = UiTransform::from_rotation(Rot2::degrees(button.rotation));
    let body = spawn_child(
        commands,
        frame,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {Val::Px(0.)}, top: {Val::Px(0.)},
                width: {Val::Px(size.x)}, height: {Val::Px(size.y)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({fill})
            BorderColor::all(border)
            ~{rotation}
            Button
        },
    );
    commands.entity(body).insert(WheelHudButton {
        action: WheelHudAction::SelectAction { set, entry },
        base: fill,
    });
    spawn_child(commands, body, text_label(&button.name, 10., text_color));

    if view.hud.editor_open {
        spawn_edge_affordances(
            commands,
            frame,
            size,
            &view.asset_server,
            [
                WheelHudAction::MoveAction { set, entry },
                WheelHudAction::RotateAction {
                    set,
                    entry,
                    delta: 15.0,
                },
                WheelHudAction::DeleteAction { set, entry },
                WheelHudAction::EditAction { set, entry },
            ],
        );
    }
}

/// Shows a controller glyph for `GP:` bindings, otherwise a bordered key badge.
fn spawn_binding_badge(commands: &mut Commands, row: Entity, key: &str, view: &HudView) {
    if key.is_empty() {
        return;
    }
    if let Some(glyph) = view.gamepad_glyph(key) {
        commands.spawn((image_icon(glyph, 22., Color::WHITE), ChildOf(row)));
        return;
    }
    let badge = spawn_child(
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
    let label = key.strip_prefix("GP:").unwrap_or(key);
    spawn_child(commands, badge, text_label(label, 8., HUD_DIM));
}

/// Ends the dry-run flash of a triggered button.
pub(crate) fn tick_dry_run_flash(time: Res<Time>, mut hud: ResMut<WheelHudState>) {
    if hud.flash_action_entry.is_some() {
        hud.flash_action_ttl -= time.delta_secs();
        if hud.flash_action_ttl <= 0.0 {
            hud.flash_action_entry = None;
        }
    }
}

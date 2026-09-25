//! Page scene: a page is a stack of component slots.

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::scene::prelude::Scene;
use bevy::ui_widgets::Button;

use crate::button::Buttons;
use crate::hud::slot::HudSlot;
use crate::page_switch::PageSwitches;
use crate::radial_menu_set::RadialMenuSets;
use crate::widgets::{
    editor_icon_path, hud_clickable, hud_layer, image_icon, spawn_child, text_label,
};
use crate::{
    HudView, WheelHudAction, WheelHudState, HUD_BLUE, HUD_DIM, HUD_DIMMER, HUD_PANEL_CARD,
    HUD_SIDEBAR_BORDER, HUD_TEXT,
};

/// A HUD page, identified by its index in the document.
#[derive(Component, Default, Clone, Copy)]
#[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
pub struct Page(pub usize);

/// Slot for the page's background image.
#[derive(Component, Default, Clone, Copy)]
#[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
pub struct PageBackground(pub usize);

/// Slot for the page selector bar.
#[derive(Component, Default, Clone, Copy)]
#[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
pub struct PageTabs;

impl HudSlot for PageTabs {
    type Key = usize;

    fn key(&self, hud: &WheelHudState) -> usize {
        hud.active_set
    }
}

/// Page `index`, bottom layer first.
pub(crate) fn page(index: usize) -> impl Scene {
    bsn! {
        Page(index)
        Children [
            PageBackground(index)
            -- RadialMenuSets(index)
            -- Buttons(index)
            -- PageSwitches(index)
        ]
    }
}

/// Shown instead of a page when the document has no enabled page.
pub(crate) fn no_page() -> impl Scene {
    bsn! {
        Node { position_type: PositionType::Absolute, left: { Val::Px(14.) }, bottom: { Val::Px(14.) } }
        Children [ @text_label("No pages — open the editor to add one.", 12., HUD_DIMMER) ]
    }
}

pub(crate) fn fill_page_background(
    add: On<Add<PageBackground>>,
    slots: Query<&PageBackground>,
    view: HudView,
    mut commands: Commands,
) {
    let Ok(&PageBackground(index)) = slots.get(add.entity) else {
        return;
    };
    let Some(page) = view.cfg.sets.get(index).filter(|p| !p.bg_image.is_empty()) else {
        return;
    };
    commands.spawn((
        crate::widgets::hud_layer(),
        ImageNode {
            image: view.asset_server.load(page.bg_image.clone()),
            color: Color::WHITE.with_alpha(page.bg_image_opacity),
            ..default()
        },
        ChildOf(add.entity),
    ));
}

pub(crate) fn fill_page_tabs(add: On<Add<PageTabs>>, view: HudView, mut commands: Commands) {
    if !view.cfg.show_set_bar {
        return;
    }
    let (cfg, active) = (&view.cfg, view.hud.active_set);
    let bar = spawn_child(
        &mut commands,
        add.entity,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(12.)}, left: {Val::Px(0.)}, right: {Val::Px(0.)},
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            Pickable::IGNORE
        },
    );

    let prev = active.saturating_sub(1);
    spawn_page_arrow(
        &mut commands,
        bar,
        &view,
        prev,
        "cil-chevron-left",
        &cfg.prev_set_key,
        true,
    );
    for (i, page) in cfg.sets.iter().enumerate() {
        let (bg, text, border) = if i == active {
            (Color::srgba(0.38, 0.62, 0.95, 0.20), HUD_TEXT, HUD_BLUE)
        } else {
            (HUD_PANEL_CARD, HUD_DIM, HUD_SIDEBAR_BORDER)
        };
        let tab = hud_clickable(
            &mut commands,
            bar,
            bsn! {
                Node {
                    padding: {UiRect::axes(Val::Px(14.), Val::Px(7.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                }
                BorderColor::all(border)
                BackgroundColor({bg})
                Button
            },
            WheelHudAction::SetActiveSet(i),
            bg,
        );
        spawn_child(&mut commands, tab, text_label(&page.name, 11., text));
    }
    let next = (active + 1).min(cfg.sets.len().saturating_sub(1));
    spawn_page_arrow(
        &mut commands,
        bar,
        &view,
        next,
        "cil-chevron-right",
        &cfg.next_set_key,
        false,
    );
}

fn spawn_page_arrow(
    commands: &mut Commands,
    bar: Entity,
    view: &HudView,
    target: usize,
    icon: &str,
    binding: &str,
    left: bool,
) {
    let radius = if left {
        BorderRadius::left(Val::Px(6.))
    } else {
        BorderRadius::right(Val::Px(6.))
    };
    let arrow = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                width: {Val::Px(28.)}, height: {Val::Px(32.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {radius},
            }
            BorderColor::all(HUD_SIDEBAR_BORDER)
            BackgroundColor({HUD_PANEL_CARD})
            Button
        },
        WheelHudAction::SetActiveSet(target),
        HUD_PANEL_CARD,
    );
    let chevron = view.asset_server.load(editor_icon_path(icon));
    commands.spawn((image_icon(chevron, 16., HUD_DIM), ChildOf(arrow)));
    if let Some(glyph) = view.gamepad_glyph(binding) {
        commands.spawn((image_icon(glyph, 18., Color::WHITE), ChildOf(arrow)));
    }
}

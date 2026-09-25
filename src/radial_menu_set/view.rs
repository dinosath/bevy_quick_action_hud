//! The [`RadialMenuSets`] slot: the active radial-menu set of a page.

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::scene::prelude::Scene;

use crate::editor::overlays::decorate_radial_menu;
use crate::radial_menu::widget::{radial_menu, resolved_menu};
use crate::widgets::{hud_layer, spawn_child, text_label};
use crate::{HudRadialMenu, HudView, HUD_DIMMER};

/// Page, entry, and menu index identifying one radial menu on the HUD.
pub(crate) type RadialMenuRef = (usize, usize, Option<usize>);

/// Slot holding the active radial-menu set of page `.0`.
#[derive(Component, Default, Clone, Copy)]
#[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
pub struct RadialMenuSets(pub usize);

pub(crate) fn fill_radial_menu_sets(
    add: On<Add<RadialMenuSets>>,
    slots: Query<&RadialMenuSets>,
    view: HudView,
    mut commands: Commands,
) {
    let Ok(&RadialMenuSets(page)) = slots.get(add.entity) else {
        return;
    };
    let slot = add.entity;
    let active = view
        .cfg
        .sets
        .get(page)
        .and_then(|p| p.radial_menu_set(view.hud.active_wheel_entry));
    let menu_index = active.map_or(0, |(_, set)| {
        view.hud
            .active_wheel_index
            .min(set.radial_menu_count().saturating_sub(1))
    });
    let Some(((entry, _), menu)) =
        active.and_then(|(entry, set)| Some(((entry, set), resolved_menu(set, menu_index)?)))
    else {
        let hint = text_label("No radial menus in this set.", 11., HUD_DIMMER);
        spawn_child(&mut commands, slot, hint);
        return;
    };

    let menu_ref: RadialMenuRef = (page, entry, Some(menu_index));
    let highlighted = sector_in(view.hud.highlighted, menu_ref);
    let menu = radial_menu(&menu, highlighted);
    // The editor decorates the menu once its whole hierarchy has spawned.
    let decorate: Box<dyn Scene> = if view.hud.editor_open {
        Box::new(bsn! { on(decorate_radial_menu) })
    } else {
        Box::new(bsn! {})
    };
    spawn_child(
        &mut commands,
        slot,
        bsn! {
            @{menu}
            HudRadialMenu(menu_ref)
            @{decorate}
        },
    );
}

/// Returns the sector index of `id` when it belongs to `menu`.
pub(crate) fn sector_in(
    id: Option<(usize, usize, Option<usize>, usize)>,
    menu: RadialMenuRef,
) -> Option<usize> {
    id.filter(|(page, entry, wheel, _)| (*page, *entry, *wheel) == menu)
        .map(|(.., sector)| sector)
}

//! The radial-menu widget: one BSN scene per menu, one nested scene per sector.
//!
//! Consumers find the parts they decorate through [`SectorIndex`],
//! [`RadialMenuHub`], and [`RadialMenuCenter`] instead of returned entity handles.

use std::f32::consts::TAU;

use bevy::picking::hover::PickingInteraction;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::scene::prelude::{Scene, SceneList};
use bevy::scene::EntityScene;
use bevy::ui::{AngularColorStop, BackgroundGradient, BorderGradient, ConicGradient, UiPosition};
use bevy::ui_widgets::Button;

use super::geometry::{
    conic_sector, radial_menu_hub_radius, rendered_sector_outer_radius, slice_angles, SectorPanel,
};
use super::model::{RadialMenu, Sector};
use super::style::*;
use crate::radial_menu_set::{wheelset_visuals, RadialMenuSet};
use crate::widgets::text_label;

/// Index of a sector entity within its radial menu.
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectorIndex(pub usize);

/// Zero-size origin of a radial menu; menu-local overlays are spawned under it.
#[derive(Component, Default, Clone, Copy)]
pub struct RadialMenuHub;

/// Center hub ring of a radial menu.
#[derive(Component, Default, Clone, Copy)]
pub struct RadialMenuCenter;

/// Returns menu `index` of `menu_set` (clamped) with the set's shared visuals applied.
pub(crate) fn resolved_menu(menu_set: &RadialMenuSet, index: usize) -> Option<RadialMenu> {
    let index = index.min(menu_set.radial_menu_count().saturating_sub(1));
    let mut menu = menu_set.radial_menu(index)?.clone();
    wheelset_visuals(menu_set).apply_to(&mut menu);
    Some(menu)
}

/// One radial menu, drawn bottom to top, with `highlighted` sector selected.
pub(crate) fn radial_menu(menu: &RadialMenu, highlighted: Option<usize>) -> impl Scene {
    let colors = RadialMenuColors::new(menu);
    let selected = highlighted.filter(|&index| index < menu.slots.len());
    let anchor = radial_menu_anchor();
    let hub = radial_menu_hub(menu);
    let background = radial_menu_background(menu.outer_radius, colors.background);
    let ring = radial_menu_sector_ring(menu, selected);
    let sectors: Vec<Box<dyn SceneList>> = menu
        .slots
        .iter()
        .enumerate()
        .map(|(index, sector)| {
            boxed(radial_menu_sector_entity(
                menu,
                index,
                sector,
                selected == Some(index),
            ))
        })
        .collect();
    let outer_ring = radial_menu_outer_ring(
        menu.outer_radius,
        colors.outer_border,
        colors.outer_border_width,
    );
    // Drawn above the outer ring so the selected sector overlaps it.
    let selected_wedge = selected.map(|index| {
        EntityScene(radial_menu_selected_wedge(
            menu,
            index,
            colors.selected_fill,
            colors.highlight,
        ))
    });
    let dividers = radial_menu_dividers(menu, highlighted, colors.highlight);
    let center = radial_menu_center(menu, selected, &colors);
    bsn! {
        @{anchor}
        Children [
            RadialMenuHub
            @{hub}
            Children [
                @{background}
                -- {ring}
                -- {sectors}
                -- @{outer_ring}
                -- {selected_wedge}
                -- {dividers}
                -- @{center}
            ]
        ]
    }
}

fn boxed(scene: impl Scene) -> Box<dyn SceneList> {
    Box::new(EntityScene(scene))
}

/// Declares the flex-layout anchor used for a centered radial menu.
fn radial_menu_anchor() -> impl Scene {
    bsn! {
        Node {
            width: { Val::Percent(100.) },
            height: { Val::Percent(100.) },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

/// Declares the zero-size positioning origin that carries the menu transform.
fn radial_menu_hub(menu: &RadialMenu) -> impl Scene {
    let translation = Val2::px(menu.offset_x, -menu.offset_y);
    let rotation = Rot2::degrees(menu.rotation);
    bsn! {
        Node {
            width: { Val::Px(0.) },
            height: { Val::Px(0.) },
        }
        UiTransform {
            translation: { translation },
            scale: { Vec2::ONE },
            rotation: { rotation },
        }
    }
}

/// Declares the circular background surface for a radial menu.
fn radial_menu_background(outer_radius: f32, color: Color) -> impl Scene {
    let radius = outer_radius + RADIAL_MENU_BACKGROUND_BLEED;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(-radius) },
            top: { Val::Px(-radius) },
            width: { Val::Px(radius * 2.0) },
            height: { Val::Px(radius * 2.0) },
            border_radius: { BorderRadius::all(Val::Px(radius)) },
        }
        BackgroundColor({ color })
    }
}

/// Declares the outer border ring for a radial menu.
fn radial_menu_outer_ring(outer_radius: f32, color: Color, border_width: f32) -> impl Scene {
    let radius = outer_radius + RADIAL_MENU_OUTER_RING_OUTSET;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(-radius) },
            top: { Val::Px(-radius) },
            width: { Val::Px(radius * 2.0) },
            height: { Val::Px(radius * 2.0) },
            border_radius: { BorderRadius::all(Val::Px(radius)) },
            border: { UiRect::all(Val::Px(border_width)) },
        }
        BackgroundColor({ Color::NONE })
        BorderColor::all(color)
    }
}

/// One interactive sector: an invisible rotated hit panel holding its upright content.
fn radial_menu_sector_entity(
    menu: &RadialMenu,
    index: usize,
    sector: &Sector,
    selected: bool,
) -> impl Scene {
    let panel = SectorPanel::new(menu, index, selected);
    let SectorPanel {
        center,
        width,
        height,
        rotation,
    } = panel;
    let content = radial_menu_sector_content(menu, index, sector, panel, selected);
    bsn! {
        SectorIndex(index)
        Button
        PickingInteraction::None
        Node {
            position_type: PositionType::Absolute,
            left:   { Val::Px(center.x - width / 2.0) },
            top:    { Val::Px(-center.y - height / 2.0) },
            width:  { Val::Px(width) },
            height: { Val::Px(height) },
        }
        // Native UI resolves `UiTransform` during layout; `Transform` would not rotate UI.
        UiTransform::from_rotation(Rot2::radians(rotation))
        Children [ @{content} ]
    }
}

/// A conic gradient painting `color` over each `(offset, span)` range after `start`.
fn sector_gradient(start: f32, ranges: &[(f32, f32)], color: Color) -> ConicGradient {
    let mut stops = vec![AngularColorStop::new(Color::NONE, 0.)];
    for &(offset, span) in ranges {
        let end = (offset + span).min(TAU);
        stops.extend([
            AngularColorStop::new(Color::NONE, offset),
            AngularColorStop::new(color, offset),
            AngularColorStop::new(color, end),
            AngularColorStop::new(Color::NONE, end),
        ]);
    }
    stops.push(AngularColorStop::new(Color::NONE, TAU));
    ConicGradient::new(UiPosition::CENTER, stops).with_start(start)
}

/// Circle node of `radius` centered on the hub; the hub drawn later covers its inner part.
fn radial_menu_disc(radius: f32, border: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(-radius),
        top: Val::Px(-radius),
        width: Val::Px(radius * 2.0),
        height: Val::Px(radius * 2.0),
        border_radius: BorderRadius::all(Val::Px(radius)),
        border: UiRect::all(Val::Px(border)),
        ..default()
    }
}

/// Every unselected sector's annular wedge, painted by a single conic gradient.
fn radial_menu_sector_ring(
    menu: &RadialMenu,
    selected: Option<usize>,
) -> Option<EntityScene<impl Scene>> {
    let (start, _) = conic_sector(menu, 0);
    let mut ranges: Vec<(f32, f32)> = (0..menu.slots.len())
        .filter(|&index| selected != Some(index))
        .map(|index| {
            let (begin, span) = conic_sector(menu, index);
            ((begin - start).rem_euclid(TAU), span)
        })
        .collect();
    if ranges.is_empty() {
        return None;
    }
    ranges.sort_by(|a, b| a.0.total_cmp(&b.0));
    let node = radial_menu_disc(rendered_sector_outer_radius(menu, false), 0.);
    let gradient = BackgroundGradient::from(sector_gradient(start, &ranges, RADIAL_MENU_SECTOR));
    Some(EntityScene(bsn! { ~{node} ~{gradient} Pickable::IGNORE }))
}

/// The selected sector's wedge: `fill` inside, with a `highlight` outline along its outer arc.
fn radial_menu_selected_wedge(
    menu: &RadialMenu,
    index: usize,
    fill: Color,
    highlight: Color,
) -> impl Scene {
    let (begin, span) = conic_sector(menu, index);
    let radius = rendered_sector_outer_radius(menu, true);
    let node = radial_menu_disc(radius, RADIAL_MENU_SELECTED_OUTLINE_WIDTH);
    let background = BackgroundGradient::from(sector_gradient(begin, &[(0., span)], fill));
    let border = BorderGradient::from(sector_gradient(begin, &[(0., span)], highlight));
    bsn! { ~{node} ~{background} ~{border} Pickable::IGNORE }
}

/// Upright label/icon layer inside a rotated sector panel.
fn radial_menu_sector_content(
    menu: &RadialMenu,
    index: usize,
    sector: &Sector,
    panel: SectorPanel,
    selected: bool,
) -> impl Scene {
    let label_size = ((menu.outer_radius - menu.inner_radius) * RADIAL_MENU_LABEL_SIZE_RATIO)
        .clamp(
            RADIAL_MENU_LABEL_SIZE_RANGE.0,
            RADIAL_MENU_LABEL_SIZE_RANGE.1,
        );
    let mut items: Vec<Box<dyn SceneList>> = Vec::new();
    if menu.show_labels {
        let name = sector.name.to_uppercase();
        items.push(boxed(text_label(&name, label_size, RADIAL_MENU_LABEL)));
    }
    if menu.show_icon && !sector.icon.is_empty() {
        let size = (panel.height * RADIAL_MENU_ICON_SIZE_RATIO)
            .clamp(RADIAL_MENU_ICON_SIZE_RANGE.0, RADIAL_MENU_ICON_SIZE_RANGE.1);
        items.push(boxed(radial_menu_icon(&sector.icon, size, index)));
    } else if menu.show_labels {
        items.push(boxed(
            bsn! { Node { width: { Val::Px(4.) }, height: { Val::Px(4.) } } },
        ));
    }
    let raised: Box<dyn Scene> = if selected {
        Box::new(bsn! { GlobalZIndex(1) })
    } else {
        Box::new(bsn! {})
    };
    // The panel runs beneath the hub, so shift the content back into the
    // visible band and counter-rotate it to stay upright.
    let translation = Val2::px(0., -menu.inner_radius * 0.5);
    let rotation = Rot2::radians(-panel.rotation);
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(0.) },
            top: { Val::Px(0.) },
            width: { Val::Percent(100.) },
            height: { Val::Percent(100.) },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            padding: { UiRect::all(Val::Px(RADIAL_MENU_SECTOR_CONTENT_PADDING)) },
        }
        UiTransform {
            translation: { translation },
            scale: { Vec2::ONE },
            rotation: { rotation },
        }
        @{raised}
        Children [{ items }]
    }
}

/// Straight dividers on every sector boundary; boundaries of the selected sector are highlighted.
fn radial_menu_dividers(
    menu: &RadialMenu,
    highlighted: Option<usize>,
    highlight: Color,
) -> Vec<Box<dyn SceneList>> {
    let count = menu.slots.len().max(1);
    (0..count)
        .map(|index| {
            let (angle, _) = slice_angles(menu, index);
            let previous = (index + count - 1) % count;
            let on_selected = highlighted.is_some_and(|s| s == index || s == previous);
            let color = if on_selected {
                highlight
            } else {
                RADIAL_MENU_DIVIDER
            };
            let outer_radius = rendered_sector_outer_radius(menu, on_selected);
            let width = RADIAL_MENU_DIVIDER_WIDTH;
            let length = (outer_radius - menu.inner_radius).max(0.0);
            let center_radius = (menu.inner_radius + outer_radius) * 0.5;
            let center = Vec2::new(angle.cos() * center_radius, angle.sin() * center_radius);
            boxed(bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: { Val::Px(center.x - width * 0.5) },
                    top: { Val::Px(-center.y - length * 0.5) },
                    width: { Val::Px(width) },
                    height: { Val::Px(length) },
                }
                BackgroundColor({ color })
                UiTransform::from_rotation(Rot2::radians(std::f32::consts::FRAC_PI_2 - angle))
            })
        })
        .collect()
}

/// The hub ring; shows the selected sector's icon and name.
fn radial_menu_center(
    menu: &RadialMenu,
    selected: Option<usize>,
    colors: &RadialMenuColors,
) -> impl Scene {
    let radius = radial_menu_hub_radius(menu);
    let (background, ring_color, ring_width) =
        (colors.hub, colors.hub_border, colors.hub_border_width);
    let info: Vec<Box<dyn SceneList>> = selected
        .map(|index| boxed(radial_menu_hub_info(&menu.slots[index], index, radius)))
        .into_iter()
        .collect();
    bsn! {
        RadialMenuCenter
        Node {
            position_type: PositionType::Absolute,
            left: { Val::Px(-radius) },
            top: { Val::Px(-radius) },
            width: { Val::Px(radius * 2.0) },
            height: { Val::Px(radius * 2.0) },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: { BorderRadius::all(Val::Px(radius)) },
            border: { UiRect::all(Val::Px(ring_width)) },
        }
        BackgroundColor({ background })
        BorderColor::all(ring_color)
        Children [{ info }]
    }
}

fn radial_menu_hub_info(sector: &Sector, index: usize, hub_radius: f32) -> impl Scene {
    let mut items: Vec<Box<dyn SceneList>> = Vec::new();
    if !sector.icon.is_empty() {
        let size = (hub_radius * 0.58).clamp(24.0, 42.0);
        items.push(boxed(radial_menu_icon(&sector.icon, size, index)));
    }
    if !sector.name.is_empty() {
        let size = (hub_radius * 0.22).clamp(7.0, 10.0);
        items.push(boxed(text_label(&sector.name, size, RADIAL_MENU_HUB_LABEL)));
    }
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: { Val::Px(2.) },
        }
        Children [{ items }]
    }
}

/// Declares a round icon badge; its fill cycles with the sector index.
fn radial_menu_icon(icon: &str, size: f32, index: usize) -> impl Scene {
    let color = RADIAL_MENU_ICON_COLORS[index % RADIAL_MENU_ICON_COLORS.len()];
    let glyph = text_label(icon, size * 0.48, Color::WHITE);
    bsn! {
        Node {
            width: { Val::Px(size) },
            height: { Val::Px(size) },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: { BorderRadius::all(Val::Px(size * 0.5)) },
        }
        BackgroundColor({ color })
        Children [ @{glyph} ]
    }
}

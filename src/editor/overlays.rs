//! Edit-mode affordances drawn on top of HUD components.

use bevy::prelude::*;
use bevy::scene::Ready;
use bevy::ui_widgets::Button;

use crate::radial_menu::style::radial_menu_hub_color;
use crate::radial_menu::widget::{resolved_menu, RadialMenuCenter, RadialMenuHub};
use crate::radial_menu_set::{sector_in, RadialMenuRef};
use crate::widgets::{editor_icon_path, hud_clickable, image_icon, spawn_child, text_label};
use crate::{
    hud_control_owner, sector_anchor, HudContextControl, HudControlOwner, HudRadialMenu, HudView,
    RadialMenu, SectorAnchor, SegmentInsertSide, SetEntry, WheelHudAction, WheelHudButton,
    WheelHudState, HUD_AMBER, HUD_BADGE_BORDER, HUD_DIM, HUD_PANEL_CARD, HUD_TEXT,
};

use super::components::{spawn_segment_editor_card, spawn_wheel_settings_card};

const BADGE: f32 = 22.;

/// Shows edit badges only for the selected component while the editor is open.
pub(crate) fn context_visibility(
    hud: Res<WheelHudState>,
    mut controls: Query<(&HudContextControl, &mut Visibility)>,
) {
    for (control, mut visibility) in &mut controls {
        let selected = match control.owner {
            HudControlOwner::Action(set, entry) => hud.selected_action == Some((set, entry)),
            HudControlOwner::Wheel(set, entry, wheel) => {
                hud.selected_wheel == Some((set, entry, wheel))
            }
            HudControlOwner::HudSwitch(set, entry) => hud.selected_hud_switch == Some((set, entry)),
        };
        let next = if hud.editor_open && selected {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

/// Spawns move/transform/delete/settings badges on the top, right, bottom,
/// and left edges of a rectangular component of `size`.
pub(crate) fn spawn_edge_affordances(
    commands: &mut Commands,
    parent: Entity,
    size: Vec2,
    asset_server: &AssetServer,
    [top, right, bottom, left]: [WheelHudAction; 4],
) {
    let half = BADGE * 0.5;
    for (action, icon, position, color) in [
        (
            top,
            "cil-camera-control",
            Vec2::new(size.x * 0.5 - half, -BADGE - 2.),
            HUD_TEXT,
        ),
        (
            right,
            "cil-aperture",
            Vec2::new(size.x + 2., size.y * 0.5 - half),
            HUD_TEXT,
        ),
        (
            bottom,
            "cil-trash",
            Vec2::new(size.x * 0.5 - half, size.y + 2.),
            HUD_AMBER,
        ),
        (
            left,
            "cil-cog",
            Vec2::new(-BADGE - 2., size.y * 0.5 - half),
            HUD_TEXT,
        ),
    ] {
        let badge = spawn_badge(commands, parent, position, action, color, false);
        let image = asset_server.load(editor_icon_path(icon));
        commands.spawn((image_icon(image, 14., color), ChildOf(badge)));
    }
}

/// Adds edit controls to a HUD radial menu once its scene has fully spawned.
pub(crate) fn decorate_radial_menu(
    ready: On<Ready>,
    menus: Query<&HudRadialMenu>,
    children: Query<&Children>,
    hubs: Query<(), With<RadialMenuHub>>,
    centers: Query<(), With<RadialMenuCenter>>,
    view: HudView,
    mut commands: Commands,
) {
    let root = ready.entity;
    let Ok(&HudRadialMenu(menu_ref)) = menus.get(root) else {
        return;
    };
    let find = |parts: &dyn Fn(Entity) -> bool| children.iter_descendants(root).find(|&e| parts(e));
    let (Some(hub), Some(center)) = (find(&|e| hubs.contains(e)), find(&|e| centers.contains(e)))
    else {
        return;
    };
    let (page, entry, wheel) = menu_ref;
    let menu = view
        .cfg
        .sets
        .get(page)
        .and_then(|p| match p.entries.get(entry) {
            Some(SetEntry::RadialMenuSet(set)) => resolved_menu(set, wheel.unwrap_or(0)),
            _ => None,
        });
    if let Some(menu) = menu {
        spawn_radial_menu_overlay(&mut commands, hub, center, &menu, menu_ref, &view);
    }
}

/// Spawns the radial-menu edit controls, sector controls, and inspector cards.
fn spawn_radial_menu_overlay(
    commands: &mut Commands,
    hub: Entity,
    center: Entity,
    menu: &RadialMenu,
    menu_ref: RadialMenuRef,
    ctx: &HudView,
) {
    let hud = &ctx.hud;
    let (set, entry, wheel) = menu_ref;
    let highlighted = sector_in(hud.highlighted, menu_ref).filter(|&s| s < menu.slots.len());

    commands.entity(center).insert((
        WheelHudButton {
            action: WheelHudAction::SelectWheel { set, entry, wheel },
            base: radial_menu_hub_color(menu),
        },
        Button,
    ));
    if highlighted.is_some() {
        spawn_child(commands, center, text_label("▣  Apply", 9., HUD_DIM));
        spawn_child(commands, center, text_label("▣  Back", 9., HUD_DIM));
    }

    if let Some(slot) = sector_in(hud.selected_segment, menu_ref) {
        if let Some(sector) = menu.slots.get(slot) {
            spawn_segment_editor_card(
                commands,
                hub,
                sector,
                set,
                entry,
                wheel,
                slot,
                menu.outer_radius,
                hud.edit_control_focus,
            );
        }
    }
    let menu_focused = hud.selected_wheel == Some(menu_ref) || hud.hovered_wheel == Some(menu_ref);
    if hud.highlighted.is_none() && menu_focused {
        spawn_wheel_settings_card(commands, hub, menu, hud.theme_popup_open);
    }

    let ring = menu.outer_radius + 28.0;
    for (position, action, label, color) in [
        (
            Vec2::new(0., -ring),
            WheelHudAction::WheelSettings { set, entry, wheel },
            "⚙",
            HUD_TEXT,
        ),
        (
            Vec2::new(ring, 0.),
            WheelHudAction::DeleteWheel { set, entry, wheel },
            "×",
            HUD_AMBER,
        ),
        (
            Vec2::new(0., ring),
            WheelHudAction::MoveWheel { set, entry, wheel },
            "↕",
            HUD_TEXT,
        ),
        (
            Vec2::new(-ring, 0.),
            WheelHudAction::ResizeWheel {
                set,
                entry,
                wheel,
                delta: 10.0,
            },
            "⌗",
            HUD_TEXT,
        ),
    ] {
        spawn_radial_edit_button(commands, hub, position, action, label, color, false);
    }

    let Some(slot) = highlighted else {
        return;
    };
    let insert = |side| WheelHudAction::AddSegment {
        set,
        entry,
        wheel,
        side,
    };
    for (focus, anchor, action, label) in [
        (
            1,
            SectorAnchor::StartEdge,
            insert(SegmentInsertSide::Before),
            "+",
        ),
        (
            2,
            SectorAnchor::InnerArc,
            WheelHudAction::RemoveSegment {
                set,
                entry,
                wheel,
                slot,
            },
            "×",
        ),
        (
            3,
            SectorAnchor::EndEdge,
            insert(SegmentInsertSide::After),
            "+",
        ),
        (
            4,
            SectorAnchor::OuterArc,
            insert(SegmentInsertSide::Outer),
            "+",
        ),
    ] {
        spawn_radial_edit_button(
            commands,
            hub,
            sector_anchor(menu, slot, anchor),
            action,
            label,
            HUD_TEXT,
            hud.edit_control_focus == Some(focus),
        );
    }
}

/// Spawns a round text badge centred on `position` (menu-local, y up).
pub(crate) fn spawn_radial_edit_button(
    commands: &mut Commands,
    parent: Entity,
    position: Vec2,
    action: WheelHudAction,
    label: &str,
    color: Color,
    focused: bool,
) {
    let top_left = Vec2::new(position.x, -position.y) - Vec2::splat(BADGE * 0.5);
    let badge = spawn_badge(commands, parent, top_left, action, color, focused);
    spawn_child(commands, badge, text_label(label, 15., color));
}

/// Round, context-visible edit badge at `top_left` within `parent`.
fn spawn_badge(
    commands: &mut Commands,
    parent: Entity,
    top_left: Vec2,
    action: WheelHudAction,
    accent: Color,
    focused: bool,
) -> Entity {
    let owner = hud_control_owner(&action);
    let (fill, border) = match (focused, accent == HUD_AMBER) {
        (true, _) => (HUD_AMBER, HUD_TEXT),
        (false, true) => (HUD_PANEL_CARD, HUD_AMBER),
        (false, false) => (HUD_PANEL_CARD, HUD_BADGE_BORDER),
    };
    let badge = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {Val::Px(top_left.x)},
                top: {Val::Px(top_left.y)},
                width: {Val::Px(BADGE)}, height: {Val::Px(BADGE)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(BADGE * 0.5))},
            }
            BackgroundColor({fill})
            BorderColor::all(border)
            Button
        },
        action,
        HUD_PANEL_CARD,
    );
    if let Some(owner) = owner {
        commands
            .entity(badge)
            .insert((HudContextControl { owner }, Visibility::Hidden));
    }
    badge
}

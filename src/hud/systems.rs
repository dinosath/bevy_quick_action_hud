//! HUD lifecycle systems: rebuild on document/state change and button feedback.

use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

use super::{hud, WheelHudButton, WheelHudRoot, WheelHudState, HUD_BG};
use crate::page::Page;
use crate::widgets::parse_hex_color;
use crate::{enabled_hud_pages, QuickActionConfig};

pub(crate) fn button_feedback(
    mut buttons: Query<
        (&WheelHudButton, &PickingInteraction, &mut BackgroundColor),
        Changed<PickingInteraction>,
    >,
) {
    for (button, interaction, mut background) in &mut buttons {
        let next = match interaction {
            PickingInteraction::Hovered => BackgroundColor(Color::srgba(1., 1., 1., 0.05)),
            PickingInteraction::Pressed => BackgroundColor(Color::srgba(0.38, 0.62, 0.95, 0.16)),
            PickingInteraction::None => BackgroundColor(button.base),
        };
        if *background != next {
            *background = next;
        }
    }
}

/// Respawns the HUD scene when the config changes and keeps the active page valid.
pub(crate) fn rebuild_hud(
    mut commands: Commands,
    mut hud_state: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    old_roots: Query<Entity, With<WheelHudRoot>>,
) {
    let pages = enabled_hud_pages(&cfg);
    if (cfg.is_changed() || hud_state.is_changed()) && !pages.contains(&hud_state.active_set) {
        if let Some(&page) = pages.first() {
            hud_state.active_set = page;
        }
    }
    if !cfg.is_changed() && !old_roots.is_empty() {
        return;
    }
    debug!("[hud] rebuild: {} pages", pages.len());

    for root in &old_roots {
        commands.entity(root).despawn();
    }
    let background = if cfg.hud_bg_color.is_empty() {
        HUD_BG.with_alpha(cfg.hud_bg_opacity)
    } else {
        parse_hex_color(&cfg.hud_bg_color, cfg.hud_bg_opacity)
    };
    commands
        .spawn_scene(hud(&pages))
        .insert(BackgroundColor(background));
}

/// Shows the HUD while it is open, and only its active page.
pub(crate) fn show_hud(
    hud_state: Res<WheelHudState>,
    mut roots: Query<&mut Node, With<WheelHudRoot>>,
    mut pages: Query<(&Page, &mut Node), Without<WheelHudRoot>>,
    added: Query<(), Added<WheelHudRoot>>,
) {
    if !hud_state.is_changed() && added.is_empty() {
        return;
    }
    for mut node in &mut roots {
        set_shown(&mut node, hud_state.open);
    }
    for (&Page(index), mut node) in &mut pages {
        set_shown(&mut node, index == hud_state.active_set);
    }
}

/// `Display::None` also removes the subtree from layout and picking.
fn set_shown(node: &mut Mut<Node>, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}

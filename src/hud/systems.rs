//! HUD lifecycle systems: rebuild on document/state change and button feedback.

use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

use super::{hud, WheelHudButton, WheelHudRoot, WheelHudState, HUD_BG};
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

/// Respawns the HUD scene when [`WheelHudState::dirty`] is set.
pub(crate) fn rebuild_hud(
    mut commands: Commands,
    mut hud_state: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    old_roots: Query<Entity, With<WheelHudRoot>>,
) {
    if !hud_state.dirty {
        return;
    }
    hud_state.dirty = false;
    if cfg
        .sets
        .get(hud_state.active_set)
        .is_none_or(|page| !page.enabled)
    {
        if let Some(page) = enabled_hud_pages(&cfg).first().copied() {
            hud_state.active_set = page;
        }
    }
    if !cfg.sets.is_empty() && hud_state.active_set >= cfg.sets.len() {
        hud_state.active_set = cfg.sets.len() - 1;
    }
    debug!(
        "[hud] rebuild: open={} editor_open={} active_set={}",
        hud_state.open, hud_state.editor_open, hud_state.active_set
    );

    for root in &old_roots {
        commands.entity(root).despawn();
    }
    if !hud_state.open {
        return;
    }
    let background = if cfg.hud_bg_color.is_empty() {
        HUD_BG.with_alpha(cfg.hud_bg_opacity)
    } else {
        parse_hex_color(&cfg.hud_bg_color, cfg.hud_bg_opacity)
    };
    let active_page = cfg
        .sets
        .get(hud_state.active_set)
        .filter(|page| page.enabled)
        .map(|_| hud_state.active_set);
    commands
        .spawn_scene(hud(active_page))
        .insert(BackgroundColor(background));
}

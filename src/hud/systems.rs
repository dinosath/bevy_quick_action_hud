//! Runtime HUD systems.
//!
//! BSN owns stable hierarchy; these systems only update runtime interaction
//! state and visual components in place.

use super::{HudContextControl, HudControlOwner, WheelHudButton, WheelHudState};
use bevy::prelude::*;

pub(crate) fn button_feedback(
    mut buttons: Query<(&WheelHudButton, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (button, interaction, mut background) in &mut buttons {
        let next = match interaction {
            Interaction::Hovered => BackgroundColor(Color::srgba(1., 1., 1., 0.05)),
            Interaction::Pressed => BackgroundColor(Color::srgba(0.38, 0.62, 0.95, 0.16)),
            Interaction::None => BackgroundColor(button.base),
        };
        if *background != next {
            *background = next;
        }
    }
}

pub(crate) fn context_visibility(
    hud: Res<WheelHudState>,
    mut controls: Query<(&HudContextControl, &mut Visibility)>,
) {
    for (control, mut visibility) in &mut controls {
        let visible = hud.editor_open
            && match control.owner {
                HudControlOwner::Action(set, entry) => hud.selected_action == Some((set, entry)),
                HudControlOwner::Wheel(set, entry, wheel) => {
                    hud.selected_wheel == Some((set, entry, wheel))
                }
                HudControlOwner::HudSwitch(set, entry) => {
                    hud.selected_hud_switch == Some((set, entry))
                }
            };
        let next = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

pub(crate) fn tick_dry_run_flash(time: Res<Time>, mut hud: ResMut<WheelHudState>) {
    if hud.flash_action_entry.is_some() && !hud.dirty {
        hud.flash_action_ttl -= time.delta_secs();
        if hud.flash_action_ttl <= 0.0 {
            hud.flash_action_entry = None;
            hud.dirty = true;
        }
    }
}

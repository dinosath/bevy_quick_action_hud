//! Runtime systems for switching menus inside a radial-menu set.

use bevy::prelude::*;

use super::components::RadialMenuSetState;
use super::messages::WheelSwitched;

/// Cycles the active radial menu when the configured shoulder buttons are pressed.
pub fn update_wheel_set(
    gamepads: Query<&Gamepad>,
    mut query: Query<(Entity, &mut RadialMenuSetState)>,
    mut switched: MessageWriter<WheelSwitched>,
) {
    for (entity, mut set) in &mut query {
        if set.count < 2 {
            continue;
        }
        for gamepad in &gamepads {
            if gamepad.just_pressed(set.next_button) {
                let previous = set.active;
                set.active = (set.active + 1) % set.count;
                switched.write(WheelSwitched {
                    previous,
                    current: set.active,
                    menu_entity: entity,
                });
            }
            if gamepad.just_pressed(set.prev_button) {
                let previous = set.active;
                set.active = (set.active + set.count - 1) % set.count;
                switched.write(WheelSwitched {
                    previous,
                    current: set.active,
                    menu_entity: entity,
                });
            }
        }
    }
}

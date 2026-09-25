//! Keyed HUD slots: a slot respawns its content only when the state it shows changes.
//!
//! Config edits rebuild the whole HUD (see [`super::rebuild_hud`]); runtime and
//! editor state reaches each slot through its [`HudSlot::key`].

use bevy::prelude::*;

use super::{rebuild_hud, WheelHudState};

/// A slot whose content is a function of the config and [`HudSlot::key`].
pub(crate) trait HudSlot: Component + Clone {
    type Key: PartialEq + Send + Sync + 'static;

    /// The part of `hud` this slot's content depends on.
    fn key(&self, hud: &WheelHudState) -> Self::Key;
}

/// Systems that bring HUD slots and visibility up to date with [`WheelHudState`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HudRefresh;

#[derive(Component)]
struct ShownKey<K: Send + Sync + 'static>(K);

/// Makes slot `S` respawn its content whenever its key changes.
pub(crate) fn register<S: HudSlot>(app: &mut App) {
    app.configure_sets(Update, HudRefresh.after(rebuild_hud))
        .add_observer(record_key::<S>)
        .add_systems(Update, refresh::<S>.in_set(HudRefresh));
}

fn record_key<S: HudSlot>(
    add: On<Add<S>>,
    slots: Query<&S>,
    hud: Res<WheelHudState>,
    mut commands: Commands,
) {
    if let Ok(slot) = slots.get(add.entity) {
        commands.entity(add.entity).insert(ShownKey(slot.key(&hud)));
    }
}

fn refresh<S: HudSlot>(
    hud: Res<WheelHudState>,
    mut slots: Query<(Entity, &S, &mut ShownKey<S::Key>)>,
    mut commands: Commands,
) {
    if !hud.is_changed() {
        return;
    }
    for (entity, slot, mut shown) in &mut slots {
        let key = slot.key(&hud);
        if shown.0 == key {
            continue;
        }
        shown.0 = key;
        // Re-adding the slot runs its fill observer again.
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .remove::<S>()
            .insert(slot.clone());
    }
}

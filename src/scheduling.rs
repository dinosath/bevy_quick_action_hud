//! Named schedule ownership for the feature plugins.
//!
//! The current systems retain their proven local chains, while these sets make
//! feature boundaries explicit and provide a safe seam for the next extraction
//! into `core`, `hud`, and `editor` system modules.

use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WheelCoreSet {
    Runtime,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HudSet {
    Runtime,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum EditorSet {
    Runtime,
}

pub(crate) fn configure(app: &mut App) {
    app.configure_sets(
        Update,
        (WheelCoreSet::Runtime, HudSet::Runtime, EditorSet::Runtime),
    );
}

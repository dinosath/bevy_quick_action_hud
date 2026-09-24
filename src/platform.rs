//! Platform feature composition.
//!
//! Browser viewport/orientation support and touch gestures are independent of
//! wheel rendering and editor state. Keeping their plugin composition here
//! makes the root HUD plugin a facade rather than the owner of platform logic.

use bevy::prelude::*;

use crate::{touch, wasm};

/// Installs the platform adapters used by the HUD on desktop, mobile and WASM.
///
/// The adapters remain separate plugins so applications can add only the
/// pieces they need directly. This wrapper is the default composition used by
/// [`crate::QuickActionHudPlugin`].
pub(crate) struct PlatformSupportPlugin;

impl Plugin for PlatformSupportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            touch::TouchInteractionPlugin,
            wasm::WasmSupportPlugin,
            wasm::MobileSupportPlugin,
        ));
    }
}

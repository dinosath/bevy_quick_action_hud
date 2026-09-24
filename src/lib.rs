//! Headless wheel menu library for Bevy.
//!
//! This library provides the logic and data structures for wheel menus.
//! Rendering is left to the application.

pub mod editor;
mod hud;
mod persistence;
mod platform;
mod scheduling;
pub mod touch;
pub mod wasm;
mod wheel_core;

use hud::*;
pub use hud::{
    HudContextControl, HudControlOwner, HudSegmentSelected, WedgeMaterial, WedgeParams,
    WheelHudButton, WheelHudRoot, WheelHudSegmentHit, WheelHudState,
};
pub use wheel_core::messages::*;
pub use wheel_core::{
    resolve_input, ActiveSlotContext, GlobalBindings, InputAction, RadialMenuAudio,
    RadialMenuConfig, RadialMenuEditMode, RadialMenuHierarchy, RadialMenuHoldState,
    RadialMenuSetState, RadialMenuState, RadialMenuStyle, SectorContent, SectorCount, SectorEntity,
    WheelAction, WheelAudio, WheelEditMode, WheelHierarchy, WheelHoldState, WheelInputOverride,
    WheelMenuConfig, WheelSet, WheelSlice, WheelSliceContent, WheelSliceCount, WheelSliceLink,
    WheelState, WheelStyle,
};

use bevy::asset::embedded_asset;
use bevy::color::Alpha;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

/// Default filename for the persisted [`QuickActionConfig`].
/// Resolved relative to the process working directory (the project root when
/// running via `cargo run`).
pub const CONFIG_FILE: &str = "quickactions_config.ron";

// ─── gamepad icon set ─────────────────────────────────────────────────────────────

/// Which family of controller button icons to show in the editor UI.
///
/// Auto-detected from the connected gamepad's USB vendor/product IDs and device
/// name when a controller connects; defaults to [`GamepadIconSet::Xbox`] when no
/// controller is present or the type is unknown.
///
/// Icon assets live under `assets/icons/<set>/Default/`.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GamepadIconSet {
    /// Xbox / generic controller — A / B / X / Y / LB / RB / LT / RT …
    #[default]
    Xbox,
    /// PlayStation 4 DualShock 4 — Cross / Circle / Square / Triangle …
    PS4,
    /// PlayStation 5 DualSense — Cross / Circle / Square / Triangle …
    PS5,
    /// Nintendo Switch Pro Controller / Joy-Con — B / A / Y / X / + / − …
    Switch,
}

impl GamepadIconSet {
    /// Base asset path (relative to `assets/`) for this set's Default style.
    pub fn base_path(self) -> &'static str {
        match self {
            Self::Xbox => "icons/XGamepad/Default",
            Self::PS4 => "icons/P4Gamepad/Default",
            Self::PS5 => "icons/P5Gamepad/Default",
            Self::Switch => "icons/SGamepad/Default",
        }
    }

    /// Returns the asset path for a gamepad button label (e.g. `"LB"`, `"A"`,
    /// `"Start"`).
    ///
    /// Returns `None` when the label has no mapped asset for this set.
    pub fn icon_path(self, label: &str) -> Option<String> {
        let base = self.base_path();
        let file: &str = match self {
            Self::Xbox => match label {
                "A" => "T_X_A_Color.png",
                "B" => "T_X_B_Color.png",
                "X" => "T_X_X_Color.png",
                "Y" => "T_X_Y_Color.png",
                "LB" => "T_X_LB.png",
                "RB" => "T_X_RB.png",
                "LT" => "T_X_LT.png",
                "RT" => "T_X_RT.png",
                "Start" => "T_X_Share.png",
                "Select" => "T_X_Share-1.png",
                "LS" => "T_X_Left_Stick_Click.png",
                "RS" => "T_X_Right_Stick_Click.png",
                "DUp" => "T_X_Dpad_Up.png",
                "DDown" => "T_X_Dpad_Down.png",
                "DLeft" => "T_X_Dpad_Left.png",
                "DRight" => "T_X_Dpad_Right.png",
                _ => return None,
            },
            Self::PS4 => match label {
                "A" => "T_P4_Cross.png",
                "B" => "T_P4_Circle.png",
                "X" => "T_P4_Square.png",
                "Y" => "T_P4_Triangle.png",
                "LB" => "T_P4_L1.png",
                "RB" => "T_P4_R1.png",
                "LT" => "T_P4_L2.png",
                "RT" => "T_P4_R2.png",
                "Start" => "T_P4_Options.png",
                "Select" => "T_P4_Share.png",
                "LS" => "T_P4_Left_Stick_Click.png",
                "RS" => "T_P4_Right_Stick_Click.png",
                "DUp" => "T_P4_Dpad_UP.png",
                "DDown" => "T_P4_Dpad_Down.png",
                "DLeft" => "T_P4_Dpad_Left.png",
                "DRight" => "T_P4_Dpad_Right.png",
                _ => return None,
            },
            Self::PS5 => match label {
                "A" => "T_P5_Cross.png",
                "B" => "T_P5_Circle.png",
                "X" => "T_P5_Square.png",
                "Y" => "T_P5_Triangle.png",
                "LB" => "T_P5_L1.png",
                "RB" => "T_P5_R1.png",
                "LT" => "T_P5_L2.png",
                "RT" => "T_P5_R2.png",
                "Start" => "T_P5_Options.png",
                "Select" => "T_P5_Share.png",
                "LS" => "T_P5_Left_Stick_Click_Alt.png",
                "RS" => "T_P5_Right_Stick_Click_Alt.png",
                "DUp" => "T_P5_Dpad_UP.png",
                "DDown" => "T_P5_Dpad_Down.png",
                "DLeft" => "T_P5_Dpad_Left.png",
                "DRight" => "T_P5_Dpad_Right.png",
                _ => return None,
            },
            Self::Switch => match label {
                // Nintendo physical layout: South=B, East=A, West=Y, North=X
                "A" => "T_S_B.png",
                "B" => "T_S_A.png",
                "X" => "T_S_Y.png",
                "Y" => "T_S_X.png",
                "LB" => "T_S_LB.png",
                "RB" => "T_S_RB.png",
                "LT" => "T_S_LT.png",
                "RT" => "T_S_RT.png",
                "Start" => "T_S_Plus.png",
                "Select" => "T_S_Minus.png",
                "LS" => "T_S_L.png",
                "RS" => "T_S_R.png",
                "DUp" => "T_S_Dpad_Up.png",
                "DDown" => "T_S_Dpad_Down.png",
                "DLeft" => "T_S_Dpad_Left.png",
                "DRight" => "T_S_Dpad_Right.png",
                _ => return None,
            },
        };
        Some(format!("{}/{}", base, file))
    }

    /// Returns the embedded asset path for a gamepad button label, for use
    /// in internal `asset_server.load()` calls.
    pub(crate) fn embedded_icon_path(self, label: &str) -> Option<String> {
        self.icon_path(label)
            .map(|p| format!("embedded://bevy_quick_action_hud/embedded/{p}"))
    }

    /// Detect icon set from USB vendor / product IDs reported by gilrs.
    pub fn from_ids(vendor: Option<u16>, product: Option<u16>) -> Self {
        match (vendor, product) {
            (Some(0x054C), Some(0x0CE6)) => Self::PS5, // DualSense
            (Some(0x054C), _) => Self::PS4,            // Other Sony
            (Some(0x057E), _) => Self::Switch,         // Nintendo
            (Some(0x045E), _) => Self::Xbox,           // Microsoft
            _ => Self::Xbox,
        }
    }

    /// Detect icon set from the controller's human-readable name string.
    pub fn from_name(name: &str) -> Self {
        let n = name.to_lowercase();
        if n.contains("dualsense") || n.contains("ps5") {
            Self::PS5
        } else if n.contains("dualshock") || n.contains("ps4") || n.contains("ps3") {
            Self::PS4
        } else if n.contains("switch")
            || n.contains("joy-con")
            || n.contains("joycon")
            || n.contains("pro controller")
            || n.contains("nintendo")
        {
            Self::Switch
        } else {
            Self::Xbox
        }
    }
}

/// System: updates [`GamepadIconSet`] when a gamepad is detected.
/// USB IDs take priority; controller name is used as a fallback.
fn detect_gamepad_icon_set(
    added: Query<(&Gamepad, Option<&Name>), Added<Gamepad>>,
    mut icon_set: ResMut<GamepadIconSet>,
) {
    if let Some((gamepad, name)) = added.iter().next() {
        let by_id = GamepadIconSet::from_ids(gamepad.vendor_id(), gamepad.product_id());
        *icon_set = if by_id != GamepadIconSet::Xbox {
            by_id
        } else {
            // IDs inconclusive — try device name
            name.map(|n| GamepadIconSet::from_name(n.as_str()))
                .unwrap_or(GamepadIconSet::Xbox)
        };
    }
}

/// Leafwing-backed input actions for gamepad wheel navigation.
///
/// Attach an `InputMap<WheelNavAction>` + `ActionState<WheelNavAction>` to an entity
/// (done automatically by `WheelMenuPlugin` via `setup_wheel_nav_input`), then read
/// `ActionState` to drive `WheelState`.
#[derive(Actionlike, Reflect, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WheelNavAction {
    /// Right-stick direction — selects the hovered segment.
    #[actionlike(DualAxis)]
    Navigate,
    /// Confirm / select hovered segment (South / A).
    Confirm,
    /// Cancel / close wheel (East / B).
    Cancel,
    /// Cycle active item forward in the hovered slot (North / Y).
    CycleForward,
    /// Cycle active item backward in the hovered slot (West / X).
    CycleBack,
}

// ─── action data model ─────────────────────────────────────────────────────

/// Trait for game-specific actions stored in a [`WheelSlot`] and executed when
/// [`ActionTriggered`] fires.  Implement this to add custom behaviour.
pub trait ActionBehavior: Send + Sync + 'static {
    fn execute(&self, commands: &mut Commands);
    fn label(&self) -> &str;
    fn icon(&self) -> &str;
}

/// The kind of item a wheel slot can hold.
///
/// Use [`ActionItem::Custom`] with a boxed [`ActionBehavior`] for any
/// game-specific item not covered by the built-in variants.
pub enum ActionItem {
    Weapon {
        name: String,
        icon: String,
    },
    Spell {
        name: String,
        icon: String,
    },
    Consumable {
        name: String,
        icon: String,
        count: u32,
    },
    Shout {
        name: String,
        icon: String,
    },
    Custom(Box<dyn ActionBehavior>),
}

impl ActionItem {
    pub fn label(&self) -> &str {
        match self {
            Self::Weapon { name, .. }
            | Self::Spell { name, .. }
            | Self::Shout { name, .. }
            | Self::Consumable { name, .. } => name,
            Self::Custom(b) => b.label(),
        }
    }
    pub fn icon(&self) -> &str {
        match self {
            Self::Weapon { icon, .. }
            | Self::Spell { icon, .. }
            | Self::Shout { icon, .. }
            | Self::Consumable { icon, .. } => icon,
            Self::Custom(b) => b.icon(),
        }
    }
}

/// A wheel slot that can hold **multiple** [`ActionItem`]s.
///
/// The player cycles through items with a button (right/left thumbstick press
/// by default via [`update_slot_cycle`]) without leaving the hovered slice.
/// Attach alongside [`WheelSlice`] on the slice entity.
#[derive(Component)]
pub struct WheelSlot {
    pub items: Vec<ActionItem>,
    /// Index of the currently displayed / active item.
    pub current_item: usize,
}

impl WheelSlot {
    pub fn new(items: Vec<ActionItem>) -> Self {
        Self {
            items,
            current_item: 0,
        }
    }
    pub fn current(&self) -> Option<&ActionItem> {
        self.items.get(self.current_item)
    }
    pub fn cycle_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.current_item = (self.current_item + 1) % self.items.len();
    }
    pub fn cycle_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.current_item = (self.current_item + self.items.len() - 1) % self.items.len();
    }
}

// ─── config enums ─────────────────────────────────────────────────────────────────

/// How the wheel interacts with game time while it is open.
///
/// Exactly **one** variant is active per wheel entity.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub enum TimeMode {
    /// No time manipulation.
    #[default]
    Normal,
    /// Slow [`Time<Virtual>`] to `scale` (0.0 = pause, 1.0 = real time).
    Slow(f32),
    /// Fully pause virtual time.
    Pause,
}

/// How an action is confirmed and triggered from the wheel.
///
/// Using an enum guarantees exactly **one** mode is active; there are no
/// conflicting flag combinations.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub enum CastingMode {
    /// Standard: press the confirm button (South/A) while hovering a slice.
    #[default]
    Vanilla,
    /// Release-to-Use: release the stick from a hovered slice to select it.
    ReleaseToUse,
    /// Dwell on a slice for `duration` seconds to trigger it.
    HoldToActivate { duration: f32 },
    /// Activate immediately when a new slice is hovered (no confirm step).
    Direct,
}

/// How the wheel opens and closes.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub enum WheelToggleMode {
    /// Press the hotkey once to open; press again to close.
    Toggle,
    /// Hold the key / button to keep open; release to close.
    #[default]
    Hold,
    /// Short press → toggle mode; long press → hold mode.
    Hybrid { hold_threshold_secs: f32 },
}

// ─── contextual input override system ──────────────────────────────────────────

// ─── additional messages ──────────────────────────────────────────────────────

// ─── lifecycle & action messages ─────────────────────────────────────────────────────

// ─── plugin ───────────────────────────────────────────────────────────────────

/// The unified wheel-menu plugin.
///
/// | Constructor | Core wheel logic | HUD canvas | Editor sidebar |
/// |---|---|---|---|
/// | `QuickActionHudPlugin::core()` | ✓ | | |
/// | `QuickActionHudPlugin::default()` | ✓ | ✓ | |
/// | `QuickActionHudPlugin::with_editor()` | ✓ | ✓ | ✓ |
///
/// The **core** systems are the full wheel input / hover / selection / hold
/// pipeline (everything that was in the old `WheelMenuPlugin`).
/// The **HUD canvas** renders [`QuickActionConfig`]-driven wheels and action
/// buttons on screen.
/// The **editor sidebar** adds the in-app config editor.
pub struct QuickActionHudPlugin {
    /// Render the [`QuickActionConfig`]-driven HUD canvas.
    pub hud: bool,
    /// Enable the in-app editor sidebar.
    pub editor: bool,
}

impl Default for QuickActionHudPlugin {
    fn default() -> Self {
        Self {
            hud: true,
            editor: false,
        }
    }
}

impl QuickActionHudPlugin {
    /// Core wheel systems only — no HUD canvas, no editor.
    /// Drop-in replacement for the old `WheelMenuPlugin` when you manage your
    /// own rendering.
    pub fn core() -> Self {
        Self {
            hud: false,
            editor: false,
        }
    }

    /// Full HUD canvas **with** the editor sidebar enabled.
    pub fn with_editor() -> Self {
        Self {
            hud: true,
            editor: true,
        }
    }
}

impl Plugin for QuickActionHudPlugin {
    fn build(&self, app: &mut App) {
        scheduling::configure(app);
        // Embed plugin-internal assets so the plugin works as a library
        // without requiring the consumer project to copy these files.
        embedded_asset!(app, "embedded/shaders/wedge.wgsl");
        // Editor UI icons
        embedded_asset!(app, "embedded/icons/editor/cil-aperture.png");
        embedded_asset!(app, "embedded/icons/editor/cil-applications-settings.png");
        embedded_asset!(app, "embedded/icons/editor/cil-camera-control.png");
        embedded_asset!(app, "embedded/icons/editor/cil-chevron-left.png");
        embedded_asset!(app, "embedded/icons/editor/cil-chevron-right.png");
        embedded_asset!(app, "embedded/icons/editor/cil-cog.png");
        embedded_asset!(app, "embedded/icons/editor/cil-minus.png");
        embedded_asset!(app, "embedded/icons/editor/cil-plus.png");
        embedded_asset!(app, "embedded/icons/editor/cil-trash.png");
        embedded_asset!(app, "embedded/icons/editor/cil-x.png");
        // Xbox gamepad button icons
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_A_Color.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_B_Color.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_X_Color.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_Y_Color.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_LB.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_RB.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_LT.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_RT.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_Share.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_Share-1.png");
        embedded_asset!(
            app,
            "embedded/icons/XGamepad/Default/T_X_Left_Stick_Click.png"
        );
        embedded_asset!(
            app,
            "embedded/icons/XGamepad/Default/T_X_Right_Stick_Click.png"
        );
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_Dpad_Up.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_Dpad_Down.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_Dpad_Left.png");
        embedded_asset!(app, "embedded/icons/XGamepad/Default/T_X_Dpad_Right.png");
        // PS4 gamepad button icons
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Cross.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Circle.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Square.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Triangle.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_L1.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_R1.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_L2.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_R2.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Options.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Share.png");
        embedded_asset!(
            app,
            "embedded/icons/P4Gamepad/Default/T_P4_Left_Stick_Click.png"
        );
        embedded_asset!(
            app,
            "embedded/icons/P4Gamepad/Default/T_P4_Right_Stick_Click.png"
        );
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Dpad_UP.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Dpad_Down.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Dpad_Left.png");
        embedded_asset!(app, "embedded/icons/P4Gamepad/Default/T_P4_Dpad_Right.png");
        // PS5 gamepad button icons
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Cross.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Circle.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Square.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Triangle.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_L1.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_R1.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_L2.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_R2.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Options.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Share.png");
        embedded_asset!(
            app,
            "embedded/icons/P5Gamepad/Default/T_P5_Left_Stick_Click_Alt.png"
        );
        embedded_asset!(
            app,
            "embedded/icons/P5Gamepad/Default/T_P5_Right_Stick_Click_Alt.png"
        );
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Dpad_UP.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Dpad_Down.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Dpad_Left.png");
        embedded_asset!(app, "embedded/icons/P5Gamepad/Default/T_P5_Dpad_Right.png");
        // Switch gamepad button icons
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_A.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_B.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_X.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_Y.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_LB.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_RB.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_LT.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_RT.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_Plus.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_Minus.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_L.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_R.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_Dpad_Up.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_Dpad_Down.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_Dpad_Left.png");
        embedded_asset!(app, "embedded/icons/SGamepad/Default/T_S_Dpad_Right.png");

        // ── core wheel input / logic ──────────────────────────────────────────
        app.add_plugins(InputManagerPlugin::<WheelNavAction>::default())
            .init_resource::<GlobalBindings>()
            // selection messages
            .add_message::<WheelMenuSelected>()
            .add_message::<WheelMenuHoverChanged>()
            // lifecycle messages
            .add_message::<WheelOpened>()
            .add_message::<WheelClosed>()
            // action messages
            .add_message::<SlotSelected>()
            .add_message::<ActionTriggered>()
            .add_message::<WheelSlotItemChanged>()
            .add_message::<WheelActionResolved>()
            // wheel-set messages
            .add_message::<WheelSwitched>()
            // hold messages
            .add_message::<WheelMenuHoldProgress>()
            .add_message::<WheelMenuHoldActivated>()
            // misc
            .add_message::<WheelMenuLowCount>()
            .add_message::<WheelEditModeChanged>()
            .add_message::<WheelSliceReorder>()
            .add_systems(Startup, setup_wheel_nav_input)
            .add_systems(
                Update,
                (
                    update_wheel_input,
                    update_wheel_hover,
                    emit_selection,
                    emit_lifecycle,
                    update_slot_cycle,
                    update_wheel_time_scale,
                    update_wheel_hold,
                    update_wheel_set,
                    check_low_counts,
                    update_edit_mode,
                    update_active_slot_context,
                    resolve_wheel_input,
                )
                    .chain()
                    .in_set(scheduling::WheelCoreSet::Runtime),
            );

        // ── HUD canvas ────────────────────────────────────────────────────────
        if self.hud {
            app.add_plugins(UiMaterialPlugin::<WedgeMaterial>::default())
                .init_resource::<QuickActionConfig>()
                .init_resource::<WheelHudState>()
                .init_resource::<GamepadIconSet>()
                .add_message::<HudSegmentSelected>()
                .add_systems(PostStartup, persistence::try_autoload_config)
                .add_systems(
                    Update,
                    (
                        detect_gamepad_icon_set,
                        hud_button_feedback,
                        hud_context_visibility,
                        hud_stick_nav,
                        tick_hud_dry_run_flash,
                        rebuild_hud,
                    )
                        .chain()
                        .in_set(scheduling::HudSet::Runtime),
                );
        }

        // ── editor sidebar ───────────────────────────────────────────────────────────────────────────
        if self.editor {
            app.add_plugins(bevy::feathers::FeathersPlugins);
            // Always insert the dark theme so feathers widgets are styled.
            // Users who need a custom theme should insert their own UiTheme
            // *after* adding this plugin.
            app.insert_resource(bevy::feathers::theme::UiTheme(
                bevy::feathers::dark_theme::create_dark_theme(),
            ));
            editor::register_editor_systems(app);
        }

        // ── platform support ────────────────────────────────────────────────
        app.add_plugins(platform::PlatformSupportPlugin);
    }
}

/// Backward-compat wrapper — use [`QuickActionHudPlugin::core()`] instead.
///
/// Provides core wheel logic with no HUD canvas.
pub struct WheelMenuPlugin;
impl Plugin for WheelMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickActionHudPlugin::core());
    }
}

/// Alias kept for call-sites that used `ActionWheelPlugin`.
pub type ActionWheelPlugin = WheelMenuPlugin;

/// Spawns the global entity that holds the [`WheelNavAction`] input map.
/// Called once at startup by [`WheelMenuPlugin`].
fn setup_wheel_nav_input(mut commands: Commands) {
    commands.spawn((
        ActionState::<WheelNavAction>::default(),
        InputMap::<WheelNavAction>::default()
            .with_dual_axis(WheelNavAction::Navigate, GamepadStick::RIGHT)
            .with(WheelNavAction::Confirm, GamepadButton::South)
            .with(WheelNavAction::Cancel, GamepadButton::East)
            .with(WheelNavAction::CycleForward, GamepadButton::North)
            .with(WheelNavAction::CycleBack, GamepadButton::West),
    ));
}

/// Reads the right-stick via [`WheelNavAction::Navigate`] (leafwing) and updates every
/// [`WheelState::dir`]. Falls back to the raw left stick when no leafwing entity exists.
pub fn update_wheel_input(
    nav_states: Query<&ActionState<WheelNavAction>>,
    gamepads: Query<&Gamepad>,
    mut wheel_states: Query<&mut WheelState, With<WheelData>>,
) {
    // Primary: right stick via leafwing
    let mut nav_dir = Vec2::ZERO;
    for action_state in nav_states.iter() {
        let pair = action_state.axis_pair(&WheelNavAction::Navigate);
        if pair.length() > 0.25 {
            nav_dir = pair;
            break;
        }
    }

    // Fallback: raw left stick (keeps existing example code working)
    if nav_dir == Vec2::ZERO {
        for gamepad in &gamepads {
            let x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
            let y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
            let v = Vec2::new(x, y);
            if v.length() > 0.25 {
                nav_dir = v;
                break;
            }
        }
    }

    let dir = if nav_dir.length() > 0.25 {
        nav_dir.normalize()
    } else {
        Vec2::ZERO
    };
    for mut ws in &mut wheel_states {
        ws.dir = dir;
    }
}

/// Determines which slice is hovered and handles per-mode activation:
/// - [`CastingMode::ReleaseToUse`]: fires [`WheelMenuSelected`] when the stick
///   returns to centre.
/// - [`CastingMode::Direct`]: fires [`WheelMenuSelected`] immediately on hover.
pub fn update_wheel_hover(
    mut q: Query<(
        Entity,
        &WheelData,
        &mut WheelState,
        Option<&WheelMenuConfig>,
    )>,
    mut hover_ev: MessageWriter<WheelMenuHoverChanged>,
    mut select_ev: MessageWriter<WheelMenuSelected>,
) {
    for (entity, menu, mut state, config) in &mut q {
        let previous = state.hovered;

        if state.dir.length() < menu.deadzone {
            state.hovered = None;
        } else {
            let a = state.dir.y.atan2(state.dir.x);
            // Angle relative to the arc start, wrapped into [0, TAU).
            let rel = (a - menu.arc_offset).rem_euclid(std::f32::consts::TAU);
            if rel <= menu.arc_span {
                let idx = ((rel / menu.arc_span) * menu.slots.len().max(1) as f32).floor() as usize;
                state.hovered = Some(idx.min(menu.slots.len().max(1).saturating_sub(1)));
            } else {
                // Direction points outside a partial arc.
                state.hovered = None;
            }
        }

        if previous != state.hovered {
            hover_ev.write(WheelMenuHoverChanged {
                previous,
                current: state.hovered,
                menu_entity: entity,
            });

            if let Some(cfg) = config {
                match &cfg.casting_mode {
                    CastingMode::ReleaseToUse => {
                        // Fire when stick returns to centre after hovering.
                        if let (Some(prev_idx), None) = (previous, state.hovered) {
                            select_ev.write(WheelMenuSelected {
                                index: prev_idx,
                                menu_entity: entity,
                            });
                        }
                    }
                    CastingMode::Direct => {
                        // Fire immediately when a new slice is entered.
                        if let Some(idx) = state.hovered {
                            select_ev.write(WheelMenuSelected {
                                index: idx,
                                menu_entity: entity,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Emits [`WheelMenuSelected`] on confirm-button press for [`CastingMode::Vanilla`].
///
/// All other casting modes handle their own activation logic.
pub fn emit_selection(
    gamepads: Query<&Gamepad>,
    q: Query<(Entity, &WheelState, Option<&WheelMenuConfig>), With<WheelData>>,
    mut ev: MessageWriter<WheelMenuSelected>,
    hud: Option<Res<WheelHudState>>,
) {
    // `WheelMenuPlugin` is also valid without the HUD canvas, so the HUD
    // state resource is not guaranteed to exist in core-only applications.
    if hud.as_ref().is_some_and(|state| state.editor_open) {
        return;
    }
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            for (entity, state, config) in &q {
                if let Some(cfg) = config {
                    match cfg.casting_mode {
                        CastingMode::Vanilla => {} // fall through
                        _ => continue,             // another mode is active
                    }
                }
                if let Some(i) = state.hovered {
                    ev.write(WheelMenuSelected {
                        index: i,
                        menu_entity: entity,
                    });
                }
            }
        }
    }
}

// ─── additional systems ───────────────────────────────────────────────────────

/// Emits [`WheelOpened`] the first frame a wheel gains a hovered slice, and
/// [`WheelClosed`] the first frame it loses one.  Runs after [`update_wheel_hover`].
pub fn emit_lifecycle(
    mut q: Query<(Entity, &mut WheelState), With<WheelData>>,
    mut opened_ev: MessageWriter<WheelOpened>,
    mut closed_ev: MessageWriter<WheelClosed>,
) {
    for (entity, mut state) in &mut q {
        let is_open = state.hovered.is_some();
        if is_open && !state.open {
            state.open = true;
            opened_ev.write(WheelOpened {
                menu_entity: entity,
            });
        } else if !is_open && state.open {
            state.open = false;
            closed_ev.write(WheelClosed {
                menu_entity: entity,
            });
        }
    }
}

/// Cycles the active item in a [`WheelSlot`] when right-thumb (next) or
/// left-thumb (prev) is pressed while hovering the corresponding slice.
/// Emits [`WheelSlotItemChanged`] on each cycle.
pub fn update_slot_cycle(
    gamepads: Query<&Gamepad>,
    wheel_q: Query<(Entity, &WheelState), With<WheelData>>,
    mut slot_q: Query<(&WheelSlice, &mut WheelSlot)>,
    mut ev: MessageWriter<WheelSlotItemChanged>,
) {
    for gamepad in &gamepads {
        let next = gamepad.just_pressed(GamepadButton::RightThumb);
        let prev = gamepad.just_pressed(GamepadButton::LeftThumb);
        if !next && !prev {
            continue;
        }

        for (menu_entity, state) in &wheel_q {
            if let Some(hovered) = state.hovered {
                for (slice, mut slot) in &mut slot_q {
                    if slice.index == hovered {
                        let previous_item = slot.current_item;
                        if next {
                            slot.cycle_next();
                        } else {
                            slot.cycle_prev();
                        }
                        if slot.current_item != previous_item {
                            ev.write(WheelSlotItemChanged {
                                slot_index: hovered,
                                previous_item,
                                current_item: slot.current_item,
                                menu_entity,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Applies the configured time scale to [`Time<Virtual>`] based on each
/// [`WheelMenuConfig::time_mode`].  When multiple wheel entities are alive the
/// most restrictive (lowest) scale wins.
pub fn update_wheel_time_scale(q: Query<&WheelMenuConfig>, mut time: ResMut<Time<Virtual>>) {
    let effective = q.iter().fold(1.0_f32, |acc, cfg| {
        let scale = match cfg.time_mode {
            TimeMode::Normal => 1.0,
            TimeMode::Slow(s) => s,
            TimeMode::Pause => 0.0,
        };
        acc.min(scale)
    });
    time.set_relative_speed(effective);
}

/// Tracks dwell time on a hovered slice for [`CastingMode::HoldToActivate`].
/// Emits [`WheelMenuHoldProgress`] each frame and [`WheelMenuHoldActivated`]
/// when `duration` is reached.
pub fn update_wheel_hold(
    time: Res<Time>,
    mut q: Query<(Entity, &WheelMenuConfig, &WheelState, &mut WheelHoldState)>,
    mut progress_ev: MessageWriter<WheelMenuHoldProgress>,
    mut activate_ev: MessageWriter<WheelMenuHoldActivated>,
) {
    for (entity, config, state, mut hold) in &mut q {
        let duration = match config.casting_mode {
            CastingMode::HoldToActivate { duration } => duration,
            _ => {
                hold.progress = 0.0;
                hold.holding = false;
                continue;
            }
        };
        match state.hovered {
            Some(index) => {
                hold.holding = true;
                hold.progress = (hold.progress + time.delta_secs() / duration).clamp(0.0, 1.0);
                progress_ev.write(WheelMenuHoldProgress {
                    index,
                    progress: hold.progress,
                    menu_entity: entity,
                });
                if hold.progress >= 1.0 {
                    activate_ev.write(WheelMenuHoldActivated {
                        index,
                        menu_entity: entity,
                    });
                    hold.progress = 0.0;
                }
            }
            None => {
                hold.holding = false;
                hold.progress = 0.0;
            }
        }
    }
}

/// Cycles the active wheel in a [`WheelSet`] when the configured shoulder
/// buttons are pressed.  The index wraps around at both ends.
pub fn update_wheel_set(
    gamepads: Query<&Gamepad>,
    mut q: Query<(Entity, &mut WheelSet)>,
    mut ev: MessageWriter<WheelSwitched>,
) {
    for (entity, mut set) in &mut q {
        if set.count < 2 {
            continue;
        }
        for gamepad in &gamepads {
            if gamepad.just_pressed(set.next_button) {
                let previous = set.active;
                set.active = (set.active + 1) % set.count;
                ev.write(WheelSwitched {
                    previous,
                    current: set.active,
                    menu_entity: entity,
                });
            }
            if gamepad.just_pressed(set.prev_button) {
                let previous = set.active;
                set.active = (set.active + set.count - 1) % set.count;
                ev.write(WheelSwitched {
                    previous,
                    current: set.active,
                    menu_entity: entity,
                });
            }
        }
    }
}

/// Emits [`WheelMenuLowCount`] once each time a [`WheelSliceCount`] transitions
/// from above to at-or-below its threshold.  The flag resets when the count
/// rises above the threshold again.
pub fn check_low_counts(
    mut q: Query<(Entity, &WheelSlice, &mut WheelSliceCount)>,
    mut ev: MessageWriter<WheelMenuLowCount>,
) {
    for (entity, slice, mut count) in &mut q {
        let is_low = count.max > 0 && count.current <= count.low_threshold;
        if is_low && !count.low_notified {
            count.low_notified = true;
            ev.write(WheelMenuLowCount {
                index: slice.index,
                current: count.current,
                threshold: count.low_threshold,
                slice_entity: entity,
            });
        } else if !is_low {
            count.low_notified = false;
        }
    }
}

/// Toggles edit mode when the configured button is pressed, and emits
/// [`WheelSliceReorder`] events when D-pad Up/Down is pressed while hovering a
/// slice in edit mode.
pub fn update_edit_mode(
    gamepads: Query<&Gamepad>,
    mut q: Query<(Entity, &WheelData, &WheelState, &mut WheelEditMode)>,
    mut mode_ev: MessageWriter<WheelEditModeChanged>,
    mut reorder_ev: MessageWriter<WheelSliceReorder>,
    hud: Option<Res<WheelHudState>>,
) {
    // `WheelMenuPlugin` is also valid without the HUD canvas, so the HUD
    // state resource is not guaranteed to exist in core-only applications.
    if hud.as_ref().is_some_and(|state| state.editor_open) {
        return;
    }
    for (entity, menu, state, mut edit) in &mut q {
        for gamepad in &gamepads {
            if let Some(btn) = edit.toggle_button {
                if gamepad.just_pressed(btn) {
                    edit.active = !edit.active;
                    mode_ev.write(WheelEditModeChanged {
                        active: edit.active,
                        menu_entity: entity,
                    });
                }
            }
            if edit.active {
                if let Some(hovered) = state.hovered {
                    if gamepad.just_pressed(GamepadButton::DPadUp) && hovered > 0 {
                        reorder_ev.write(WheelSliceReorder {
                            from_index: hovered,
                            to_index: hovered - 1,
                            menu_entity: entity,
                        });
                    }
                    if gamepad.just_pressed(GamepadButton::DPadDown)
                        && hovered + 1 < menu.slots.len().max(1)
                    {
                        reorder_ev.write(WheelSliceReorder {
                            from_index: hovered,
                            to_index: hovered + 1,
                            menu_entity: entity,
                        });
                    }
                }
            }
        }
    }
}

/// Maintains [`ActiveSlotContext`] on each wheel from its currently hovered
/// slice.  Only slices carrying a [`WheelSliceLink`] participate, so the link
/// back to the owning wheel is explicit and query scans stay cheap.
pub fn update_active_slot_context(
    mut commands: Commands,
    wheel_q: Query<(Entity, &WheelState)>,
    slice_q: Query<(Entity, &WheelSlice, &WheelSliceLink)>,
) {
    for (menu, state) in &wheel_q {
        let mut found: Option<Entity> = None;
        if let Some(hovered) = state.hovered {
            for (slice_entity, slice, link) in &slice_q {
                if link.menu == menu && slice.index == hovered {
                    found = Some(slice_entity);
                    break;
                }
            }
        }
        match found {
            Some(slot_entity) => {
                commands
                    .entity(menu)
                    .insert(ActiveSlotContext { slot_entity });
            }
            None => {
                commands.entity(menu).remove::<ActiveSlotContext>();
            }
        }
    }
}

/// Reads gamepad face/thumb buttons, maps them onto [`InputAction`]s, resolves
/// each against the slot → wheel → global override chain, and emits
/// [`WheelActionResolved`].  This implements the contextual input-override
/// system: a hovered slot's bindings take priority over the wheel's, which take
/// priority over [`GlobalBindings`].
#[allow(clippy::type_complexity)]
pub fn resolve_wheel_input(
    gamepads: Query<&Gamepad>,
    global: Res<GlobalBindings>,
    wheel_q: Query<
        (
            Entity,
            Option<&WheelInputOverride>,
            Option<&ActiveSlotContext>,
        ),
        With<WheelState>,
    >,
    slot_q: Query<&WheelInputOverride, Without<WheelState>>,
    mut ev: MessageWriter<WheelActionResolved>,
) {
    for (menu, wheel_override, active_slot) in &wheel_q {
        let slot_override = active_slot.and_then(|ctx| slot_q.get(ctx.slot_entity).ok());
        for gamepad in &gamepads {
            for (button, input) in wheel_core::DEFAULT_BUTTON_MAP {
                if gamepad.just_pressed(*button) {
                    if let Some(action) =
                        resolve_input(*input, slot_override, wheel_override, &global)
                    {
                        ev.write(WheelActionResolved {
                            input: *input,
                            action,
                            menu_entity: menu,
                        });
                    }
                }
            }
        }
    }
}

/// Helper to calculate slice angles with gap.
pub fn slice_angles(menu: &WheelData, index: usize) -> (f32, f32) {
    let n = menu.slots.len().max(1);
    let slice_angle = menu.arc_span / n as f32;
    let half_gap = if menu.overlap { 0.0 } else { menu.gap / 2.0 };
    let a0 = menu.arc_offset + index as f32 * slice_angle + half_gap;
    let a1 = menu.arc_offset + (index + 1) as f32 * slice_angle - half_gap;
    (a0, a1)
}

/// Helper to get the center position of a slice (for placing icons/text).
pub fn slice_center(menu: &WheelData, index: usize) -> Vec2 {
    let (a0, a1) = slice_angles(menu, index);
    let center_angle = (a0 + a1) / 2.0;
    let center_radius = (menu.inner_radius + menu.outer_radius) / 2.0;
    Vec2::new(
        center_angle.cos() * center_radius,
        center_angle.sin() * center_radius,
    )
}

/// Returns a full-screen `bevy_ui` overlay [`Node`] that centers its children,
/// authored with the [`bsn!`] macro.
///
/// Spawn it with `commands.spawn_scene(wheel_overlay())` and attach the
/// wheel-menu logic components ([`WheelData`] and [`WheelState`]) to the
/// resulting entity.
pub fn wheel_overlay() -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(0.)},
            top: {px(0.)},
            width: {percent(100.)},
            height: {percent(100.)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

/// Returns a zero-size hub [`Node`] used as the positioning origin for slices,
/// authored with the [`bsn!`] macro.
///
/// Spawn it as a child of [`wheel_overlay`] and parent each slice panel to it so
/// the absolutely-positioned panels are laid out relative to the screen center.
pub fn wheel_hub() -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node { width: {px(0.)}, height: {px(0.)} }
    }
}

/// Returns an absolutely-positioned, rounded slice panel centered on the radial
/// position of slice `index`, authored with the [`bsn!`] macro.
///
/// `size` is the panel's width/height in logical pixels and `color` its
/// background color. The panel is laid out as a centered vertical column so
/// icons and labels can be added as children.
pub fn wheel_slice_panel(
    menu: &WheelData,
    index: usize,
    size: f32,
    color: Color,
) -> impl bevy::scene::prelude::Scene {
    wheel_slice_panel_styled(menu, index, size, color, size * 0.18)
}

/// Like [`wheel_slice_panel`] but with an explicit `corner_radius`, letting the
/// caller pick a slot shape: `size * 0.5` ≈ round, `size * 0.18` ≈ rounded,
/// `0.0` = square.
pub fn wheel_slice_panel_styled(
    menu: &WheelData,
    index: usize,
    size: f32,
    color: Color,
    corner_radius: f32,
) -> impl bevy::scene::prelude::Scene {
    let center = slice_center(menu, index);
    // Math coordinates are y-up and centered on the wheel; UI is y-down relative
    // to the hub, so flip the y axis and offset by half the panel size.
    let left = center.x - size / 2.0;
    let top = -center.y - size / 2.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(left)},
            top: {px(top)},
            width: {px(size)},
            height: {px(size)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: {px(2.)},
            border_radius: {BorderRadius::all(px(corner_radius))},
        }
        BackgroundColor({color})
    }
}

/// Returns a circular center-disc [`Node`] of diameter `radius * 2`, centered
/// on the hub via absolute positioning.
///
/// Spawn as a child of [`wheel_hub`].  Add label or icon children afterward
/// with `commands.entity(disc).add_child(...)`.
pub fn wheel_center_disc(radius: f32, color: Color) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(-radius)},
            top: {px(-radius)},
            width: {px(radius * 2.0)},
            height: {px(radius * 2.0)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(radius))},
        }
        BackgroundColor({color})
    }
}

/// Like [`wheel_center_disc`] but draws a coloured ring border around the hub.
///
/// Use this instead of [`wheel_center_disc`] to get the golden ring shown in
/// the reference screenshots.
pub fn wheel_center_ring(
    radius: f32,
    bg: Color,
    ring_color: Color,
    ring_width: f32,
) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(-radius)},
            top: {px(-radius)},
            width: {px(radius * 2.0)},
            height: {px(radius * 2.0)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(radius))},
            border: {UiRect::all(px(ring_width))},
        }
        BackgroundColor({bg})
        BorderColor::all(ring_color)
    }
}

/// A large dark disc that fills the full wheel area, placed behind all slices.
///
/// Spawn as a child of [`wheel_hub`] **before** the slices so it sits at the
/// back of the z-order.
pub fn wheel_bg_disc(outer_radius: f32, color: Color) -> impl bevy::scene::prelude::Scene {
    let r = outer_radius + 4.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {Val::Px(-r)},
            top: {Val::Px(-r)},
            width: {Val::Px(r * 2.0)},
            height: {Val::Px(r * 2.0)},
            border_radius: {BorderRadius::all(Val::Px(r))},
        }
        BackgroundColor({color})
    }
}

/// A thin amber/gold ring just outside the wheel — approximates the dashed
/// outer border visible in the reference screenshots.
pub fn wheel_outer_ring(
    outer_radius: f32,
    color: Color,
    border_w: f32,
) -> impl bevy::scene::prelude::Scene {
    let r = outer_radius + 1.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {Val::Px(-r)},
            top: {Val::Px(-r)},
            width: {Val::Px(r * 2.0)},
            height: {Val::Px(r * 2.0)},
            border_radius: {BorderRadius::all(Val::Px(r))},
            border: {UiRect::all(Val::Px(border_w))},
        }
        BackgroundColor({Color::NONE})
        BorderColor::all(color)
    }
}

/// Absolutely-positioned rectangular slice panel, sized to better fill a
/// segment of the wheel than the square [`wheel_slice_panel_styled`].
///
/// `width` and `height` are in logical pixels.  Use `corner_radius` ≈
/// `min(width, height) * 0.15` for the rounded look shown in the screenshots.
pub fn wheel_slice_panel_rect(
    menu: &WheelData,
    index: usize,
    width: f32,
    height: f32,
    color: Color,
    corner_radius: f32,
) -> impl bevy::scene::prelude::Scene {
    let center = slice_center(menu, index);
    let left = center.x - width / 2.0;
    let top = -center.y - height / 2.0;
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {px(left)},
            top: {px(top)},
            width: {px(width)},
            height: {px(height)},
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            padding: {UiRect::all(px(6.))},
            border_radius: {BorderRadius::all(px(corner_radius))},
        }
        BackgroundColor({color})
    }
}

/// Returns a [`Text`] node sized for a slice **icon** (typically an emoji or
/// large glyph).
///
/// Spawn as a child of [`wheel_slice_panel`] or insert marker components with
/// `.insert(MyMarker)` on the returned `EntityCommands`.
pub fn wheel_slice_icon(
    icon: String,
    font_size: f32,
    color: Color,
) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Text({icon})
        TextFont { font_size: {FontSize::Px(font_size)} }
        TextColor({color})
    }
}

/// Returns a [`Text`] node sized for a slice **label** (name, count, cooldown,
/// etc.).
///
/// Spawn as a child of [`wheel_slice_panel`] or insert marker components with
/// `.insert(MyMarker)` on the returned `EntityCommands`.
pub fn wheel_slice_label(
    text: String,
    font_size: f32,
    color: Color,
) -> impl bevy::scene::prelude::Scene {
    bsn! {
        Text({text})
        TextFont { font_size: {FontSize::Px(font_size)} }
        TextColor({color})
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// QUICK-ACTION CONFIG — data model for the editor and the HUD
// ─────────────────────────────────────────────────────────────────────────────────

/// Placement reference for a floating quick-action button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum PositionMode {
    #[default]
    Relative,
    Absolute,
}
impl PositionMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Relative => "Relative",
            Self::Absolute => "Absolute",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Relative => Self::Absolute,
            Self::Absolute => Self::Relative,
        }
    }
}

/// Shape of a quick-action HUD button.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum ActionShape {
    #[default]
    Rounded,
    Round,
    Square,
    Diamond,
}
impl ActionShape {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rounded => "Rounded",
            Self::Round => "Round",
            Self::Square => "Square",
            Self::Diamond => "Diamond",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Rounded => Self::Round,
            Self::Round => Self::Square,
            Self::Square => Self::Diamond,
            Self::Diamond => Self::Rounded,
        }
    }
}

/// Visual theme for a wheel.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum WheelTheme {
    #[default]
    Dark,
    Light,
}
impl WheelTheme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

/// Shape rendered for each segment panel inside the wheel.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum SegmentShape {
    #[default]
    Rounded,
    Square,
    Circle,
    /// Asymmetric corners — outer large, inner small.
    Wedge,
    /// Real `Mesh2d` wedge arc (uses `bevy::mesh`).
    Pie,
}
impl SegmentShape {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rounded => "Rounded",
            Self::Square => "Square",
            Self::Circle => "Circle",
            Self::Wedge => "Wedge",
            Self::Pie => "Pie",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Rounded => Self::Square,
            Self::Square => Self::Circle,
            Self::Circle => Self::Wedge,
            Self::Wedge => Self::Pie,
            Self::Pie => Self::Rounded,
        }
    }
}

// ── palette helpers ─────────────────────────────────────────────────────────────
pub const ICON_PALETTE: &[&str] = &["◆", "●", "★", "▲", "✦", "✚", "◈", "○", "◐", "✱"];
pub const COMMAND_PALETTE: &[&str] = &[
    "none", "attack", "heal", "block", "dash", "reload", "interact", "jump", "crouch", "sprint",
];
pub fn cycle_palette<'a>(list: &[&'a str], current: &str) -> &'a str {
    let idx = list.iter().position(|s| *s == current).unwrap_or(0);
    list[(idx + 1) % list.len()]
}

// ── serde defaults ──────────────────────────────────────────────────────────────
fn _default_true() -> bool {
    true
}
fn _default_action_color() -> String {
    "#3b82f6".into()
}
fn _default_action_width() -> f32 {
    80.0
}
fn _default_action_height() -> f32 {
    28.0
}
fn _default_hold_command() -> String {
    "none".into()
}
fn _default_action_command() -> String {
    "none".into()
}
fn _default_outer_radius() -> f32 {
    270.0
}
fn _default_inner_radius() -> f32 {
    130.0
}
fn _full_opacity() -> f32 {
    1.0
}
fn _default_highlight_color() -> String {
    "#ef8b92".into()
}
fn _default_segment_scale() -> f32 {
    1.0
}
fn _default_border_width() -> f32 {
    2.0
}
fn _default_deadzone() -> f32 {
    0.3
}
fn _default_gap() -> f32 {
    0.012
}
fn _default_arc_span() -> f32 {
    std::f32::consts::TAU
}
fn _default_arc_offset() -> f32 {
    std::f32::consts::FRAC_PI_6
}
fn _default_wheelset_min() -> usize {
    1
}
fn _default_wheelset_max() -> usize {
    8
}

// ── slot / item data ─────────────────────────────────────────────────────────────

/// One item in a slot's cycle carousel.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SlotItem {
    pub name: String,
    pub icon: String,
}

/// Per-segment data for the editor config.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Sector {
    pub name: String,
    /// Optional explanatory text for this sector.
    #[serde(default)]
    pub description: String,
    pub icon: String,
    /// Captured input label: keyboard key or "GP:…" gamepad.
    pub input: String,
    /// Action mapping executed when this segment is applied.
    #[serde(default = "_default_action_command")]
    pub command: String,
    /// Action mapping executed while this segment is held.
    #[serde(default = "_default_hold_command")]
    pub hold_command: String,
    /// Whether the hold action is enabled for this segment.
    #[serde(default)]
    pub hold: bool,
    pub items: Vec<SlotItem>,
    /// Close the HUD overlay when this slot is selected.
    #[serde(default = "_default_true")]
    pub close_on_select: bool,
}
impl Default for Sector {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            icon: String::new(),
            input: String::new(),
            command: "none".into(),
            hold_command: "none".into(),
            hold: false,
            items: Vec::new(),
            close_on_select: true,
        }
    }
}
impl Sector {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

// ── quick action ─────────────────────────────────────────────────────────────────

/// A key-bound floating HUD button.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct QuickAction {
    pub name: String,
    /// Optional explanatory text for this HUD button.
    #[serde(default)]
    pub description: String,
    /// Keyboard key or gamepad button ("GP:\u{2026}" prefix) that triggers this action.
    pub key: String,
    pub icon: String,
    pub command: String,
    /// Command/mapping used while the input is held.
    #[serde(default = "_default_hold_command")]
    pub hold_command: String,
    pub hold: bool,
    pub show_on_menu: bool,
    pub opacity: f32,
    pub position: PositionMode,
    pub radius: f32,
    /// Horizontal editor offset from the HUD's default action area.
    #[serde(default)]
    pub offset_x: f32,
    /// Vertical editor offset from the HUD's default action area.
    #[serde(default)]
    pub offset_y: f32,
    /// Rotation of the floating button in degrees.
    #[serde(default)]
    pub rotation: f32,
    pub shape: ActionShape,
    #[serde(default = "_default_action_color")]
    pub color: String,
    #[serde(default = "_default_action_width")]
    pub width: f32,
    #[serde(default = "_default_action_height")]
    pub height: f32,
    #[serde(default = "_default_true")]
    pub enabled: bool,
    /// Close the HUD overlay when this action's shortcut is pressed.
    #[serde(default = "_default_true")]
    pub close_on_select: bool,
}
impl Default for QuickAction {
    fn default() -> Self {
        Self {
            name: "Action".into(),
            description: String::new(),
            key: String::new(),
            icon: "◆".into(),
            command: "none".into(),
            hold_command: "none".into(),
            hold: false,
            show_on_menu: true,
            opacity: 1.0,
            position: PositionMode::Relative,
            radius: 48.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            shape: ActionShape::Rounded,
            color: _default_action_color(),
            width: _default_action_width(),
            height: _default_action_height(),
            enabled: true,
            close_on_select: true,
        }
    }
}

// ── wheel config data ────────────────────────────────────────────────────────────

/// Editor data-model wheel — one radial menu with named segments.
#[derive(Component, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenu {
    pub name: String,
    pub cooldown_secs: f32,
    pub slots: Vec<WheelSlotData>,
    /// Horizontal editor offset from the centered wheel position.
    #[serde(default)]
    pub offset_x: f32,
    /// Vertical editor offset from the centered wheel position.
    #[serde(default)]
    pub offset_y: f32,
    /// Rotation shared by all wheels when this wheel belongs to a wheel set.
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub theme: WheelTheme,
    #[serde(default = "_default_outer_radius")]
    pub outer_radius: f32,
    #[serde(default = "_default_inner_radius")]
    pub inner_radius: f32,
    #[serde(default = "_default_true")]
    pub show_labels: bool,
    #[serde(default)]
    pub segment_shape: SegmentShape,
    #[serde(default = "_default_true")]
    pub show_icon: bool,
    #[serde(default = "_default_highlight_color")]
    pub highlight_color: String,
    #[serde(default = "_default_segment_scale")]
    pub segment_scale: f32,
    /// Overall opacity of the wheel overlay (0.0 – 1.0).
    #[serde(default = "_full_opacity")]
    pub opacity: f32,
    /// Hex color for the inner-radius border ring; empty = no border.
    #[serde(default)]
    pub inner_border: String,
    /// Hex color for the outer-radius border ring; empty = no border.
    #[serde(default)]
    pub outer_border: String,
    /// Width in px of the outer border ring; only used when `outer_border` is set.
    #[serde(default = "_default_border_width")]
    pub outer_border_width: f32,
    /// Width in px of the inner hub ring; only used when `inner_border` is set.
    #[serde(default = "_default_border_width")]
    pub inner_border_width: f32,
    /// Hex background color for the full wheel disc; empty = use theme.
    #[serde(default)]
    pub bg_color: String,
    /// Opacity of the wheel background disc (0.0 – 1.0).
    #[serde(default = "_full_opacity")]
    pub bg_opacity: f32,
    /// Hex background color for the hub (inner circle); empty = use theme.
    #[serde(default)]
    pub hub_color: String,
    /// Opacity of the hub (inner circle) background (0.0 – 1.0).
    #[serde(default = "_full_opacity")]
    pub hub_opacity: f32,
    /// Stick deadzone for the headless ECS input API (0.0–1.0).
    #[serde(default = "_default_deadzone")]
    pub deadzone: f32,
    /// Gap between segments in radians (used by the headless ECS API).
    #[serde(default = "_default_gap")]
    pub gap: f32,
    /// Total angular span in radians (TAU = full circle).
    #[serde(default = "_default_arc_span")]
    pub arc_span: f32,
    /// Angle of the first segment, CCW from +X axis.
    #[serde(default = "_default_arc_offset")]
    pub arc_offset: f32,
    /// When true, segments touch with no gap.
    #[serde(default)]
    pub overlap: bool,
    /// Which analogue stick navigates this wheel in the HUD.
    #[serde(default)]
    pub stick: StickSide,
}
impl Default for RadialMenu {
    fn default() -> Self {
        Self {
            name: "Radial menu".into(),
            cooldown_secs: 6.0,
            slots: vec![WheelSlotData::named("Slot 1")],
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            theme: WheelTheme::Dark,
            outer_radius: _default_outer_radius(),
            inner_radius: _default_inner_radius(),
            show_labels: false,
            segment_shape: SegmentShape::Pie,
            show_icon: true,
            highlight_color: "#ef8b92".into(),
            segment_scale: 1.0,
            opacity: 1.0,
            inner_border: String::new(),
            outer_border: String::new(),
            outer_border_width: 2.0,
            inner_border_width: 2.0,
            bg_color: String::new(),
            bg_opacity: 1.0,
            hub_color: String::new(),
            hub_opacity: 1.0,
            deadzone: _default_deadzone(),
            gap: _default_gap(),
            arc_span: _default_arc_span(),
            arc_offset: _default_arc_offset(),
            overlap: false,
            stick: StickSide::Right,
        }
    }
}
impl RadialMenu {
    pub fn new(name: impl Into<String>, n: usize) -> Self {
        Self {
            name: name.into(),
            slots: (0..n.max(1))
                .map(|i| WheelSlotData::named(format!("Slot {}", i + 1)))
                .collect(),
            ..Default::default()
        }
    }
}

/// Shared presentation/configuration applied to every wheel in a wheel set.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenuSetVisuals {
    pub offset_x: f32,
    pub offset_y: f32,
    pub rotation: f32,
    pub theme: WheelTheme,
    pub outer_radius: f32,
    pub inner_radius: f32,
    pub show_labels: bool,
    pub segment_shape: SegmentShape,
    pub show_icon: bool,
    pub highlight_color: String,
    pub segment_scale: f32,
    pub opacity: f32,
    pub inner_border: String,
    pub outer_border: String,
    pub outer_border_width: f32,
    pub inner_border_width: f32,
    pub bg_color: String,
    pub bg_opacity: f32,
    pub hub_color: String,
    pub hub_opacity: f32,
    pub deadzone: f32,
    pub gap: f32,
    pub arc_span: f32,
    pub arc_offset: f32,
    pub overlap: bool,
    pub stick: StickSide,
}
impl Default for RadialMenuSetVisuals {
    fn default() -> Self {
        let w = WheelData::default();
        Self::from(&w)
    }
}
impl From<&WheelData> for RadialMenuSetVisuals {
    fn from(w: &WheelData) -> Self {
        Self {
            offset_x: w.offset_x,
            offset_y: w.offset_y,
            rotation: w.rotation,
            theme: w.theme,
            outer_radius: w.outer_radius,
            inner_radius: w.inner_radius,
            show_labels: w.show_labels,
            segment_shape: w.segment_shape,
            show_icon: w.show_icon,
            highlight_color: w.highlight_color.clone(),
            segment_scale: w.segment_scale,
            opacity: w.opacity,
            inner_border: w.inner_border.clone(),
            outer_border: w.outer_border.clone(),
            outer_border_width: w.outer_border_width,
            inner_border_width: w.inner_border_width,
            bg_color: w.bg_color.clone(),
            bg_opacity: w.bg_opacity,
            hub_color: w.hub_color.clone(),
            hub_opacity: w.hub_opacity,
            deadzone: w.deadzone,
            gap: w.gap,
            arc_span: w.arc_span,
            arc_offset: w.arc_offset,
            overlap: w.overlap,
            stick: w.stick,
        }
    }
}
impl RadialMenuSetVisuals {
    pub fn apply_to(&self, w: &mut WheelData) {
        w.offset_x = self.offset_x;
        w.offset_y = self.offset_y;
        w.rotation = self.rotation;
        w.theme = self.theme;
        w.outer_radius = self.outer_radius;
        w.inner_radius = self.inner_radius;
        w.show_labels = self.show_labels;
        w.segment_shape = self.segment_shape;
        w.show_icon = self.show_icon;
        w.highlight_color = self.highlight_color.clone();
        w.segment_scale = self.segment_scale;
        w.opacity = self.opacity;
        w.inner_border = self.inner_border.clone();
        w.outer_border = self.outer_border.clone();
        w.outer_border_width = self.outer_border_width;
        w.inner_border_width = self.inner_border_width;
        w.bg_color = self.bg_color.clone();
        w.bg_opacity = self.bg_opacity;
        w.hub_color = self.hub_color.clone();
        w.hub_opacity = self.hub_opacity;
        w.deadzone = self.deadzone;
        w.gap = self.gap;
        w.arc_span = self.arc_span;
        w.arc_offset = self.arc_offset;
        w.overlap = self.overlap;
        w.stick = self.stick;
    }
}

/// A named group of wheels that share presentation/configuration and can be switched.
/// Serialized as `WheelSet` for RON compatibility.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RadialMenuSet {
    pub name: String,
    pub wheels: Vec<WheelData>,
    /// Shared settings. `None` is accepted for legacy configs and resolved from wheel 0.
    #[serde(default)]
    pub visuals: Option<WheelSetVisuals>,
    #[serde(default = "_default_wheelset_min")]
    pub min_wheels: usize,
    #[serde(default = "_default_wheelset_max")]
    pub max_wheels: usize,
    #[serde(default)]
    pub prev_wheel_key: String,
    #[serde(default)]
    pub next_wheel_key: String,
    #[serde(default)]
    pub cycle_wheels: bool,
    /// Legacy single switch key retained for old RON files.
    #[serde(default)]
    pub switch_key: String,
    /// Which analogue stick navigates wheels in this set.
    #[serde(default)]
    pub stick: StickSide,
}

/// A visible HUD component that switches to another enabled HUD page.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct HudSwitch {
    pub name: String,
    pub key: String,
    pub target_page: usize,
    pub enabled: bool,
    #[serde(default)]
    pub offset_x: f32,
    #[serde(default)]
    pub offset_y: f32,
    #[serde(default = "_default_action_width")]
    pub width: f32,
    #[serde(default = "_default_action_height")]
    pub height: f32,
}
impl Default for HudSwitch {
    fn default() -> Self {
        Self {
            name: "HUD Switch".into(),
            key: String::new(),
            target_page: 0,
            enabled: true,
            offset_x: 0.0,
            offset_y: 0.0,
            width: 100.0,
            height: 28.0,
        }
    }
}

/// Common editor contract for components that can appear on a HUD canvas.
pub trait HudComponent {
    fn hud_name(&self) -> &str;
    fn hud_enabled(&self) -> bool;
}
impl HudComponent for QuickAction {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        self.enabled
    }
}
impl HudComponent for WheelSetData {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        true
    }
}
impl HudComponent for HudSwitch {
    fn hud_name(&self) -> &str {
        &self.name
    }
    fn hud_enabled(&self) -> bool {
        self.enabled
    }
}
impl Default for RadialMenuSet {
    fn default() -> Self {
        Self {
            name: "Radial menu set".into(),
            wheels: vec![WheelData::default()],
            visuals: Some(WheelSetVisuals::default()),
            min_wheels: 1,
            max_wheels: 8,
            prev_wheel_key: String::new(),
            next_wheel_key: String::new(),
            cycle_wheels: false,
            switch_key: String::new(),
            stick: StickSide::Right,
        }
    }
}

pub fn wheelset_visuals(ws: &WheelSetData) -> WheelSetVisuals {
    ws.visuals
        .clone()
        .or_else(|| ws.wheels.first().map(WheelSetVisuals::from))
        .unwrap_or_default()
}

pub fn normalize_wheelset(ws: &mut WheelSetData) {
    let visuals = wheelset_visuals(ws);
    ws.visuals = Some(visuals.clone());
    ws.min_wheels = ws.min_wheels.max(1);
    ws.max_wheels = ws.max_wheels.max(ws.min_wheels).max(ws.wheels.len());
    if ws.wheels.len() < ws.min_wheels {
        while ws.wheels.len() < ws.min_wheels {
            let mut wheel = WheelData::new(format!("Radial menu {}", ws.wheels.len() + 1), 6);
            visuals.apply_to(&mut wheel);
            ws.wheels.push(wheel);
        }
    }
    for wheel in &mut ws.wheels {
        visuals.apply_to(wheel);
    }
}

pub fn normalize_wheelset_config(cfg: &mut QuickActionConfig) {
    // A HUD document always contains at least one page, including after
    // loading an empty hand-authored or legacy RON document.
    if cfg.sets.is_empty() {
        cfg.sets.push(ActionSet::default());
    }
    for set in &mut cfg.sets {
        for entry in &mut set.entries {
            if let SetEntry::WheelSet(ws) = entry {
                normalize_wheelset(ws);
            }
        }
    }
}

/// One entry inside an [`ActionSet`].
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SetEntry {
    Action(QuickAction),
    Wheel(WheelData),
    WheelSet(WheelSetData),
    HudSwitch(HudSwitch),
}

/// A named context group that holds quick actions and wheels.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ActionSet {
    pub name: String,
    /// Optional page icon path or symbolic icon identifier.
    #[serde(default)]
    pub icon: String,
    /// Whether this HUD page participates in the page switcher.
    #[serde(default = "_default_true")]
    pub enabled: bool,
    #[serde(default = "_full_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub input_override: bool,
    pub entries: Vec<SetEntry>,
    #[serde(default)]
    pub bg_image: String,
    #[serde(default = "_full_opacity")]
    pub bg_image_opacity: f32,
    #[serde(default)]
    pub next_wheel_key: String,
    #[serde(default)]
    pub prev_wheel_key: String,
    #[serde(default)]
    pub cycle_wheels: bool,
}
impl Default for ActionSet {
    fn default() -> Self {
        Self {
            name: "Set".into(),
            icon: String::new(),
            enabled: true,
            opacity: 1.0,
            input_override: false,
            entries: Vec::new(),
            bg_image: String::new(),
            bg_image_opacity: 1.0,
            next_wheel_key: String::new(),
            prev_wheel_key: String::new(),
            cycle_wheels: false,
        }
    }
}

/// Canonical HUD terminology. Original type names remain available for API
/// and RON compatibility.
pub type WheelSlotData = Sector;
pub type WheelData = RadialMenu;
pub type WheelSetData = RadialMenuSet;
pub type WheelSetVisuals = RadialMenuSetVisuals;
pub type HudPage = ActionSet;
pub type HudButton = QuickAction;

/// Returns the number of `Wheel` and `WheelSet` entries in a set.
pub fn count_wheel_entries(set: &ActionSet) -> usize {
    set.entries
        .iter()
        .filter(|e| matches!(e, SetEntry::Wheel(_) | SetEntry::WheelSet(_)))
        .count()
}

/// Returns the page indices currently included in HUD page navigation.
pub fn enabled_hud_pages(cfg: &QuickActionConfig) -> Vec<usize> {
    cfg.sets
        .iter()
        .enumerate()
        .filter_map(|(i, page)| page.enabled.then_some(i))
        .collect()
}

/// Whether the HUD overlay opens while a button is held (released = close)
/// or toggles open/closed on each press.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum HudOpenMode {
    /// Hold the trigger to keep the HUD open; releasing closes it.
    Hold,
    /// First press opens the HUD; second press closes it.
    #[default]
    Toggle,
}

/// Which analogue stick navigates the HUD wheel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StickSide {
    /// Use the right analogue stick (default).
    #[default]
    Right,
    /// Use the left analogue stick.
    Left,
}
impl StickSide {
    pub fn label(self) -> &'static str {
        match self {
            Self::Right => "R Stick",
            Self::Left => "L Stick",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
}

impl HudOpenMode {
    pub fn label(&self) -> &'static str {
        match self {
            HudOpenMode::Hold => "Hold",
            HudOpenMode::Toggle => "Toggle",
        }
    }
    pub fn next(&self) -> Self {
        match self {
            HudOpenMode::Hold => HudOpenMode::Toggle,
            HudOpenMode::Toggle => HudOpenMode::Hold,
        }
    }
}

/// The complete editable document (Bevy `Resource`).
#[derive(Resource, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct QuickActionConfig {
    #[serde(default)]
    pub next_set_key: String,
    #[serde(default)]
    pub prev_set_key: String,
    /// Show the ActionSet tab bar in the HUD overlay.
    #[serde(default = "_default_true")]
    pub show_set_bar: bool,
    /// Pressing Next on the last set wraps to the first (cycle), otherwise stops.
    #[serde(default)]
    pub cycle_sets: bool,
    /// Key or gamepad button ("GP:…" prefix) that opens/closes the editor sidebar.
    #[serde(default)]
    pub edit_shortcut: String,
    /// Whether the HUD trigger button is a hold (release = close) or a toggle.
    #[serde(default)]
    pub hud_open_mode: HudOpenMode,
    /// Opacity of the full-screen HUD background overlay (0.0 = invisible, 1.0 = opaque).
    #[serde(default = "_full_opacity")]
    pub hud_bg_opacity: f32,
    /// Hex tint color for the HUD background overlay (e.g. "#0d1520"); empty = default dark.
    #[serde(default)]
    pub hud_bg_color: String,
    pub sets: Vec<ActionSet>,
}

impl Default for QuickActionConfig {
    fn default() -> Self {
        let mut combat_wheel = WheelData::new("Combat radial menu", 6);
        combat_wheel.slots = vec![
            WheelSlotData {
                name: "Open map".into(),
                icon: "⌖".into(),
                ..default()
            },
            WheelSlotData {
                name: "Attack".into(),
                icon: "⚒".into(),
                ..default()
            },
            WheelSlotData {
                name: "Block".into(),
                icon: "✹".into(),
                ..default()
            },
            WheelSlotData {
                name: "Heal".into(),
                icon: "♧".into(),
                ..default()
            },
            WheelSlotData {
                name: "Ability".into(),
                icon: "△".into(),
                ..default()
            },
            WheelSlotData {
                name: "Sprint".into(),
                icon: "◌".into(),
                ..default()
            },
        ];
        Self {
            next_set_key: "Tab".into(),
            prev_set_key: "Q".into(),
            show_set_bar: true,
            cycle_sets: false,
            edit_shortcut: "GP:Start".into(),
            hud_open_mode: HudOpenMode::Toggle,
            hud_bg_opacity: 1.0,
            hud_bg_color: String::new(),
            sets: vec![
                ActionSet {
                    name: "Combat".into(),
                    icon: String::new(),
                    enabled: true,
                    opacity: 1.0,
                    input_override: false,
                    entries: vec![
                        SetEntry::WheelSet(WheelSetData {
                            name: "Combat radial menu set".into(),
                            wheels: vec![combat_wheel, WheelData::new("Radial menu 2", 6)],
                            stick: StickSide::Right,
                            ..default()
                        }),
                        SetEntry::Action(QuickAction {
                            name: "Interact".into(),
                            key: "E".into(),
                            icon: "◆".into(),
                            command: "interact".into(),
                            color: "#14b8a6".into(),
                            width: 90.0,
                            height: 28.0,
                            ..default()
                        }),
                        SetEntry::Action(QuickAction {
                            name: "Inventory".into(),
                            key: "I".into(),
                            icon: "◈".into(),
                            command: "none".into(),
                            color: "#8b5cf6".into(),
                            width: 80.0,
                            height: 28.0,
                            ..default()
                        }),
                    ],
                    bg_image: String::new(),
                    bg_image_opacity: 1.0,
                    next_wheel_key: String::new(),
                    prev_wheel_key: String::new(),
                    cycle_wheels: false,
                },
                ActionSet {
                    name: "Stealth".into(),
                    icon: String::new(),
                    enabled: true,
                    opacity: 1.0,
                    input_override: false,
                    entries: vec![
                        SetEntry::WheelSet(WheelSetData {
                            name: "Stealth radial menus".into(),
                            wheels: vec![WheelData::new("Stealth radial menu", 4)],
                            stick: StickSide::Right,
                            ..default()
                        }),
                        SetEntry::Action(QuickAction {
                            name: "Hide".into(),
                            key: "H".into(),
                            icon: "◐".into(),
                            command: "crouch".into(),
                            color: "#6366f1".into(),
                            width: 70.0,
                            height: 28.0,
                            ..default()
                        }),
                    ],
                    bg_image: String::new(),
                    bg_image_opacity: 1.0,
                    next_wheel_key: String::new(),
                    prev_wheel_key: String::new(),
                    cycle_wheels: false,
                },
            ],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// HUD STATE, COMPONENTS, AND RENDERING
// ─────────────────────────────────────────────────────────────────────────────────

/// Actions that can be triggered directly from the HUD.
#[derive(Clone, Debug)]
pub enum WheelHudAction {
    SetActiveSet(usize),
    PrevSet,
    NextSet,
    ToggleEditor,
    /// Add a segment to the wheel currently shown in the HUD editor.
    AddSegment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        side: SegmentInsertSide,
    },
    /// Remove the selected segment from the wheel currently shown in the HUD editor.
    RemoveSegment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    EditSegmentName {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    EditSegmentIcon {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    EditSegmentInput {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    CycleSegmentMapping {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    ToggleSegmentHold {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    CycleSegmentHoldAction {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    ToggleSegmentCloseOnApply {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    DeleteSegment {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        slot: usize,
    },
    SaveConfig,
    AddNewButton,
    ToggleSettings,
    SelectAction {
        set: usize,
        entry: usize,
    },
    DeleteAction {
        set: usize,
        entry: usize,
    },
    MoveAction {
        set: usize,
        entry: usize,
    },
    RotateAction {
        set: usize,
        entry: usize,
        delta: f32,
    },
    EditAction {
        set: usize,
        entry: usize,
    },
    SelectHudSwitch {
        set: usize,
        entry: usize,
    },
    DeleteHudSwitch {
        set: usize,
        entry: usize,
    },
    MoveHudSwitch {
        set: usize,
        entry: usize,
    },
    ResizeHudSwitch {
        set: usize,
        entry: usize,
        delta: f32,
    },
    EditHudSwitch {
        set: usize,
        entry: usize,
    },
    EditActionName {
        set: usize,
        entry: usize,
    },
    CaptureActionKey {
        set: usize,
        entry: usize,
    },
    CycleActionIcon {
        set: usize,
        entry: usize,
    },
    CycleActionMapping {
        set: usize,
        entry: usize,
    },
    ToggleActionHold {
        set: usize,
        entry: usize,
    },
    CycleHoldAction {
        set: usize,
        entry: usize,
    },
    ToggleActionCloseOnApply {
        set: usize,
        entry: usize,
    },
    ActionWidthDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    ActionHeightDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    ActionRadiusDelta {
        set: usize,
        entry: usize,
        delta: f32,
    },
    CycleActionPosition {
        set: usize,
        entry: usize,
    },
    WheelSettings {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    MoveWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    DeleteWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    ResizeWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
        delta: f32,
    },
    SelectWheel {
        set: usize,
        entry: usize,
        wheel: Option<usize>,
    },
    CloseSelection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentInsertSide {
    Before,
    After,
    Outer,
}

// ── internal helpers ─────────────────────────────────────────────────────────────

fn hud_child(
    commands: &mut Commands,
    parent: Entity,
    scene: impl bevy::scene::prelude::Scene,
) -> Entity {
    let e = commands.spawn_scene(scene).id();
    commands.entity(parent).add_child(e);
    e
}

fn hud_text(s: &str, size: f32, color: Color) -> impl bevy::scene::prelude::Scene {
    let s = s.to_string();
    let sz = size;
    bsn! {
        Text({s})
        TextFont { font_size: {FontSize::Px(sz)} }
        TextColor({color})
    }
}

fn hud_wheel_icon(commands: &mut Commands, parent: Entity, icon: &str, size: f32, index: usize) {
    let colors = [
        Color::srgb(0.68, 1.0, 0.08),
        Color::srgb(0.05, 0.20, 1.0),
        Color::srgb(1.0, 0.10, 0.12),
        Color::srgb(0.02, 0.72, 0.66),
        Color::srgb(0.08, 0.72, 0.98),
        Color::srgb(0.55, 0.55, 0.55),
    ];
    let badge = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                width: {Val::Px(size)}, height: {Val::Px(size)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: {BorderRadius::all(Val::Px(size * 0.5))},
            }
            BackgroundColor({colors[index % colors.len()]})
        },
    );
    hud_child(commands, badge, hud_text(icon, size * 0.48, Color::WHITE));
}

fn hud_clickable(
    commands: &mut Commands,
    parent: Entity,
    scene: impl bevy::scene::prelude::Scene,
    action: WheelHudAction,
    base: Color,
) -> Entity {
    let e = commands
        .spawn_scene(scene)
        .insert(WheelHudButton { action, base })
        .id();
    commands.entity(parent).add_child(e);
    e
}

/// Parse `#rrggbb` hex string into a Bevy [`Color`].
pub fn parse_hex_color(hex: &str, alpha: f32) -> Color {
    let s = hex.trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return Color::srgba(r as f32 / 255., g as f32 / 255., b as f32 / 255., alpha);
        }
    }
    Color::srgba(0.23, 0.51, 0.96, alpha)
}

pub fn hud_label_or(key: &str) -> String {
    if key.is_empty() {
        "—".into()
    } else {
        key.into()
    }
}

// ── canvas root ──────────────────────────────────────────────────────────────────

fn hud_canvas_root() -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {Val::Px(0.)}, top: {Val::Px(0.)},
            right: {Val::Px(0.)}, bottom: {Val::Px(0.)},
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor({HUD_BG.with_alpha(1.0)})
    }
}

// ── main HUD build ───────────────────────────────────────────────────────────────

pub fn build_hud_canvas(
    commands: &mut Commands,
    cfg: &QuickActionConfig,
    hud: &WheelHudState,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
    wedge_materials: &mut Assets<WedgeMaterial>,
) {
    let root = commands
        .spawn_scene(hud_canvas_root())
        .insert(WheelHudRoot)
        .id();

    // Nothing to render while the wheel overlay is closed.
    if !hud.open {
        commands.entity(root).insert(BackgroundColor(Color::NONE));
        return;
    }

    // Apply the user-configured HUD background opacity.
    let hud_bg = if cfg.hud_bg_color.is_empty() {
        HUD_BG.with_alpha(cfg.hud_bg_opacity)
    } else {
        parse_hex_color(&cfg.hud_bg_color, cfg.hud_bg_opacity)
    };
    commands.entity(root).insert(BackgroundColor(hud_bg));

    // Edit toggle button — visible only while the wheel is open, hidden when
    // the editor sidebar is already showing.
    if !hud.editor_open {
        let btn = hud_clickable(
            commands,
            root,
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    top: {Val::Px(14.)}, left: {Val::Px(14.)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                    padding: {UiRect::axes(Val::Px(10.), Val::Px(6.))},
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(5.))},
                }
                BorderColor::all(HUD_BADGE_BORDER)
                BackgroundColor({HUD_PANEL_CARD})
                Button
            },
            WheelHudAction::ToggleEditor,
            HUD_PANEL_CARD,
        );
        // Show the assigned edit shortcut icon (if it's a gamepad button).
        if let Some(lbl) = cfg.edit_shortcut.strip_prefix("GP:") {
            if let Some(path) = icon_set.embedded_icon_path(lbl) {
                let handle = asset_server.load::<Image>(path);
                let icon_e = commands
                    .spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        ImageNode::new(handle),
                    ))
                    .id();
                commands.entity(btn).add_child(icon_e);
            }
        }
        {
            let handle = asset_server.load::<Image>(
                "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-cog.png",
            );
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    ImageNode {
                        image: handle,
                        color: HUD_DIM,
                        ..default()
                    },
                ))
                .id();
            commands.entity(btn).add_child(e);
        }
        hud_child(commands, btn, hud_text("Edit", 10., HUD_DIM));
    }

    // Set tabs at the top centre (only when enabled in config).
    if cfg.show_set_bar {
        build_hud_set_tabs(commands, root, cfg, hud, asset_server, icon_set);
    }
    if hud.editor_open {
        build_hud_editor_toolbar(
            commands,
            root,
            hud.settings_open,
            hud.edit_control_focus,
            &cfg.edit_shortcut,
        );
    }

    if cfg.sets.is_empty() {
        hud_child(
            commands,
            root,
            hud_text("No sets — open the editor to add one.", 12., HUD_DIMMER),
        );
        return;
    }

    if let Some(set) = cfg.sets.get(hud.active_set) {
        // Background image for this set, if configured.
        if !set.bg_image.is_empty() {
            let handle = asset_server.load::<Image>(set.bg_image.clone());
            let bg_e = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.),
                        top: Val::Px(0.),
                        right: Val::Px(0.),
                        bottom: Val::Px(0.),
                        ..default()
                    },
                    ImageNode {
                        image: handle,
                        color: Color::WHITE.with_alpha(set.bg_image_opacity),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(bg_e);
        }

        // Clamp active_wheel_entry to a valid range.
        let n_wheels = count_wheel_entries(set);
        let target = if n_wheels == 0 {
            0
        } else {
            hud.active_wheel_entry.min(n_wheels - 1)
        };

        // Find the target-th Wheel / WheelSet entry.
        let mut rendered = false;
        let mut wcount = 0usize;
        for (ei, entry) in set.entries.iter().enumerate() {
            let is_wheel = matches!(entry, SetEntry::Wheel(_) | SetEntry::WheelSet(_));
            if !is_wheel {
                continue;
            }
            if wcount != target {
                wcount += 1;
                continue;
            }
            match entry {
                SetEntry::Wheel(w) => {
                    build_centered_wheel_hud(
                        commands,
                        root,
                        w,
                        hud.active_set,
                        ei,
                        None,
                        hud.highlighted,
                        hud.selected_wheel,
                        hud.hovered_wheel,
                        hud.editor_open,
                        hud.edit_control_focus,
                        wedge_materials,
                    );
                    rendered = true;
                }
                SetEntry::WheelSet(ws) => {
                    let wheel_index = hud
                        .active_wheel_index
                        .min(ws.wheels.len().saturating_sub(1));
                    if let Some(w) = ws.wheels.get(wheel_index) {
                        let mut display_wheel = w.clone();
                        wheelset_visuals(ws).apply_to(&mut display_wheel);
                        build_centered_wheel_hud(
                            commands,
                            root,
                            &display_wheel,
                            hud.active_set,
                            ei,
                            Some(wheel_index),
                            hud.highlighted,
                            hud.selected_wheel,
                            hud.hovered_wheel,
                            hud.editor_open,
                            hud.edit_control_focus,
                            wedge_materials,
                        );
                        rendered = true;
                    }
                }
                _ => {}
            }
            break;
        }
        if !rendered {
            hud_child(
                commands,
                root,
                hud_text("No radial menus in this set.", 11., HUD_DIMMER),
            );
        }
        build_hud_action_buttons(
            commands,
            root,
            hud.active_set,
            set,
            asset_server,
            icon_set,
            hud.flash_action_entry,
            hud.editor_open,
        );
        build_hud_switch_buttons(
            commands,
            root,
            hud.active_set,
            set,
            asset_server,
            hud.editor_open,
        );
        if hud.highlighted.is_none() && hud.selected_wheel.is_none() && hud.hovered_wheel.is_none()
        {
            if let Some((selected_set, selected_entry)) = hud.selected_action {
                if selected_set == hud.active_set {
                    if let Some(SetEntry::Action(action)) = set.entries.get(selected_entry) {
                        build_hud_action_editor_card(
                            commands,
                            root,
                            selected_set,
                            selected_entry,
                            action,
                        );
                    }
                }
            }
        }
    }
}

fn build_hud_action_editor_card(
    commands: &mut Commands,
    parent: Entity,
    set: usize,
    entry: usize,
    action: &QuickAction,
) {
    editor::components::build_hud_action_editor_card(commands, parent, set, entry, action);
}
fn hud_action_field(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    height: f32,
    action: WheelHudAction,
    value_color: Color,
) {
    let field = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                height: {Val::Px(height)},
                padding: {UiRect::horizontal(Val::Px(9.))},
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        action,
        HUD_PANEL_CARD,
    );
    hud_child(commands, field, hud_text(label, 9., HUD_DIM));
    hud_child(commands, field, hud_text(value, 11., value_color));
}

fn hud_action_stepper(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    decrement: WheelHudAction,
    increment: WheelHudAction,
) {
    let row = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                height: {Val::Px(26.)},
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: {Val::Px(4.)},
            }
        },
    );
    hud_child(commands, row, hud_text(label, 10., HUD_TEXT));
    let minus = hud_clickable(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(26.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        decrement,
        HUD_PANEL_CARD,
    );
    hud_child(commands, minus, hud_text("−", 14., HUD_TEXT));
    let value_node = hud_child(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(52.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
        },
    );
    hud_child(commands, value_node, hud_text(value, 10., HUD_DIM));
    let plus = hud_clickable(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(26.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        increment,
        HUD_PANEL_CARD,
    );
    hud_child(commands, plus, hud_text("+", 14., HUD_TEXT));
}

fn build_hud_editor_toolbar(
    commands: &mut Commands,
    parent: Entity,
    settings_open: bool,
    edit_focus: Option<usize>,
    edit_shortcut: &str,
) {
    let bar = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(14.)}, left: {Val::Px(14.)},
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: {Val::Px(6.)},
                padding: {UiRect::all(Val::Px(0.))},
            }
            BackgroundColor({Color::NONE})
        },
    );
    let close = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                padding: {UiRect::axes(Val::Px(10.), Val::Px(6.))},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({if edit_focus == Some(11) { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if edit_focus == Some(11) { HUD_TEXT } else { HUD_AMBER })
            Button
        },
        WheelHudAction::ToggleEditor,
        HUD_PANEL_CARD,
    );
    hud_child(
        commands,
        close,
        hud_text(&format!("Close  [{}]", edit_shortcut), 10., HUD_AMBER),
    );
    for (label, shortcut, action, accent, focus) in [
        (
            "Save",
            "LB / Ctrl+S",
            WheelHudAction::SaveConfig,
            HUD_GREEN,
            8,
        ),
        (
            "+ Add new",
            "RB / Ctrl+N",
            WheelHudAction::AddNewButton,
            HUD_TEXT,
            9,
        ),
        (
            if settings_open {
                "Settings ✓"
            } else {
                "Settings"
            },
            "Select / Ctrl+,",
            WheelHudAction::ToggleSettings,
            HUD_TEXT,
            10,
        ),
    ] {
        let button = hud_clickable(
            commands,
            bar,
            bsn! {
                Node {
                    height: {Val::Px(28.)},
                    padding: {UiRect::horizontal(Val::Px(10.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(3.))},
                }
                BackgroundColor({if edit_focus == Some(focus) { HUD_AMBER } else { HUD_PANEL_CARD }})
                BorderColor::all(if edit_focus == Some(focus) {
                    HUD_TEXT
                } else {
                    HUD_BADGE_BORDER
                })
                Button
            },
            action,
            HUD_PANEL_CARD,
        );
        hud_child(
            commands,
            button,
            hud_text(&format!("{label}  [{shortcut}]"), 10., accent),
        );
    }
}

/// Renders a radial wheel preview centred in the HUD.
#[allow(clippy::too_many_arguments)]
pub fn build_centered_wheel_hud(
    commands: &mut Commands,
    parent: Entity,
    wheel: &WheelData,
    set: usize,
    entry: usize,
    w_idx: Option<usize>,
    highlighted: Option<(usize, usize, Option<usize>, usize)>,
    selected_wheel: Option<(usize, usize, Option<usize>)>,
    hovered_wheel: Option<(usize, usize, Option<usize>)>,
    editor_open: bool,
    edit_control_focus: Option<usize>,
    wedge_materials: &mut Assets<WedgeMaterial>,
) {
    let n_slices = wheel.slots.len().max(1);
    let hub = hud_child(commands, parent, wheel_hub());
    commands.entity(hub).insert(Node {
        position_type: PositionType::Relative,
        left: Val::Px(wheel.offset_x),
        top: Val::Px(-wheel.offset_y),
        ..default()
    });
    if wheel.rotation != 0.0 {
        commands
            .entity(hub)
            .insert(Transform::from_rotation(Quat::from_rotation_z(
                wheel.rotation.to_radians(),
            )));
    }

    let is_pie = wheel.segment_shape == SegmentShape::Pie;
    if !is_pie {
        let bg_col = if wheel.bg_color.is_empty() {
            Color::srgba(0.096, 0.118, 0.157, wheel.bg_opacity)
        } else {
            parse_hex_color(&wheel.bg_color, wheel.bg_opacity)
        };
        hud_child(commands, hub, wheel_bg_disc(wheel.outer_radius, bg_col));
    }
    let outer_col = if wheel.outer_border.is_empty() {
        Color::srgba(0.38, 0.39, 0.39, 0.90)
    } else {
        parse_hex_color(&wheel.outer_border, 1.0)
    };
    let outer_bw = if wheel.outer_border.is_empty() {
        1.0_f32
    } else {
        wheel.outer_border_width.max(0.0)
    };
    hud_child(
        commands,
        hub,
        wheel_outer_ring(wheel.outer_radius, outer_col, outer_bw),
    );

    let slice_angle = std::f32::consts::TAU / n_slices as f32;
    let base_pw = (2.0 * wheel.outer_radius * (slice_angle / 2.0).sin() * 0.72).max(48.0);
    let base_ph = ((wheel.outer_radius - wheel.inner_radius) * 0.85).max(40.0);
    let panel_w = (base_pw * wheel.segment_scale).max(32.0);
    let panel_h = (base_ph * wheel.segment_scale).max(24.0);
    let min_dim = panel_w.min(panel_h);
    let highlight_col = parse_hex_color(&wheel.highlight_color, 1.0);
    let slice_bg = Color::srgb(0.115, 0.12, 0.12);
    let label_c = Color::srgb(0.91, 0.91, 0.89);
    let label_sz = (panel_h * 0.18).clamp(9.0, 13.0);

    for (i, slot) in wheel.slots.iter().enumerate() {
        if i >= n_slices {
            break;
        }
        let is_sel = highlighted
            .map(|(s, e, w, sl)| s == set && e == entry && w == w_idx && sl == i)
            .unwrap_or(false);
        // The reference keeps the selected sector translucent so the dark
        // wheel surface remains visible beneath the coral tint.
        let seg_color = if is_sel {
            highlight_col.with_alpha(0.38)
        } else {
            slice_bg
        };

        if is_pie {
            let (a0, a1) = slice_angles(wheel, i);
            let mat_handle = wedge_materials.add(WedgeMaterial {
                params: WedgeParams {
                    color: seg_color.to_linear().to_vec4(),
                    border_color: if is_sel {
                        highlight_col.to_linear().to_vec4()
                    } else {
                        Color::srgb(0.30, 0.31, 0.31).to_linear().to_vec4()
                    },
                    inner_r: wheel.inner_radius,
                    outer_r: wheel.outer_radius,
                    angle_start: a0,
                    angle_end: a1,
                    edge_width: if is_sel { 2.0 } else { 0.8 },
                },
            });
            let dia = wheel.outer_radius * 2.0;
            let wedge_e = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(-wheel.outer_radius),
                        top: Val::Px(-wheel.outer_radius),
                        width: Val::Px(dia),
                        height: Val::Px(dia),
                        ..default()
                    },
                    MaterialNode(mat_handle),
                ))
                .id();
            commands.entity(hub).add_child(wedge_e);
            let ctr = slice_center(wheel, i);
            let panel_e = commands
                .spawn_scene(bsn! {
                    Node {
                        position_type: PositionType::Absolute,
                        left:   {Val::Px(ctr.x - panel_w / 2.0)},
                        top:    {Val::Px(-ctr.y - panel_h / 2.0)},
                        width:  {Val::Px(panel_w)}, height: {Val::Px(panel_h)},
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: {UiRect::all(Val::Px(6.))},
                    }
                    BackgroundColor({Color::NONE})
                })
                .insert((
                    WheelHudSegmentHit {
                        set,
                        entry,
                        wheel: w_idx,
                        slot: i,
                    },
                    Interaction::None,
                    Button,
                ))
                .id();
            commands.entity(hub).add_child(panel_e);
            if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    wheel_slice_label(slot.name.to_uppercase(), label_sz, label_c),
                );
            }
            if wheel.show_icon && !slot.icon.is_empty() {
                hud_wheel_icon(
                    commands,
                    panel_e,
                    &slot.icon,
                    (panel_h * 0.42).clamp(24.0, 44.0),
                    i,
                );
            } else if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    bsn! { Node { width: {Val::Px(4.)}, height: {Val::Px(4.)} } },
                );
            }
        } else {
            let seg_br = match wheel.segment_shape {
                SegmentShape::Square => BorderRadius::all(Val::Px(0.0)),
                SegmentShape::Rounded => BorderRadius::all(Val::Px(min_dim * 0.14)),
                SegmentShape::Circle => BorderRadius::all(Val::Px(min_dim * 0.5)),
                SegmentShape::Wedge => BorderRadius {
                    top_left: Val::Px(min_dim * 0.40),
                    top_right: Val::Px(min_dim * 0.40),
                    bottom_left: Val::Px(min_dim * 0.05),
                    bottom_right: Val::Px(min_dim * 0.05),
                },
                SegmentShape::Pie => unreachable!(),
            };
            let ctr = slice_center(wheel, i);
            let panel_e = commands
                .spawn_scene(bsn! {
                    Node {
                        position_type: PositionType::Absolute,
                        left:   {Val::Px(ctr.x - panel_w / 2.0)},
                        top:    {Val::Px(-ctr.y - panel_h / 2.0)},
                        width:  {Val::Px(panel_w)}, height: {Val::Px(panel_h)},
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: {UiRect::all(Val::Px(6.))},
                        border_radius: {seg_br},
                    }
                    BackgroundColor({seg_color})
                })
                .insert((
                    WheelHudSegmentHit {
                        set,
                        entry,
                        wheel: w_idx,
                        slot: i,
                    },
                    Interaction::None,
                    Button,
                ))
                .id();
            commands.entity(hub).add_child(panel_e);
            if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    wheel_slice_label(slot.name.to_uppercase(), label_sz, label_c),
                );
            }
            if wheel.show_icon && !slot.icon.is_empty() {
                hud_wheel_icon(
                    commands,
                    panel_e,
                    &slot.icon,
                    (panel_h * 0.42).clamp(24.0, 44.0),
                    i,
                );
            } else if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    bsn! { Node { width: {Val::Px(4.)}, height: {Val::Px(4.)} } },
                );
            }
        }
    }

    // Centre hub ring.
    let disc_r = (wheel.inner_radius - 4.0).max(8.0);
    let ring_col = if wheel.inner_border.is_empty() {
        Color::srgb(0.34, 0.35, 0.35)
    } else {
        parse_hex_color(&wheel.inner_border, 1.0)
    };
    let hub_bg = if wheel.hub_color.is_empty() {
        Color::srgba(0.10, 0.10, 0.10, wheel.hub_opacity)
    } else {
        parse_hex_color(&wheel.hub_color, wheel.hub_opacity)
    };
    let inner_bw = if wheel.inner_border.is_empty() {
        1.0_f32
    } else {
        wheel.inner_border_width.max(0.0)
    };
    let center = hud_child(
        commands,
        hub,
        wheel_center_ring(disc_r, hub_bg, ring_col, inner_bw),
    );
    if editor_open {
        commands.entity(center).insert((
            WheelHudButton {
                action: WheelHudAction::SelectWheel {
                    set,
                    entry,
                    wheel: w_idx,
                },
                base: hub_bg,
            },
            Button,
            Interaction::None,
        ));
    }

    // Show highlighted slot info; show nothing by default.
    let hub_slot = highlighted.and_then(|(hs, he, hw, si)| {
        if hs == set && he == entry && hw == w_idx {
            wheel.slots.get(si)
        } else {
            None
        }
    });
    if let Some(slot) = hub_slot {
        let info_col = hud_child(
            commands,
            center,
            bsn! {
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: {Val::Px(2.)},
                }
            },
        );
        let name_sz = (disc_r * 0.22).clamp(7.0, 10.0);
        if !slot.icon.is_empty() {
            let icon_index = highlighted.map(|(_, _, _, index)| index).unwrap_or(0);
            hud_wheel_icon(
                commands,
                info_col,
                &slot.icon,
                (disc_r * 0.58).clamp(24.0, 42.0),
                icon_index,
            );
        }
        if !slot.name.is_empty() {
            hud_child(commands, info_col, hud_text(&slot.name, name_sz, HUD_DIM));
        }
        if editor_open {
            hud_child(commands, info_col, hud_text("▣  Apply", 9., HUD_DIM));
            hud_child(commands, info_col, hud_text("▣  Back", 9., HUD_DIM));
            spawn_segment_editor_card(
                commands,
                hub,
                slot,
                set,
                entry,
                w_idx,
                highlighted.map(|(_, _, _, index)| index).unwrap_or(0),
                wheel.outer_radius,
                edit_control_focus,
            );
        }
    }
    if editor_open
        && highlighted.is_none()
        && (selected_wheel == Some((set, entry, w_idx))
            || hovered_wheel == Some((set, entry, w_idx)))
    {
        spawn_wheel_settings_card(commands, hub, wheel);
    }

    // Wheel-level controls are always visible in edit mode so hovering or
    // selecting the wheel exposes the same affordances as action buttons.
    if editor_open {
        let r = wheel.outer_radius + 28.0;
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(0.0, -r),
            WheelHudAction::WheelSettings {
                set,
                entry,
                wheel: w_idx,
            },
            "⚙",
            HUD_TEXT,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(r, 0.0),
            WheelHudAction::DeleteWheel {
                set,
                entry,
                wheel: w_idx,
            },
            "×",
            HUD_AMBER,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(0.0, r),
            WheelHudAction::MoveWheel {
                set,
                entry,
                wheel: w_idx,
            },
            "↕",
            HUD_TEXT,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(-r, 0.0),
            WheelHudAction::ResizeWheel {
                set,
                entry,
                wheel: w_idx,
                delta: 10.0,
            },
            "⌗",
            HUD_TEXT,
            false,
        );
    }

    // In edit mode, place the same compact radial controls used by the
    // reference UI: plus buttons on both sides of the selected sector and a
    // trash button on its inner edge.
    if editor_open {
        if let Some((hs, he, hw, slot)) = highlighted {
            if hs == set && he == entry && hw == w_idx && slot < n_slices {
                let (a0, a1) = slice_angles(wheel, slot);
                let mid = (a0 + a1) * 0.5;
                let p = Vec2::new(
                    a0.cos() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                    a0.sin() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::Before,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(1),
                );
                let p = Vec2::new(
                    mid.cos() * (wheel.inner_radius + 2.0),
                    mid.sin() * (wheel.inner_radius + 2.0),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::RemoveSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        slot,
                    },
                    "×",
                    HUD_TEXT,
                    edit_control_focus == Some(2),
                );
                // The second boundary is the after/right insertion point.
                let p = Vec2::new(
                    a1.cos() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                    a1.sin() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::After,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(3),
                );
                // Outer-side add affordance, matching the reference editor.
                let p = Vec2::new(
                    mid.cos() * (wheel.outer_radius + 2.0),
                    mid.sin() * (wheel.outer_radius + 2.0),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::Outer,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(4),
                );
            }
        }
    }
}

fn spawn_wheel_settings_card(commands: &mut Commands, parent: Entity, wheel: &WheelData) {
    editor::components::spawn_wheel_settings_card(commands, parent, wheel);
}
fn spawn_segment_editor_card(
    commands: &mut Commands,
    parent: Entity,
    slot: &WheelSlotData,
    set: usize,
    entry: usize,
    wheel: Option<usize>,
    slot_index: usize,
    outer_radius: f32,
    edit_control_focus: Option<usize>,
) {
    editor::components::spawn_segment_editor_card(
        commands,
        parent,
        slot,
        set,
        entry,
        wheel,
        slot_index,
        outer_radius,
        edit_control_focus,
    );
}
fn spawn_radial_edit_button(
    commands: &mut Commands,
    parent: Entity,
    position: Vec2,
    action: WheelHudAction,
    label: &str,
    color: Color,
    focused: bool,
) {
    editor::components::spawn_radial_edit_button(
        commands, parent, position, action, label, color, focused,
    );
}
/// Floating quick-action buttons in the bottom-right corner.
fn build_hud_action_buttons(
    commands: &mut Commands,
    parent: Entity,
    set_index: usize,
    set: &ActionSet,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
    flash_entry: Option<usize>,
    editor_open: bool,
) {
    let btns: Vec<(usize, &QuickAction)> = set
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if let SetEntry::Action(a) = e {
                Some((i, a))
            } else {
                None
            }
        })
        .filter(|(_, a)| a.enabled)
        .collect();
    if btns.is_empty() {
        return;
    }

    let container = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(60.)}, right: {Val::Px(36.)},
                flex_direction: FlexDirection::Column,
                row_gap: {Val::Px(8.)},
                align_items: AlignItems::FlexEnd,
            }
        })
        .id();
    commands.entity(parent).add_child(container);

    for (entry_idx, qa) in btns.iter().rev() {
        let is_flash = flash_entry == Some(*entry_idx);
        let eff = (set.opacity * qa.opacity).clamp(0.05, 1.0);
        let w = qa.width.max(40.0);
        let h = qa.height.max(20.0);
        let bg = if is_flash {
            Color::srgba(0.38, 0.62, 0.95, 0.90) // bright blue flash
        } else {
            parse_hex_color(&qa.color, eff * 0.85)
        };
        let tc = HUD_TEXT.with_alpha(if is_flash { 1.0 } else { eff });
        let bc = HUD_BADGE_BORDER.with_alpha(eff);

        let row = hud_child(
            commands,
            container,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    left: {Val::Px(qa.offset_x)},
                    top: {Val::Px(-qa.offset_y)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                }
            },
        );
        if !qa.key.is_empty() {
            // GP: key → show the controller button icon; keyboard key → text badge.
            let mut showed_icon = false;
            if let Some(btn_label) = qa.key.strip_prefix("GP:") {
                if let Some(path) = icon_set.embedded_icon_path(btn_label) {
                    let handle = asset_server.load::<Image>(path);
                    let icon_e = commands
                        .spawn((
                            Node {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                ..default()
                            },
                            ImageNode::new(handle),
                        ))
                        .id();
                    commands.entity(row).add_child(icon_e);
                    showed_icon = true;
                }
            }
            if !showed_icon {
                // Keyboard fallback — bordered text badge.
                let key_disp = qa.key.strip_prefix("GP:").unwrap_or(&qa.key);
                let kb = hud_child(
                    commands,
                    row,
                    bsn! {
                        Node {
                            min_width: {Val::Px(16.)}, height: {Val::Px(16.)},
                            padding: {UiRect::horizontal(Val::Px(3.))},
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: {UiRect::all(Val::Px(1.))},
                            border_radius: {BorderRadius::all(Val::Px(2.))},
                        }
                        BorderColor::all(HUD_BADGE_BORDER)
                    },
                );
                hud_child(commands, kb, hud_text(key_disp, 8., HUD_DIM));
            }
        }
        let button_wrap = hud_child(
            commands,
            row,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    width: {Val::Px(w)}, height: {Val::Px(h)},
                }
            },
        );
        let btn_node = commands
            .spawn_scene(bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: {Val::Px(0.)}, top: {Val::Px(0.)},
                    width: {Val::Px(w)}, height: {Val::Px(h)},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(4.))},
                }
                BackgroundColor({bg})
                BorderColor::all(bc)
            })
            .insert((
                WheelHudButton {
                    action: WheelHudAction::SelectAction {
                        set: set_index,
                        entry: *entry_idx,
                    },
                    base: bg,
                },
                Button,
                Interaction::None,
            ))
            .id();
        if qa.rotation != 0.0 {
            commands
                .entity(btn_node)
                .insert(Transform::from_rotation(Quat::from_rotation_z(
                    qa.rotation.to_radians(),
                )));
        }
        commands.entity(button_wrap).add_child(btn_node);
        hud_child(commands, btn_node, hud_text(&qa.name, 10., tc));
        if editor_open {
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w * 0.5 - 11.),
                Val::Px(-24.),
                asset_server,
                "cil-camera-control",
                WheelHudAction::MoveAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w + 2.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-aperture",
                WheelHudAction::RotateAction {
                    set: set_index,
                    entry: *entry_idx,
                    delta: 15.0,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w * 0.5 - 11.),
                Val::Px(h + 2.),
                asset_server,
                "cil-trash",
                WheelHudAction::DeleteAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_AMBER,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(-24.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-cog",
                WheelHudAction::EditAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_TEXT,
            );
        }
    }
}

/// Floating HUD-switch components share the same contextual editor affordances
/// as quick-action buttons: settings, move, resize, and delete.
fn build_hud_switch_buttons(
    commands: &mut Commands,
    parent: Entity,
    set_index: usize,
    set: &ActionSet,
    asset_server: &AssetServer,
    editor_open: bool,
) {
    let switches: Vec<(usize, &HudSwitch)> = set
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if let SetEntry::HudSwitch(s) = e {
                s.enabled.then_some((i, s))
            } else {
                None
            }
        })
        .collect();
    if switches.is_empty() {
        return;
    }
    let container = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(104.)}, right: {Val::Px(36.)},
                flex_direction: FlexDirection::Column, row_gap: {Val::Px(8.)},
                align_items: AlignItems::FlexEnd,
            }
        })
        .id();
    commands.entity(parent).add_child(container);
    for (entry, switch) in switches.iter().rev() {
        let row = hud_child(
            commands,
            container,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    left: {Val::Px(switch.offset_x)}, top: {Val::Px(-switch.offset_y)},
                    flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                }
            },
        );
        let button = hud_clickable(
            commands,
            row,
            bsn! {
                Node {
                    width: {Val::Px(switch.width.max(40.))}, height: {Val::Px(switch.height.max(20.))},
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))}, border_radius: {BorderRadius::all(Val::Px(4.))},
                }
                BackgroundColor({Color::srgba(0.38, 0.26, 0.62, 0.85)})
                BorderColor::all(HUD_BADGE_BORDER)
                Button
            },
            WheelHudAction::SelectHudSwitch {
                set: set_index,
                entry: *entry,
            },
            HUD_PANEL_CARD,
        );
        hud_child(commands, button, hud_text(&switch.name, 10., HUD_TEXT));
        if editor_open {
            let w = switch.width.max(40.);
            let h = switch.height.max(20.);
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w * 0.5 - 11.),
                Val::Px(-24.),
                asset_server,
                "cil-camera-control",
                WheelHudAction::MoveHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w + 2.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-aperture",
                WheelHudAction::ResizeHudSwitch {
                    set: set_index,
                    entry: *entry,
                    delta: 8.0,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w * 0.5 - 11.),
                Val::Px(h + 2.),
                asset_server,
                "cil-trash",
                WheelHudAction::DeleteHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_AMBER,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(-24.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-cog",
                WheelHudAction::EditHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_TEXT,
            );
        }
    }
}

fn spawn_action_edge_button(
    commands: &mut Commands,
    parent: Entity,
    left: Val,
    top: Val,
    asset_server: &AssetServer,
    icon: &str,
    action: WheelHudAction,
    color: Color,
) {
    let owner = hud_control_owner(&action);
    let button = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {left}, top: {top},
                width: {Val::Px(22.)}, height: {Val::Px(22.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(11.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(if color == HUD_AMBER { HUD_AMBER } else { HUD_BADGE_BORDER })
            Button
        },
        action,
        HUD_PANEL_CARD,
    );
    if let Some(owner) = owner {
        commands
            .entity(button)
            .insert((HudContextControl { owner }, Visibility::Hidden));
    }
    let handle = asset_server.load::<Image>(format!(
        "embedded://bevy_quick_action_hud/embedded/icons/editor/{icon}.png"
    ));
    let icon_node = commands
        .spawn((
            Node {
                width: Val::Px(14.),
                height: Val::Px(14.),
                ..default()
            },
            ImageNode {
                image: handle,
                color,
                ..default()
            },
        ))
        .id();
    commands.entity(button).add_child(icon_node);
}

fn hud_control_owner(action: &WheelHudAction) -> Option<HudControlOwner> {
    match action {
        WheelHudAction::MoveAction { set, entry }
        | WheelHudAction::RotateAction { set, entry, .. }
        | WheelHudAction::DeleteAction { set, entry }
        | WheelHudAction::EditAction { set, entry }
        | WheelHudAction::EditActionName { set, entry }
        | WheelHudAction::CaptureActionKey { set, entry }
        | WheelHudAction::CycleActionIcon { set, entry }
        | WheelHudAction::CycleActionMapping { set, entry }
        | WheelHudAction::ToggleActionHold { set, entry }
        | WheelHudAction::CycleHoldAction { set, entry }
        | WheelHudAction::ToggleActionCloseOnApply { set, entry }
        | WheelHudAction::ActionWidthDelta { set, entry, .. }
        | WheelHudAction::ActionHeightDelta { set, entry, .. } => {
            Some(HudControlOwner::Action(*set, *entry))
        }
        WheelHudAction::WheelSettings { set, entry, wheel }
        | WheelHudAction::MoveWheel { set, entry, wheel }
        | WheelHudAction::SelectWheel { set, entry, wheel }
        | WheelHudAction::ResizeWheel {
            set, entry, wheel, ..
        }
        | WheelHudAction::DeleteWheel { set, entry, wheel } => {
            Some(HudControlOwner::Wheel(*set, *entry, *wheel))
        }
        WheelHudAction::AddSegment {
            set, entry, wheel, ..
        }
        | WheelHudAction::RemoveSegment {
            set, entry, wheel, ..
        }
        | WheelHudAction::EditSegmentName {
            set, entry, wheel, ..
        }
        | WheelHudAction::EditSegmentIcon {
            set, entry, wheel, ..
        }
        | WheelHudAction::EditSegmentInput {
            set, entry, wheel, ..
        }
        | WheelHudAction::CycleSegmentMapping {
            set, entry, wheel, ..
        }
        | WheelHudAction::ToggleSegmentHold {
            set, entry, wheel, ..
        }
        | WheelHudAction::CycleSegmentHoldAction {
            set, entry, wheel, ..
        }
        | WheelHudAction::ToggleSegmentCloseOnApply {
            set, entry, wheel, ..
        }
        | WheelHudAction::DeleteSegment {
            set, entry, wheel, ..
        } => Some(HudControlOwner::Wheel(*set, *entry, *wheel)),
        WheelHudAction::MoveHudSwitch { set, entry }
        | WheelHudAction::ResizeHudSwitch { set, entry, .. }
        | WheelHudAction::DeleteHudSwitch { set, entry }
        | WheelHudAction::EditHudSwitch { set, entry }
        | WheelHudAction::SelectHudSwitch { set, entry } => {
            Some(HudControlOwner::HudSwitch(*set, *entry))
        }
        _ => None,
    }
}

/// Set-selection tab bar pinned to the bottom of the HUD.
fn build_hud_set_tabs(
    commands: &mut Commands,
    parent: Entity,
    cfg: &QuickActionConfig,
    hud: &WheelHudState,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
) {
    let bar = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(12.)}, left: {Val::Px(0.)}, right: {Val::Px(0.)},
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
        })
        .id();
    commands.entity(parent).add_child(bar);

    let prev_idx = hud.active_set.saturating_sub(1);
    let larrow = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                width: {Val::Px(28.)}, height: {Val::Px(32.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::left(Val::Px(6.))},
            }
            BorderColor::all(HUD_SIDEBAR_BORDER)
            BackgroundColor({HUD_PANEL_CARD})
            Button
        },
        WheelHudAction::SetActiveSet(prev_idx),
        HUD_PANEL_CARD,
    );
    // Chevron-left PNG icon; gamepad icon overlays it when a button is assigned.
    {
        let handle = asset_server.load::<Image>(
            "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-chevron-left.png",
        );
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    color: HUD_DIM,
                    ..default()
                },
            ))
            .id();
        commands.entity(larrow).add_child(e);
    }
    // Overlay the assigned prev-set icon if it's a gamepad button.
    if let Some(lbl) = cfg.prev_set_key.strip_prefix("GP:") {
        if let Some(path) = icon_set.embedded_icon_path(lbl) {
            let handle = asset_server.load::<Image>(path);
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    ImageNode::new(handle),
                ))
                .id();
            commands.entity(larrow).add_child(e);
        }
    }

    for (i, set) in cfg.sets.iter().enumerate() {
        let active = i == hud.active_set;
        let (bg, tc, bc) = if active {
            (Color::srgba(0.38, 0.62, 0.95, 0.20), HUD_TEXT, HUD_BLUE)
        } else {
            (HUD_PANEL_CARD, HUD_DIM, HUD_SIDEBAR_BORDER)
        };
        let tab = hud_clickable(
            commands,
            bar,
            bsn! {
                Node {
                    padding: {UiRect::axes(Val::Px(14.), Val::Px(7.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                }
                BorderColor::all(bc)
                BackgroundColor({bg})
                Button
            },
            WheelHudAction::SetActiveSet(i),
            bg,
        );
        hud_child(commands, tab, hud_text(&set.name, 11., tc));
    }

    let next_idx = (hud.active_set + 1).min(cfg.sets.len().saturating_sub(1));
    let rarrow = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                width: {Val::Px(28.)}, height: {Val::Px(32.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::right(Val::Px(6.))},
            }
            BorderColor::all(HUD_SIDEBAR_BORDER)
            BackgroundColor({HUD_PANEL_CARD})
            Button
        },
        WheelHudAction::SetActiveSet(next_idx),
        HUD_PANEL_CARD,
    );
    // Chevron-right PNG icon; gamepad icon overlays it when a button is assigned.
    {
        let handle = asset_server.load::<Image>(
            "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-chevron-right.png",
        );
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    color: HUD_DIM,
                    ..default()
                },
            ))
            .id();
        commands.entity(rarrow).add_child(e);
    }
    // Overlay the assigned next-set icon if it's a gamepad button.
    if let Some(lbl) = cfg.next_set_key.strip_prefix("GP:") {
        if let Some(path) = icon_set.embedded_icon_path(lbl) {
            let handle = asset_server.load::<Image>(path);
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    ImageNode::new(handle),
                ))
                .id();
            commands.entity(rarrow).add_child(e);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// WheelHudPlugin
// ─────────────────────────────────────────────────────────────────────────────────

/// Renders a full-screen HUD showing the active [`QuickActionConfig`] set.
///
/// Add this plugin (alongside [`WheelMenuPlugin`]) to display wheels and
/// quick-action buttons.  Add [`crate::editor::QuickActionEditorPlugin`] on top
/// to get the editor sidebar.
///
/// ```ignore
/// app.add_plugins((WheelMenuPlugin, WheelHudPlugin));
/// ```
/// Backward-compat wrapper — use [`QuickActionHudPlugin::default()`] instead.
///
/// Provides core wheel logic + HUD canvas (no editor).
pub struct WheelHudPlugin;
impl Plugin for WheelHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickActionHudPlugin::default());
    }
}

fn hud_button_feedback(
    mut buttons: Query<(&WheelHudButton, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (btn, interaction, mut bg) in &mut buttons {
        let next = match interaction {
            Interaction::Hovered => BackgroundColor(Color::srgba(1., 1., 1., 0.05)),
            Interaction::Pressed => BackgroundColor(Color::srgba(0.38, 0.62, 0.95, 0.16)),
            Interaction::None => BackgroundColor(btn.base),
        };
        if *bg != next {
            *bg = next;
        }
    }
}

fn hud_context_visibility(
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

/// Updates [`WheelHudState::highlighted`] while the HUD wheel is open.
///
/// Uses **release-to-use**: the slot that was highlighted when the stick
/// returns to the dead-zone is emitted as a [`HudSegmentSelected`] event.
/// The stick side (L/R) is read from the active wheel entry's `stick` field.
fn hud_stick_nav(
    gamepads: Query<&Gamepad>,
    mut hud: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    mut select_ev: MessageWriter<HudSegmentSelected>,
) {
    if !hud.open || hud.editor_open {
        return;
    }

    // Locate the active wheel entry (honoring active_wheel_entry).
    let Some(set) = cfg.sets.get(hud.active_set) else {
        return;
    };
    let mut found: Option<(usize, Option<usize>, usize, StickSide)> = None;
    let target = hud.active_wheel_entry;
    let mut wcount = 0usize;
    for (ei, entry) in set.entries.iter().enumerate() {
        let is_wheel = matches!(entry, SetEntry::Wheel(_) | SetEntry::WheelSet(_));
        if !is_wheel {
            continue;
        }
        if wcount != target {
            wcount += 1;
            continue;
        }
        match entry {
            SetEntry::Wheel(w) => {
                found = Some((ei, None, w.slots.len(), w.stick));
            }
            SetEntry::WheelSet(ws) => {
                let wheel_index = hud
                    .active_wheel_index
                    .min(ws.wheels.len().saturating_sub(1));
                if let Some(w) = ws.wheels.get(wheel_index) {
                    found = Some((ei, Some(wheel_index), w.slots.len(), ws.stick));
                }
            }
            _ => {}
        }
        break;
    }
    let Some((entry_idx, wheel_idx, n_slots, stick_side)) = found else {
        return;
    };
    if n_slots == 0 {
        return;
    }

    // Read raw gamepad axes for the configured stick.
    let mut stick = Vec2::ZERO;
    if let Some(gamepad) = gamepads.iter().next() {
        let (xa, ya) = match stick_side {
            StickSide::Right => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
            StickSide::Left => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
        };
        stick = Vec2::new(
            gamepad.get(xa).unwrap_or(0.0),
            gamepad.get(ya).unwrap_or(0.0),
        );
    }

    const DEADZONE: f32 = 0.2;
    let prev = hud.highlighted;

    let new_highlight = if stick.length() < DEADZONE {
        if hud.editor_open {
            prev
        } else {
            None
        }
    } else {
        // Same angle mapping as WheelData::arc_offset default (FRAC_PI_6).
        let a = stick.y.atan2(stick.x);
        let rel = (a - std::f32::consts::FRAC_PI_6).rem_euclid(std::f32::consts::TAU);
        let idx = ((rel / std::f32::consts::TAU) * n_slots as f32).floor() as usize;
        Some((hud.active_set, entry_idx, wheel_idx, idx.min(n_slots - 1)))
    };

    if prev != new_highlight {
        debug!(
            "[hud] stick highlight changed: {:?} -> {:?} editor_open={}",
            prev, new_highlight, hud.editor_open
        );
        // Release-to-use: emit selection when stick returns to dead-zone.
        if let (Some((s, e, w, slot)), None) = (prev, new_highlight) {
            if !hud.editor_open {
                // Normal mode: fire selection and optionally close.
                let slot_close = cfg
                    .sets
                    .get(s)
                    .and_then(|set| set.entries.get(e))
                    .and_then(|entry| match (entry, w) {
                        (SetEntry::Wheel(wd), None) => wd.slots.get(slot),
                        (SetEntry::WheelSet(ws), Some(wi)) => {
                            ws.wheels.get(wi).and_then(|wd| wd.slots.get(slot))
                        }
                        _ => None,
                    })
                    .map(|s| s.close_on_select)
                    .unwrap_or(false);

                select_ev.write(HudSegmentSelected {
                    set: s,
                    entry: e,
                    wheel: w,
                    slot,
                });

                if slot_close {
                    hud.open = false;
                }
            }
            // Dry-run mode: highlight clears visually — no event, no close.
        }
        hud.highlighted = new_highlight;
        if hud.editor_open && new_highlight.is_some() {
            hud.mouse_hovered_segment = None;
            hud.edit_control_focus = Some(0);
        }
        hud.dirty = true;
    }
}

/// Counts down the dry-run flash timer.  When expired it clears the flash entry and
/// triggers a HUD rebuild so the button returns to its normal colour.
fn tick_hud_dry_run_flash(time: Res<Time>, mut hud: ResMut<WheelHudState>) {
    // Only tick while a flash is active and the HUD is not already queued for rebuild.
    if hud.flash_action_entry.is_some() && !hud.dirty {
        hud.flash_action_ttl -= time.delta_secs();
        if hud.flash_action_ttl <= 0.0 {
            hud.flash_action_entry = None;
            hud.dirty = true;
        }
    }
}

fn rebuild_hud(
    mut commands: Commands,
    mut hud: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    asset_server: Res<AssetServer>,
    icon_set: Res<GamepadIconSet>,
    old_hud: Query<Entity, With<WheelHudRoot>>,
    children: Query<&Children>,
    mut wedge_materials: ResMut<Assets<WedgeMaterial>>,
) {
    if !hud.dirty {
        return;
    }
    debug!("[hud] rebuild requested: open={} editor_open={} active_set={} selected_action={:?} hovered_action={:?} selected_wheel={:?} hovered_wheel={:?}",
        hud.open, hud.editor_open, hud.active_set, hud.selected_action, hud.hovered_action,
        hud.selected_wheel, hud.hovered_wheel);
    hud.dirty = false;
    if cfg
        .sets
        .get(hud.active_set)
        .is_none_or(|page| !page.enabled)
    {
        if let Some(page) = enabled_hud_pages(&cfg).first().copied() {
            hud.active_set = page;
        }
    }

    debug!(
        "[hud] rebuild_hud — open={} editor_open={} active_set={} active_wheel_entry={}",
        hud.open, hud.editor_open, hud.active_set, hud.active_wheel_entry
    );

    for e in &old_hud {
        debug!("[hud] recursively despawning HUD root {:?}", e);
        despawn_hud_tree(&mut commands, e, &children);
        commands.entity(e).despawn();
    }

    if !cfg.sets.is_empty() && hud.active_set >= cfg.sets.len() {
        hud.active_set = cfg.sets.len() - 1;
    }

    build_hud_canvas(
        &mut commands,
        &cfg,
        &hud,
        &asset_server,
        *icon_set,
        &mut wedge_materials,
    );
}

fn despawn_hud_tree(commands: &mut Commands, entity: Entity, children: &Query<&Children>) {
    if let Ok(kids) = children.get(entity) {
        for child in kids.iter() {
            despawn_hud_tree(commands, child, children);
            commands.entity(child).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ─── RON serialisation ──────────────────────────────────────────────────────

    #[test]
    fn ron_round_trip() {
        let cfg = QuickActionConfig::default();
        let serialized = ron::ser::to_string_pretty(&cfg, ron::ser::PrettyConfig::default())
            .expect("serialize default config");
        let deserialized: QuickActionConfig =
            ron::from_str(&serialized).expect("deserialize round-tripped config");
        assert_eq!(cfg.sets.len(), deserialized.sets.len());
        assert_eq!(cfg.next_set_key, deserialized.next_set_key);
        assert_eq!(cfg.prev_set_key, deserialized.prev_set_key);
        assert_eq!(cfg.hud_open_mode, deserialized.hud_open_mode);
    }

    // ─── GamepadIconSet ──────────────────────────────────────────────────────────

    #[test]
    fn gamepad_icon_set_xbox_default() {
        assert_eq!(GamepadIconSet::default(), GamepadIconSet::Xbox);
    }

    #[test]
    fn gamepad_icon_set_from_ids_xbox() {
        let set = GamepadIconSet::from_ids(Some(0x045E), Some(0x0202));
        assert_eq!(set, GamepadIconSet::Xbox);
    }

    #[test]
    fn gamepad_icon_set_from_ids_ps5() {
        let set = GamepadIconSet::from_ids(Some(0x054C), Some(0x0CE6));
        assert_eq!(set, GamepadIconSet::PS5);
    }

    #[test]
    fn gamepad_icon_set_from_ids_nintendo() {
        let set = GamepadIconSet::from_ids(Some(0x057E), Some(0x2009));
        assert_eq!(set, GamepadIconSet::Switch);
    }

    #[test]
    fn gamepad_icon_set_from_name_ps5() {
        let set = GamepadIconSet::from_name("DualSense Wireless Controller");
        assert_eq!(set, GamepadIconSet::PS5);
    }

    #[test]
    fn gamepad_icon_set_from_name_ps4() {
        let set = GamepadIconSet::from_name("DualShock 4");
        assert_eq!(set, GamepadIconSet::PS4);
    }

    #[test]
    fn gamepad_icon_set_from_name_switch() {
        let set = GamepadIconSet::from_name("Nintendo Switch Pro Controller");
        assert_eq!(set, GamepadIconSet::Switch);
    }

    #[test]
    fn gamepad_icon_set_icon_path_xbox_a() {
        let set = GamepadIconSet::Xbox;
        let path = set.icon_path("A");
        assert!(path.is_some());
        assert!(path.unwrap().contains("T_X_A_Color.png"));
    }

    #[test]
    fn gamepad_icon_set_icon_path_unknown_label() {
        let set = GamepadIconSet::Xbox;
        assert!(set.icon_path("UNKNOWN").is_none());
    }

    #[test]
    fn gamepad_icon_set_base_path() {
        assert!(GamepadIconSet::Xbox.base_path().contains("XGamepad"));
        assert!(GamepadIconSet::PS4.base_path().contains("P4Gamepad"));
        assert!(GamepadIconSet::PS5.base_path().contains("P5Gamepad"));
        assert!(GamepadIconSet::Switch.base_path().contains("SGamepad"));
    }

    // ─── WheelState ──────────────────────────────────────────────────────────────

    #[test]
    fn wheel_state_default_dir_is_zero() {
        let state = WheelState::default();
        assert_eq!(state.dir, Vec2::ZERO);
        assert!(state.hovered.is_none());
        assert!(!state.open);
    }

    #[test]
    fn wheel_state_hovered_updates() {
        let mut state = WheelState {
            dir: Vec2::new(1.0, 0.0),
            hovered: Some(0),
            ..Default::default()
        };
        assert_eq!(state.hovered, Some(0));
        state.hovered = None;
        assert!(state.hovered.is_none());
    }

    // ─── WheelSlot ───────────────────────────────────────────────────────────────

    #[test]
    fn wheel_slot_new_empty() {
        let slot: WheelSlot = WheelSlot::new(vec![]);
        assert!(slot.items.is_empty());
        assert_eq!(slot.current_item, 0);
    }

    #[test]
    fn wheel_slot_current_none_when_empty() {
        let slot = WheelSlot::new(vec![]);
        assert!(slot.current().is_none());
    }

    #[test]
    fn wheel_slot_cycle_next() {
        let mut slot = WheelSlot::new(vec![
            ActionItem::Weapon {
                name: "Sword".into(),
                icon: "⚔".into(),
            },
            ActionItem::Weapon {
                name: "Bow".into(),
                icon: "🏹".into(),
            },
        ]);
        assert_eq!(slot.current_item, 0);
        slot.cycle_next();
        assert_eq!(slot.current_item, 1);
        slot.cycle_next();
        assert_eq!(slot.current_item, 0); // wraps around
    }

    #[test]
    fn wheel_slot_cycle_prev() {
        let mut slot = WheelSlot::new(vec![
            ActionItem::Weapon {
                name: "Sword".into(),
                icon: "⚔".into(),
            },
            ActionItem::Weapon {
                name: "Bow".into(),
                icon: "🏹".into(),
            },
        ]);
        slot.cycle_prev();
        assert_eq!(slot.current_item, 1); // wraps around
    }

    #[test]
    fn wheel_slot_cycle_empty_noop() {
        let mut slot = WheelSlot::new(vec![]);
        slot.cycle_next();
        assert_eq!(slot.current_item, 0);
        slot.cycle_prev();
        assert_eq!(slot.current_item, 0);
    }

    // ─── ActionItem ──────────────────────────────────────────────────────────────

    #[test]
    fn action_item_weapon_label() {
        let item = ActionItem::Weapon {
            name: "Sword".into(),
            icon: "⚔".into(),
        };
        assert_eq!(item.label(), "Sword");
        assert_eq!(item.icon(), "⚔");
    }

    #[test]
    fn action_item_spell_label() {
        let item = ActionItem::Spell {
            name: "Fireball".into(),
            icon: "🔥".into(),
        };
        assert_eq!(item.label(), "Fireball");
        assert_eq!(item.icon(), "🔥");
    }

    #[test]
    fn action_item_consumable_label() {
        let item = ActionItem::Consumable {
            name: "Potion".into(),
            icon: "💊".into(),
            count: 5,
        };
        assert_eq!(item.label(), "Potion");
        assert_eq!(item.icon(), "💊");
    }

    #[test]
    fn action_item_shout_label() {
        let item = ActionItem::Shout {
            name: "Fus Ro".into(),
            icon: "🗣".into(),
        };
        assert_eq!(item.label(), "Fus Ro");
    }

    // ─── WheelMenuConfig ─────────────────────────────────────────────────────────

    #[test]
    fn wheel_menu_config_default() {
        let cfg = WheelMenuConfig::default();
        assert_eq!(cfg.time_mode, TimeMode::Normal);
        assert_eq!(cfg.casting_mode, CastingMode::Vanilla);
        assert_eq!(cfg.toggle_mode, WheelToggleMode::Hold);
        assert!(cfg.auto_snap);
    }

    // ─── CastingMode ─────────────────────────────────────────────────────────────

    #[test]
    fn casting_mode_default_is_vanilla() {
        assert_eq!(CastingMode::default(), CastingMode::Vanilla);
    }

    #[test]
    fn casting_mode_hold_to_activate() {
        match (CastingMode::HoldToActivate { duration: 0.8 }) {
            CastingMode::HoldToActivate { duration } => assert!((duration - 0.8).abs() < 1e-6),
            _ => panic!("expected HoldToActivate"),
        }
    }

    // ─── TimeMode ────────────────────────────────────────────────────────────────

    #[test]
    fn time_mode_default_is_normal() {
        assert_eq!(TimeMode::default(), TimeMode::Normal);
    }

    #[test]
    fn time_mode_slow_scale() {
        match TimeMode::Slow(0.2) {
            TimeMode::Slow(scale) => assert!((scale - 0.2).abs() < 1e-6),
            _ => panic!("expected Slow"),
        }
    }

    // ─── WheelToggleMode ─────────────────────────────────────────────────────────

    #[test]
    fn toggle_mode_default_is_hold() {
        assert_eq!(WheelToggleMode::default(), WheelToggleMode::Hold);
    }

    // ─── WheelSet ────────────────────────────────────────────────────────────────

    #[test]
    fn wheel_set_default() {
        let set = WheelSet::default();
        assert_eq!(set.active, 0);
        assert_eq!(set.count, 1);
        assert_eq!(set.prev_button, GamepadButton::LeftTrigger);
        assert_eq!(set.next_button, GamepadButton::RightTrigger);
    }

    // ─── WheelStyle ──────────────────────────────────────────────────────────────

    #[test]
    fn wheel_style_default_colors() {
        let style = WheelStyle::default();
        assert_eq!(style.skin, "default");
        assert_eq!(style.base_color, [0.08, 0.12, 0.18, 0.85]);
    }

    #[test]
    fn wheel_style_color_conversion() {
        let style = WheelStyle::default();
        let base = style.base();
        assert!((base.to_srgba().alpha - 0.85).abs() < 0.01);
    }

    // ─── WheelHoldState ──────────────────────────────────────────────────────────

    #[test]
    fn wheel_hold_state_default() {
        let state = WheelHoldState::default();
        assert!((state.progress - 0.0).abs() < f32::EPSILON);
        assert!(!state.holding);
    }

    // ─── WheelSliceCount ─────────────────────────────────────────────────────────

    #[test]
    fn wheel_slice_count_default() {
        let count = WheelSliceCount::default();
        assert_eq!(count.current, 0);
        assert_eq!(count.max, 0);
        assert_eq!(count.low_threshold, 0);
        assert!(!count.low_notified);
    }

    // ─── QuickActionConfig ───────────────────────────────────────────────────────

    #[test]
    fn quick_action_config_default_has_sets() {
        let cfg = QuickActionConfig::default();
        assert!(!cfg.sets.is_empty(), "default config should have sets");
    }

    #[test]
    fn quick_action_config_default_edit_shortcut() {
        let cfg = QuickActionConfig::default();
        assert_eq!(cfg.edit_shortcut, "GP:Start");
    }

    #[test]
    fn quick_action_config_default_set_count() {
        let cfg = QuickActionConfig::default();
        assert_eq!(cfg.sets.len(), 2);
    }

    // ─── QuickAction ─────────────────────────────────────────────────────────────

    #[test]
    fn quick_action_default() {
        let qa = QuickAction::default();
        assert_eq!(qa.name, "Action");
        assert!(qa.enabled);
        assert!(qa.show_on_menu);
        assert_eq!(qa.shape, ActionShape::Rounded);
    }

    // ─── WheelData ───────────────────────────────────────────────────────────────

    #[test]
    fn wheel_data_default_has_one_slot() {
        let wd = WheelData::default();
        assert_eq!(wd.slots.len(), 1);
    }

    #[test]
    fn wheel_data_new_creates_n_slots() {
        let wd = WheelData::new("Test", 6);
        assert_eq!(wd.slots.len(), 6);
        assert_eq!(wd.name, "Test");
    }

    #[test]
    fn wheel_data_new_min_one_slot() {
        let wd = WheelData::new("Min", 0);
        assert_eq!(wd.slots.len(), 1);
    }

    // ─── WheelSlotData ───────────────────────────────────────────────────────────

    #[test]
    fn wheel_slot_data_default_is_empty() {
        let sd = WheelSlotData::default();
        assert!(sd.name.is_empty());
        assert!(sd.items.is_empty());
    }

    #[test]
    fn wheel_slot_data_named() {
        let sd = WheelSlotData::named("Test Slot");
        assert_eq!(sd.name, "Test Slot");
    }

    // ─── Enums ────────────────────────────────────────────────────────────────────

    #[test]
    fn position_mode_cycle() {
        assert_eq!(PositionMode::Relative.next(), PositionMode::Absolute);
        assert_eq!(PositionMode::Absolute.next(), PositionMode::Relative);
    }

    #[test]
    fn action_shape_cycle() {
        let shapes = [
            ActionShape::Rounded,
            ActionShape::Round,
            ActionShape::Square,
            ActionShape::Diamond,
        ];
        for i in 0..shapes.len() {
            assert_eq!(shapes[i].next(), shapes[(i + 1) % shapes.len()]);
        }
    }

    #[test]
    fn wheel_theme_cycle() {
        assert_eq!(WheelTheme::Dark.next(), WheelTheme::Light);
        assert_eq!(WheelTheme::Light.next(), WheelTheme::Dark);
    }

    #[test]
    fn segment_shape_cycle() {
        let shapes = [
            SegmentShape::Rounded,
            SegmentShape::Square,
            SegmentShape::Circle,
            SegmentShape::Wedge,
            SegmentShape::Pie,
        ];
        for i in 0..shapes.len() {
            assert_eq!(shapes[i].next(), shapes[(i + 1) % shapes.len()]);
        }
    }

    #[test]
    fn hud_open_mode_cycle() {
        assert_eq!(HudOpenMode::Hold.next(), HudOpenMode::Toggle);
        assert_eq!(HudOpenMode::Toggle.next(), HudOpenMode::Hold);
    }

    #[test]
    fn stick_side_cycle() {
        assert_eq!(StickSide::Right.next(), StickSide::Left);
        assert_eq!(StickSide::Left.next(), StickSide::Right);
    }

    // ─── HudOpenMode ─────────────────────────────────────────────────────────────

    #[test]
    fn hud_open_mode_labels() {
        assert_eq!(HudOpenMode::Hold.label(), "Hold");
        assert_eq!(HudOpenMode::Toggle.label(), "Toggle");
    }

    // ─── StickSide ───────────────────────────────────────────────────────────────

    #[test]
    fn stick_side_labels() {
        assert_eq!(StickSide::Right.label(), "R Stick");
        assert_eq!(StickSide::Left.label(), "L Stick");
    }

    // ─── Palette helpers ─────────────────────────────────────────────────────────

    #[test]
    fn cycle_palette_wraps() {
        let list = &["a", "b", "c"];
        assert_eq!(cycle_palette(list, "c"), "a");
        assert_eq!(cycle_palette(list, "a"), "b");
    }

    #[test]
    fn cycle_palette_unknown_starts_at_zero() {
        let list = &["x", "y", "z"];
        assert_eq!(cycle_palette(list, "unknown"), "y");
    }

    #[test]
    fn icon_palette_not_empty() {
        assert!(!ICON_PALETTE.is_empty());
    }

    #[test]
    fn command_palette_not_empty() {
        assert!(!COMMAND_PALETTE.is_empty());
    }

    // ─── parse_hex_color ─────────────────────────────────────────────────────────

    #[test]
    fn parse_hex_color_valid() {
        let c = parse_hex_color("#3b82f6", 1.0);
        let srgba = c.to_srgba();
        assert!((srgba.red - 0.231).abs() < 0.01);
        assert!((srgba.green - 0.509).abs() < 0.01);
        assert!((srgba.blue - 0.964).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_invalid_fallback() {
        let c = parse_hex_color("not-a-color", 0.5);
        let srgba = c.to_srgba();
        assert!((srgba.alpha - 0.5).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_empty() {
        let c = parse_hex_color("", 1.0);
        let srgba = c.to_srgba();
        assert!((srgba.alpha - 1.0).abs() < 0.01);
    }

    // ─── WheelHudState ───────────────────────────────────────────────────────────

    #[test]
    fn wheel_hud_state_default() {
        let state = WheelHudState::default();
        assert!(state.dirty);
        assert!(!state.open);
        assert!(!state.editor_open);
        assert_eq!(state.active_set, 0);
    }

    // ─── count_wheel_entries ─────────────────────────────────────────────────────

    #[test]
    fn count_wheel_entries_empty_set() {
        let set = ActionSet {
            name: "Empty".into(),
            entries: vec![],
            ..default()
        };
        assert_eq!(count_wheel_entries(&set), 0);
    }

    #[test]
    fn count_wheel_entries_mixed() {
        let set = ActionSet {
            name: "Mixed".into(),
            entries: vec![
                SetEntry::Wheel(WheelData::default()),
                SetEntry::Action(QuickAction::default()),
                SetEntry::WheelSet(WheelSetData::default()),
                SetEntry::Action(QuickAction::default()),
            ],
            ..default()
        };
        assert_eq!(count_wheel_entries(&set), 2);
    }

    // ─── Resolve input ───────────────────────────────────────────────────────────

    #[test]
    fn resolve_input_global_only() {
        let global = GlobalBindings {
            bindings: {
                let mut m = HashMap::new();
                m.insert(InputAction::PrimaryConfirm, WheelAction::UseSlot);
                m
            },
        };
        let result = resolve_input(InputAction::PrimaryConfirm, None, None, &global);
        assert_eq!(result, Some(WheelAction::UseSlot));
    }

    #[test]
    fn resolve_input_slot_overrides_wheel() {
        let global = GlobalBindings::default();
        let wheel = WheelInputOverride {
            bindings: {
                let mut m = HashMap::new();
                m.insert(InputAction::PrimaryConfirm, WheelAction::UseSlot);
                m
            },
            priority: 0,
        };
        let slot = WheelInputOverride {
            bindings: {
                let mut m = HashMap::new();
                m.insert(InputAction::PrimaryConfirm, WheelAction::UseItem(3));
                m
            },
            priority: 0,
        };
        let result = resolve_input(
            InputAction::PrimaryConfirm,
            Some(&slot),
            Some(&wheel),
            &global,
        );
        assert_eq!(result, Some(WheelAction::UseItem(3)));
    }

    #[test]
    fn resolve_input_unbound_returns_none() {
        let global = GlobalBindings::default();
        let result = resolve_input(InputAction::Custom(99), None, None, &global);
        assert_eq!(result, None);
    }

    // ─── Editor actions ──────────────────────────────────────────────────────────

    #[test]
    fn selection_default_is_none() {
        assert_eq!(editor::Selection::default(), editor::Selection::None);
    }

    #[test]
    fn edit_focus_default_is_none() {
        assert_eq!(editor::EditFocus::default(), editor::EditFocus::None);
    }

    // ─── Gamepad button label mapping ────────────────────────────────────────────

    #[test]
    fn gamepad_btn_label_mapping() {
        // The editor module has a gamepad_btn_label function, but it's in a
        // different module. We test the mapping through the icon set.
        assert_eq!(GamepadIconSet::Xbox.base_path(), "icons/XGamepad/Default");
    }

    // ─── SetEntry ────────────────────────────────────────────────────────────────

    #[test]
    fn set_entry_wheel_creation() {
        let entry = SetEntry::Wheel(WheelData::new("Test", 3));
        match entry {
            SetEntry::Wheel(w) => assert_eq!(w.name, "Test"),
            _ => panic!("expected Wheel"),
        }
    }

    #[test]
    fn set_entry_action_creation() {
        let entry = SetEntry::Action(QuickAction {
            name: "Test".into(),
            ..default()
        });
        match entry {
            SetEntry::Action(a) => assert_eq!(a.name, "Test"),
            _ => panic!("expected Action"),
        }
    }

    // ─── WheelSetData ────────────────────────────────────────────────────────────

    #[test]
    fn wheel_set_data_default() {
        let data = WheelSetData::default();
        assert_eq!(data.name, "Radial menu set");
        assert_eq!(data.wheels.len(), 1);
        assert_eq!(data.min_wheels, 1);
        assert_eq!(data.max_wheels, 8);
    }

    #[test]
    fn normalize_wheelset_applies_shared_visuals_and_minimum() {
        let mut ws = WheelSetData {
            wheels: vec![WheelData::new("First", 3)],
            min_wheels: 2,
            max_wheels: 4,
            ..default()
        };
        ws.wheels[0].offset_x = 42.0;
        ws.wheels[0].rotation = 0.75;
        ws.wheels[0].outer_radius = 123.0;
        ws.visuals = None;

        normalize_wheelset(&mut ws);

        assert_eq!(ws.wheels.len(), 2);
        assert_eq!(ws.wheels[1].offset_x, 42.0);
        assert_eq!(ws.wheels[1].rotation, 0.75);
        assert_eq!(ws.wheels[1].outer_radius, 123.0);
        assert!(ws.visuals.is_some());
    }

    #[test]
    fn wheelset_visual_sync_preserves_sector_data() {
        let mut ws = WheelSetData {
            wheels: vec![WheelData::new("First", 2), WheelData::new("Second", 4)],
            ..default()
        };
        ws.wheels[0].offset_y = -18.0;
        ws.wheels[0].rotation = 1.25;
        ws.visuals = None;
        normalize_wheelset(&mut ws);

        assert_eq!(ws.wheels[0].slots.len(), 2);
        assert_eq!(ws.wheels[1].slots.len(), 4);
        assert_eq!(ws.wheels[1].offset_y, -18.0);
        assert_eq!(ws.wheels[1].rotation, 1.25);
    }

    // ─── SlotItem ────────────────────────────────────────────────────────────────

    #[test]
    fn slot_item_default() {
        let item = SlotItem::default();
        assert!(item.name.is_empty());
        assert!(item.icon.is_empty());
    }

    // ─── ActionSet ───────────────────────────────────────────────────────────────

    #[test]
    fn action_set_default() {
        let set = ActionSet::default();
        assert_eq!(set.name, "Set");
        assert_eq!(set.opacity, 1.0);
        assert!(set.entries.is_empty());
    }

    // ─── WedgeParams ─────────────────────────────────────────────────────────────

    #[test]
    fn wedge_params_default_values() {
        let params = WedgeParams {
            color: Vec4::new(1.0, 0.0, 0.0, 0.5),
            border_color: Vec4::new(1.0, 0.5, 0.5, 1.0),
            inner_r: 40.0,
            outer_r: 140.0,
            angle_start: 0.0,
            angle_end: std::f32::consts::FRAC_PI_2,
            edge_width: 1.0,
        };
        assert!((params.inner_r - 40.0).abs() < f32::EPSILON);
        assert!((params.outer_r - 140.0).abs() < f32::EPSILON);
    }

    // ─── GlobalBindings ──────────────────────────────────────────────────────────

    #[test]
    fn global_bindings_default_empty() {
        let gb = GlobalBindings::default();
        assert!(gb.bindings.is_empty());
    }

    // ─── ActiveSlotContext ───────────────────────────────────────────────────────

    #[test]
    fn active_slot_context_creation() {
        // Just verify it can be created; actual entity requires a running app
        // We only test the type exists and derives are correct
        let _ = format!("{:?}", editor::EditFocus::None);
    }

    // ─── ActionShape labels ──────────────────────────────────────────────────────

    #[test]
    fn action_shape_labels() {
        assert_eq!(ActionShape::Rounded.label(), "Rounded");
        assert_eq!(ActionShape::Round.label(), "Round");
        assert_eq!(ActionShape::Square.label(), "Square");
        assert_eq!(ActionShape::Diamond.label(), "Diamond");
    }

    // ─── PositionMode labels ─────────────────────────────────────────────────────

    #[test]
    fn position_mode_labels() {
        assert_eq!(PositionMode::Relative.label(), "Relative");
        assert_eq!(PositionMode::Absolute.label(), "Absolute");
    }

    // ─── SegmentShape labels ─────────────────────────────────────────────────────

    #[test]
    fn segment_shape_labels() {
        assert_eq!(SegmentShape::Rounded.label(), "Rounded");
        assert_eq!(SegmentShape::Pie.label(), "Pie");
    }

    // ─── WheelTheme labels ───────────────────────────────────────────────────────

    #[test]
    fn wheel_theme_labels() {
        assert_eq!(WheelTheme::Dark.label(), "dark");
        assert_eq!(WheelTheme::Light.label(), "light");
    }

    // ─── Default constants ───────────────────────────────────────────────────────

    #[test]
    fn config_file_constant() {
        assert_eq!(CONFIG_FILE, "quickactions_config.ron");
    }

    #[test]
    fn default_action_color() {
        let color = _default_action_color();
        assert_eq!(color, "#3b82f6");
    }

    #[test]
    fn default_outer_radius() {
        assert!((_default_outer_radius() - 270.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_inner_radius() {
        assert!((_default_inner_radius() - 130.0).abs() < f32::EPSILON);
    }
}

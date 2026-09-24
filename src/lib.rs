//! Headless wheel menu library for Bevy.
//!
//! This library provides the logic and data structures for wheel menus.
//! Rendering is left to the application.
#![warn(missing_docs)]

pub mod editor;
mod hud;
mod persistence;
mod platform;
mod radial_menu;
mod radial_menu_set;
mod scheduling;
pub mod touch;
pub mod wasm;

pub use hud::*;
use hud::{
    detect_gamepad_icon_set, hud_button_feedback, hud_context_visibility, hud_control_owner,
    hud_stick_nav, rebuild_hud, tick_hud_dry_run_flash,
};
pub use radial_menu::messages::*;
pub use radial_menu::{
    check_low_counts, emit_lifecycle, emit_selection, resolve_wheel_input, slice_angles,
    slice_center, update_active_slot_context, update_edit_mode, update_wheel_hold,
    update_wheel_hover, wheel_bg_disc, wheel_center_ring, wheel_hub, wheel_outer_ring,
    wheel_slice_label,
};
pub use radial_menu::{
    resolve_input, ActiveSlotContext, CastingMode, GlobalBindings, InputAction, RadialMenuAudio,
    RadialMenuConfig, RadialMenuEditMode, RadialMenuHierarchy, RadialMenuHoldState,
    RadialMenuState, RadialMenuStyle, RadialMenuToggleMode, SectorContent, SectorCount,
    SectorEntity, WheelAction, WheelInputOverride, WheelSliceLink,
};
pub use radial_menu::{
    RadialMenu, RadialMenuGeometry, Sector, SegmentShape, WheelTheme, DEFAULT_STICK_BINDING,
};
pub use radial_menu_set::messages::*;
use radial_menu_set::update_wheel_set;
pub use radial_menu_set::{
    normalize_wheelset, wheelset_visuals, RadialMenuSet, RadialMenuSetState, RadialMenuSetVisuals,
};

use bevy::asset::embedded_asset;
use bevy::prelude::*;
#[allow(unused_imports)]
pub(crate) use hud::config::_default_action_color;
pub(crate) use hud::ui::{
    hud_action_field, hud_action_stepper, hud_child, hud_clickable, hud_label_or, hud_text,
};

/// Default filename for the persisted [`QuickActionConfig`].
/// Resolved relative to the process working directory (the project root when
/// running via `cargo run`).
pub const CONFIG_FILE: &str = "quickactions_config.ron";

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
    /// Creates the runtime HUD plugin without the editor.
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
    /// Registers core messages, systems, assets, and optional HUD/editor systems.
    fn build(&self, app: &mut App) {
        scheduling::configure(app);
        embedded_asset!(app, "embedded/shaders/wedge.wgsl");
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

        app.init_resource::<GlobalBindings>()
            .add_message::<WheelMenuSelected>()
            .add_message::<WheelMenuHoverChanged>()
            .add_message::<WheelOpened>()
            .add_message::<WheelClosed>()
            .add_message::<WheelActionResolved>()
            .add_message::<WheelSwitched>()
            .add_message::<WheelMenuHoldProgress>()
            .add_message::<WheelMenuHoldActivated>()
            .add_message::<WheelMenuLowCount>()
            .add_message::<WheelEditModeChanged>()
            .add_message::<WheelSliceReorder>()
            .add_systems(
                Update,
                (
                    update_wheel_hover,
                    emit_selection,
                    emit_lifecycle,
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

        if self.editor {
            app.add_plugins(bevy::feathers::FeathersPlugins);
            app.insert_resource(bevy::feathers::theme::UiTheme(
                bevy::feathers::dark_theme::create_dark_theme(),
            ));
            editor::register_editor_systems(app);
        }

        app.add_plugins(platform::PlatformSupportPlugin);
    }
}

/// Backward-compat wrapper — use [`QuickActionHudPlugin::core()`] instead.
///
/// Provides core wheel logic with no HUD canvas.
pub struct WheelMenuPlugin;
impl Plugin for WheelMenuPlugin {
    /// Registers the core wheel systems through [`QuickActionHudPlugin::core`].
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickActionHudPlugin::core());
    }
}

/// Alias kept for call-sites that used `ActionWheelPlugin`.
pub type ActionWheelPlugin = WheelMenuPlugin;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

        let wheel = RadialMenu::new("Geometry", 1);
        let wheel_serialized = ron::ser::to_string(&wheel).expect("serialize radial menu");
        let wheel_deserialized: RadialMenu =
            ron::from_str(&wheel_serialized).expect("deserialize radial menu");
        assert_eq!(wheel.outer_radius, wheel_deserialized.outer_radius);
        assert_eq!(wheel.arc_offset, wheel_deserialized.arc_offset);
    }

    #[test]
    fn existing_flat_geometry_config_deserializes() {
        let cfg: QuickActionConfig = ron::from_str(include_str!("../quickactions_config.ron"))
            .expect("deserialize existing flat-key configuration");
        let wheel = cfg
            .sets
            .iter()
            .flat_map(|set| set.entries.iter())
            .find_map(|entry| match entry {
                SetEntry::Wheel(wheel) => Some(wheel),
                SetEntry::RadialMenuSet(set) => set.wheels.first(),
                _ => None,
            })
            .expect("existing configuration contains a radial menu");
        assert!(wheel.outer_radius > wheel.inner_radius);
    }

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

    #[test]
    fn wheel_state_default_dir_is_zero() {
        let state = RadialMenuState::default();
        assert_eq!(state.dir, Vec2::ZERO);
        assert!(state.hovered.is_none());
        assert!(!state.open);
    }

    #[test]
    fn wheel_state_hovered_updates() {
        let mut state = RadialMenuState {
            dir: Vec2::new(1.0, 0.0),
            hovered: Some(0),
            ..Default::default()
        };
        assert_eq!(state.hovered, Some(0));
        state.hovered = None;
        assert!(state.hovered.is_none());
    }

    #[test]
    fn wheel_menu_config_default() {
        let cfg = RadialMenuConfig::default();
        assert_eq!(cfg.casting_mode, CastingMode::Vanilla);
        assert_eq!(cfg.toggle_mode, RadialMenuToggleMode::Hold);
        assert!(cfg.auto_snap);
    }

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

    #[test]
    fn wheel_set_default() {
        let set = RadialMenuSetState::default();
        assert_eq!(set.active, 0);
        assert_eq!(set.count, 1);
        assert_eq!(set.prev_button, GamepadButton::LeftTrigger);
        assert_eq!(set.next_button, GamepadButton::RightTrigger);
    }

    #[test]
    fn wheel_style_default_colors() {
        let style = RadialMenuStyle::default();
        assert_eq!(style.skin, "default");
        assert_eq!(style.base_color, [0.08, 0.12, 0.18, 0.85]);
    }

    #[test]
    fn wheel_style_color_conversion() {
        let style = RadialMenuStyle::default();
        let base = style.base();
        assert!((base.to_srgba().alpha - 0.85).abs() < 0.01);
    }

    #[test]
    fn wheel_hold_state_default() {
        let state = RadialMenuHoldState::default();
        assert!((state.progress - 0.0).abs() < f32::EPSILON);
        assert!(!state.holding);
    }

    #[test]
    fn wheel_slice_count_default() {
        let count = SectorCount::default();
        assert_eq!(count.current, 0);
        assert_eq!(count.max, 0);
        assert_eq!(count.low_threshold, 0);
        assert!(!count.low_notified);
    }

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

    #[test]
    fn quick_action_default() {
        let qa = QuickAction::default();
        assert_eq!(qa.name, "Action");
        assert!(qa.enabled);
        assert!(qa.show_on_menu);
        assert_eq!(qa.shape, ActionShape::Rounded);
    }

    #[test]
    fn wheel_data_default_has_one_slot() {
        let wd = RadialMenu::default();
        assert_eq!(wd.slots.len(), 1);
    }

    #[test]
    fn wheel_data_new_creates_n_slots() {
        let wd = RadialMenu::new("Test", 6);
        assert_eq!(wd.slots.len(), 6);
        assert_eq!(wd.name, "Test");
    }

    #[test]
    fn wheel_data_new_min_one_slot() {
        let wd = RadialMenu::new("Min", 0);
        assert_eq!(wd.slots.len(), 1);
    }

    #[test]
    fn wheel_slot_data_default_is_empty() {
        let sd = Sector::default();
        assert!(sd.name.is_empty());
    }

    #[test]
    fn wheel_slot_data_named() {
        let sd = Sector::named("Test Slot");
        assert_eq!(sd.name, "Test Slot");
    }

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
    fn hud_open_mode_labels() {
        assert_eq!(HudOpenMode::Hold.label(), "Hold");
        assert_eq!(HudOpenMode::Toggle.label(), "Toggle");
    }

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

    #[test]
    fn wheel_hud_state_default() {
        let state = WheelHudState::default();
        assert!(state.dirty);
        assert!(!state.open);
        assert!(!state.editor_open);
        assert_eq!(state.active_set, 0);
    }

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
                SetEntry::Wheel(RadialMenu::default()),
                SetEntry::Action(QuickAction::default()),
                SetEntry::RadialMenuSet(RadialMenuSet::default()),
                SetEntry::Action(QuickAction::default()),
            ],
            ..default()
        };
        assert_eq!(count_wheel_entries(&set), 2);
    }

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

    #[test]
    fn selection_default_is_none() {
        assert_eq!(editor::Selection::default(), editor::Selection::None);
    }

    #[test]
    fn edit_focus_default_is_none() {
        assert_eq!(editor::EditFocus::default(), editor::EditFocus::None);
    }

    #[test]
    fn gamepad_btn_label_mapping() {
        assert_eq!(GamepadIconSet::Xbox.base_path(), "icons/XGamepad/Default");
    }

    #[test]
    fn set_entry_wheel_creation() {
        let entry = SetEntry::Wheel(RadialMenu::new("Test", 3));
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

    #[test]
    fn wheel_set_data_default() {
        let data = RadialMenuSet::default();
        assert_eq!(data.name, "Radial menu set");
        assert_eq!(data.wheels.len(), 1);
        assert_eq!(data.min_wheels, 1);
        assert_eq!(data.max_wheels, 8);
    }

    #[test]
    fn normalize_wheelset_applies_shared_visuals_and_minimum() {
        let mut ws = RadialMenuSet {
            wheels: vec![RadialMenu::new("First", 3)],
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
        let mut ws = RadialMenuSet {
            wheels: vec![RadialMenu::new("First", 2), RadialMenu::new("Second", 4)],
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

    #[test]
    fn action_set_default() {
        let set = ActionSet::default();
        assert_eq!(set.name, "Set");
        assert_eq!(set.opacity, 1.0);
        assert!(set.entries.is_empty());
    }

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

    #[test]
    fn global_bindings_default_empty() {
        let gb = GlobalBindings::default();
        assert!(gb.bindings.is_empty());
    }

    #[test]
    fn active_slot_context_creation() {
        let _ = format!("{:?}", editor::EditFocus::None);
    }

    #[test]
    fn action_shape_labels() {
        assert_eq!(ActionShape::Rounded.label(), "Rounded");
        assert_eq!(ActionShape::Round.label(), "Round");
        assert_eq!(ActionShape::Square.label(), "Square");
        assert_eq!(ActionShape::Diamond.label(), "Diamond");
    }

    #[test]
    fn position_mode_labels() {
        assert_eq!(PositionMode::Relative.label(), "Relative");
        assert_eq!(PositionMode::Absolute.label(), "Absolute");
    }

    #[test]
    fn segment_shape_labels() {
        assert_eq!(SegmentShape::Rounded.label(), "Rounded");
        assert_eq!(SegmentShape::Pie.label(), "Pie");
    }

    #[test]
    fn wheel_theme_labels() {
        assert_eq!(WheelTheme::Dark.label(), "dark");
        assert_eq!(WheelTheme::Light.label(), "light");
    }

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
        assert!((RadialMenuGeometry::default().outer_radius - 270.0).abs() < f32::EPSILON);
    }

    #[test]
    fn radial_menu_defaults_to_right_thumbstick_binding() {
        assert_eq!(RadialMenu::default().stick_binding, DEFAULT_STICK_BINDING);
        assert_eq!(
            RadialMenuSet::default().stick_binding,
            DEFAULT_STICK_BINDING
        );
    }

    #[test]
    fn default_inner_radius() {
        assert!((RadialMenuGeometry::default().inner_radius - 130.0).abs() < f32::EPSILON);
    }
}

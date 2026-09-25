//! Visual regression probe for the native BSN radial-menu sector.
//!
//! The probe renders the default centered menu with its first sector selected,
//! then captures `/private/tmp/radial-menu-default.png` for manual comparison
//! with the project reference image.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy_quick_action_hud::{QuickActionConfig, QuickActionHudPlugin, WheelHudState};

#[derive(Resource)]
struct CaptureState {
    wait_frames: u32,
    requested: bool,
    finished: bool,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self {
            wait_frames: 15,
            requested: false,
            finished: false,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Radial menu visual probe".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(QuickActionHudPlugin::default())
        .init_resource::<CaptureState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (install_probe_config, capture_default).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn install_probe_config(
    mut installed: Local<bool>,
    mut config: ResMut<QuickActionConfig>,
    mut hud: ResMut<WheelHudState>,
) {
    if *installed {
        return;
    }

    let default_config = QuickActionConfig::default();
    config.sets = vec![default_config
        .sets
        .into_iter()
        .next()
        .expect("the default HUD configuration contains a page")];
    hud.active_set = 0;
    hud.highlighted = Some((0, 0, Some(0), 0));
    hud.open = true;
    // Editor preview prevents release-to-use input from consuming the
    // synthetic selected state before the screenshot is captured.
    hud.editor_open = true;
    *installed = true;
}

fn capture_default(
    mut commands: Commands,
    mut state: ResMut<CaptureState>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.finished {
        exit.write(AppExit::Success);
        return;
    }
    if state.wait_frames > 0 {
        state.wait_frames -= 1;
        return;
    }
    if state.requested {
        return;
    }

    state.requested = true;
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut state: ResMut<CaptureState>| {
            save_to_disk("/private/tmp/radial-menu-default.png")(captured);
            state.finished = true;
        },
    );
}

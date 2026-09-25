//! Minimal HUD host used while developing the editor.

use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::scene::prelude::{bsn, Scene};
use bevy_quick_action_hud::{
    HudSegmentSelected, QuickActionConfig, QuickActionHudPlugin, SetEntry, WheelHudButton,
    WheelHudState,
};
use std::collections::VecDeque;

#[derive(Resource, Default)]
struct InputLog {
    lines: VecDeque<String>,
}

#[derive(Component)]
struct InputLogText;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Quick Action HUD".into(),
                resolution: (1280, 720).into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(QuickActionHudPlugin::with_editor())
        .init_resource::<InputLog>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                open_hud_shortcut,
                log_hud_button_presses,
                log_hud_selection,
                update_log_window,
            ),
        )
        .run();
}

fn open_hud_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut hud: ResMut<WheelHudState>,
    mut log: ResMut<InputLog>,
) {
    let q_pressed = keys.just_pressed(KeyCode::KeyQ);
    let l2_pressed = gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::LeftTrigger2));

    if (q_pressed || l2_pressed) && !hud.editor_open {
        hud.open = !hud.open;
        hud.dirty = true;
        let source = if q_pressed { "Q" } else { "L2" };
        push_log(
            &mut log,
            format!(
                "{source} pressed → HUD {}",
                if hud.open { "opened" } else { "closed" }
            ),
        );
    }

    for gamepad in &gamepads {
        for (button, label) in [
            (GamepadButton::South, "A / South"),
            (GamepadButton::East, "B / East"),
            (GamepadButton::North, "Y / North"),
            (GamepadButton::West, "X / West"),
            (GamepadButton::LeftTrigger, "LB"),
            (GamepadButton::RightTrigger, "RB"),
            (GamepadButton::LeftThumb, "Left stick click"),
            (GamepadButton::RightThumb, "Right stick click"),
            (GamepadButton::DPadUp, "D-pad up"),
            (GamepadButton::DPadDown, "D-pad down"),
            (GamepadButton::DPadLeft, "D-pad left"),
            (GamepadButton::DPadRight, "D-pad right"),
        ] {
            if gamepad.just_pressed(button) && button != GamepadButton::LeftTrigger2 {
                push_log(&mut log, format!("{label} pressed"));
            }
        }
    }
}

fn log_hud_selection(
    mut events: MessageReader<HudSegmentSelected>,
    cfg: Res<QuickActionConfig>,
    mut log: ResMut<InputLog>,
) {
    for event in events.read() {
        let Some(entry) = cfg
            .sets
            .get(event.set)
            .and_then(|set| set.entries.get(event.entry))
        else {
            push_log(
                &mut log,
                format!(
                    "Segment selected: invalid entry {}/{}",
                    event.set, event.entry
                ),
            );
            continue;
        };
        let (component, slot) = match (entry, event.wheel) {
            (SetEntry::RadialMenuSet(wheel_set), Some(index)) => (
                wheel_set.name.as_str(),
                wheel_set
                    .radial_menu(index)
                    .and_then(|wheel| wheel.slots.get(event.slot)),
            ),
            _ => ("unknown", None),
        };
        if let Some(slot) = slot {
            push_log(
                &mut log,
                format!(
                    "{} → {} | action={} | hold={} | close={}",
                    component,
                    if slot.name.is_empty() {
                        "unnamed segment"
                    } else {
                        &slot.name
                    },
                    slot.command,
                    if slot.hold {
                        slot.hold_command.as_str()
                    } else {
                        "disabled"
                    },
                    slot.close_on_select,
                ),
            );
        }
    }
}

fn log_hud_button_presses(
    buttons: Query<(&WheelHudButton, &PickingInteraction), Changed<PickingInteraction>>,
    mut log: ResMut<InputLog>,
) {
    for (button, interaction) in &buttons {
        if *interaction == PickingInteraction::Pressed {
            push_log(
                &mut log,
                format!("HUD button pressed → {:?}", button.action),
            );
        }
    }
}

fn update_log_window(log: Res<InputLog>, mut texts: Query<&mut Text, With<InputLogText>>) {
    if !log.is_changed() {
        return;
    }
    let content = if log.lines.is_empty() {
        "Press Q or L2 to open the HUD".to_string()
    } else {
        log.lines.iter().cloned().collect::<Vec<_>>().join("\n")
    };
    for mut text in &mut texts {
        text.0 = content.clone();
    }
}

fn push_log(log: &mut InputLog, message: String) {
    log.lines.push_back(message);
    while log.lines.len() > 8 {
        log.lines.pop_front();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text2d::new("Bevy placeholder"),
        TextFont {
            font_size: FontSize::Px(28.0),
            ..default()
        },
        TextColor(Color::srgb(0.35, 0.35, 0.35)),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

    // Static hierarchy is declarative; only the log text is updated at runtime.
    let panel = commands.spawn_scene(input_log_panel()).id();
    let text = commands
        .spawn_scene(bsn! {
            Text("Press Q or L2 to open the HUD")
            TextFont { font_size: {FontSize::Px(13.0)} }
            TextColor(Color::srgb(0.75, 0.75, 0.75))
        })
        .insert(InputLogText)
        .id();
    commands.entity(panel).add_child(text);
}

fn input_log_panel() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {Val::Px(16.0)},
            bottom: {Val::Px(16.0)},
            width: {Val::Px(500.0)},
            min_height: {Val::Px(150.0)},
            padding: {UiRect::all(Val::Px(10.0))},
        }
        BackgroundColor(Color::srgba(0.03, 0.03, 0.03, 0.92))
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2))
    }
}

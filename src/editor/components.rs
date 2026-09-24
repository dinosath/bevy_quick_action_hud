//! ECS components owned by the editor feature.

use super::EditorAction;
use crate::{
    hud_action_field, hud_action_stepper, hud_child, hud_clickable, hud_control_owner,
    hud_label_or, hud_text, HudContextControl, QuickAction, RadialMenu, Sector, WheelHudAction,
    WheelTheme, HUD_AMBER, HUD_BADGE_BORDER, HUD_DIM, HUD_GREEN, HUD_PANEL_CARD, HUD_TEXT,
};
use bevy::feathers::controls::{ButtonVariant, FeathersButton};
use bevy::prelude::*;

/// A compact editor control used by the in-canvas settings cards.
///
/// These controls deliberately use the same `EditorButton`/`Activate`
/// observer path as the former navigation surface.  That keeps input capture,
/// undo snapshots, and persistence in one place instead of creating a second
/// radial-menu configuration protocol for the HUD canvas.
fn settings_action_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    action: EditorAction,
    accent: Color,
) -> Entity {
    let button = commands
        .spawn_scene(bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node {
                min_height: {Val::Px(28.)},
                padding: {UiRect::horizontal(Val::Px(8.))},
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
        })
        .insert(EditorButton {
            action,
            base: HUD_PANEL_CARD,
        })
        .id();
    commands.entity(parent).add_child(button);
    let left = hud_child(commands, button, hud_text(label, 10., HUD_DIM));
    let right = hud_child(commands, button, hud_text(value, 10., accent));
    commands.entity(left).insert(Node {
        flex_grow: 1.,
        ..default()
    });
    commands.entity(right).insert(Node {
        flex_shrink: 0.,
        ..default()
    });
    button
}

/// Temporary hover color used while rendering editable segments.
#[derive(Component)]
pub struct SegmentHoverColor(pub Color);

/// Action dispatched by an editor button.
#[derive(Component, Clone)]
pub struct EditorButton {
    pub action: EditorAction,
    pub base: Color,
}

/// Marks the retained settings panel so sector hit testing cannot treat a
/// click inside the panel as a click on the radial menu underneath it.
#[derive(Component)]
pub struct WheelSettingsPanel;

/// Marks the item currently focused by keyboard/gamepad navigation.
#[derive(Component)]
pub struct FocusedEditorItem;

pub(crate) fn build_hud_action_editor_card(
    commands: &mut Commands,
    parent: Entity,
    set: usize,
    entry: usize,
    action: &QuickAction,
) {
    let card = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                // Keep the detail window in the upper-right utility area;
                // action buttons live along the lower-right edge.
                right: {Val::Px(28.)}, top: {Val::Px(72.)},
                width: {Val::Px(266.)},
                padding: {UiRect::all(Val::Px(10.))},
                flex_direction: FlexDirection::Column,
                row_gap: {Val::Px(6.)},
                border: {UiRect::all(Val::Px(1.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
        },
    );
    hud_child(commands, card, hud_text("Button", 11., HUD_TEXT));
    hud_action_field(
        commands,
        card,
        "Name",
        &action.name,
        34.,
        WheelHudAction::EditActionName { set, entry },
        HUD_TEXT,
    );
    hud_action_field(
        commands,
        card,
        "Input binding",
        &hud_label_or(&action.key),
        44.,
        WheelHudAction::CaptureActionKey { set, entry },
        HUD_TEXT,
    );
    hud_action_field(
        commands,
        card,
        "Icon  ·  select",
        &action.icon,
        32.,
        WheelHudAction::CycleActionIcon { set, entry },
        HUD_TEXT,
    );
    hud_action_field(
        commands,
        card,
        "Action mapping",
        &action.command,
        32.,
        WheelHudAction::CycleActionMapping { set, entry },
        HUD_TEXT,
    );
    hud_action_field(
        commands,
        card,
        if action.hold {
            "Hold  ·  enabled"
        } else {
            "Hold  ·  disabled"
        },
        if action.hold { "On" } else { "Off" },
        30.,
        WheelHudAction::ToggleActionHold { set, entry },
        if action.hold { HUD_GREEN } else { HUD_DIM },
    );
    hud_action_field(
        commands,
        card,
        "Hold action",
        &action.hold_command,
        32.,
        WheelHudAction::CycleHoldAction { set, entry },
        HUD_TEXT,
    );
    hud_action_field(
        commands,
        card,
        if action.close_on_select {
            "Close HUD on apply  ·  enabled"
        } else {
            "Close HUD on apply  ·  disabled"
        },
        if action.close_on_select { "On" } else { "Off" },
        30.,
        WheelHudAction::ToggleActionCloseOnApply { set, entry },
        if action.close_on_select {
            HUD_GREEN
        } else {
            HUD_DIM
        },
    );

    // Keep destructive and layout controls together at the top of the card.
    // The controls use the same compact, bordered language as the wheel editor.
    let delete = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(28.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({Color::srgba(0.55, 0.12, 0.14, 0.72)})
            BorderColor::all(HUD_AMBER)
            Button
        },
        WheelHudAction::DeleteAction { set, entry },
        Color::srgba(0.55, 0.12, 0.14, 0.72),
    );
    hud_child(commands, delete, hud_text("Delete button", 10., HUD_TEXT));

    hud_child(commands, card, hud_text("Resize", 10., HUD_DIM));
    hud_action_stepper(
        commands,
        card,
        "Width",
        &format!("{:.0}", action.width),
        WheelHudAction::ActionWidthDelta {
            set,
            entry,
            delta: -4.0,
        },
        WheelHudAction::ActionWidthDelta {
            set,
            entry,
            delta: 4.0,
        },
    );
    hud_action_stepper(
        commands,
        card,
        "Height",
        &format!("{:.0}", action.height),
        WheelHudAction::ActionHeightDelta {
            set,
            entry,
            delta: -2.0,
        },
        WheelHudAction::ActionHeightDelta {
            set,
            entry,
            delta: 2.0,
        },
    );

    hud_child(commands, card, hud_text("Move", 10., HUD_DIM));
    hud_action_stepper(
        commands,
        card,
        "Distance",
        &format!("{:.0}", action.radius),
        WheelHudAction::ActionRadiusDelta {
            set,
            entry,
            delta: -4.0,
        },
        WheelHudAction::ActionRadiusDelta {
            set,
            entry,
            delta: 4.0,
        },
    );
    let position = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(26.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        WheelHudAction::CycleActionPosition { set, entry },
        HUD_PANEL_CARD,
    );
    hud_child(
        commands,
        position,
        hud_text(
            &format!("Placement: {}", action.position.label()),
            10.,
            HUD_TEXT,
        ),
    );

    let close = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(28.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        WheelHudAction::CloseSelection,
        HUD_PANEL_CARD,
    );
    hud_child(commands, close, hud_text("Close", 10., HUD_TEXT));
}

pub(crate) fn spawn_wheel_settings_card(
    commands: &mut Commands,
    parent: Entity,
    wheel: &RadialMenu,
    theme_popup_open: bool,
) {
    let card = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {Val::Px(wheel.outer_radius + 24.)},
                top: {Val::Px(-154.)},
                width: {Val::Px(320.)},
                padding: {UiRect::all(Val::Px(10.))},
                flex_direction: FlexDirection::Column,
                row_gap: {Val::Px(7.)},
                border: {UiRect::all(Val::Px(1.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
    );
    commands.entity(card).insert(WheelSettingsPanel);
    hud_child(
        commands,
        card,
        hud_text("Radial menu settings", 11., HUD_TEXT),
    );
    hud_child(commands, card, hud_text(&wheel.name, 16., HUD_TEXT));
    hud_child(
        commands,
        card,
        hud_text("The selected radial menu owns these settings.", 9., HUD_DIM),
    );

    // Radial-menu configuration previously lived in the sidebar.  Keep the
    // controls next to the preview now, using the canonical EditorAction
    // pathway so all edits still participate in validation and undo/redo.
    settings_action_button(
        commands,
        card,
        "Name",
        &wheel.name,
        EditorAction::EditWheelName,
        HUD_TEXT,
    );
    settings_action_button(
        commands,
        card,
        "Theme",
        wheel.theme.label(),
        EditorAction::ToggleWheelThemePopup,
        HUD_TEXT,
    );
    if theme_popup_open {
        let popup = hud_child(
            commands,
            card,
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: {Val::Px(10.)},
                    top: {Val::Px(112.)},
                    width: {Val::Px(298.)},
                    padding: {UiRect::all(Val::Px(6.))},
                    flex_direction: FlexDirection::Column,
                    row_gap: {Val::Px(5.)},
                    border: {UiRect::all(Val::Px(1.))},
                }
                BackgroundColor({HUD_PANEL_CARD})
                BorderColor::all(HUD_BADGE_BORDER)
                Button
            },
        );
        commands.entity(popup).insert(WheelSettingsPanel);
        settings_action_button(
            commands,
            popup,
            "Dark",
            if wheel.theme == WheelTheme::Dark {
                "Selected"
            } else {
                ""
            },
            EditorAction::SetWheelTheme {
                theme: WheelTheme::Dark,
            },
            HUD_TEXT,
        );
        settings_action_button(
            commands,
            popup,
            "Light",
            if wheel.theme == WheelTheme::Light {
                "Selected"
            } else {
                ""
            },
            EditorAction::SetWheelTheme {
                theme: WheelTheme::Light,
            },
            HUD_TEXT,
        );
    }
    settings_action_button(
        commands,
        card,
        "Thumbstick",
        wheel.stick.label(),
        EditorAction::CaptureWheelStick,
        HUD_TEXT,
    );
    settings_action_button(
        commands,
        card,
        "Cooldown",
        &format!("{:.1}s", wheel.cooldown_secs),
        EditorAction::WheelCooldownDelta { delta: 0.5 },
        HUD_TEXT,
    );
    settings_action_button(
        commands,
        card,
        "Labels",
        if wheel.show_labels { "On" } else { "Off" },
        EditorAction::ToggleWheelShowLabels,
        if wheel.show_labels {
            HUD_GREEN
        } else {
            HUD_DIM
        },
    );
    settings_action_button(
        commands,
        card,
        "Icons",
        if wheel.show_icon { "On" } else { "Off" },
        EditorAction::ToggleWheelShowIcon,
        if wheel.show_icon { HUD_GREEN } else { HUD_DIM },
    );
    settings_action_button(
        commands,
        card,
        "Inner radius",
        &format!("{:.0}", wheel.inner_radius),
        EditorAction::WheelInnerRadiusDelta { delta: 2. },
        HUD_TEXT,
    );
    settings_action_button(
        commands,
        card,
        "Opacity",
        &format!("{:.0}%", wheel.opacity * 100.),
        EditorAction::WheelOpacityDelta { delta: 0.05 },
        HUD_TEXT,
    );
    let close = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(28.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        WheelHudAction::CloseSelection,
        HUD_PANEL_CARD,
    );
    hud_child(commands, close, hud_text("Close", 10., HUD_TEXT));
}

pub(crate) fn spawn_segment_editor_card(
    commands: &mut Commands,
    parent: Entity,
    slot: &Sector,
    set: usize,
    entry: usize,
    wheel: Option<usize>,
    slot_index: usize,
    outer_radius: f32,
    edit_control_focus: Option<usize>,
) {
    let card = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {Val::Px(outer_radius + 24.)},
                top: {Val::Px(-154.)},
                width: {Val::Px(264.)},
                padding: {UiRect::all(Val::Px(10.))},
                flex_direction: FlexDirection::Column,
                row_gap: {Val::Px(7.)},
                border: {UiRect::all(Val::Px(1.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
        },
    );
    hud_child(commands, card, hud_text("Sector settings", 11., HUD_TEXT));
    hud_child(commands, card, hud_text(&slot.name, 16., HUD_TEXT));
    hud_child(
        commands,
        card,
        hud_text("Edit the selected sector", 9., HUD_DIM),
    );
    let name = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(30.)},
                padding: {UiRect::horizontal(Val::Px(9.))},
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
            }
            BackgroundColor({if edit_control_focus == Some(5) { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if edit_control_focus == Some(5) { HUD_TEXT } else { HUD_BADGE_BORDER })
            Button
        },
        WheelHudAction::EditSegmentName {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        HUD_PANEL_CARD,
    );
    hud_child(commands, name, hud_text("Set name  ›", 10., HUD_TEXT));
    let icon = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(30.)},
                padding: {UiRect::horizontal(Val::Px(9.))},
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
            }
            BackgroundColor({if edit_control_focus == Some(6) { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if edit_control_focus == Some(6) { HUD_TEXT } else { HUD_BADGE_BORDER })
            Button
        },
        WheelHudAction::EditSegmentIcon {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        HUD_PANEL_CARD,
    );
    hud_child(commands, icon, hud_text("Set icon  ›", 10., HUD_TEXT));
    let input = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(30.)},
                padding: {UiRect::horizontal(Val::Px(9.))},
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
            }
            BackgroundColor({if edit_control_focus == Some(7) { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if edit_control_focus == Some(7) { HUD_TEXT } else { HUD_BADGE_BORDER })
            Button
        },
        WheelHudAction::EditSegmentInput {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        HUD_PANEL_CARD,
    );
    hud_child(
        commands,
        input,
        hud_text(
            if slot.input.is_empty() {
                "Set input  ›"
            } else {
                "Change input  ›"
            },
            10.,
            HUD_TEXT,
        ),
    );
    hud_action_field(
        commands,
        card,
        "Action mapping",
        &slot.command,
        32.,
        WheelHudAction::CycleSegmentMapping {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        HUD_TEXT,
    );
    hud_action_field(
        commands,
        card,
        if slot.hold {
            "Hold action  ·  enabled"
        } else {
            "Hold action  ·  disabled"
        },
        if slot.hold { "On" } else { "Off" },
        30.,
        WheelHudAction::ToggleSegmentHold {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        if slot.hold { HUD_GREEN } else { HUD_DIM },
    );
    hud_action_field(
        commands,
        card,
        "Hold mapping",
        &slot.hold_command,
        32.,
        WheelHudAction::CycleSegmentHoldAction {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        HUD_TEXT,
    );
    hud_action_field(
        commands,
        card,
        if slot.close_on_select {
            "Close HUD on apply  ·  enabled"
        } else {
            "Close HUD on apply  ·  disabled"
        },
        if slot.close_on_select { "On" } else { "Off" },
        30.,
        WheelHudAction::ToggleSegmentCloseOnApply {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        if slot.close_on_select {
            HUD_GREEN
        } else {
            HUD_DIM
        },
    );
    let delete = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(28.)}, justify_content: JustifyContent::Center,
                align_items: AlignItems::Center, border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({Color::srgba(0.55, 0.12, 0.14, 0.72)})
            BorderColor::all(HUD_AMBER)
            Button
        },
        WheelHudAction::DeleteSegment {
            set,
            entry,
            wheel,
            slot: slot_index,
        },
        Color::srgba(0.55, 0.12, 0.14, 0.72),
    );
    hud_child(commands, delete, hud_text("Delete segment", 10., HUD_TEXT));
    let close = hud_clickable(
        commands,
        card,
        bsn! {
            Node {
                height: {Val::Px(28.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        WheelHudAction::CloseSelection,
        HUD_PANEL_CARD,
    );
    hud_child(commands, close, hud_text("Close", 10., HUD_TEXT));
}

pub(crate) fn spawn_radial_edit_button(
    commands: &mut Commands,
    parent: Entity,
    position: Vec2,
    action: WheelHudAction,
    label: &str,
    color: Color,
    focused: bool,
) {
    let owner = hud_control_owner(&action);
    let button = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {Val::Px(position.x - 11.)},
                top: {Val::Px(-position.y - 11.)},
                width: {Val::Px(22.)}, height: {Val::Px(22.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(11.))},
            }
            BackgroundColor({if focused { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if focused { HUD_TEXT } else { HUD_BADGE_BORDER })
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
    hud_child(commands, button, hud_text(label, 15., color));
}

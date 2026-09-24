//! ECS components owned by the editor feature.

use super::EditorAction;
use crate::{
    hud_action_field, hud_action_stepper, hud_child, hud_clickable, hud_control_owner,
    hud_label_or, hud_text, HudContextControl, QuickAction, WheelData, WheelHudAction,
    WheelSlotData, HUD_AMBER, HUD_BADGE_BORDER, HUD_DIM, HUD_GREEN, HUD_PANEL_CARD, HUD_TEXT,
};
use bevy::prelude::*;

/// Root entity for the editor sidebar.
#[derive(Component)]
pub struct EditorRoot;

/// Scrollable content entity inside the wheel editor panel.
#[derive(Component)]
pub struct EditorScrollArea;

/// Temporary hover color used while rendering editable segments.
#[derive(Component)]
pub struct SegmentHoverColor(pub Color);

/// Action dispatched by an editor button.
#[derive(Component, Clone)]
pub struct EditorButton {
    pub action: EditorAction,
    pub base: Color,
}

/// Action dispatched by a Feathers checkbox/toggle.
#[derive(Component, Clone)]
pub struct EditorToggle {
    pub action: EditorAction,
}

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
    wheel: &WheelData,
) {
    let card = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {Val::Px(wheel.outer_radius + 24.)},
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
    hud_child(
        commands,
        card,
        hud_text("Radial menu settings", 11., HUD_TEXT),
    );
    hud_child(commands, card, hud_text(&wheel.name, 16., HUD_TEXT));
    hud_child(
        commands,
        card,
        hud_text(&format!("{} sectors", wheel.slots.len()), 9., HUD_DIM),
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
    slot: &WheelSlotData,
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

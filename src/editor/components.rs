//! ECS components owned by the editor feature.

use super::EditorAction;
use crate::{
    hud_action_field, hud_action_stepper, hud_child, hud_clickable, hud_control_owner,
    hud_label_or, hud_text, HudContextControl, QuickAction, RadialMenu, Sector, WheelHudAction,
    WheelTheme, HUD_AMBER, HUD_BADGE_BORDER, HUD_DIM, HUD_GREEN, HUD_PANEL_CARD, HUD_TEXT,
};
use bevy::feathers::controls::{ButtonVariant, FeathersButton};
use bevy::feathers::controls::{
    FeathersSlider, FeathersTextInput, FeathersTextInputContainer, FeathersToggleSwitch,
};
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Checked;
use bevy::ui_widgets::{checkbox_self_update, slider_self_update, SliderPrecision, SliderStep};

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
    /// Editor action applied when the button is activated.
    pub action: EditorAction,
    /// Base background color restored when the button is not hovered.
    pub base: Color,
}

/// Marks the retained settings panel so sector hit testing cannot treat a
/// click inside the panel as a click on the radial menu underneath it.
#[derive(Component)]
pub struct WheelSettingsPanel;

/// Marks the item currently focused by keyboard/gamepad navigation.
#[derive(Component)]
pub struct FocusedEditorItem;

/// Shared property-grid primitives used by every inspector window.
#[derive(Component, Clone)]
/// Associates a slider with the editor action it updates.
pub struct EditorSlider(pub EditorAction);

#[derive(Component, Clone)]
/// Associates a text field with the editor action it updates.
pub struct EditorTextValue(pub EditorAction);

pub(crate) fn editor_window(
    commands: &mut Commands,
    parent: Entity,
    title: &str,
    width: f32,
) -> Entity {
    let window = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                right: { Val::Px(28.) }, top: { Val::Px(72.) },
                width: { Val::Px(width) }, max_height: { Val::Px(620.) },
                padding: { UiRect::all(Val::Px(10.)) },
                flex_direction: FlexDirection::Column, row_gap: { Val::Px(7.) },
                overflow: { Overflow::scroll_y() },
                border: { UiRect::all(Val::Px(1.)) },
            }
            BackgroundColor({ HUD_PANEL_CARD })
            BorderColor::all(HUD_BADGE_BORDER)
        },
    );
    let title_bar = hud_child(
        commands,
        window,
        bsn! {
            Node { min_height: { Val::Px(30.) }, flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center, column_gap: { Val::Px(6.) } }
        },
    );
    let title_node = hud_child(commands, title_bar, hud_text(title, 12., HUD_TEXT));
    commands.entity(title_node).insert(Node {
        flex_grow: 1.,
        ..default()
    });
    let close = hud_clickable(
        commands,
        title_bar,
        bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node { width: { Val::Px(28.) }, height: { Val::Px(28.) },
                justify_content: JustifyContent::Center, align_items: AlignItems::Center }
        },
        WheelHudAction::CloseSelection,
        HUD_PANEL_CARD,
    );
    hud_child(commands, close, hud_text("X", 12., HUD_TEXT));
    window
}

pub(crate) fn property_row(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    control: Entity,
) -> Entity {
    let row = hud_child(
        commands,
        parent,
        bsn! {
            Node { min_height: { Val::Px(34.) }, flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center, column_gap: { Val::Px(8.) } }
        },
    );
    let label_entity = hud_child(commands, row, hud_text(label, 10., HUD_DIM));
    commands.entity(label_entity).insert(Node {
        width: Val::Px(92.),
        ..default()
    });
    // Feathers owns the control's node tree. Put it in a grid cell rather than
    // patching its `Node`, which would replace the widget's authored layout.
    let control_cell = hud_child(
        commands,
        row,
        bsn! {
            Node {
                flex_grow: 1.,
                min_width: { Val::Px(0.) },
                align_items: AlignItems::Center,
            }
        },
    );
    commands.entity(control_cell).add_child(control);
    row
}

pub(crate) fn text_field(commands: &mut Commands, value: &str, action: EditorAction) -> Entity {
    let container = commands
        .spawn_scene(bsn! { @FeathersTextInputContainer })
        .id();
    let input = commands
        .spawn_scene(bsn! {
            @FeathersTextInput { @visible_width: 18f32, @max_characters: 80usize }
        })
        .insert((EditableText::new(value), EditorTextValue(action)))
        .id();
    commands.entity(container).add_child(input);
    container
}

pub(crate) fn slider_field(
    commands: &mut Commands,
    value: f32,
    min: f32,
    max: f32,
    action: EditorAction,
) -> Entity {
    let entity = commands
        .spawn_scene(bsn! {
            @FeathersSlider { @value: value, @min: min, @max: max }
            SliderStep({ (max - min) / 100. })
            SliderPrecision(2)
            on(slider_self_update)
        })
        .insert(EditorSlider(action))
        .id();
    entity
}

pub(crate) fn toggle_field(commands: &mut Commands, checked: bool, action: EditorAction) -> Entity {
    let entity = commands
        .spawn_scene(bsn! {
            @FeathersToggleSwitch
            on(checkbox_self_update)
        })
        .insert(EditorButton {
            action,
            base: HUD_PANEL_CARD,
        })
        .id();
    if checked {
        commands.entity(entity).insert(Checked);
    }
    entity
}

pub(crate) fn input_capture_field(
    commands: &mut Commands,
    value: &str,
    action: EditorAction,
) -> Entity {
    let field = commands
        .spawn_scene(bsn! {
            @FeathersButton { @variant: ButtonVariant::Plain }
            Node { min_height: { Val::Px(30.) }, padding: { UiRect::horizontal(Val::Px(8.)) } }
        })
        .insert(EditorButton {
            action,
            base: HUD_PANEL_CARD,
        })
        .id();
    hud_child(commands, field, hud_text(value, 10., HUD_TEXT));
    field
}

pub(crate) fn build_hud_action_editor_card(
    commands: &mut Commands,
    parent: Entity,
    set: usize,
    entry: usize,
    action: &QuickAction,
) {
    let card = editor_window(commands, parent, "Button Editor", 320.);
    let name = text_field(
        commands,
        &action.name,
        EditorAction::SetActionName {
            set,
            entry,
            value: action.name.clone(),
        },
    );
    property_row(commands, card, "Name", name);
    let binding = input_capture_field(
        commands,
        &hud_label_or(&action.key),
        EditorAction::CaptureKey { set, entry },
    );
    property_row(commands, card, "Button binding", binding);
    let icon = hud_clickable(
        commands,
        card,
        bsn! { @FeathersButton { @variant: ButtonVariant::Plain } Node { min_height: { Val::Px(30.) } } },
        WheelHudAction::CycleActionIcon { set, entry },
        HUD_PANEL_CARD,
    );
    hud_child(commands, icon, hud_text(&action.icon, 10., HUD_TEXT));
    property_row(commands, card, "Icon", icon);
    let cooldown = slider_field(
        commands,
        action.cooldown_secs,
        0.,
        10.,
        EditorAction::SetActionCooldown {
            set,
            entry,
            value: action.cooldown_secs,
        },
    );
    property_row(commands, card, "Cooldown", cooldown);
    let labels = toggle_field(
        commands,
        action.show_labels,
        EditorAction::ToggleActionLabels { set, entry },
    );
    property_row(commands, card, "Labels", labels);
    let icons = toggle_field(
        commands,
        action.show_icon,
        EditorAction::ToggleActionIcons { set, entry },
    );
    property_row(commands, card, "Show icons", icons);
    let opacity = slider_field(
        commands,
        action.opacity,
        0.,
        1.,
        EditorAction::SetActionOpacity {
            set,
            entry,
            value: action.opacity,
        },
    );
    property_row(commands, card, "Opacity", opacity);
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

    // Keep destructive and layout controls together at the bottom of the window.
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
}

pub(crate) fn spawn_wheel_settings_card(
    commands: &mut Commands,
    parent: Entity,
    wheel: &RadialMenu,
    theme_popup_open: bool,
) {
    let card = editor_window(commands, parent, "Radial Menu Editor", 320.);
    commands.entity(card).insert(WheelSettingsPanel);
    let name = text_field(
        commands,
        &wheel.name,
        EditorAction::SetWheelName {
            value: wheel.name.clone(),
        },
    );
    property_row(commands, card, "Name", name);
    let theme_button = input_capture_field(
        commands,
        wheel.theme.label(),
        EditorAction::ToggleWheelThemePopup,
    );
    property_row(commands, card, "Theme", theme_button);
    if theme_popup_open {
        let theme_list = hud_child(
            commands,
            card,
            bsn! {
                Node { flex_direction: FlexDirection::Column, row_gap: { Val::Px(4.) }, padding: { UiRect::all(Val::Px(4.)) } }
                BackgroundColor({ HUD_PANEL_CARD })
            },
        );
        for (label, theme) in [("Dark", WheelTheme::Dark), ("Light", WheelTheme::Light)] {
            settings_action_button(
                commands,
                theme_list,
                label,
                if wheel.theme == theme { "Selected" } else { "" },
                EditorAction::SetWheelTheme { theme },
                HUD_TEXT,
            );
        }
    }
    let stick = input_capture_field(
        commands,
        &wheel.stick_binding,
        EditorAction::CaptureWheelStick,
    );
    property_row(commands, card, "Thumbstick", stick);
    let cooldown = slider_field(
        commands,
        wheel.cooldown_secs,
        0.,
        10.,
        EditorAction::SetWheelCooldown {
            value: wheel.cooldown_secs,
        },
    );
    property_row(commands, card, "Cooldown", cooldown);
    let labels = toggle_field(
        commands,
        wheel.show_labels,
        EditorAction::ToggleWheelShowLabels,
    );
    property_row(commands, card, "Labels", labels);
    let icons = toggle_field(commands, wheel.show_icon, EditorAction::ToggleWheelShowIcon);
    property_row(commands, card, "Icons", icons);
    let inner = slider_field(
        commands,
        wheel.inner_radius,
        8.,
        100.,
        EditorAction::SetWheelInnerRadius {
            value: wheel.inner_radius,
        },
    );
    property_row(commands, card, "Inner radius", inner);
    let opacity = slider_field(
        commands,
        wheel.opacity,
        0.,
        1.,
        EditorAction::SetWheelOpacity {
            value: wheel.opacity,
        },
    );
    property_row(commands, card, "Opacity", opacity);
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

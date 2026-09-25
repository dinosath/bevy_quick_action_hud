//! Entity components owned by the rendered HUD feature.

use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

use crate::widgets::hud_layer;
use crate::WheelHudAction;

#[derive(Component, Default, Clone)]
#[require(Node = hud_layer())]
/// Root marker for the retained HUD hierarchy.
pub struct WheelHudRoot;

#[derive(Component, Clone)]
#[require(PickingInteraction)]
/// Presentation action attached to an interactive HUD control.
pub struct WheelHudButton {
    /// Intent emitted when the control is pressed.
    pub action: WheelHudAction,
    /// Base background color used by interaction feedback.
    pub base: Color,
}

#[derive(Component, Clone, Copy)]
/// Associates an editor affordance with its owning HUD component.
pub struct HudContextControl {
    /// Component identity used by context-visibility systems.
    pub owner: HudControlOwner,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Identity of a HUD component that owns editor controls.
pub enum HudControlOwner {
    /// A floating action identified by page and entry.
    Action(usize, usize),
    /// A radial menu identified by page, entry, and optional wheel index.
    Wheel(usize, usize, Option<usize>),
    /// A page-switch control identified by page and entry.
    HudSwitch(usize, usize),
}

#[derive(Component, Default, Clone, Copy)]
/// HUD identity (page, entry, menu index) of a spawned radial menu.
pub struct HudRadialMenu(pub (usize, usize, Option<usize>));

/// Maps an editor affordance back to the HUD component it belongs to.
pub(crate) fn control_owner(action: &WheelHudAction) -> Option<HudControlOwner> {
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

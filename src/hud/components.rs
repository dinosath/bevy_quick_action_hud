//! Entity components owned by the rendered HUD feature.

use bevy::prelude::*;

use crate::WheelHudAction;

#[derive(Component)]
pub struct WheelHudRoot;

#[derive(Component, Clone)]
pub struct WheelHudButton {
    pub action: WheelHudAction,
    pub base: Color,
}

#[derive(Component, Clone, Copy)]
pub struct HudContextControl {
    pub owner: HudControlOwner,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HudControlOwner {
    Action(usize, usize),
    Wheel(usize, usize, Option<usize>),
    HudSwitch(usize, usize),
}

#[derive(Component, Clone, Copy)]
pub struct WheelHudSegmentHit {
    pub set: usize,
    pub entry: usize,
    pub wheel: Option<usize>,
    pub slot: usize,
}

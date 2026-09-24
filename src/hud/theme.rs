//! HUD presentation tokens shared by the canvas and editor-facing integration.

#![allow(dead_code)]

use bevy::prelude::Color;

/// Main HUD background color.
pub const HUD_BG: Color = Color::srgb(0.105, 0.11, 0.115);
/// Sidebar background color.
pub const HUD_SIDEBAR_BG: Color = Color::srgb(0.075, 0.08, 0.085);
/// Sidebar border color.
pub const HUD_SIDEBAR_BORDER: Color = Color::srgb(0.23, 0.24, 0.25);
/// Positive/active accent color.
pub const HUD_GREEN: Color = Color::srgb(0.62, 0.92, 0.10);
/// Translucent positive/active background.
pub const HUD_GREEN_BG: Color = Color::srgba(0.62, 0.92, 0.10, 0.14);
/// Primary HUD text color.
pub const HUD_TEXT: Color = Color::srgb(0.91, 0.91, 0.90);
/// Secondary HUD text color.
pub const HUD_DIM: Color = Color::srgb(0.65, 0.65, 0.64);
/// Tertiary HUD text color.
pub const HUD_DIMMER: Color = Color::srgb(0.39, 0.40, 0.40);
/// Icon tint color.
pub const HUD_ICON: Color = Color::srgb(0.80, 0.80, 0.78);
/// Warning/destructive accent color.
pub const HUD_AMBER: Color = Color::srgb(0.95, 0.54, 0.57);
/// Blue informational accent color.
pub const HUD_BLUE: Color = Color::srgb(0.23, 0.47, 1.0);
/// Teal informational accent color.
pub const HUD_TEAL: Color = Color::srgb(0.10, 0.72, 0.63);
/// Border color for badges and property controls.
pub const HUD_BADGE_BORDER: Color = Color::srgb(0.34, 0.35, 0.35);
/// Translucent selected-row color.
pub const HUD_ROW_SEL: Color = Color::srgba(0.94, 0.48, 0.52, 0.18);
/// Background color for editor cards.
pub const HUD_PANEL_CARD: Color = Color::srgb(0.15, 0.16, 0.16);

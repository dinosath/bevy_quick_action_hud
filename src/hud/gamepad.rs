//! Controller icon-family selection and detection.

use bevy::prelude::*;

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
pub(crate) fn detect_gamepad_icon_set(
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

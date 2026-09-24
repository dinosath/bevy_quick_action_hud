//! Retained HUD canvas rendering and HUD-specific runtime systems.

use crate::*;
use bevy::prelude::*;

pub(crate) fn hud_child(
    commands: &mut Commands,
    parent: Entity,
    scene: impl bevy::scene::prelude::Scene,
) -> Entity {
    let e = commands.spawn_scene(scene).id();
    commands.entity(parent).add_child(e);
    e
}

pub(crate) fn hud_text(s: &str, size: f32, color: Color) -> impl bevy::scene::prelude::Scene {
    let s = s.to_string();
    let sz = size;
    bsn! {
        Text({s})
        TextFont { font_size: {FontSize::Px(sz)} }
        TextColor({color})
    }
}

fn hud_wheel_icon(commands: &mut Commands, parent: Entity, icon: &str, size: f32, index: usize) {
    let colors = [
        Color::srgb(0.68, 1.0, 0.08),
        Color::srgb(0.05, 0.20, 1.0),
        Color::srgb(1.0, 0.10, 0.12),
        Color::srgb(0.02, 0.72, 0.66),
        Color::srgb(0.08, 0.72, 0.98),
        Color::srgb(0.55, 0.55, 0.55),
    ];
    let badge = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                width: {Val::Px(size)}, height: {Val::Px(size)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: {BorderRadius::all(Val::Px(size * 0.5))},
            }
            BackgroundColor({colors[index % colors.len()]})
        },
    );
    hud_child(commands, badge, hud_text(icon, size * 0.48, Color::WHITE));
}

pub(crate) fn hud_clickable(
    commands: &mut Commands,
    parent: Entity,
    scene: impl bevy::scene::prelude::Scene,
    action: WheelHudAction,
    base: Color,
) -> Entity {
    let e = commands
        .spawn_scene(scene)
        .insert(WheelHudButton { action, base })
        .id();
    commands.entity(parent).add_child(e);
    e
}

/// Parse `#rrggbb` hex string into a Bevy [`Color`].
pub fn parse_hex_color(hex: &str, alpha: f32) -> Color {
    let s = hex.trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return Color::srgba(r as f32 / 255., g as f32 / 255., b as f32 / 255., alpha);
        }
    }
    Color::srgba(0.23, 0.51, 0.96, alpha)
}

pub fn hud_label_or(key: &str) -> String {
    if key.is_empty() {
        "—".into()
    } else {
        key.into()
    }
}

// ── canvas root ──────────────────────────────────────────────────────────────────

fn hud_canvas_root() -> impl bevy::scene::prelude::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {Val::Px(0.)}, top: {Val::Px(0.)},
            right: {Val::Px(0.)}, bottom: {Val::Px(0.)},
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor({HUD_BG.with_alpha(1.0)})
    }
}

// ── main HUD build ───────────────────────────────────────────────────────────────

pub fn build_hud_canvas(
    commands: &mut Commands,
    cfg: &QuickActionConfig,
    hud: &WheelHudState,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
    wedge_materials: &mut Assets<WedgeMaterial>,
) {
    let root = commands
        .spawn_scene(hud_canvas_root())
        .insert(WheelHudRoot)
        .id();

    // Nothing to render while the wheel overlay is closed.
    if !hud.open {
        commands.entity(root).insert(BackgroundColor(Color::NONE));
        return;
    }

    // Apply the user-configured HUD background opacity.
    let hud_bg = if cfg.hud_bg_color.is_empty() {
        HUD_BG.with_alpha(cfg.hud_bg_opacity)
    } else {
        parse_hex_color(&cfg.hud_bg_color, cfg.hud_bg_opacity)
    };
    commands.entity(root).insert(BackgroundColor(hud_bg));

    // Edit toggle button — visible only while the wheel is open, hidden when
    // the editor sidebar is already showing.
    if !hud.editor_open {
        let btn = hud_clickable(
            commands,
            root,
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    top: {Val::Px(14.)}, left: {Val::Px(14.)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                    padding: {UiRect::axes(Val::Px(10.), Val::Px(6.))},
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(5.))},
                }
                BorderColor::all(HUD_BADGE_BORDER)
                BackgroundColor({HUD_PANEL_CARD})
                Button
            },
            WheelHudAction::ToggleEditor,
            HUD_PANEL_CARD,
        );
        // Show the assigned edit shortcut icon (if it's a gamepad button).
        if let Some(lbl) = cfg.edit_shortcut.strip_prefix("GP:") {
            if let Some(path) = icon_set.embedded_icon_path(lbl) {
                let handle = asset_server.load::<Image>(path);
                let icon_e = commands
                    .spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        ImageNode::new(handle),
                    ))
                    .id();
                commands.entity(btn).add_child(icon_e);
            }
        }
        {
            let handle = asset_server.load::<Image>(
                "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-cog.png",
            );
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    ImageNode {
                        image: handle,
                        color: HUD_DIM,
                        ..default()
                    },
                ))
                .id();
            commands.entity(btn).add_child(e);
        }
        hud_child(commands, btn, hud_text("Edit", 10., HUD_DIM));
    }

    // Set tabs at the top centre (only when enabled in config).
    if cfg.show_set_bar {
        build_hud_set_tabs(commands, root, cfg, hud, asset_server, icon_set);
    }
    if hud.editor_open {
        build_hud_editor_toolbar(
            commands,
            root,
            hud.settings_open,
            hud.edit_control_focus,
            &cfg.edit_shortcut,
        );
    }

    if cfg.sets.is_empty() {
        hud_child(
            commands,
            root,
            hud_text("No sets — open the editor to add one.", 12., HUD_DIMMER),
        );
        return;
    }

    if let Some(set) = cfg.sets.get(hud.active_set) {
        // Background image for this set, if configured.
        if !set.bg_image.is_empty() {
            let handle = asset_server.load::<Image>(set.bg_image.clone());
            let bg_e = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.),
                        top: Val::Px(0.),
                        right: Val::Px(0.),
                        bottom: Val::Px(0.),
                        ..default()
                    },
                    ImageNode {
                        image: handle,
                        color: Color::WHITE.with_alpha(set.bg_image_opacity),
                        ..default()
                    },
                ))
                .id();
            commands.entity(root).add_child(bg_e);
        }

        // Clamp active_wheel_entry to a valid range.
        let n_wheels = count_wheel_entries(set);
        let target = if n_wheels == 0 {
            0
        } else {
            hud.active_wheel_entry.min(n_wheels - 1)
        };

        // Find the target-th Wheel / RadialMenuSetState entry.
        let mut rendered = false;
        let mut wcount = 0usize;
        for (ei, entry) in set.entries.iter().enumerate() {
            let is_wheel = matches!(entry, SetEntry::Wheel(_) | SetEntry::RadialMenuSet(_));
            if !is_wheel {
                continue;
            }
            if wcount != target {
                wcount += 1;
                continue;
            }
            match entry {
                SetEntry::Wheel(w) => {
                    build_centered_wheel_hud(
                        commands,
                        root,
                        w,
                        hud.active_set,
                        ei,
                        None,
                        hud.highlighted,
                        hud.selected_wheel,
                        hud.hovered_wheel,
                        hud.editor_open,
                        hud.edit_control_focus,
                        hud.theme_popup_open,
                        wedge_materials,
                    );
                    rendered = true;
                }
                SetEntry::RadialMenuSet(ws) => {
                    let wheel_index = hud
                        .active_wheel_index
                        .min(ws.wheels.len().saturating_sub(1));
                    if let Some(w) = ws.wheels.get(wheel_index) {
                        let mut display_wheel = w.clone();
                        wheelset_visuals(ws).apply_to(&mut display_wheel);
                        build_centered_wheel_hud(
                            commands,
                            root,
                            &display_wheel,
                            hud.active_set,
                            ei,
                            Some(wheel_index),
                            hud.highlighted,
                            hud.selected_wheel,
                            hud.hovered_wheel,
                            hud.editor_open,
                            hud.edit_control_focus,
                            hud.theme_popup_open,
                            wedge_materials,
                        );
                        rendered = true;
                    }
                }
                _ => {}
            }
            break;
        }
        if !rendered {
            hud_child(
                commands,
                root,
                hud_text("No radial menus in this set.", 11., HUD_DIMMER),
            );
        }
        build_hud_action_buttons(
            commands,
            root,
            hud.active_set,
            set,
            asset_server,
            icon_set,
            hud.flash_action_entry,
            hud.editor_open,
        );
        build_hud_switch_buttons(
            commands,
            root,
            hud.active_set,
            set,
            asset_server,
            hud.editor_open,
        );
        if hud.highlighted.is_none() && hud.selected_wheel.is_none() && hud.hovered_wheel.is_none()
        {
            if let Some((selected_set, selected_entry)) = hud.selected_action {
                if selected_set == hud.active_set {
                    if let Some(SetEntry::Action(action)) = set.entries.get(selected_entry) {
                        build_hud_action_editor_card(
                            commands,
                            root,
                            selected_set,
                            selected_entry,
                            action,
                        );
                    }
                }
            }
        }
    }
}

fn build_hud_action_editor_card(
    commands: &mut Commands,
    parent: Entity,
    set: usize,
    entry: usize,
    action: &QuickAction,
) {
    editor::components::build_hud_action_editor_card(commands, parent, set, entry, action);
}
pub(crate) fn hud_action_field(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    height: f32,
    action: WheelHudAction,
    value_color: Color,
) {
    let field = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                height: {Val::Px(height)},
                padding: {UiRect::horizontal(Val::Px(9.))},
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        action,
        HUD_PANEL_CARD,
    );
    hud_child(commands, field, hud_text(label, 9., HUD_DIM));
    hud_child(commands, field, hud_text(value, 11., value_color));
}

pub(crate) fn hud_action_stepper(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    decrement: WheelHudAction,
    increment: WheelHudAction,
) {
    let row = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                height: {Val::Px(26.)},
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: {Val::Px(4.)},
            }
        },
    );
    hud_child(commands, row, hud_text(label, 10., HUD_TEXT));
    let minus = hud_clickable(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(26.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        decrement,
        HUD_PANEL_CARD,
    );
    hud_child(commands, minus, hud_text("−", 14., HUD_TEXT));
    let value_node = hud_child(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(52.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
        },
    );
    hud_child(commands, value_node, hud_text(value, 10., HUD_DIM));
    let plus = hud_clickable(
        commands,
        row,
        bsn! {
            Node {
                width: {Val::Px(26.)}, height: {Val::Px(24.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(4.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(HUD_BADGE_BORDER)
            Button
        },
        increment,
        HUD_PANEL_CARD,
    );
    hud_child(commands, plus, hud_text("+", 14., HUD_TEXT));
}

fn build_hud_editor_toolbar(
    commands: &mut Commands,
    parent: Entity,
    settings_open: bool,
    edit_focus: Option<usize>,
    edit_shortcut: &str,
) {
    let bar = hud_child(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(14.)}, left: {Val::Px(14.)},
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: {Val::Px(6.)},
                padding: {UiRect::all(Val::Px(0.))},
            }
            BackgroundColor({Color::NONE})
        },
    );
    let close = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                padding: {UiRect::axes(Val::Px(10.), Val::Px(6.))},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(5.))},
            }
            BackgroundColor({if edit_focus == Some(11) { HUD_AMBER } else { HUD_PANEL_CARD }})
            BorderColor::all(if edit_focus == Some(11) { HUD_TEXT } else { HUD_AMBER })
            Button
        },
        WheelHudAction::ToggleEditor,
        HUD_PANEL_CARD,
    );
    hud_child(
        commands,
        close,
        hud_text(&format!("Close  [{}]", edit_shortcut), 10., HUD_AMBER),
    );
    for (label, shortcut, action, accent, focus) in [
        (
            "Save",
            "LB / Ctrl+S",
            WheelHudAction::SaveConfig,
            HUD_GREEN,
            8,
        ),
        (
            "+ Add new",
            "RB / Ctrl+N",
            WheelHudAction::AddNewButton,
            HUD_TEXT,
            9,
        ),
        (
            if settings_open {
                "Settings ✓"
            } else {
                "Settings"
            },
            "Select / Ctrl+,",
            WheelHudAction::ToggleSettings,
            HUD_TEXT,
            10,
        ),
    ] {
        let button = hud_clickable(
            commands,
            bar,
            bsn! {
                Node {
                    height: {Val::Px(28.)},
                    padding: {UiRect::horizontal(Val::Px(10.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(3.))},
                }
                BackgroundColor({if edit_focus == Some(focus) { HUD_AMBER } else { HUD_PANEL_CARD }})
                BorderColor::all(if edit_focus == Some(focus) {
                    HUD_TEXT
                } else {
                    HUD_BADGE_BORDER
                })
                Button
            },
            action,
            HUD_PANEL_CARD,
        );
        hud_child(
            commands,
            button,
            hud_text(&format!("{label}  [{shortcut}]"), 10., accent),
        );
    }
}

/// Renders a radial wheel preview centred in the HUD.
#[allow(clippy::too_many_arguments)]
pub fn build_centered_wheel_hud(
    commands: &mut Commands,
    parent: Entity,
    wheel: &RadialMenu,
    set: usize,
    entry: usize,
    w_idx: Option<usize>,
    highlighted: Option<(usize, usize, Option<usize>, usize)>,
    selected_wheel: Option<(usize, usize, Option<usize>)>,
    hovered_wheel: Option<(usize, usize, Option<usize>)>,
    editor_open: bool,
    edit_control_focus: Option<usize>,
    theme_popup_open: bool,
    wedge_materials: &mut Assets<WedgeMaterial>,
) {
    let n_slices = wheel.slots.len().max(1);
    let hub = hud_child(commands, parent, wheel_hub());
    commands.entity(hub).insert(Node {
        position_type: PositionType::Relative,
        left: Val::Px(wheel.offset_x),
        top: Val::Px(-wheel.offset_y),
        ..default()
    });
    if wheel.rotation != 0.0 {
        commands
            .entity(hub)
            .insert(Transform::from_rotation(Quat::from_rotation_z(
                wheel.rotation.to_radians(),
            )));
    }

    let is_pie = wheel.segment_shape == SegmentShape::Pie;
    if !is_pie {
        let bg_col = if wheel.bg_color.is_empty() {
            Color::srgba(0.096, 0.118, 0.157, wheel.bg_opacity)
        } else {
            parse_hex_color(&wheel.bg_color, wheel.bg_opacity)
        };
        hud_child(commands, hub, wheel_bg_disc(wheel.outer_radius, bg_col));
    }
    let outer_col = if wheel.outer_border.is_empty() {
        Color::srgba(0.38, 0.39, 0.39, 0.90)
    } else {
        parse_hex_color(&wheel.outer_border, 1.0)
    };
    let outer_bw = if wheel.outer_border.is_empty() {
        1.0_f32
    } else {
        wheel.outer_border_width.max(0.0)
    };
    hud_child(
        commands,
        hub,
        wheel_outer_ring(wheel.outer_radius, outer_col, outer_bw),
    );

    let slice_angle = std::f32::consts::TAU / n_slices as f32;
    let base_pw = (2.0 * wheel.outer_radius * (slice_angle / 2.0).sin() * 0.72).max(48.0);
    let base_ph = ((wheel.outer_radius - wheel.inner_radius) * 0.85).max(40.0);
    let panel_w = (base_pw * wheel.segment_scale).max(32.0);
    let panel_h = (base_ph * wheel.segment_scale).max(24.0);
    let min_dim = panel_w.min(panel_h);
    let highlight_col = parse_hex_color(&wheel.highlight_color, 1.0);
    let slice_bg = Color::srgb(0.115, 0.12, 0.12);
    let label_c = Color::srgb(0.91, 0.91, 0.89);
    let label_sz = (panel_h * 0.18).clamp(9.0, 13.0);

    for (i, slot) in wheel.slots.iter().enumerate() {
        if i >= n_slices {
            break;
        }
        let is_sel = highlighted
            .map(|(s, e, w, sl)| s == set && e == entry && w == w_idx && sl == i)
            .unwrap_or(false);
        // The reference keeps the selected sector translucent so the dark
        // wheel surface remains visible beneath the coral tint.
        let seg_color = if is_sel {
            highlight_col.with_alpha(0.38)
        } else {
            slice_bg
        };

        if is_pie {
            let (a0, a1) = slice_angles(wheel, i);
            let mat_handle = wedge_materials.add(WedgeMaterial {
                params: WedgeParams {
                    color: seg_color.to_linear().to_vec4(),
                    border_color: if is_sel {
                        highlight_col.to_linear().to_vec4()
                    } else {
                        Color::srgb(0.30, 0.31, 0.31).to_linear().to_vec4()
                    },
                    inner_r: wheel.inner_radius,
                    outer_r: wheel.outer_radius,
                    angle_start: a0,
                    angle_end: a1,
                    edge_width: if is_sel { 2.0 } else { 0.8 },
                },
            });
            let dia = wheel.outer_radius * 2.0;
            let wedge_e = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(-wheel.outer_radius),
                        top: Val::Px(-wheel.outer_radius),
                        width: Val::Px(dia),
                        height: Val::Px(dia),
                        ..default()
                    },
                    MaterialNode(mat_handle),
                ))
                .id();
            commands.entity(hub).add_child(wedge_e);
            let ctr = slice_center(wheel, i);
            let panel_e = commands
                .spawn_scene(bsn! {
                    Node {
                        position_type: PositionType::Absolute,
                        left:   {Val::Px(ctr.x - panel_w / 2.0)},
                        top:    {Val::Px(-ctr.y - panel_h / 2.0)},
                        width:  {Val::Px(panel_w)}, height: {Val::Px(panel_h)},
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: {UiRect::all(Val::Px(6.))},
                    }
                    BackgroundColor({Color::NONE})
                })
                .insert((
                    WheelHudSegmentHit {
                        set,
                        entry,
                        wheel: w_idx,
                        slot: i,
                    },
                    Interaction::None,
                    Button,
                ))
                .id();
            commands.entity(hub).add_child(panel_e);
            if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    wheel_slice_label(slot.name.to_uppercase(), label_sz, label_c),
                );
            }
            if wheel.show_icon && !slot.icon.is_empty() {
                hud_wheel_icon(
                    commands,
                    panel_e,
                    &slot.icon,
                    (panel_h * 0.42).clamp(24.0, 44.0),
                    i,
                );
            } else if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    bsn! { Node { width: {Val::Px(4.)}, height: {Val::Px(4.)} } },
                );
            }
        } else {
            let seg_br = match wheel.segment_shape {
                SegmentShape::Square => BorderRadius::all(Val::Px(0.0)),
                SegmentShape::Rounded => BorderRadius::all(Val::Px(min_dim * 0.14)),
                SegmentShape::Circle => BorderRadius::all(Val::Px(min_dim * 0.5)),
                SegmentShape::Wedge => BorderRadius {
                    top_left: Val::Px(min_dim * 0.40),
                    top_right: Val::Px(min_dim * 0.40),
                    bottom_left: Val::Px(min_dim * 0.05),
                    bottom_right: Val::Px(min_dim * 0.05),
                },
                SegmentShape::Pie => unreachable!(),
            };
            let ctr = slice_center(wheel, i);
            let panel_e = commands
                .spawn_scene(bsn! {
                    Node {
                        position_type: PositionType::Absolute,
                        left:   {Val::Px(ctr.x - panel_w / 2.0)},
                        top:    {Val::Px(-ctr.y - panel_h / 2.0)},
                        width:  {Val::Px(panel_w)}, height: {Val::Px(panel_h)},
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        padding: {UiRect::all(Val::Px(6.))},
                        border_radius: {seg_br},
                    }
                    BackgroundColor({seg_color})
                })
                .insert((
                    WheelHudSegmentHit {
                        set,
                        entry,
                        wheel: w_idx,
                        slot: i,
                    },
                    Interaction::None,
                    Button,
                ))
                .id();
            commands.entity(hub).add_child(panel_e);
            if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    wheel_slice_label(slot.name.to_uppercase(), label_sz, label_c),
                );
            }
            if wheel.show_icon && !slot.icon.is_empty() {
                hud_wheel_icon(
                    commands,
                    panel_e,
                    &slot.icon,
                    (panel_h * 0.42).clamp(24.0, 44.0),
                    i,
                );
            } else if wheel.show_labels {
                hud_child(
                    commands,
                    panel_e,
                    bsn! { Node { width: {Val::Px(4.)}, height: {Val::Px(4.)} } },
                );
            }
        }
    }

    // Centre hub ring.
    let disc_r = (wheel.inner_radius - 4.0).max(8.0);
    let ring_col = if wheel.inner_border.is_empty() {
        Color::srgb(0.34, 0.35, 0.35)
    } else {
        parse_hex_color(&wheel.inner_border, 1.0)
    };
    let hub_bg = if wheel.hub_color.is_empty() {
        Color::srgba(0.10, 0.10, 0.10, wheel.hub_opacity)
    } else {
        parse_hex_color(&wheel.hub_color, wheel.hub_opacity)
    };
    let inner_bw = if wheel.inner_border.is_empty() {
        1.0_f32
    } else {
        wheel.inner_border_width.max(0.0)
    };
    let center = hud_child(
        commands,
        hub,
        wheel_center_ring(disc_r, hub_bg, ring_col, inner_bw),
    );
    if editor_open {
        commands.entity(center).insert((
            WheelHudButton {
                action: WheelHudAction::SelectWheel {
                    set,
                    entry,
                    wheel: w_idx,
                },
                base: hub_bg,
            },
            Button,
            Interaction::None,
        ));
    }

    // Show highlighted slot info; show nothing by default.
    let hub_slot = highlighted.and_then(|(hs, he, hw, si)| {
        if hs == set && he == entry && hw == w_idx {
            wheel.slots.get(si)
        } else {
            None
        }
    });
    if let Some(slot) = hub_slot {
        let info_col = hud_child(
            commands,
            center,
            bsn! {
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: {Val::Px(2.)},
                }
            },
        );
        let name_sz = (disc_r * 0.22).clamp(7.0, 10.0);
        if !slot.icon.is_empty() {
            let icon_index = highlighted.map(|(_, _, _, index)| index).unwrap_or(0);
            hud_wheel_icon(
                commands,
                info_col,
                &slot.icon,
                (disc_r * 0.58).clamp(24.0, 42.0),
                icon_index,
            );
        }
        if !slot.name.is_empty() {
            hud_child(commands, info_col, hud_text(&slot.name, name_sz, HUD_DIM));
        }
        if editor_open {
            hud_child(commands, info_col, hud_text("▣  Apply", 9., HUD_DIM));
            hud_child(commands, info_col, hud_text("▣  Back", 9., HUD_DIM));
            spawn_segment_editor_card(
                commands,
                hub,
                slot,
                set,
                entry,
                w_idx,
                highlighted.map(|(_, _, _, index)| index).unwrap_or(0),
                wheel.outer_radius,
                edit_control_focus,
            );
        }
    }
    if editor_open
        && highlighted.is_none()
        && (selected_wheel == Some((set, entry, w_idx))
            || hovered_wheel == Some((set, entry, w_idx)))
    {
        spawn_wheel_settings_card(commands, hub, wheel, theme_popup_open);
    }

    // Wheel-level controls are always visible in edit mode so hovering or
    // selecting the wheel exposes the same affordances as action buttons.
    if editor_open {
        let r = wheel.outer_radius + 28.0;
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(0.0, -r),
            WheelHudAction::WheelSettings {
                set,
                entry,
                wheel: w_idx,
            },
            "⚙",
            HUD_TEXT,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(r, 0.0),
            WheelHudAction::DeleteWheel {
                set,
                entry,
                wheel: w_idx,
            },
            "×",
            HUD_AMBER,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(0.0, r),
            WheelHudAction::MoveWheel {
                set,
                entry,
                wheel: w_idx,
            },
            "↕",
            HUD_TEXT,
            false,
        );
        spawn_radial_edit_button(
            commands,
            hub,
            Vec2::new(-r, 0.0),
            WheelHudAction::ResizeWheel {
                set,
                entry,
                wheel: w_idx,
                delta: 10.0,
            },
            "⌗",
            HUD_TEXT,
            false,
        );
    }

    // In edit mode, place the same compact radial controls used by the
    // reference UI: plus buttons on both sides of the selected sector and a
    // trash button on its inner edge.
    if editor_open {
        if let Some((hs, he, hw, slot)) = highlighted {
            if hs == set && he == entry && hw == w_idx && slot < n_slices {
                let (a0, a1) = slice_angles(wheel, slot);
                let mid = (a0 + a1) * 0.5;
                let p = Vec2::new(
                    a0.cos() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                    a0.sin() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::Before,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(1),
                );
                let p = Vec2::new(
                    mid.cos() * (wheel.inner_radius + 2.0),
                    mid.sin() * (wheel.inner_radius + 2.0),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::RemoveSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        slot,
                    },
                    "×",
                    HUD_TEXT,
                    edit_control_focus == Some(2),
                );
                // The second boundary is the after/right insertion point.
                let p = Vec2::new(
                    a1.cos() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                    a1.sin() * ((wheel.inner_radius + wheel.outer_radius) * 0.5),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::After,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(3),
                );
                // Outer-side add affordance, matching the reference editor.
                let p = Vec2::new(
                    mid.cos() * (wheel.outer_radius + 2.0),
                    mid.sin() * (wheel.outer_radius + 2.0),
                );
                spawn_radial_edit_button(
                    commands,
                    hub,
                    p,
                    WheelHudAction::AddSegment {
                        set,
                        entry,
                        wheel: w_idx,
                        side: SegmentInsertSide::Outer,
                    },
                    "+",
                    HUD_TEXT,
                    edit_control_focus == Some(4),
                );
            }
        }
    }
}

fn spawn_wheel_settings_card(
    commands: &mut Commands,
    parent: Entity,
    wheel: &RadialMenu,
    theme_popup_open: bool,
) {
    editor::components::spawn_wheel_settings_card(commands, parent, wheel, theme_popup_open);
}
fn spawn_segment_editor_card(
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
    editor::components::spawn_segment_editor_card(
        commands,
        parent,
        slot,
        set,
        entry,
        wheel,
        slot_index,
        outer_radius,
        edit_control_focus,
    );
}
fn spawn_radial_edit_button(
    commands: &mut Commands,
    parent: Entity,
    position: Vec2,
    action: WheelHudAction,
    label: &str,
    color: Color,
    focused: bool,
) {
    editor::components::spawn_radial_edit_button(
        commands, parent, position, action, label, color, focused,
    );
}
/// Floating quick-action buttons in the bottom-right corner.
fn build_hud_action_buttons(
    commands: &mut Commands,
    parent: Entity,
    set_index: usize,
    set: &ActionSet,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
    flash_entry: Option<usize>,
    editor_open: bool,
) {
    let btns: Vec<(usize, &QuickAction)> = set
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if let SetEntry::Action(a) = e {
                Some((i, a))
            } else {
                None
            }
        })
        .filter(|(_, a)| a.enabled)
        .collect();
    if btns.is_empty() {
        return;
    }

    // Keep each action on an independent anchor. A flex column would include
    // every button's current height in the layout calculation, so resizing one
    // button would move all buttons above it.
    const ACTION_STACK_STEP: f32 = 36.0;
    let container = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(60.)}, right: {Val::Px(36.)},
                width: {Val::Px(360.)}, height: {Val::Px(520.)},
            }
        })
        .id();
    commands.entity(parent).add_child(container);

    for (stack_index, (entry_idx, qa)) in btns.iter().rev().enumerate() {
        let is_flash = flash_entry == Some(*entry_idx);
        let eff = (set.opacity * qa.opacity).clamp(0.05, 1.0);
        let w = qa.width.max(40.0);
        let h = qa.height.max(20.0);
        let bg = if is_flash {
            Color::srgba(0.38, 0.62, 0.95, 0.90) // bright blue flash
        } else {
            parse_hex_color(&qa.color, eff * 0.85)
        };
        let tc = HUD_TEXT.with_alpha(if is_flash { 1.0 } else { eff });
        let bc = HUD_BADGE_BORDER.with_alpha(eff);

        let row = hud_child(
            commands,
            container,
            bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    right: {Val::Px(-qa.offset_x)},
                    bottom: {Val::Px(stack_index as f32 * ACTION_STACK_STEP + qa.offset_y)},
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                }
            },
        );
        if !qa.key.is_empty() {
            // GP: key → show the controller button icon; keyboard key → text badge.
            let mut showed_icon = false;
            if let Some(btn_label) = qa.key.strip_prefix("GP:") {
                if let Some(path) = icon_set.embedded_icon_path(btn_label) {
                    let handle = asset_server.load::<Image>(path);
                    let icon_e = commands
                        .spawn((
                            Node {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                ..default()
                            },
                            ImageNode::new(handle),
                        ))
                        .id();
                    commands.entity(row).add_child(icon_e);
                    showed_icon = true;
                }
            }
            if !showed_icon {
                // Keyboard fallback — bordered text badge.
                let key_disp = qa.key.strip_prefix("GP:").unwrap_or(&qa.key);
                let kb = hud_child(
                    commands,
                    row,
                    bsn! {
                        Node {
                            min_width: {Val::Px(16.)}, height: {Val::Px(16.)},
                            padding: {UiRect::horizontal(Val::Px(3.))},
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: {UiRect::all(Val::Px(1.))},
                            border_radius: {BorderRadius::all(Val::Px(2.))},
                        }
                        BorderColor::all(HUD_BADGE_BORDER)
                    },
                );
                hud_child(commands, kb, hud_text(key_disp, 8., HUD_DIM));
            }
        }
        let button_wrap = hud_child(
            commands,
            row,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    width: {Val::Px(w)}, height: {Val::Px(h)},
                }
            },
        );
        let btn_node = commands
            .spawn_scene(bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: {Val::Px(0.)}, top: {Val::Px(0.)},
                    width: {Val::Px(w)}, height: {Val::Px(h)},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                    border_radius: {BorderRadius::all(Val::Px(4.))},
                }
                BackgroundColor({bg})
                BorderColor::all(bc)
            })
            .insert((
                WheelHudButton {
                    action: WheelHudAction::SelectAction {
                        set: set_index,
                        entry: *entry_idx,
                    },
                    base: bg,
                },
                Button,
                Interaction::None,
            ))
            .id();
        if qa.rotation != 0.0 {
            commands
                .entity(btn_node)
                .insert(Transform::from_rotation(Quat::from_rotation_z(
                    qa.rotation.to_radians(),
                )));
        }
        commands.entity(button_wrap).add_child(btn_node);
        hud_child(commands, btn_node, hud_text(&qa.name, 10., tc));
        if editor_open {
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w * 0.5 - 11.),
                Val::Px(-24.),
                asset_server,
                "cil-camera-control",
                WheelHudAction::MoveAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w + 2.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-aperture",
                WheelHudAction::RotateAction {
                    set: set_index,
                    entry: *entry_idx,
                    delta: 15.0,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(w * 0.5 - 11.),
                Val::Px(h + 2.),
                asset_server,
                "cil-trash",
                WheelHudAction::DeleteAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_AMBER,
            );
            spawn_action_edge_button(
                commands,
                button_wrap,
                Val::Px(-24.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-cog",
                WheelHudAction::EditAction {
                    set: set_index,
                    entry: *entry_idx,
                },
                HUD_TEXT,
            );
        }
    }
}

/// Floating HUD-switch components share the same contextual editor affordances
/// as quick-action buttons: settings, move, resize, and delete.
fn build_hud_switch_buttons(
    commands: &mut Commands,
    parent: Entity,
    set_index: usize,
    set: &ActionSet,
    asset_server: &AssetServer,
    editor_open: bool,
) {
    let switches: Vec<(usize, &HudSwitch)> = set
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if let SetEntry::HudSwitch(s) = e {
                s.enabled.then_some((i, s))
            } else {
                None
            }
        })
        .collect();
    if switches.is_empty() {
        return;
    }
    let container = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                bottom: {Val::Px(104.)}, right: {Val::Px(36.)},
                flex_direction: FlexDirection::Column, row_gap: {Val::Px(8.)},
                align_items: AlignItems::FlexEnd,
            }
        })
        .id();
    commands.entity(parent).add_child(container);
    for (entry, switch) in switches.iter().rev() {
        let row = hud_child(
            commands,
            container,
            bsn! {
                Node {
                    position_type: PositionType::Relative,
                    left: {Val::Px(switch.offset_x)}, top: {Val::Px(-switch.offset_y)},
                    flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                    column_gap: {Val::Px(5.)},
                }
            },
        );
        let button = hud_clickable(
            commands,
            row,
            bsn! {
                Node {
                    width: {Val::Px(switch.width.max(40.))}, height: {Val::Px(switch.height.max(20.))},
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))}, border_radius: {BorderRadius::all(Val::Px(4.))},
                }
                BackgroundColor({Color::srgba(0.38, 0.26, 0.62, 0.85)})
                BorderColor::all(HUD_BADGE_BORDER)
                Button
            },
            WheelHudAction::SelectHudSwitch {
                set: set_index,
                entry: *entry,
            },
            HUD_PANEL_CARD,
        );
        hud_child(commands, button, hud_text(&switch.name, 10., HUD_TEXT));
        if editor_open {
            let w = switch.width.max(40.);
            let h = switch.height.max(20.);
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w * 0.5 - 11.),
                Val::Px(-24.),
                asset_server,
                "cil-camera-control",
                WheelHudAction::MoveHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w + 2.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-aperture",
                WheelHudAction::ResizeHudSwitch {
                    set: set_index,
                    entry: *entry,
                    delta: 8.0,
                },
                HUD_TEXT,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(w * 0.5 - 11.),
                Val::Px(h + 2.),
                asset_server,
                "cil-trash",
                WheelHudAction::DeleteHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_AMBER,
            );
            spawn_action_edge_button(
                commands,
                button,
                Val::Px(-24.),
                Val::Px(h * 0.5 - 11.),
                asset_server,
                "cil-cog",
                WheelHudAction::EditHudSwitch {
                    set: set_index,
                    entry: *entry,
                },
                HUD_TEXT,
            );
        }
    }
}

fn spawn_action_edge_button(
    commands: &mut Commands,
    parent: Entity,
    left: Val,
    top: Val,
    asset_server: &AssetServer,
    icon: &str,
    action: WheelHudAction,
    color: Color,
) {
    let owner = hud_control_owner(&action);
    let button = hud_clickable(
        commands,
        parent,
        bsn! {
            Node {
                position_type: PositionType::Absolute,
                left: {left}, top: {top},
                width: {Val::Px(22.)}, height: {Val::Px(22.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::all(Val::Px(11.))},
            }
            BackgroundColor({HUD_PANEL_CARD})
            BorderColor::all(if color == HUD_AMBER { HUD_AMBER } else { HUD_BADGE_BORDER })
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
    let handle = asset_server.load::<Image>(format!(
        "embedded://bevy_quick_action_hud/embedded/icons/editor/{icon}.png"
    ));
    let icon_node = commands
        .spawn((
            Node {
                width: Val::Px(14.),
                height: Val::Px(14.),
                ..default()
            },
            ImageNode {
                image: handle,
                color,
                ..default()
            },
        ))
        .id();
    commands.entity(button).add_child(icon_node);
}

pub(crate) fn hud_control_owner(action: &WheelHudAction) -> Option<HudControlOwner> {
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
        | WheelHudAction::EditSegmentInput {
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

/// Set-selection tab bar pinned to the bottom of the HUD.
fn build_hud_set_tabs(
    commands: &mut Commands,
    parent: Entity,
    cfg: &QuickActionConfig,
    hud: &WheelHudState,
    asset_server: &AssetServer,
    icon_set: GamepadIconSet,
) {
    let bar = commands
        .spawn_scene(bsn! {
            Node {
                position_type: PositionType::Absolute,
                top: {Val::Px(12.)}, left: {Val::Px(0.)}, right: {Val::Px(0.)},
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
        })
        .id();
    commands.entity(parent).add_child(bar);

    let prev_idx = hud.active_set.saturating_sub(1);
    let larrow = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                width: {Val::Px(28.)}, height: {Val::Px(32.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::left(Val::Px(6.))},
            }
            BorderColor::all(HUD_SIDEBAR_BORDER)
            BackgroundColor({HUD_PANEL_CARD})
            Button
        },
        WheelHudAction::SetActiveSet(prev_idx),
        HUD_PANEL_CARD,
    );
    // Chevron-left PNG icon; gamepad icon overlays it when a button is assigned.
    {
        let handle = asset_server.load::<Image>(
            "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-chevron-left.png",
        );
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    color: HUD_DIM,
                    ..default()
                },
            ))
            .id();
        commands.entity(larrow).add_child(e);
    }
    // Overlay the assigned prev-set icon if it's a gamepad button.
    if let Some(lbl) = cfg.prev_set_key.strip_prefix("GP:") {
        if let Some(path) = icon_set.embedded_icon_path(lbl) {
            let handle = asset_server.load::<Image>(path);
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    ImageNode::new(handle),
                ))
                .id();
            commands.entity(larrow).add_child(e);
        }
    }

    for (i, set) in cfg.sets.iter().enumerate() {
        let active = i == hud.active_set;
        let (bg, tc, bc) = if active {
            (Color::srgba(0.38, 0.62, 0.95, 0.20), HUD_TEXT, HUD_BLUE)
        } else {
            (HUD_PANEL_CARD, HUD_DIM, HUD_SIDEBAR_BORDER)
        };
        let tab = hud_clickable(
            commands,
            bar,
            bsn! {
                Node {
                    padding: {UiRect::axes(Val::Px(14.), Val::Px(7.))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: {UiRect::all(Val::Px(1.))},
                }
                BorderColor::all(bc)
                BackgroundColor({bg})
                Button
            },
            WheelHudAction::SetActiveSet(i),
            bg,
        );
        hud_child(commands, tab, hud_text(&set.name, 11., tc));
    }

    let next_idx = (hud.active_set + 1).min(cfg.sets.len().saturating_sub(1));
    let rarrow = hud_clickable(
        commands,
        bar,
        bsn! {
            Node {
                width: {Val::Px(28.)}, height: {Val::Px(32.)},
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: {UiRect::all(Val::Px(1.))},
                border_radius: {BorderRadius::right(Val::Px(6.))},
            }
            BorderColor::all(HUD_SIDEBAR_BORDER)
            BackgroundColor({HUD_PANEL_CARD})
            Button
        },
        WheelHudAction::SetActiveSet(next_idx),
        HUD_PANEL_CARD,
    );
    // Chevron-right PNG icon; gamepad icon overlays it when a button is assigned.
    {
        let handle = asset_server.load::<Image>(
            "embedded://bevy_quick_action_hud/embedded/icons/editor/cil-chevron-right.png",
        );
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                ImageNode {
                    image: handle,
                    color: HUD_DIM,
                    ..default()
                },
            ))
            .id();
        commands.entity(rarrow).add_child(e);
    }
    // Overlay the assigned next-set icon if it's a gamepad button.
    if let Some(lbl) = cfg.next_set_key.strip_prefix("GP:") {
        if let Some(path) = icon_set.embedded_icon_path(lbl) {
            let handle = asset_server.load::<Image>(path);
            let e = commands
                .spawn((
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    ImageNode::new(handle),
                ))
                .id();
            commands.entity(rarrow).add_child(e);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// WheelHudPlugin
// ─────────────────────────────────────────────────────────────────────────────────

/// Renders a full-screen HUD showing the active [`QuickActionConfig`] set.
///
/// Add this plugin (alongside [`WheelMenuPlugin`]) to display wheels and
/// quick-action buttons.  Add [`crate::editor::QuickActionEditorPlugin`] on top
/// to get the editor sidebar.
///
/// ```ignore
/// app.add_plugins((WheelMenuPlugin, WheelHudPlugin));
/// ```
/// Backward-compat wrapper — use [`QuickActionHudPlugin::default()`] instead.
///
/// Provides core wheel logic + HUD canvas (no editor).
pub struct WheelHudPlugin;
impl Plugin for WheelHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuickActionHudPlugin::default());
    }
}

pub(crate) fn hud_button_feedback(
    mut buttons: Query<(&WheelHudButton, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (btn, interaction, mut bg) in &mut buttons {
        let next = match interaction {
            Interaction::Hovered => BackgroundColor(Color::srgba(1., 1., 1., 0.05)),
            Interaction::Pressed => BackgroundColor(Color::srgba(0.38, 0.62, 0.95, 0.16)),
            Interaction::None => BackgroundColor(btn.base),
        };
        if *bg != next {
            *bg = next;
        }
    }
}

pub(crate) fn hud_context_visibility(
    hud: Res<WheelHudState>,
    mut controls: Query<(&HudContextControl, &mut Visibility)>,
) {
    for (control, mut visibility) in &mut controls {
        let visible = hud.editor_open
            && match control.owner {
                HudControlOwner::Action(set, entry) => hud.selected_action == Some((set, entry)),
                HudControlOwner::Wheel(set, entry, wheel) => {
                    hud.selected_wheel == Some((set, entry, wheel))
                }
                HudControlOwner::HudSwitch(set, entry) => {
                    hud.selected_hud_switch == Some((set, entry))
                }
            };
        let next = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

/// Updates [`WheelHudState::highlighted`] while the HUD wheel is open.
///
/// Uses **release-to-use**: the slot that was highlighted when the stick
/// returns to the dead-zone is emitted as a [`HudSegmentSelected`] event.
/// The stick side (L/R) is read from the active wheel entry's `stick` field.
pub(crate) fn hud_stick_nav(
    gamepads: Query<&Gamepad>,
    mut hud: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    mut select_ev: MessageWriter<HudSegmentSelected>,
) {
    if !hud.open || hud.editor_open {
        return;
    }

    // Locate the active wheel entry (honoring active_wheel_entry).
    let Some(set) = cfg.sets.get(hud.active_set) else {
        return;
    };
    let mut found: Option<(usize, Option<usize>, usize, StickSide)> = None;
    let target = hud.active_wheel_entry;
    let mut wcount = 0usize;
    for (ei, entry) in set.entries.iter().enumerate() {
        let is_wheel = matches!(entry, SetEntry::Wheel(_) | SetEntry::RadialMenuSet(_));
        if !is_wheel {
            continue;
        }
        if wcount != target {
            wcount += 1;
            continue;
        }
        match entry {
            SetEntry::Wheel(w) => {
                found = Some((ei, None, w.slots.len(), w.stick));
            }
            SetEntry::RadialMenuSet(ws) => {
                let wheel_index = hud
                    .active_wheel_index
                    .min(ws.wheels.len().saturating_sub(1));
                if let Some(w) = ws.wheels.get(wheel_index) {
                    found = Some((ei, Some(wheel_index), w.slots.len(), ws.stick));
                }
            }
            _ => {}
        }
        break;
    }
    let Some((entry_idx, wheel_idx, n_slots, stick_side)) = found else {
        return;
    };
    if n_slots == 0 {
        return;
    }

    // Read raw gamepad axes for the configured stick.
    let mut stick = Vec2::ZERO;
    if let Some(gamepad) = gamepads.iter().next() {
        let (xa, ya) = match stick_side {
            StickSide::Right => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
            StickSide::Left => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
        };
        stick = Vec2::new(
            gamepad.get(xa).unwrap_or(0.0),
            gamepad.get(ya).unwrap_or(0.0),
        );
    }

    const DEADZONE: f32 = 0.2;
    let prev = hud.highlighted;

    let new_highlight = if stick.length() < DEADZONE {
        if hud.editor_open {
            prev
        } else {
            None
        }
    } else {
        // Same angle mapping as RadialMenu::arc_offset default (FRAC_PI_6).
        let a = stick.y.atan2(stick.x);
        let rel = (a - std::f32::consts::FRAC_PI_6).rem_euclid(std::f32::consts::TAU);
        let idx = ((rel / std::f32::consts::TAU) * n_slots as f32).floor() as usize;
        Some((hud.active_set, entry_idx, wheel_idx, idx.min(n_slots - 1)))
    };

    if prev != new_highlight {
        debug!(
            "[hud] stick highlight changed: {:?} -> {:?} editor_open={}",
            prev, new_highlight, hud.editor_open
        );
        // Release-to-use: emit selection when stick returns to dead-zone.
        if let (Some((s, e, w, slot)), None) = (prev, new_highlight) {
            if !hud.editor_open {
                // Normal mode: fire selection and optionally close.
                let slot_close = cfg
                    .sets
                    .get(s)
                    .and_then(|set| set.entries.get(e))
                    .and_then(|entry| match (entry, w) {
                        (SetEntry::Wheel(wd), None) => wd.slots.get(slot),
                        (SetEntry::RadialMenuSet(ws), Some(wi)) => {
                            ws.wheels.get(wi).and_then(|wd| wd.slots.get(slot))
                        }
                        _ => None,
                    })
                    .map(|s| s.close_on_select)
                    .unwrap_or(false);

                select_ev.write(HudSegmentSelected {
                    set: s,
                    entry: e,
                    wheel: w,
                    slot,
                });

                if slot_close {
                    hud.open = false;
                }
            }
            // Dry-run mode: highlight clears visually — no event, no close.
        }
        hud.highlighted = new_highlight;
        if hud.editor_open && new_highlight.is_some() {
            hud.mouse_hovered_segment = None;
            hud.edit_control_focus = Some(0);
        }
        hud.dirty = true;
    }
}

/// Counts down the dry-run flash timer.  When expired it clears the flash entry and
/// triggers a HUD rebuild so the button returns to its normal colour.
pub(crate) fn tick_hud_dry_run_flash(time: Res<Time>, mut hud: ResMut<WheelHudState>) {
    // Only tick while a flash is active and the HUD is not already queued for rebuild.
    if hud.flash_action_entry.is_some() && !hud.dirty {
        hud.flash_action_ttl -= time.delta_secs();
        if hud.flash_action_ttl <= 0.0 {
            hud.flash_action_entry = None;
            hud.dirty = true;
        }
    }
}

pub(crate) fn rebuild_hud(
    mut commands: Commands,
    mut hud: ResMut<WheelHudState>,
    cfg: Res<QuickActionConfig>,
    asset_server: Res<AssetServer>,
    icon_set: Res<GamepadIconSet>,
    old_hud: Query<Entity, With<WheelHudRoot>>,
    children: Query<&Children>,
    mut wedge_materials: ResMut<Assets<WedgeMaterial>>,
) {
    if !hud.dirty {
        return;
    }
    debug!("[hud] rebuild requested: open={} editor_open={} active_set={} selected_action={:?} hovered_action={:?} selected_wheel={:?} hovered_wheel={:?}",
        hud.open, hud.editor_open, hud.active_set, hud.selected_action, hud.hovered_action,
        hud.selected_wheel, hud.hovered_wheel);
    hud.dirty = false;
    if cfg
        .sets
        .get(hud.active_set)
        .is_none_or(|page| !page.enabled)
    {
        if let Some(page) = enabled_hud_pages(&cfg).first().copied() {
            hud.active_set = page;
        }
    }

    debug!(
        "[hud] rebuild_hud — open={} editor_open={} active_set={} active_wheel_entry={}",
        hud.open, hud.editor_open, hud.active_set, hud.active_wheel_entry
    );

    for e in &old_hud {
        debug!("[hud] recursively despawning HUD root {:?}", e);
        despawn_hud_tree(&mut commands, e, &children);
        commands.entity(e).despawn();
    }

    if !cfg.sets.is_empty() && hud.active_set >= cfg.sets.len() {
        hud.active_set = cfg.sets.len() - 1;
    }

    build_hud_canvas(
        &mut commands,
        &cfg,
        &hud,
        &asset_server,
        *icon_set,
        &mut wedge_materials,
    );
}

fn despawn_hud_tree(commands: &mut Commands, entity: Entity, children: &Query<&Children>) {
    if let Ok(kids) = children.get(entity) {
        for child in kids.iter() {
            despawn_hud_tree(commands, child, children);
            commands.entity(child).despawn();
        }
    }
}

use bevy::prelude::{Alpha, Color, Vec2};

use super::components::RadialMenuToggleMode;
use super::geometry::*;
use super::model::{RadialMenu, WheelTheme};
use super::style::*;

fn assert_color_eq(actual: Color, expected: [f32; 4]) {
    let actual = actual.to_srgba();
    let actual = [actual.red, actual.green, actual.blue, actual.alpha];
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "expected {expected:?}, got {actual:?}"
        );
    }
}

#[test]
fn toggle_mode_default_is_hold() {
    assert_eq!(RadialMenuToggleMode::default(), RadialMenuToggleMode::Hold);
}

#[test]
fn sector_resolution_excludes_configured_gaps() {
    let mut menu = RadialMenu::new("Test", 4);
    menu.arc_offset = 0.0;
    menu.gap = 0.2;

    assert_eq!(sector_index_at_angle(&menu, 0.4), Some(0));
    assert_eq!(sector_index_at_angle(&menu, 1.58), None);
}

#[test]
fn sector_resolution_respects_partial_arcs() {
    let mut menu = RadialMenu::new("Test", 2);
    menu.arc_offset = 0.0;
    menu.arc_span = std::f32::consts::PI;
    menu.gap = 0.0;

    assert_eq!(sector_index_at_angle(&menu, 0.5), Some(0));
    assert_eq!(sector_index_at_angle(&menu, 2.0), Some(1));
    assert_eq!(sector_index_at_angle(&menu, 4.0), None);
}

#[test]
fn conic_wedges_match_menu_sector_angles() {
    let menu = RadialMenu::default();
    // Sector 0 is centered at twelve o'clock: clockwise from -45° to +45°.
    let (begin, span) = conic_sector(&menu, 0);
    let expected_span = std::f32::consts::FRAC_PI_2 - menu.gap;
    assert!((span - expected_span).abs() < 1e-5);
    let expected_begin = std::f32::consts::TAU - std::f32::consts::FRAC_PI_4 + menu.gap * 0.5;
    assert!((begin - expected_begin).abs() < 1e-5);
    // Sector 1 (counter-clockwise from sector 0) is centered at nine o'clock.
    let (begin, span) = conic_sector(&menu, 1);
    let center = begin + span * 0.5;
    assert!((center - 3.0 * std::f32::consts::FRAC_PI_2).abs() < 1e-5);
}

#[test]
fn default_menu_is_centered_without_a_user_translation() {
    let menu = RadialMenu::default();

    assert_eq!(menu.offset_x, 0.0);
    assert_eq!(menu.offset_y, 0.0);
}

#[test]
fn default_four_sector_menu_starts_at_twelve_oclock() {
    let menu = RadialMenu::default();
    let (start, end) = slice_angles(&menu, 0);

    assert!(((start + end) * 0.5 - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
}

#[test]
fn sector_anchors_sit_on_the_sector_edges_and_arcs() {
    let menu = RadialMenu::default();
    let band = (menu.inner_radius + menu.outer_radius) * 0.5;
    let (start, _) = slice_angles(&menu, 0);

    let leading = sector_anchor(&menu, 0, SectorAnchor::StartEdge);
    assert!(leading.distance(Vec2::from_angle(start) * band) < 1e-3);
    let inner = sector_anchor(&menu, 0, SectorAnchor::InnerArc);
    assert!(inner.distance(Vec2::new(0.0, menu.inner_radius + 2.0)) < 1e-3);
    let outer = sector_anchor(&menu, 0, SectorAnchor::OuterArc);
    assert!(outer.distance(Vec2::new(0.0, menu.outer_radius + 2.0)) < 1e-3);
}

#[test]
fn default_layout_matches_measured_reference_image() {
    let menu = RadialMenu::default();
    let center_radius = (menu.inner_radius + menu.outer_radius) * 0.5;
    let expected_centers = [
        Vec2::new(0.0, center_radius),
        Vec2::new(-center_radius, 0.0),
        Vec2::new(0.0, -center_radius),
        Vec2::new(center_radius, 0.0),
    ];

    assert_eq!(menu.slots.len(), 4);
    assert_eq!(menu.outer_radius, 270.0);
    assert_eq!(menu.inner_radius, 145.0);
    assert_eq!(menu.highlight_color, "#a8e9ec");
    assert_eq!(HIGHLIGHTED_SECTOR_OUTSET, 38.0);
    assert_eq!(rendered_sector_outer_radius(&menu, false), 270.0);
    assert_eq!(rendered_sector_outer_radius(&menu, true), 308.0);
    assert_eq!(radial_menu_hub_radius(&menu), 141.0);

    // Measurements from image.png's menu crop: the four-sector base ring
    // has a 270 px radius, the selected top sector reaches 308 px, and
    // the circular hub masks panels at roughly 141 px.  The first sector
    // spans 45°..135°, giving these two selected outer endpoints.
    let (start, end) = slice_angles(&menu, 0);
    assert!((start - (std::f32::consts::FRAC_PI_4 + menu.gap * 0.5)).abs() < 1e-5);
    assert!((end - (std::f32::consts::FRAC_PI_4 * 3.0 - menu.gap * 0.5)).abs() < 1e-5);
    let selected_radius = rendered_sector_outer_radius(&menu, true);
    let left_endpoint = Vec2::from_angle(end) * selected_radius;
    let right_endpoint = Vec2::from_angle(start) * selected_radius;
    assert!(left_endpoint.distance(Vec2::new(-216.5, 219.1)) < 0.2);
    assert!(right_endpoint.distance(Vec2::new(216.5, 219.1)) < 0.2);

    // Hit panels run from the selected outer edge to the hub center, so the
    // hub covers their inner end.
    let panel_height = radial_menu_sector_panel_height(selected_radius);
    assert!(selected_radius - panel_height < radial_menu_hub_radius(&menu));
    for (index, expected) in expected_centers.into_iter().enumerate() {
        assert!(slice_center(&menu, index).distance(expected) < 1e-4);
    }
}

#[test]
fn default_reference_presentation_locks_colors_spacing_and_control_style() {
    let menu = RadialMenu::default();

    // Authored default state: the reference is a centered, dark, icon-led
    // four-sector menu.  Empty border/color fields deliberately select
    // the fallback presentation values asserted below.
    assert_eq!(menu.theme, WheelTheme::Dark);
    assert!(!menu.show_labels);
    assert!(menu.show_icon);
    assert_eq!(menu.opacity, 1.0);
    assert_eq!(menu.bg_opacity, 1.0);
    assert_eq!(menu.hub_opacity, 1.0);
    assert!(!menu.overlap);
    assert_eq!(menu.gap, 0.012);
    assert_eq!(menu.rotation, 0.0);
    assert!(menu.bg_color.is_empty());
    assert!(menu.hub_color.is_empty());
    assert!(menu.inner_border.is_empty());
    assert!(menu.outer_border.is_empty());
    assert_eq!(menu.outer_border_width, 2.0);
    assert_eq!(menu.inner_border_width, 2.0);

    // Exact fallback palette used by the BSN scene.
    assert_color_eq(
        RADIAL_MENU_BACKGROUND.with_alpha(menu.bg_opacity),
        [0.096, 0.118, 0.157, 1.0],
    );
    assert_color_eq(RADIAL_MENU_OUTER_BORDER, [0.38, 0.39, 0.39, 0.90]);
    assert_color_eq(RADIAL_MENU_SECTOR, [0.10, 0.11, 0.12, 1.0]);
    assert_color_eq(RADIAL_MENU_DIVIDER, [0.30, 0.32, 0.33, 0.85]);
    assert_color_eq(RADIAL_MENU_HUB_BORDER, [0.34, 0.35, 0.35, 1.0]);
    assert_color_eq(RADIAL_MENU_LABEL, [0.91, 0.91, 0.89, 1.0]);

    // #a8e9ec is the selected outline and, blended at 34%, produces the
    // exact selected fill emitted by the retained scene.
    let selected = blend_radial_menu_colors(
        RADIAL_MENU_SECTOR,
        Color::srgb(168.0 / 255.0, 233.0 / 255.0, 236.0 / 255.0),
        RADIAL_MENU_SELECTED_HIGHLIGHT_WEIGHT,
    );
    assert_color_eq(selected, [0.29, 0.383_266_7, 0.393_866_7, 1.0]);

    // Structural styling and spacing of the native BSN sector scene.
    assert_eq!(RADIAL_MENU_BACKGROUND_BLEED, 4.0);
    assert_eq!(RADIAL_MENU_OUTER_RING_OUTSET, 1.0);
    assert_eq!(RADIAL_MENU_DEFAULT_BORDER_WIDTH, 1.0);
    assert_eq!(RADIAL_MENU_DIVIDER_WIDTH, 1.5);
    assert_eq!(RADIAL_MENU_SELECTED_OUTLINE_WIDTH, 2.0);
    assert_eq!(RADIAL_MENU_HUB_OVERLAP, 4.0);
    assert_eq!(RADIAL_MENU_SECTOR_CONTENT_PADDING, 6.0);

    let thickness = menu.outer_radius - menu.inner_radius;
    let label_size = (thickness * RADIAL_MENU_LABEL_SIZE_RATIO).clamp(
        RADIAL_MENU_LABEL_SIZE_RANGE.0,
        RADIAL_MENU_LABEL_SIZE_RANGE.1,
    );
    let icon_size = (rendered_sector_outer_radius(&menu, false) * RADIAL_MENU_ICON_SIZE_RATIO)
        .clamp(RADIAL_MENU_ICON_SIZE_RANGE.0, RADIAL_MENU_ICON_SIZE_RANGE.1);
    assert_eq!(label_size, 13.0);
    assert_eq!(icon_size, 44.0);
}

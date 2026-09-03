//! Z5 receipts: the design fit, and the board's pixel grid.
//!
//! Same instrument as [`super::host_routing`] — [`Harness`] is the shipping
//! host with `window: None` — but with `HostOptions::fit_design` set and the
//! short window this laptop actually opens. The routing receipts deliberately
//! stay at zoom 1 with no fit, so between the two files the board is proved at
//! the identity and under a real fractional zoom.
//!
//! Windowless, the device scale is 1.0, so a receipt about a 200% display
//! states its `(device scale, zoom)` pair directly — which is exactly what the
//! frame hook hands [`UiState::set_pixel_grid`] headed, and the only input the
//! board's scale has.

use cambium_genet_winit_host::{inert_hooks, Harness, HostOptions, Init};
use genet_probe::Selector;
use layout_dom_api::{LayoutDom as _, LocalName, Namespace};

use super::*;

type BoardHarness = Harness<UiState, Logic, UiChild>;

/// The window this laptop opens: 1100 wide, and 752 tall because the host
/// clamps the asked-for 820 to the display's logical height less 48.
const SHORT_WINDOW: (f32, f32) = (1_100.0, 752.0);

/// This laptop's other half: a 200% display, which is where a fractional zoom
/// and a whole device pixel actually have to be reconciled.
const DEVICE_SCALE: f32 = 2.0;

/// A board laid out in `window`, with or without the design fit.
fn board(window: (f32, f32), fit: bool) -> BoardHarness {
    let mut ui = UiState::new(demo_map());
    ui.camera = (420.0, 140.0);
    let mut hooks = inert_hooks();
    hooks.key_intercept = Box::new(hooks::key_intercept);
    hooks.focused_text = Box::new(hooks::focused_text);
    let mut harness = Harness::with_hooks_and_options(
        Init {
            state: ui,
            logic: board_root as Logic,
            sheet: board_css(),
        },
        hooks,
        HostOptions {
            fit_design: fit.then_some(DESIGN_SIZE),
            ..Default::default()
        },
    );
    harness.layout_at(window.0, window.1);
    // What `hooks::init` and the frame hook do headed, in the same order: the
    // board's pixel grid from the host's pair, then the pane from the post-zoom
    // size. Windowless the device scale is 1.0.
    let grid = (1.0, harness.ui_zoom());
    let (logical_w, logical_h) = harness.logical_size();
    harness.update(|ui| {
        ui.set_pixel_grid(grid);
        ui.viewport = ((logical_w - isometry_views::PANEL_W).max(0.0), logical_h);
    });
    harness.relayout();
    harness
}

/// The painted box of the first element carrying `class`.
fn rect(harness: &BoardHarness, class: &str) -> (f32, f32, f32, f32) {
    harness
        .with_dom(|dom| {
            genet_probe::matching(dom, &Selector::class(class))
                .first()
                .copied()
        })
        .and_then(|node| harness.painted_rect(node))
        .unwrap_or_else(|| panic!("`{class}` has a laid-out box"))
}

/// Whether a device-pixel measurement lands on a whole pixel. The tolerance is
/// f32 accumulation over three multiplies, not a slack allowance.
fn whole(device: f32) -> bool {
    (device - device.round()).abs() < 1e-3
}

/// The bottom edge of the panel's last painted row. `.side` adds its own 14px
/// of bottom padding under it, which is part of the 20 the plan's target
/// leaves free rather than part of this figure.
///
/// A maximum over the panel's own rows rather than a named last row, because
/// which row is last is a matter of state now — the key hints stand only while
/// a target pick is armed.
fn panel_bottom(harness: &BoardHarness) -> f32 {
    let rows = harness.with_dom(|dom| {
        let side = genet_probe::matching(dom, &Selector::class("side"))
            .first()
            .copied()
            .expect("the panel strip is in the retained tree");
        dom.dom_children(side).collect::<Vec<_>>()
    });
    assert!(!rows.is_empty(), "the panel draws rows at all");
    rows.into_iter()
        .filter_map(|node| harness.painted_rect(node))
        .map(|(_, y, _, h)| y + h)
        .fold(0.0_f32, f32::max)
}

/// The fit lays the interface out at the design rather than at the window.
///
/// Both halves of the arithmetic are asserted, because `min` of the two ratios
/// means only the *binding* axis lands on the design figure: the height comes
/// out at 820 and the width keeps its slack at about 1199.5. That is the plan's
/// Z1 note, taken from the consumer's side.
#[test]
fn the_fit_lays_out_at_the_design_rather_than_the_window() {
    let harness = board(SHORT_WINDOW, true);
    let (logical_w, logical_h) = harness.logical_size();
    assert!(
        (harness.ui_zoom() - SHORT_WINDOW.1 / DESIGN_SIZE.1).abs() < 1e-4,
        "the height is the binding axis: {}",
        harness.ui_zoom()
    );
    assert!(
        (logical_h - DESIGN_SIZE.1).abs() < 0.5,
        "the binding axis lands on the design within rounding: {logical_h}"
    );
    assert!(
        logical_w > DESIGN_SIZE.0,
        "the other axis keeps its slack rather than shrinking: {logical_w}"
    );
    // And the interface really is laid out in that space: the pane is what is
    // left of the post-zoom width after the panel strip, not of the window's.
    let (pane_x, _, pane_w, pane_h) = rect(&harness, "pane");
    assert_eq!(pane_x, isometry_views::PANEL_W);
    assert!(
        (pane_w - (logical_w - isometry_views::PANEL_W)).abs() < 0.5
            && (pane_h - logical_h).abs() < 0.5,
        "the pane fills the post-zoom surface: {pane_w}x{pane_h}"
    );
}

/// A short window under the fit lays out exactly as a full-height window does
/// with no fit: same boxes, same coordinates, top to bottom of the panel.
///
/// This is the whole promise of `fit_design` stated as an identity, and it is
/// what the headed capture is showing when it shows more of the panel than the
/// window has room for. Its counterpart is the negative control below it: the
/// same short window with no fit, where every panel row sits 68 logical pixels
/// lower relative to the surface.
#[test]
fn the_fit_makes_a_short_window_lay_out_like_the_design() {
    let fitted = board(SHORT_WINDOW, true);
    let designed = board(DESIGN_SIZE, false);
    for class in ["side", "side-title", "turn-list", "messages", "side-status"] {
        let (fx, fy, fw, fh) = rect(&fitted, class);
        let (dx, dy, dw, dh) = rect(&designed, class);
        // Within a rounding of each other rather than bit-equal: the fitted
        // surface is 752/0.9170732, which is 820 to six figures and not to
        // seven.
        assert!(
            (fx - dx).abs() < 0.01
                && (fy - dy).abs() < 0.01
                && (fw - dw).abs() < 0.01
                && (fh - dh).abs() < 0.01,
            "`{class}` lands where the design puts it: \
             ({fx}, {fy}, {fw}, {fh}) vs ({dx}, {dy}, {dw}, {dh})"
        );
    }

    // Negative control: without the fit the same window lays out at 752 and the
    // panel's own box is 68 shorter, which is exactly the room the fit bought.
    let unfitted = board(SHORT_WINDOW, false);
    assert_eq!(unfitted.ui_zoom(), 1.0);
    let (_, _, _, fitted_side) = rect(&fitted, "side");
    let (_, _, _, unfitted_side) = rect(&unfitted, "side");
    assert!(
        (fitted_side - unfitted_side - (DESIGN_SIZE.1 - SHORT_WINDOW.1)).abs() < 0.5,
        "the fit is worth the whole difference: {fitted_side} vs {unfitted_side}"
    );
}

/// The side panel fits the design figure it is being fitted to.
///
/// This is the standing form of a receipt that used to say the opposite. From
/// 2026-09-03 `the_side_panel_is_still_taller_than_the_design` recorded that
/// the panel's last row ended at **1024** logical px, 1038 with the column's
/// own padding, where `DESIGN_SIZE` names 820 — so the whisper composer, the
/// status line and the key hints sat below the fold on any display shorter
/// than about 1040, and the fit only shrank the interface without revealing
/// them. The side panel diet
/// (`design_docs/2026-09-03_side_panel_diet_plan.md`) took it to 796, and this
/// is what keeps it there: the target is 800, which leaves 20 for padding and
/// slack inside the 820 design.
///
/// Stated twice on purpose. At zoom 1 in a design-sized window, which is the
/// figure the plan names and the one a cut is measured against; and in the
/// short window the fit actually opens on this laptop, where what has to hold
/// is that the panel ends inside the post-zoom surface rather than under it.
#[test]
fn the_side_panel_fits_the_design_height() {
    let designed = board(DESIGN_SIZE, false);
    assert_eq!(designed.ui_zoom(), 1.0, "the plan's figure is a zoom-1 one");
    let needed = panel_bottom(&designed);
    assert!(
        needed <= 800.0,
        "the panel's last painted row ends at or above 800 on the demo skirmish: {needed}"
    );

    let fitted = board(SHORT_WINDOW, true);
    let surface = fitted.logical_size().1;
    let bottom = panel_bottom(&fitted) + 14.0;
    assert!(
        bottom <= surface,
        "and the fit puts the whole column, padding and all, on screen: \
         {bottom} of {surface}"
    );
}

/// With rounding on, a tile's device size is a whole number of pixels; with it
/// off, at the same zoom, it is not.
///
/// Stated on the painted box rather than on `geo`, so the receipt covers the
/// emitted element as well as the arithmetic — the two are one scale or the
/// board tears. The elevation step is asserted too: it is the unit the rounding
/// is actually taken on, and an elevation column that stacked on a half pixel
/// would seam exactly where a tile would not.
#[test]
fn rounding_lands_the_board_on_whole_device_pixels() {
    let mut harness = board(SHORT_WINDOW, true);
    let grid = (DEVICE_SCALE, harness.ui_zoom());
    harness.update(|ui| ui.set_pixel_grid(grid));
    harness.relayout();

    let device = |css: f32| css * grid.0 * grid.1;
    let (_, _, tile_w, tile_h) = rect(&harness, "tile");
    let geo = harness.state().geo;
    assert!(
        (tile_w - geo.tile_w).abs() < 1e-3 && (tile_h - geo.tile_h).abs() < 1e-3,
        "the emitted box is the scaled projection: {tile_w}x{tile_h} vs {geo:?}"
    );
    assert!(
        whole(device(tile_w)) && whole(device(tile_h)),
        "a tile is a whole number of device pixels: {}x{}",
        device(tile_w),
        device(tile_h)
    );
    assert!(
        whole(device(geo.elev_step)),
        "and so is the elevation step: {}",
        device(geo.elev_step)
    );

    // Off: the raw fractional zoom, which is what the setting exists to allow.
    harness.update(|ui| ui.toggle_integer_pixel_rounding());
    harness.relayout();
    let (_, _, raw_w, raw_h) = rect(&harness, "tile");
    assert_eq!(harness.state().board_scale, 1.0);
    assert_eq!(
        (raw_w, raw_h),
        (32.0, 16.0),
        "rounding off is the sheet's own geometry"
    );
    assert!(
        !whole(device(raw_w)) && !whole(device(raw_h)),
        "and at this zoom it lands between device pixels: {}x{}",
        device(raw_w),
        device(raw_h)
    );
}

/// Zoom 1 is the identity on every ordinary device scale, so the setting costs
/// the existing receipts — and any consumer on a display with the room —
/// nothing at all.
#[test]
fn the_pixel_grid_is_inert_at_zoom_one() {
    let mut harness = board(SHORT_WINDOW, true);
    for scale in [1.0, 1.25, 1.5, 2.0, 3.0] {
        harness.update(|ui| ui.set_pixel_grid((scale, 1.0)));
        assert_eq!(
            harness.state().board_scale,
            1.0,
            "device scale {scale} at zoom 1 needs no board scale"
        );
        assert_eq!(harness.state().geo, isometry_core::IsoGeometry::default());
    }
    // And `BOARD_UNIT` really is the projection's own finest unit, which is the
    // whole reason rounding it lands the tile's two axes as well.
    assert_eq!(
        isometry_views::BOARD_UNIT,
        isometry_core::IsoGeometry::default().elev_step
    );
}

/// The panel's toggle moves the board at runtime, through the shipping click
/// path rather than by writing the flag.
#[test]
fn the_panel_toggle_relayouts_the_board() {
    let mut harness = board(SHORT_WINDOW, true);
    assert!(harness.state().integer_pixel_rounding, "on by default");
    let (_, _, rounded_w, _) = rect(&harness, "tile");
    assert!(
        (rounded_w - 32.0).abs() > 0.1,
        "the fit is fractional here, so rounding really is doing something: {rounded_w}"
    );

    assert!(
        harness.click_on(&Selector::class("px-grid")),
        "the Map section carries the toggle"
    );
    harness.relayout();
    assert!(!harness.state().integer_pixel_rounding);
    let (_, _, raw_w, _) = rect(&harness, "tile");
    assert_eq!(raw_w, 32.0, "off, the board is back to the sheet's geometry");

    assert!(harness.click_on(&Selector::class("px-grid")));
    harness.relayout();
    assert!(harness.state().integer_pixel_rounding);
    let (_, _, again_w, _) = rect(&harness, "tile");
    assert_eq!(again_w, rounded_w, "and back on again is where it started");
}

/// Hit testing stays exact under the board's own scale: a click at a tile's
/// projected centre selects whichever tile is painted there.
///
/// The receipt is deliberately the same shape as
/// `host_routing::a_click_on_a_tile_selects_it` — "the tile *under the pointer*
/// is the one that got selected", read back from the tree — because diamonds
/// are drawn as overlapping rectangles and which one owns a point is the
/// engine's business. What must hold under a scaled board is that a press and a
/// paint still agree, which they do by construction: one `geo`.
#[test]
fn a_click_still_lands_on_the_tile_under_it_at_a_fractional_zoom() {
    let mut harness = board(SHORT_WINDOW, true);
    assert!(
        harness.state().board_scale != 1.0,
        "the board really is scaled here"
    );
    let (board_x, board_y, _, _) = rect(&harness, "board");
    let (sx, sy) = harness.state().geo.tile_to_screen((2, 2), 0);
    let (x, y) = (board_x + sx, board_y + sy);

    let class_at = |harness: &mut BoardHarness| {
        harness.move_to(x, y);
        let hit = harness.hit().expect("something is painted under the pointer");
        harness
            .with_dom(|dom| {
                dom.attribute(hit, &Namespace::from(""), &LocalName::from("class"))
                    .map(str::to_owned)
            })
            .unwrap_or_default()
    };
    assert!(class_at(&mut harness).starts_with("tile "));
    harness.click_at(x, y);
    harness.relayout();
    let selected = harness
        .state()
        .selected
        .expect("a click on the scaled board selects a tile");
    assert!(
        harness.state().map.ground.in_bounds(selected.0, selected.1),
        "the selected tile is a real board tile: {selected:?}"
    );
    assert!(
        class_at(&mut harness).contains("tile-selected"),
        "the tile under the pointer is the one that got selected"
    );
}

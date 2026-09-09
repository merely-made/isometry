//! The board screen: side panel plus the iso board pane.
//!
//! Every ground tile, elevation-column filler, prop, and token is one
//! absolutely-positioned element inside the `.board` container; the
//! container's inline `left`/`top` carries the camera, so a pan is a
//! single attribute change on one element. Depth comes from
//! [`isometry_core::depth_key`] as a plain z-index.

use std::collections::HashSet;

use cambium::{
    AnyView, CommandEvent, CommandItem, CommandState, ElementView, GenetCtx, GenetElement,
    HoverEvent, HoverPhase, OverlayDismiss, OverlayRole, OverlaySurface, Placement, PointerButton,
    PointerEvent, PointerPhase, WheelEvent, clickable, command_menu, el, lens, map_action,
    on_hover, on_pointer, on_wheel, overlay_surface,
};
use isometry_core::{MapDocument, TileCoord, TileKindId, Token, depth_key, path_to};

use crate::panel::side_panel;
use crate::projection::tile_board_cells;
use crate::state::{EditMode, FogLevel, PANEL_W, UiState};

pub type UiChild = Box<dyn AnyView<UiState, (), GenetCtx, GenetElement>>;

/// The unscaled box the stylesheet gives each kind of board element, in CSS
/// pixels. These restate `theme.rs`, because a board element's size and its
/// projected position have to move together and only one of the two can live
/// in a sheet; `the_board_screen_lays_out_where_the_gestures_expect` is the
/// receipt that the pair still agrees.
const TILE_BOX: (f32, f32) = (32.0, 16.0);
const PROP_BOX: (f32, f32) = (20.0, 24.0);
const TOKEN_BOX: (f32, f32) = (24.0, 36.0);
const MARKER_BOX: (f32, f32) = (28.0, 14.0);

/// One board element's inline geometry: where it sits, how big it is, and how
/// deep it stands.
///
/// `size` is the sheet's own unscaled number for the box, and it is emitted
/// only when the board is not at scale 1 — where it would restate what the
/// stylesheet already says, character for character. Above or below 1 it
/// overrides the sheet, which is what keeps the diamonds meeting: the
/// projection in [`UiState::geo`] and the boxes drawn on it are one scale, or
/// the board tears along every tile edge.
fn placed(ui: &UiState, (x, y): (f32, f32), size: (f32, f32), z: i32) -> String {
    let mut style = format!("left: {x}px; top: {y}px; z-index: {z};");
    if ui.board_scale != 1.0 {
        let (w, h) = (size.0 * ui.board_scale, size.1 * ui.board_scale);
        style.push_str(&format!(" width: {w}px; height: {h}px;"));
    }
    style
}

/// Report the tile a board element stands on as the pointer enters and leaves
/// it, so the play-mode path preview and the measure template follow the
/// cursor.
///
/// The granularity comes from the tree because it cannot come from the host:
/// the shared host routes `on_hover` Enter and Leave as the *hit element*
/// changes and deliberately routes no Move, so a single handler on the pane
/// would only ever learn that the pointer is somewhere over the board. Every
/// element that stands on a tile carries this instead, and one crossing is one
/// Leave followed by one Enter. [`UiState::hover_tile_enter`] holds the gate
/// that decides whether the change is worth a rebuild.
fn standing_on<V>(child: V, at: TileCoord) -> UiChild
where
    V: ElementView<UiState, ()> + 'static,
{
    Box::new(on_hover(
        child,
        move |ui: &mut UiState, event: HoverEvent| match event.phase {
            HoverPhase::Enter => ui.hover_tile_enter(Some(at)),
            HoverPhase::Leave => ui.hover_tile_enter(None),
            HoverPhase::Move => {},
        },
    ))
}

/// One diamond at tile `at`, drawn at `elevation`, with `class` deciding
/// its paint. Clicking selects the tile.
fn tile_el(
    ui: &UiState,
    at: TileCoord,
    elevation: i32,
    class: String,
    label: Option<String>,
    accessible_label: Option<String>,
) -> UiChild {
    let geo = &ui.geo;
    let (cx, cy) = geo.tile_to_screen(at, elevation);
    let (x, y) = (cx - geo.tile_w / 2.0, cy - geo.tile_h / 2.0);
    let z = depth_key(at, elevation);
    let mut tile = el("div", ())
        .attr("class", class)
        .attr("style", placed(ui, (x, y), TILE_BOX, z));
    if let Some(label) = label {
        tile = tile.attr("title", label);
    }
    if let Some(label) = accessible_label {
        tile = tile.attr("aria-label", label);
    }
    standing_on(
        clickable(tile, move |ui: &mut UiState, _| {
            ui.click_tile(at);
        }),
        at,
    )
}

fn kind_name(map: &MapDocument, kind: TileKindId) -> &str {
    map.tile_kinds
        .get(kind.0 as usize)
        .map(String::as_str)
        .unwrap_or("empty")
}

/// Viewport culling margins (logical px). A tile is kept when its diamond,
/// elevation column, or standing sprite can still touch the pane, so the
/// margins are generous and asymmetric: a tile above the pane only pokes
/// down by half a diamond, while a tile below the pane can poke up by a
/// full elevation column plus a sprite. Over-emitting a thin ring is cheap;
/// clipping a visible tile is a bug.
const CULL_MARGIN_X: f32 = 32.0;
const CULL_MARGIN_ABOVE: f32 = 24.0;
const CULL_MARGIN_BELOW: f32 = 176.0;

/// Whether tile `at` can touch the board pane under the current camera, so
/// the whole-grid emitters only build elements the viewport can show. Until
/// the host reports a viewport (`(0, 0)`), this returns `true`, so an unset
/// viewport degrades to the pre-windowing "emit everything" behavior.
/// Battle-to-region scale is the design center; a pathologically tall tile
/// (elevation past ~18) below the pane edge is the one case the fixed bottom
/// margin does not cover, and that is outside the aesthetic.
fn in_view(ui: &UiState, at: TileCoord) -> bool {
    let (vw, vh) = ui.viewport;
    if vw <= 0.0 || vh <= 0.0 {
        return true;
    }
    let (bx, by) = ui.geo.tile_to_screen(at, 0);
    let px = bx + ui.camera.0;
    let py = by + ui.camera.1;
    px >= -CULL_MARGIN_X
        && px <= vw + CULL_MARGIN_X
        && py >= -CULL_MARGIN_ABOVE
        && py <= vh + CULL_MARGIN_BELOW
}

mod menu;
mod tiles;
mod tokens;

// The 2026-09-04 split moved the drawing passes into the modules above; this
// file keeps the shared imports, the box sizes, the placement and culling
// helpers every pass reads, and the two roots that assemble what they return.
// The five items `board_root` calls back into are `pub(super)` for exactly
// that reason and no wider; imported here so their call sites are unchanged.
use menu::context_menu_overlay;
use tiles::{ground_tiles, prop_tiles};
use tokens::{marker_el, token_el};
/// The screen root the runner diffs.
pub fn board_root(ui: &UiState) -> UiChild {
    let mut layers: Vec<UiChild> = ground_tiles(ui);
    layers.extend(prop_tiles(ui));
    // Markers and tokens follow fog: a marker only shows on a token the
    // viewer can currently see.
    let marker_shown = |id: isometry_core::TokenId| {
        ui.map
            .token(id)
            .map(|t| ui.token_visible(t))
            .unwrap_or(false)
    };
    if let Some(id) = ui.selected_token {
        if marker_shown(id) {
            layers.extend(marker_el(ui, id, "marker marker-select"));
        }
    }
    if let Some(active) = ui.turns.active() {
        if marker_shown(active) {
            layers.extend(marker_el(ui, active, "marker marker-turn"));
        }
    }
    layers.extend(
        ui.map
            .tokens
            .iter()
            .filter(|t| ui.token_visible(t))
            .map(|t| token_el(ui, t)),
    );
    // Windowing metric: with `ISOMETRY_PROFILE` on, report how many
    // elements the viewport emits. It should stay bounded by the pane, not
    // grow with the board (see the windowing plan).
    if std::env::var_os("ISOMETRY_PROFILE").is_some() {
        eprintln!("[isometry] board elements emitted: {}", layers.len());
    }
    let (camx, camy) = ui.camera;
    let mut pane_children: Vec<UiChild> = vec![Box::new(
        el("div", layers)
            .attr("class", "board")
            .attr("style", format!("left: {camx}px; top: {camy}px;")),
    )];
    if let Some(overlay) = crate::character::character_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::sheet::sheet_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::compendium::compendium_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::generator::generator_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::storylet::storylet_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::downtime::downtime_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::overmap::overmap_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::governance::governance_overlay(ui) {
        pane_children.push(overlay);
    }
    // The token menu is the one overlay that is *not* a pane child: its
    // dismissal layer has to cover the side panel too, and `.pane` clips.
    let mut screen: Vec<UiChild> = vec![side_panel(ui), board_pane(pane_children)];
    if let Some(menu) = context_menu_overlay(ui) {
        screen.push(menu);
    }
    Box::new(el("div", screen).attr("class", "app"))
}

/// The board pane, carrying the board's own pointer and wheel gestures.
///
/// Both handlers hang here rather than on the window, which is what keeps the
/// side panel out of them: a drag can never spam the panel's buttons and a
/// wheel notch over the panel never pans the board, by construction rather
/// than by a coordinate test. `local` is measured against this element's
/// painted box, so a handler receives the pointer in the pane's own
/// coordinates — the space [`UiState::open_context_menu`] already anchors in,
/// and the space every gesture on [`UiState`] takes.
fn board_pane(children: Vec<UiChild>) -> UiChild {
    let pane = el("div", children).attr("class", "pane");
    let pane = on_wheel(pane, |ui: &mut UiState, event: WheelEvent| {
        // The wheel pans, so the host's scrolling default must not also run:
        // `.pane` is `overflow: hidden`, and an unconsumed notch would chain
        // out to the document viewport and drag the whole board with it.
        event.prevent_default();
        ui.board_wheel(event.delta.0, event.delta.1);
    });
    Box::new(on_pointer(pane, |ui: &mut UiState, event: PointerEvent| {
        match (event.phase, event.button) {
            // A secondary press is one-shot: it captures nothing and no Move
            // or Up follows it, so opening the menu is the whole gesture.
            (PointerPhase::Down, PointerButton::Secondary) => ui.board_context_menu(event.local),
            (PointerPhase::Down, PointerButton::Primary) => {
                if std::env::var_os("ISOMETRY_PROFILE").is_some() {
                    eprintln!(
                        "[isometry] board press at {:?} tile {:?}",
                        event.local,
                        ui.tile_at_pane(event.local)
                    );
                }
                ui.board_press(event.local);
            },
            (PointerPhase::Move, _) => ui.board_drag(event.local),
            (PointerPhase::Up, _) => ui.board_release(event.local),
        }
    }))
}

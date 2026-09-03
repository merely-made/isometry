//! Host-routing receipts: the board's gestures, driven through the real host.
//!
//! [`Harness`] is the shipping host with `window: None` — the same hit test,
//! the same pointer capture, the same `key_intercept`, the same dispatch order,
//! minus a swapchain. So a test here exercises production routing rather than a
//! parallel imitation of it, which is exactly what the migration needed proving:
//! the gestures that used to be hand-routed by `input.rs` now ride
//! `on_pointer`, `on_hover` and `on_wheel` on the board pane, and nothing
//! between a press and `UiState` is isometry's any more.
//!
//! Coordinates are **measured, not assumed**. The board's origin is read off
//! the laid-out `.board` container, so these receipts say nothing about where
//! the panel ends — that is the host's flex layout to decide, and a test that
//! hard-coded it would fail for the wrong reason the day it changed.

use cambium_genet_winit_host::{inert_hooks, Harness, Init, KeyPress, NamedKey};
use genet_probe::Selector;
use isometry_core::{TileCoord, TokenId};
use layout_dom_api::{LayoutDom as _, LocalName, Namespace};

use super::*;

type BoardHarness = Harness<UiState, Logic, UiChild>;

/// A laid-out board with isometry's own key policy and text seam installed.
///
/// The other five hooks stay inert: nothing here is about the campaign store,
/// the session bridge or the self-tests, and a hook that reached for a `System`
/// this harness never loaded would be testing the fixture.
/// The window the receipts lay out in. The pane is what is left of it after
/// the panel strip, which is what `init` and the frame hook seed `viewport`
/// with in the shipping host.
const WINDOW: (f32, f32) = (1_100.0, 820.0);

fn board() -> BoardHarness {
    let mut ui = UiState::new(demo_map());
    // The same camera `init` opens on, so the demo skirmish sits inside the
    // pane rather than off its left edge.
    ui.camera = (420.0, 140.0);
    // What `App::sync_viewport` would report for this window. Viewport culling
    // reads it, and so does the token menu's dismissal layer, which is sized
    // to the window; leaving it at `(0, 0)` would give that layer no area and
    // quietly prove nothing.
    ui.viewport = (WINDOW.0 - isometry_views::PANEL_W, WINDOW.1);
    let mut hooks = inert_hooks();
    hooks.key_intercept = Box::new(hooks::key_intercept);
    hooks.focused_text = Box::new(hooks::focused_text);
    let mut harness = Harness::with_hooks(
        Init {
            state: ui,
            logic: board_root as Logic,
            sheet: board_css(),
        },
        hooks,
    );
    harness.layout_at(WINDOW.0, WINDOW.1);
    harness
}

/// The board container's top-left, in the coordinates the pointer uses.
fn board_origin(harness: &BoardHarness) -> (f32, f32) {
    let node = harness
        .with_dom(|dom| {
            genet_probe::matching(dom, &Selector::class("board"))
                .first()
                .copied()
        })
        .expect("the board container is in the retained tree");
    let (x, y, _, _) = harness
        .painted_rect(node)
        .expect("the board container has a laid-out box");
    (x, y)
}

/// Where a tile's diamond centre lands, in the coordinates the pointer uses.
fn tile_point(harness: &BoardHarness, at: TileCoord) -> (f32, f32) {
    let (ox, oy) = board_origin(harness);
    let ui = harness.state();
    let elevation = *ui
        .map
        .elevation
        .get(at.0.max(0) as u32, at.1.max(0) as u32)
        .unwrap_or(&0) as i32;
    let (sx, sy) = ui.geo.tile_to_screen(at, elevation);
    (ox + sx, oy + sy)
}

/// The `class` of whatever the pointer is over.
fn class_at(harness: &mut BoardHarness, x: f32, y: f32) -> String {
    harness.move_to(x, y);
    let hit = harness
        .hit()
        .expect("something is painted under the pointer");
    harness
        .with_dom(|dom| {
            dom.attribute(hit, &Namespace::from(""), &LocalName::from("class"))
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

/// A click reaches the tile's own handler, which is what selects it. The whole
/// chain is under test: hit test, click dispatch, then the pointer-down that
/// begins the drag on the pane above it.
///
/// The receipt is deliberately "the tile *under the pointer* is the one that
/// got selected", read back from the tree, rather than a hard-coded coordinate.
/// Diamonds are drawn as overlapping rectangles, so which of several tiles owns
/// a given point is the engine's business, not this test's; what must hold is
/// that the click reached whichever one it was.
#[test]
fn a_click_on_a_tile_selects_it() {
    let mut harness = board();
    let (x, y) = tile_point(&harness, (2, 2));
    assert!(class_at(&mut harness, x, y).starts_with("tile "));
    assert!(harness.state().selected.is_none());

    harness.click_at(x, y);
    harness.relayout();

    let selected = harness
        .state()
        .selected
        .expect("a click on the board selects a tile");
    assert!(
        harness.state().map.ground.in_bounds(selected.0, selected.1),
        "the selected tile is a real board tile: {selected:?}"
    );
    assert!(
        class_at(&mut harness, x, y).contains("tile-selected"),
        "the tile under the pointer is the one that got selected"
    );
}

/// A right press opens the token's context menu, anchored where it landed.
///
/// This is the one gesture the shared host could not deliver before genet's M4:
/// a secondary press is routed into the same `on_pointer` element a primary
/// press would capture, marked `Secondary`, and captures nothing.
#[test]
fn a_right_click_on_a_token_opens_its_menu() {
    let mut harness = board();
    let knight = harness
        .state()
        .map
        .token(TokenId(1))
        .expect("the demo skirmish stands a knight")
        .at;
    let (x, y) = tile_point(&harness, knight);
    assert!(harness.state().context_menu.is_none());
    harness.right_click_at(x, y);
    let (id, _) = harness
        .state()
        .context_menu
        .expect("a right press on a token opens its menu");
    assert_eq!(id, TokenId(1));
    // Right-click also selects, so the menu's rows act on what it named.
    assert_eq!(harness.state().selected_token, Some(TokenId(1)));

    // And the rows are still reachable through the dismissal layer the menu
    // now sits on: the layer covers the window, but the card is its later,
    // higher sibling. The first row is Sheet.
    assert!(harness.click_on(&Selector::class("command-item")));
    assert_eq!(
        harness.state().open_sheet,
        Some(TokenId(1)),
        "a press on a row activates it rather than dismissing the menu"
    );
    assert!(harness.state().context_menu.is_none(), "and then closes it");
}

/// A wheel notch over the board pans it, and the host's own scrolling default
/// does not also run — the handler consumes the notch.
#[test]
fn a_wheel_notch_over_the_board_pans_it() {
    let mut harness = board();
    let (x, y) = tile_point(&harness, (8, 12));
    harness.move_to(x, y);
    let before = harness.state().camera;
    let tile_h = harness.state().geo.tile_h;
    // One notch, in the direction the content moves. Two diagonal tile steps
    // per notch, and a step is half a tile footprint on each axis.
    harness.wheel(0.0, isometry_views::WHEEL_NOTCH_PX);
    let after = harness.state().camera;
    assert_eq!(after.0, before.0, "a vertical notch does not pan sideways");
    assert!(
        (after.1 - (before.1 - tile_h)).abs() < 0.01,
        "one notch pans one tile height: {before:?} -> {after:?}"
    );
    assert_eq!(
        harness.element_scroll_total(),
        0.0,
        "the pane consumed the notch, so nothing scrolled behind it"
    );
}

/// Escape backs out of an armed action before anything else reads it, so a
/// target pick is always cancellable without spending a turn.
#[test]
fn escape_cancels_a_target_pick() {
    let mut harness = board();
    harness.update(|ui| ui.action_pick = Some((TokenId(1), "attack".to_owned())));
    assert!(harness.state().picking_target());
    harness.press_key(&KeyPress::named(NamedKey::Escape));
    assert!(
        !harness.state().picking_target(),
        "the key intercept consumed Escape and cleared the pick"
    );
}

/// The board screen lays out where the gestures assume it does: a fixed panel
/// strip, a pane filling the rest, and tiles at their projected boxes.
///
/// Worth asserting in its own right. When the migration's first headed run came
/// back black, this receipt is what separated "the tree did not lay out" from
/// "the tree laid out and the paint pass dropped it" — and it was the second.
#[test]
fn the_board_screen_lays_out_where_the_gestures_expect() {
    let harness = board();
    let rect = |class: &str| {
        harness
            .with_dom(|dom| {
                genet_probe::matching(dom, &Selector::class(class))
                    .first()
                    .copied()
            })
            .and_then(|node| harness.painted_rect(node))
            .unwrap_or_else(|| panic!("`{class}` has a laid-out box"))
    };
    let (app_x, app_y, app_w, _) = rect("app");
    assert_eq!((app_x, app_y, app_w), (0.0, 0.0, 1_100.0));
    let (side_x, _, side_w, _) = rect("side");
    assert_eq!(
        (side_x, side_w),
        (0.0, isometry_views::PANEL_W),
        "the panel strip is exactly PANEL_W wide, which is what sizes the board viewport"
    );
    let (pane_x, _, pane_w, _) = rect("pane");
    assert_eq!(
        (pane_x, pane_w),
        (isometry_views::PANEL_W, 1_100.0 - isometry_views::PANEL_W),
        "the pane takes the rest, so pane-local coordinates start at the panel's edge"
    );
    let (_, _, tile_w, tile_h) = rect("tile");
    let geo = &harness.state().geo;
    assert_eq!((tile_w, tile_h), (geo.tile_w, geo.tile_h));
}

// ---------- M3: the text lanes, through the host's own key path ----------

/// Typing a whisper reaches the field, and Enter sends what the field holds.
///
/// The whole seam is under test: `w` opens the lane through `key_intercept`,
/// the rebuilt field asks for the caret, `focused_text` recognises it by its
/// wrapper class, the host routes the letters into the `TextInput`, and Enter
/// comes back out as an app command. Nothing between a key and the draft is
/// isometry's any more — the point of the lane's migration.
#[test]
fn typing_a_whisper_reaches_the_field_and_enter_sends_it() {
    let mut harness = board();
    harness.update(|ui| ui.whisper_target = Some("alice".to_owned()));
    harness.key_char("w");
    assert!(harness.state().composing, "`w` opens the composer");
    assert!(
        hooks::focused_text(harness.runner()).is_some(),
        "opening the lane put the caret in its field"
    );
    assert_focused_field_is_drawn(&harness, "the whisper composer");

    for c in ["h", "e", "y"] {
        harness.key_char(c);
    }
    assert_eq!(
        harness.state().whisper_draft.text(),
        "hey",
        "the letters went to the field, not to a verb"
    );

    harness.press_key(&KeyPress::named(NamedKey::Enter));
    assert!(!harness.state().composing);
    assert_eq!(harness.state().messages, vec!["to alice: hey".to_owned()]);
    assert_eq!(
        harness.state().whisper_outbox,
        vec![("alice".to_owned(), "hey".to_owned())]
    );
}

/// Typing in the compendium filters its index.
///
/// The receipt is the *rendered* index, not the query string: what the filter
/// is for is fewer rows, and a query that reached the state without reaching
/// the grid would pass a weaker test.
#[test]
fn typing_in_the_compendium_filters_its_index() {
    let mut harness = board();
    harness.update(|ui| {
        ui.bestiary = crate::catalog::bestiary_of();
        ui.open_compendium();
    });
    harness.relayout();

    let query = "gob";
    let expected = harness
        .state()
        .bestiary
        .iter()
        .filter(|m| m.name.to_lowercase().contains(query))
        .count();
    assert!(expected > 0, "the SRD bestiary stands something goblin-ish");
    let before = index_rows(&harness);
    assert!(before > expected, "the unfiltered index is longer");
    assert_focused_field_is_drawn(&harness, "the compendium filter");

    for c in ["g", "o", "b"] {
        harness.key_char(c);
    }
    assert_eq!(harness.state().compendium_search.text(), query);
    assert_eq!(
        index_rows(&harness),
        expected,
        "the index shows exactly the matches"
    );
}

/// How many entry rows the compendium index is currently drawing.
fn index_rows(harness: &BoardHarness) -> usize {
    harness.with_dom(|dom| genet_probe::matching(dom, &Selector::class("compendium-link")).len())
}

/// The lane the caret is in has a box on screen.
///
/// A field can route keys perfectly and still be invisible — a caret in a
/// zero-width box types into nothing anyone can read. The headed captures do
/// not open either of these two lanes, so this is where the geometry is
/// checked at all.
fn assert_focused_field_is_drawn(harness: &BoardHarness, lane: &str) {
    let node = hooks::focused_text(harness.runner())
        .unwrap_or_else(|| panic!("{lane} holds the caret"))
        .node;
    let (_, _, width, height) = harness
        .painted_rect(node)
        .unwrap_or_else(|| panic!("{lane}'s field has a laid-out box"));
    assert!(
        width > 0.0 && height > 0.0,
        "{lane}'s field is drawn, not collapsed: {width}x{height}"
    );
}

// ---------- M3 follow-ons: menu dismissal, and drags through overlays ----------

/// A press outside the open token menu closes it — including a press on the
/// side panel, which is the regression the migration left behind when the old
/// host's left-click-off branch went away with `input.rs`.
///
/// The dismissal is the catalog `overlay_surface`'s transparent layer, sized
/// to the whole window, which is why the menu hangs off `.app` rather than
/// `.pane` (the pane clips). Escape is the other half, and is
/// `key_intercept`'s.
#[test]
fn a_press_outside_the_token_menu_closes_it() {
    let mut harness = board();
    let knight = harness
        .state()
        .map
        .token(TokenId(1))
        .expect("the demo skirmish stands a knight")
        .at;
    let (x, y) = tile_point(&harness, knight);

    // A press on the side panel: the half that used to work and stopped.
    harness.right_click_at(x, y);
    assert!(harness.state().context_menu.is_some());
    let (panel_x, panel_y) = harness
        .resolve(&Selector::class("side-title"))
        .expect("the panel strip has a title");
    assert!(
        panel_x < isometry_views::PANEL_W,
        "that point really is on the panel, not the pane"
    );
    harness.press_at(panel_x, panel_y);
    harness.relayout();
    assert!(
        harness.state().context_menu.is_none(),
        "a press on the side panel dismisses the menu"
    );

    // And Escape, with nothing else armed to claim it.
    harness.right_click_at(x, y);
    assert!(harness.state().context_menu.is_some());
    harness.press_key(&KeyPress::named(NamedKey::Escape));
    assert!(
        harness.state().context_menu.is_none(),
        "Escape dismisses the menu"
    );
}

/// A press inside an open overlay never reaches the board.
///
/// The host routes a pointer down to the *nearest* `on_pointer` ancestor of
/// the hit element, so the no-op handler on the overlay panel's root is the
/// whole mechanism: `.pane`'s handler, and with it `board_press` and the paint
/// drag it seeds, is never called. The closed-overlay half is the positive
/// control — without it this would pass just as well if the point missed the
/// board entirely.
#[test]
fn a_press_inside_an_overlay_does_not_reach_the_board() {
    let mut harness = board();
    harness.update(|ui| {
        ui.mode = isometry_views::EditMode::PaintGround;
        ui.open_compendium();
    });
    harness.relayout();
    let (x, y) = harness
        .resolve(&Selector::class("compendium"))
        .expect("the compendium panel has a laid-out box");
    let pane = (x - isometry_views::PANEL_W, y);
    assert!(
        harness.state().tile_at_pane(pane).is_some(),
        "the point sits over the board as well as over the panel"
    );

    harness.press_at(x, y);
    assert!(
        harness.state().drag_tile.is_none(),
        "the overlay took the press; `board_press` never ran"
    );
    harness.release_at(x, y);

    // Positive control: with the overlay closed, the same point does reach the
    // board and seeds the drag.
    harness.update(|ui| ui.close_compendium());
    harness.relayout();
    harness.press_at(x, y);
    assert_eq!(
        harness.state().drag_tile,
        harness.state().tile_at_pane(pane),
        "with nothing over it, the press paints the tile beneath"
    );
}

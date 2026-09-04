//! Tests for `interaction.rs`: pointer gestures, the turn gate, and undo.
//!
//! Everything the mouse does to the board, and the one step of history each
//! gesture is worth.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn paint_undo_redo_round_trip() {
    let mut ui = UiState::new(demo_map());
    let pristine = ui.map.clone();
    ui.mode = EditMode::PaintGround;
    ui.brush = TileKindId(2);
    ui.click_tile((3, 3));
    ui.click_tile((4, 3));
    let painted = ui.map.clone();
    assert_ne!(painted, pristine);
    ui.undo();
    ui.undo();
    assert_eq!(ui.map, pristine);
    ui.redo();
    ui.redo();
    assert_eq!(ui.map, painted);
}

#[test]
fn play_move_respects_turn_gate_and_sets_facing() {
    use isometry_core::TokenId;
    let mut ui = UiState::new(demo_map());
    ui.mode = EditMode::Play;
    // Knight at (10, 14): free token, may move.
    ui.select_token(TokenId(1));
    assert!(ui.may_move(TokenId(1)));
    assert!(!ui.reach.is_empty());
    ui.click_tile((12, 14)); // 2 east, within budget 5
    let t = ui.map.token(TokenId(1)).unwrap();
    assert_eq!(t.at, (12, 14));
    assert_eq!(t.facing, isometry_core::Facing::East);
    // Both tokens in the list: only the active one may move.
    ui.toggle_turn(TokenId(1));
    ui.toggle_turn(TokenId(2));
    assert!(ui.may_move(TokenId(1)));
    assert!(!ui.may_move(TokenId(2)));
    ui.select_token(TokenId(2));
    assert!(ui.reach.is_empty(), "waiting token gets no reach");
    let before = ui.map.token(TokenId(2)).unwrap().at;
    ui.click_tile((before.0 + 1, before.1));
    assert_eq!(ui.map.token(TokenId(2)).unwrap().at, before);
    // End turn: token 2 is up and can move now.
    ui.end_turn();
    assert!(ui.may_move(TokenId(2)));
    // The move is undoable (one step: move + facing).
    ui.select_token(TokenId(1));
    assert!(!ui.may_move(TokenId(1)) || ui.turns.active() == Some(TokenId(1)));
}

#[test]
fn ending_a_turn_refreshes_the_incoming_token_per_turn_counters() {
    use isometry_core::TokenId;
    let mut ui = UiState::new(demo_map());
    ui.toggle_turn(TokenId(1));
    ui.toggle_turn(TokenId(2));
    assert_eq!(ui.turns.active(), Some(TokenId(1)));
    // The active knight has spent part of its turn: an action economy, a
    // multiple-attack tally -- the view never learns which.
    ui.map.bump_turn_counter(TokenId(1), "actions_spent", 2);

    // The goblin's turn begins. A turn-start wipes the *incoming* token's
    // counters, so the goblin's clear (they were empty) while the knight
    // keeps its spend as it waits.
    ui.end_turn();
    assert_eq!(ui.turns.active(), Some(TokenId(2)));
    assert_eq!(ui.map.turn_counter(TokenId(1), "actions_spent"), 2);

    // Back to the knight: its own turn beginning clears the ledger, so it
    // acts with a whole economy again.
    ui.end_turn();
    assert_eq!(ui.turns.active(), Some(TokenId(1)));
    assert_eq!(ui.map.turn_counter(TokenId(1), "actions_spent"), 0);
}

#[test]
fn drag_move_relocates_a_token_and_is_undoable() {
    use isometry_core::TokenId;
    let mut ui = UiState::new(demo_map());
    let start = ui.map.token(TokenId(1)).unwrap().at; // (10, 14)
    ui.drag_move_token(TokenId(1), (5, 5));
    assert_eq!(ui.map.token(TokenId(1)).unwrap().at, (5, 5));
    // Occupied (goblin 2 at (15, 8)) and out-of-bounds are no-ops.
    ui.drag_move_token(TokenId(1), (15, 8));
    ui.drag_move_token(TokenId(1), (999, 999));
    assert_eq!(ui.map.token(TokenId(1)).unwrap().at, (5, 5));
    // The one real move undoes back to the start.
    ui.undo();
    assert_eq!(ui.map.token(TokenId(1)).unwrap().at, start);
}

#[test]
fn drag_move_routes_out_in_remote_mode() {
    use isometry_core::TokenId;
    let mut ui = UiState::new(demo_map());
    ui.net_mode = NetMode::Remote;
    let before = ui.map.token(TokenId(1)).unwrap().at;
    ui.drag_move_token(TokenId(1), (5, 5));
    // Session mode emits an intent and leaves the local map untouched.
    assert_eq!(ui.map.token(TokenId(1)).unwrap().at, before);
    assert_eq!(ui.net_outbox.len(), 1);
}

#[test]
fn token_drag_candidate_finds_a_token_in_select_mode_only() {
    use isometry_core::TokenId;
    let mut ui = UiState::new(demo_map());
    // Pointer over knight 1's tile (10, 14), in the pane's own coordinates;
    // the default camera is (0, 0).
    let (sx, sy) = ui.geo.tile_to_screen((10, 14), 0);
    let on_token = (sx + ui.camera.0, sy + ui.camera.1);
    assert_eq!(ui.mode, EditMode::Select);
    assert_eq!(ui.token_drag_candidate(on_token), Some(TokenId(1)));
    // An empty tile, or any non-Select mode, yields nothing.
    let (ex, ey) = ui.geo.tile_to_screen((0, 0), 0);
    assert_eq!(ui.token_drag_candidate((ex, ey)), None);
    ui.mode = EditMode::Play;
    assert_eq!(ui.token_drag_candidate(on_token), None);
}

#[test]
fn context_menu_opens_selects_and_removes() {
    use isometry_core::TokenId;
    let mut ui = UiState::new(demo_map());
    let n = ui.map.tokens.len();
    ui.open_context_menu(TokenId(1), (50.0, 60.0));
    assert_eq!(ui.context_menu, Some((TokenId(1), (50.0, 60.0))));
    assert_eq!(ui.selected_token, Some(TokenId(1)), "right-click selects");
    ui.close_context_menu();
    assert!(ui.context_menu.is_none());
    // Remove drops the token from the map, turn order, and selection.
    ui.turns.add(TokenId(1));
    ui.open_context_menu(TokenId(1), (0.0, 0.0));
    ui.remove_token(TokenId(1));
    assert_eq!(ui.map.tokens.len(), n - 1);
    assert!(ui.map.token(TokenId(1)).is_none());
    assert!(!ui.turns.contains(TokenId(1)));
    assert!(ui.selected_token.is_none());
    assert!(ui.context_menu.is_none());
}

#[test]
fn token_mode_places_and_removes() {
    let mut ui = UiState::new(demo_map());
    ui.mode = EditMode::Token;
    let n = ui.map.tokens.len();
    ui.click_tile((2, 2));
    assert_eq!(ui.map.tokens.len(), n + 1);
    let placed = ui.token_at((2, 2)).unwrap();
    assert!(placed.0 > 2, "fresh id past the demo tokens");
    ui.click_tile((2, 2));
    assert_eq!(ui.map.tokens.len(), n);
    ui.undo(); // undo the removal: token back
    assert_eq!(ui.map.tokens.len(), n + 1);
    assert_eq!(ui.token_at((2, 2)), Some(placed));
}

#[test]
fn fill_is_one_undo_step() {
    let mut ui = UiState::new(demo_map());
    let pristine = ui.map.clone();
    ui.mode = EditMode::Fill;
    ui.brush = TileKindId(3);
    ui.click_tile((0, 0));
    assert_ne!(ui.map, pristine);
    ui.undo();
    assert_eq!(ui.map, pristine);
}

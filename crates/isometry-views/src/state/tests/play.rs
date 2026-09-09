//! Tests for `play.rs`: spawning, and where a spawn lands.
//!
//! In a session a spawn is the authority's to make, and on a narrow map the
//! fallback tile still has to be on the board.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;
use isometry_campaign::{EncounterAnchor, MapPoint, MapScale, MapTransition};

#[test]
fn spawn_in_a_session_routes_through_the_authority_not_the_local_map() {
    // The bug the adversarial review caught: a hosted DM's `>spawn` mutated
    // the local map directly, so the token never replicated and was wiped by
    // the next snapshot mirror (leaving an orphan sheet). It must emit an
    // authoritative TokenPlaced instead.
    let mut ui = UiState::new(demo_map());
    ui.net_mode = NetMode::Remote;
    ui.bestiary = vec![MonsterRow {
        key: "goblin".to_owned(),
        name: "Goblin".to_owned(),
        cr: 0.25,
        cr_label: "1/4".to_owned(),
        kind: "humanoid".to_owned(),
        size: "small".to_owned(),
        alignment: "ne".to_owned(),
        hp: 7,
        hit_dice: "2d6".to_owned(),
        ac: 15,
        speed_ft: 30,
        xp: 50,
        abilities: [8, 14, 10, 10, 8, 8],
        actions: Vec::new(),
        sprite: "goblin".to_owned(),
    }];
    let before = ui.map.tokens.len();

    ui.spawn_query("goblin");

    // The local map is untouched; the placement is queued for the authority.
    assert_eq!(
        ui.map.tokens.len(),
        before,
        "no local mutation in a session"
    );
    let placed = ui
        .net_outbox
        .iter()
        .any(|e| matches!(e, GameEvent::Map(SessionEvent::TokenPlaced(_))));
    assert!(placed, "the spawn must replicate as an authoritative event");
    // And the stat-block bind is queued for the same id.
    assert!(ui.spawn_sheet_request.is_some());
}

#[test]
fn a_spawn_tile_stays_on_the_board_on_a_narrow_map() {
    // free_spawn_tile's outward scan could walk off a map narrower than its
    // stride, yielding an off-board tile that fails placement. It must clamp.
    let mut ui = UiState::new(MapDocument::new("slot", 3, 3));
    let grass = ui.map.intern_tile_kind("grass");
    for row in 0..3 {
        for col in 0..3 {
            ui.map.ground.set(col, row, grass);
        }
    }
    // Pack the whole 3x3 but one cell, forcing the scan to the survivor.
    for row in 0..3 {
        for col in 0..3 {
            if (col, row) != (2, 2) {
                ui.map.tokens.push(Token {
                    id: TokenId(100 + (row * 3 + col) as u32),
                    at: (col, row),
                    facing: Facing::South,
                    sprite: "goblin".to_owned(),
                    owner: None,
                });
            }
        }
    }
    let at = ui.free_spawn_tile();
    assert!(
        ui.map.ground.in_bounds(at.0, at.1),
        "spawn tile {at:?} is off the 3x3 board"
    );
    assert_eq!(at, (2, 2), "the one free in-bounds cell");
}

#[test]
fn authored_sites_and_doors_have_user_facing_labels() {
    let mut ui = UiState::new(demo_map());
    let mut destination = MapDocument::new("Bellwood Reach", 3, 3);
    let grass = destination.intern_tile_kind("grass");
    destination.ground.set(0, 0, grass);
    ui.active_map = Some("tower".to_owned());
    ui.campaign_maps.insert(
        "tower".to_owned(),
        CampaignMap {
            id: "tower".to_owned(),
            scale: MapScale::Local,
            document: ui.map.clone(),
            spawn_zones: Vec::new(),
            transitions: vec![MapTransition {
                id: "forest-door".to_owned(),
                at: MapPoint { col: 1, row: 1 },
                target_map: "forest".to_owned(),
                target_entry: None,
            }],
            encounter_anchors: vec![EncounterAnchor {
                id: "wolf-trail".to_owned(),
                at: MapPoint { col: 2, row: 2 },
                tags: vec!["forest".to_owned(), "trail".to_owned()],
            }],
        },
    );
    ui.campaign_maps.insert(
        "forest".to_owned(),
        CampaignMap {
            id: "forest".to_owned(),
            scale: MapScale::Region,
            document: destination,
            spawn_zones: Vec::new(),
            transitions: Vec::new(),
            encounter_anchors: Vec::new(),
        },
    );

    assert_eq!(
        ui.authored_site_labels_at((1, 1)),
        Some((
            "Travel to Bellwood Reach".to_owned(),
            "Travel to Bellwood Reach".to_owned(),
            false,
        ))
    );
    assert_eq!(
        ui.authored_site_labels_at((2, 2)),
        Some((
            "Encounter site: Wolf trail (forest, trail)".to_owned(),
            "Encounter site: wolf-trail (forest, trail)".to_owned(),
            true,
        ))
    );
    ui.hover_tile_enter(Some((2, 2)));
    assert_eq!(ui.status, "Encounter site: Wolf trail (forest, trail)");

    let before_map = ui.map.clone();
    let before_events = ui.net_outbox.clone();
    ui.click_tile((2, 2));
    assert_eq!(ui.map, before_map, "site selection does not mutate the map");
    assert_eq!(
        ui.net_outbox, before_events,
        "site selection emits no event"
    );

    let mut hidden = ui;
    hidden.viewer = Some("player".to_owned());
    hidden.visible.clear();
    hidden.explored.clear();
    hidden.status = "keep this status".to_owned();
    hidden.hover_tile_enter(Some((2, 2)));
    assert_eq!(hidden.status, "keep this status");
}

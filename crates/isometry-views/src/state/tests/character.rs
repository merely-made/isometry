//! First-character authoring receipts.

use super::*;

#[test]
fn character_creation_waits_for_the_host_to_make_token_and_default_sheet_together() {
    let mut ui = UiState::new(demo_map());
    ui.character_name = cambium::TextInput::new("Mira");
    ui.character_owner = cambium::TextInput::new("mira-player");
    ui.character_sprite = "goblin".to_owned();
    let before = ui.map.tokens.len();

    ui.create_character();

    assert_eq!(ui.map.tokens.len(), before, "no sheetless local token");
    assert_eq!(
        ui.character_create_request,
        Some(CharacterCreateRequest {
            token: Token {
                id: ui.next_token_id(),
                at: ui.free_spawn_tile(),
                facing: Facing::South,
                sprite: "goblin".to_owned(),
                owner: Some("mira-player".to_owned()),
            },
            name: "Mira".to_owned(),
        })
    );
}

#[test]
fn remote_non_authoring_refuses_character_creation() {
    let mut ui = UiState::new(demo_map());
    ui.net_mode = NetMode::Remote;
    ui.can_edit_inventory = false;
    let before = ui.map.tokens.len();

    ui.open_character();
    ui.create_character();

    assert!(!ui.character_open);
    assert_eq!(ui.map.tokens.len(), before);
    assert!(ui.net_outbox.is_empty());
    assert!(ui.character_create_request.is_none());
    assert_eq!(ui.status, "character creation requires the host");
}

#[test]
fn remote_authoring_only_queues_the_host_validated_request() {
    let mut ui = UiState::new(demo_map());
    ui.net_mode = NetMode::Remote;
    ui.character_name = cambium::TextInput::new("Mira");
    let before = ui.map.tokens.len();

    ui.create_character();

    assert_eq!(ui.map.tokens.len(), before, "remote state waits for echo");
    assert!(
        ui.net_outbox.is_empty(),
        "the host validates defaults first"
    );
    assert!(ui.character_create_request.is_some());
}

#[test]
fn a_full_board_refuses_creation_without_leaving_a_request() {
    let mut ui = UiState::new(MapDocument::new("full", 1, 1));
    let grass = ui.map.intern_tile_kind("grass");
    ui.map.ground.set(0, 0, grass);
    ui.character_name = cambium::TextInput::new("Mira");
    ui.map.tokens.push(Token {
        id: TokenId(1),
        at: (0, 0),
        facing: Facing::South,
        sprite: "hero".to_owned(),
        owner: None,
    });

    ui.create_character();

    assert_eq!(ui.map.tokens.len(), 1);
    assert!(ui.character_create_request.is_none());
    assert_eq!(ui.status, "no free tile for a character");
}

#[test]
fn water_is_not_a_character_spawn_tile() {
    let mut ui = UiState::new(MapDocument::new("stream", 2, 1));
    let water = ui.map.intern_tile_kind("water");
    let grass = ui.map.intern_tile_kind("grass");
    ui.map.ground.set(0, 0, water);
    ui.map.ground.set(1, 0, grass);

    assert!(!ui.character_spawn_tile_available((0, 0)));
    assert!(ui.character_spawn_tile_available((1, 0)));
    assert_eq!(ui.available_spawn_tile(), Some((1, 0)));
}

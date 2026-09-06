//! Tests for `session.rs`: net routing, dice, fog, and the snapshot.
//!
//! In remote mode nothing mutates locally — an intent goes out and the
//! authority's snapshot comes back — and fog is recomputed from whatever that
//! snapshot said.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn remote_mode_routes_moves_as_events_not_local_mutation() {
    use isometry_core::TokenId;
    let mut ui = UiState::new(demo_map());
    ui.net_mode = NetMode::Remote;
    ui.mode = EditMode::Play;
    let before = ui.map.token(TokenId(1)).unwrap().at;
    ui.select_token(TokenId(1));
    // A move in a session emits intents and leaves the local map
    // untouched (the host authority echoes the real move back).
    ui.click_tile((before.0 + 1, before.1));
    assert_eq!(ui.map.token(TokenId(1)).unwrap().at, before);
    assert_eq!(ui.net_outbox.len(), 2, "move + facing emitted");
    // End turn and toggle also route out, not local.
    ui.net_outbox.clear();
    ui.end_turn();
    ui.toggle_turn(TokenId(2));
    assert_eq!(ui.net_outbox.len(), 2);
    // Editing is inert in a session.
    ui.net_outbox.clear();
    ui.mode = EditMode::PaintGround;
    ui.click_tile((0, 0));
    assert!(ui.net_outbox.is_empty());
    assert!(ui.can_undo() == false, "no local edit happened");

    // A snapshot mirrors the authoritative state in.
    let mut snap_map = ui.map.clone();
    snap_map.token_mut(TokenId(1)).unwrap().at = (before.0 + 1, before.1);
    let inventories = std::collections::BTreeMap::from([(TokenId(1), Inventory::default())]);
    let snap = GameSnapshot {
        map: snap_map,
        turns: ui.turns.clone(),
        roll_log: Vec::new(),
        journal: Vec::new(),
        inventories: inventories.clone(),
        generations: Vec::new(),
        maps: Default::default(),
        active_map: None,
        world: Default::default(),
        clocks: Default::default(),

        party_cap: isonetry::default_party_cap(),
        last_beats: Vec::new(),
        beat_seq: 0,
        applied_actions: Default::default(),
    };
    ui.apply_snapshot(snap);
    assert_eq!(
        ui.map.token(TokenId(1)).unwrap().at,
        (before.0 + 1, before.1)
    );
    assert_eq!(ui.inventories, inventories);
}

#[test]
fn local_roll_appends_to_log_and_is_reproducible() {
    let mut ui = UiState::new(demo_map());
    ui.reseed(99);
    ui.roll_dice("1d20+3");
    assert_eq!(ui.roll_log.len(), 1);
    let rec = &ui.roll_log[0];
    assert_eq!(rec.by, "dm");
    assert_eq!(rec.dice.len(), 1);
    assert_eq!(rec.total, rec.dice[0] as i32 + 3);
    // A bad expression sets a status and adds nothing.
    ui.roll_dice("nonsense");
    assert_eq!(ui.roll_log.len(), 1);
    assert!(ui.status.starts_with("bad roll"));
}

#[test]
fn roll_initiative_individual_and_side() {
    let mut ui = UiState::new(demo_map());
    ui.reseed(5);
    let ids: Vec<_> = ui.map.tokens.iter().map(|t| t.id).collect();
    for id in &ids {
        ui.turns.add(*id);
    }
    // Individual: one roll per token, order preserved as a set.
    ui.roll_initiative();
    assert_eq!(ui.turns.entries().len(), ids.len());
    assert_eq!(ui.roll_log.len(), ids.len());
    let mut sorted = ui.turns.entries().to_vec();
    sorted.sort();
    let mut expect = ids.clone();
    expect.sort();
    assert_eq!(sorted, expect, "same tokens, reordered");

    // Side-based: tokens grouped by owner, so exactly one boundary
    // between the two sides (A knights, B goblins).
    ui.initiative_mode = InitiativeMode::SideBased;
    ui.roll_initiative();
    let owners: Vec<String> = ui
        .turns
        .entries()
        .iter()
        .map(|id| ui.map.token(*id).unwrap().owner.clone().unwrap())
        .collect();
    let boundaries = owners.windows(2).filter(|w| w[0] != w[1]).count();
    assert_eq!(boundaries, 1, "two sides act in blocks");
}

#[test]
fn remote_roll_routes_out_not_local() {
    let mut ui = UiState::new(demo_map());
    ui.net_mode = NetMode::Remote;
    ui.viewer = Some("A".to_owned());
    ui.roll_dice("2d6");
    assert!(
        ui.roll_log.is_empty(),
        "remote rolls come back via snapshot"
    );
    assert_eq!(ui.net_outbox.len(), 1);
}

#[test]
fn fog_hides_out_of_sight_and_remembers_explored() {
    let mut ui = UiState::new(demo_map());
    // Knights are owner "A" near (10,14)/(9,15); goblins "B" near the
    // northeast hill. As player A, the goblins are out of sight.
    ui.viewer = Some("A".to_owned());
    ui.recompute_fog();
    let knight = ui.map.token(TokenId(1)).unwrap().at;
    let goblin = ui.map.token(TokenId(2)).unwrap().at;
    assert_eq!(ui.fog_level(knight), FogLevel::Clear);
    assert_eq!(ui.fog_level(goblin), FogLevel::Hidden);
    assert!(ui.token_visible(ui.map.token(TokenId(1)).unwrap()));
    assert!(!ui.token_visible(ui.map.token(TokenId(2)).unwrap()));

    // Explored memory: a tile seen once stays remembered (Dim) after
    // the token that saw it moves away.
    let seen_far = ui
        .visible
        .iter()
        .copied()
        .find(|&t| t != knight && (t.0 - knight.0).abs() + (t.1 - knight.1).abs() >= 3)
        .expect("some far-but-visible tile");
    // Move the knight to the opposite side so seen_far leaves sight.
    ui.mode = EditMode::Play;
    ui.apply_step(vec![SessionEvent::TokenMoved {
        id: TokenId(1),
        to: (0, 0),
    }]);
    ui.apply_step(vec![SessionEvent::TokenMoved {
        id: TokenId(3),
        to: (1, 0),
    }]);
    assert_eq!(
        ui.fog_level(seen_far),
        FogLevel::Dim,
        "a tile seen earlier is remembered, not black"
    );

    // Omniscient clears fog entirely.
    ui.viewer = None;
    ui.recompute_fog();
    assert_eq!(ui.fog_level(goblin), FogLevel::Clear);
    assert!(!ui.fog_active());
}

#[test]
fn a_viewer_commands_a_faction_only_once_granted_its_channel() {
    let mut ui = UiState::new(demo_map());
    ui.viewer = Some("B".to_owned());

    // Ungranted, a faction's token is not B's to command.
    assert!(!ui.commands(Some("tide")));
    // Grant B the Tide Court's channel (as the replicated world would carry).
    ui.world
        .faction_control
        .insert("tide".to_owned(), "B".to_owned());
    assert!(
        ui.commands(Some("tide")),
        "the grant extends command to the faction"
    );

    // Direct ownership is unchanged, and a stranger's token stays off-limits.
    assert!(ui.commands(Some("B")));
    assert!(!ui.commands(Some("A")));
    assert!(!ui.commands(Some("ash")), "an unrelated faction is not B's");
    assert!(!ui.commands(None), "a DM token is nobody's to a player");
}

#[test]
fn apply_snapshot_mirrors_the_clock_and_the_cap() {
    // A joined client mirrors the host snapshot into its UiState. Dropping
    // clocks (a C3 omission the C5 review caught) shows the wrong split-party
    // time on clients; dropping party_cap desyncs the limit.
    let mut ui = UiState::new(demo_map());
    assert_eq!(ui.party_cap, 4);
    let mut snap = GameSnapshot {
        map: demo_map(),
        turns: TurnList::new(),
        roll_log: Vec::new(),
        journal: Vec::new(),
        inventories: Default::default(),
        generations: Vec::new(),
        maps: Default::default(),
        active_map: None,
        world: Default::default(),
        clocks: Default::default(),
        party_cap: 2,
        last_beats: Vec::new(),
        beat_seq: 0,
        applied_actions: Default::default(),
    };
    snap.clocks.insert("field".to_owned(), 7);
    ui.apply_snapshot(snap);
    assert_eq!(ui.party_cap, 2, "the cap must mirror");
    assert_eq!(ui.clocks.get("field"), Some(&7), "the clock must mirror");
}

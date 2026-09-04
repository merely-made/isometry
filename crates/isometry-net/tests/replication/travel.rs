//! Map-to-map crossings: doors, arrival, and the clocks on either side.
//!
//! C2 end to end over the sim. The host rules a crossing and broadcasts a
//! payload that names every consequence. `two_map_snapshot` sits with the
//! other fixtures because the road encounter in `world.rs` needs it too.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

/// The host rules the crossing `token` is standing in, and commits the verdict.
/// The app's door sweep in one line: resolve once, on the authority, and
/// broadcast a payload that names every consequence.
fn cross(sim: &mut Sim, token: TokenId) {
    let ruled = isometry_net::resolve_transition(sim.host.state(), token, next_request())
        .expect("the traveler is standing on a door");
    sim.host_event(GameEvent::TransitionResolved(ruled));
}

#[test]
fn walking_through_a_door_crosses_maps_and_the_board_follows_the_party() {
    // The goblin is DM furniture (owner: None), so the knight is the last
    // player out and the board follows it through the door.
    let mut base = two_map_snapshot();
    base.map.tokens[1].owner = None; // goblin: DM furniture
    if let Some(field) = base.maps.get_mut("field") {
        field.document = base.map.clone();
    }
    let mut sim = Sim::new(HostSession::new(base));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(1),
        sheet: sheet("Knight", 12, 16),
    });
    sim.host_event(GameEvent::ConditionSet {
        token: TokenId(1),
        condition: "prone".to_owned(),
        value: 1,
        mobility: Some((2, 6)),
    });

    // Walk onto the gate, then through it.
    sim.host_event(GameEvent::Map(SessionEvent::TokenMoved {
        id: TokenId(1),
        to: (3, 3),
    }));
    cross(&mut sim, TokenId(1));

    let host = sim.host.state();
    // The board followed the last player out.
    assert_eq!(host.active_map.as_deref(), Some("hut"));
    // The knight arrived at the hut's entry door, carrying everything it is:
    // sheet, condition, and the condition's numbers.
    let knight = host.map.token(TokenId(1)).expect("knight in the hut");
    assert_eq!(knight.at, (1, 1), "landed at the named entry");
    assert_eq!(host.map.sheet(TokenId(1)).and_then(|s| s.int("hp_current")), Some(12));
    assert!(host.map.has_condition(TokenId(1), "prone"), "still prone: travel is not a cure");
    assert_eq!(host.map.effective_mobility(TokenId(1), (5, 6)), (2, 6));
    // And left the field entirely (the stored copy, since field is no longer
    // the active board).
    let field = &host.maps["field"].document;
    assert!(field.token(TokenId(1)).is_none());
    assert!(field.sheets.get(&TokenId(1)).is_none());
    // The goblin furniture stayed home.
    assert!(field.token(TokenId(2)).is_some());
    assert_converged(&sim);
}

#[test]
fn arriving_where_your_id_is_taken_mints_a_new_one_and_carries_the_inventory() {
    let mut base = two_map_snapshot();
    base.map.tokens[1].owner = None;
    // The hut already has a resident with the knight's id.
    if let Some(hut) = base.maps.get_mut("hut") {
        hut.document.tokens.push(Token {
            id: TokenId(1),
            at: (4, 4),
            facing: Facing::South,
            sprite: "goblin".to_owned(),
            owner: None,
        });
    }
    if let Some(field) = base.maps.get_mut("field") {
        field.document = base.map.clone();
    }
    let mut sim = Sim::new(HostSession::new(base));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::InventorySet {
        token: TokenId(1),
        inventory: sword_inventory(),
    });
    sim.host_event(GameEvent::Map(SessionEvent::TokenMoved {
        id: TokenId(1),
        to: (3, 3),
    }));
    cross(&mut sim, TokenId(1));

    let host = sim.host.state();
    assert_eq!(host.active_map.as_deref(), Some("hut"));
    // The resident kept its id; the traveler was minted a fresh one, and the
    // inventory followed the new id (they key globally).
    let arrivals: Vec<_> = host
        .map
        .tokens
        .iter()
        .filter(|t| t.sprite == "knight")
        .collect();
    assert_eq!(arrivals.len(), 1);
    let new_id = arrivals[0].id;
    assert_ne!(new_id, TokenId(1));
    assert!(host.inventories.contains_key(&new_id), "the sword crossed too");
    assert!(!host.inventories.contains_key(&TokenId(1)));
    assert_converged(&sim);
}

#[test]
fn split_party_time_drifts_freely_and_travel_reconciles_it() {
    // The knight fights in the field while the hut sits quiet: the two
    // locations' clocks drift apart, and nothing needs to agree until someone
    // crosses. Simultaneity is presentation; the door is where timelines meet.
    let mut base = two_map_snapshot();
    base.map.tokens[1].owner = None;
    if let Some(field) = base.maps.get_mut("field") {
        field.document = base.map.clone();
    }
    let mut sim = Sim::new(HostSession::new(base));
    sim.connect(PeerId(10));

    // Three rounds of fighting in the field: knight and goblin trade turns.
    sim.host_event(GameEvent::TurnAdd(TokenId(1)));
    sim.host_event(GameEvent::TurnAdd(TokenId(2)));
    for _ in 0..6 {
        sim.host_event(GameEvent::TurnAdvance);
    }
    // And the DM declares a rest on top.
    sim.host_event(GameEvent::TimeAdvanced { ticks: 4 });

    let clock = |s: &GameSnapshot, id: &str| s.clocks.get(id).copied().unwrap_or(0);
    assert_eq!(clock(sim.host.state(), "field"), 7, "3 rounds + 4 declared");
    assert_eq!(clock(sim.host.state(), "hut"), 0, "nobody home: no time passes");

    // The knight walks through the gate. Nobody arrives before they left: the
    // hut's clock catches up to the traveler's, on every peer.
    sim.host_event(GameEvent::Map(SessionEvent::TokenMoved {
        id: TokenId(1),
        to: (3, 3),
    }));
    cross(&mut sim, TokenId(1));
    assert_eq!(clock(sim.host.state(), "hut"), 7);
    assert_eq!(
        clock(sim.clients[&PeerId(10)].state().unwrap(), "hut"),
        7,
        "the reconciled clock is truth, so the client holds it too"
    );

    // A player does not declare hours passing.
    let seq = sim.host.seq();
    sim.client_intent(PeerId(10), GameEvent::TimeAdvanced { ticks: 99 });
    assert_eq!(sim.host.seq(), seq, "a client kept the clock");
    assert_converged(&sim);
}

#[test]
fn travel_off_a_door_is_refused_and_clients_cannot_rule_it() {
    let mut sim = Sim::new(HostSession::new(two_map_snapshot()));
    sim.connect(PeerId(10));
    let seq = sim.host.seq();

    // Not standing on a transition point: there is nothing to rule, so the
    // refusal now happens where the ruling would have, and no event is minted.
    let ruling = isometry_net::resolve_transition(sim.host.state(), TokenId(1), next_request());
    assert_eq!(ruling, Err(GameError::NotOnTransition(TokenId(1))));
    assert_eq!(sim.host.seq(), seq, "an off-door travel entered the log");

    // And a crossing is the host's verdict: a client walks through a door, it
    // does not pronounce where the door led. Forged whole, since a client that
    // cannot resolve one has nothing else to send.
    sim.client_intent(
        PeerId(10),
        GameEvent::TransitionResolved(TransitionResolved {
            request: next_request(),
            token: TokenId(1),
            from_map: "field".to_owned(),
            to_map: "hut".to_owned(),
            landing: (1, 1),
            arrival: TokenId(1),
            inventory_remaps: Vec::new(),
            destination_clock: 0,
            activated: Some("hut".to_owned()),
        }),
    );
    assert_eq!(sim.host.seq(), seq);
    assert_converged(&sim);
}

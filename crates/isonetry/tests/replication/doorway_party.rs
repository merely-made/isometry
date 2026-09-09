//! The bounded connection between local doorways and overmap party position.

use super::*;

fn place(id: &str, map: &str) -> WorldPlace {
    WorldPlace {
        id: id.to_owned(),
        name: id.to_owned(),
        tags: Vec::new(),
        map: Some(map.to_owned()),
        position: None,
    }
}

fn party_snapshot() -> GameSnapshot {
    let mut state = two_map_snapshot();
    // Token 2 is furniture, leaving token 1 as the final player departure.
    state.map.tokens[0].at = (3, 3);
    state.map.tokens[1].owner = None;
    state.maps.get_mut("field").unwrap().document = state.map.clone();
    state
        .world
        .places
        .insert("field-place".into(), place("field-place", "field"));
    state
        .world
        .places
        .insert("hut-place".into(), place("hut-place", "hut"));
    state
        .world
        .party_node
        .insert("trail-party".into(), "field-place".into());
    state
}

fn doorway_events(state: &GameSnapshot, request: RequestId) -> Vec<GameEvent> {
    isonetry::resolve_transition_for_party(state, TokenId(1), request, "trail-party")
        .expect("the knight is standing on a doorway")
}

#[test]
fn final_departure_moves_the_explicit_party_to_its_unique_target_place() {
    let state = party_snapshot();
    let events = doorway_events(&state, next_request());
    assert!(
        matches!(events.as_slice(), [GameEvent::TransitionResolved(_), GameEvent::World(WorldEvent::PartyMoved { party, node })] if party == "trail-party" && node == "hut-place")
    );
}

#[test]
fn a_split_party_crossing_stays_tactical_only() {
    let mut state = party_snapshot();
    state.map.tokens[1].owner = Some("B".into());
    state.maps.get_mut("field").unwrap().document = state.map.clone();

    let events = doorway_events(&state, next_request());
    assert!(
        matches!(events.as_slice(), [GameEvent::TransitionResolved(res)] if res.activated.is_none())
    );
}

#[test]
fn absent_or_ambiguous_target_place_stays_tactical_only() {
    let mut absent = party_snapshot();
    absent.world.places.remove("hut-place");
    assert!(matches!(
        doorway_events(&absent, next_request()).as_slice(),
        [GameEvent::TransitionResolved(_)]
    ));

    let mut ambiguous = party_snapshot();
    ambiguous
        .world
        .places
        .insert("other-hut".into(), place("other-hut", "hut"));
    assert!(matches!(
        doorway_events(&ambiguous, next_request()).as_slice(),
        [GameEvent::TransitionResolved(_)]
    ));
}

#[test]
fn an_unrelated_party_is_never_inferred_from_the_travelers_owner() {
    let mut state = party_snapshot();
    state
        .world
        .party_node
        .insert("trail-party".into(), "hut-place".into());

    assert!(matches!(
        doorway_events(&state, next_request()).as_slice(),
        [GameEvent::TransitionResolved(_)]
    ));
}

#[test]
fn applying_the_batch_discovers_the_destination_and_replay_is_empty() {
    let mut host = HostSession::new(party_snapshot());
    let mut client = ClientSession::new();
    let snapshot = host.on_connect(PeerId(23)).pop().unwrap().1;
    assert!(client.on_message(snapshot).is_empty());
    let request = next_request();
    let out = host
        .commit_transition_for_party(TokenId(1), request, "trail-party")
        .unwrap();

    assert_eq!(
        out.len(),
        2,
        "the verdict and party move committed together"
    );
    for (_, message) in out {
        let bytes = postcard::to_allocvec(&message).unwrap();
        let message: NetMessage = postcard::from_bytes(&bytes).unwrap();
        assert!(client.on_message(message).is_empty());
    }
    assert_eq!(
        host.state().world.party_at("trail-party"),
        Some("hut-place")
    );
    assert!(host.state().world.knows("trail-party", "hut-place"));
    assert_eq!(client.state(), Some(host.state()));
    assert_eq!(client.log_hash(), host.log_hash());
    host.local_event(GameEvent::World(WorldEvent::PartyMoved {
        party: "trail-party".into(),
        node: "field-place".into(),
    }));
    assert!(
        host.commit_transition_for_party(TokenId(1), request, "trail-party")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        host.state().world.party_at("trail-party"),
        Some("field-place")
    );
}

#[test]
fn invalid_doorway_request_keeps_host_state_and_sequence_unchanged() {
    let mut host = HostSession::new(party_snapshot());
    let before = host.state().clone();
    let seq = host.seq();

    assert!(
        host.commit_transition_for_party(TokenId(99), next_request(), "trail-party")
            .is_err()
    );
    assert_eq!(host.state(), &before);
    assert_eq!(host.seq(), seq);
}

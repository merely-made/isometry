//! Atomic character authoring receipts.

use super::*;

fn created() -> GameEvent {
    GameEvent::CharacterCreated {
        token: Token {
            id: TokenId(40),
            at: (3, 3),
            facing: Facing::South,
            sprite: "hero".to_owned(),
            owner: Some("Mira".to_owned()),
        },
        sheet: sheet("Mira", 12, 14),
    }
}

#[test]
fn character_creation_replays_as_one_complete_token_and_sheet() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));

    sim.host_event(created());

    for state in std::iter::once(sim.host.state()).chain(
        sim.clients
            .values()
            .map(|client| client.state().expect("connected client has a snapshot")),
    ) {
        let token = state.map.token(TokenId(40)).expect("character token");
        assert_eq!(token.at, (3, 3));
        assert_eq!(token.owner.as_deref(), Some("Mira"));
        assert_eq!(
            state.map.sheet(TokenId(40)).and_then(|s| s.text("name")),
            Some("Mira")
        );
    }
    assert_eq!(
        sim.host.history().len(),
        1,
        "one replay record owns both halves"
    );
    assert_converged(&sim);
}

#[test]
fn refused_character_placement_does_not_attach_an_orphan_sheet() {
    let mut state = snapshot();
    let before = state.clone();
    let mut event = created();
    let GameEvent::CharacterCreated { token, .. } = &mut event else {
        unreachable!()
    };
    token.at = (-1, 0);
    assert!(isonetry::apply_game(&mut state, &event).is_err());
    assert_eq!(state, before);
}

//! What a joined peer may declare, and what stays the DM's.
//!
//! The substrate document is not a free-for-all: a player commands its own
//! tokens and nothing else. These two are kept together because the gap they
//! close was one permissive catch-all, not one missing check.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

/// The substrate document is not a free-for-all. A joined peer may declare
/// things about the tokens it commands and nothing else: the board itself, the
/// initiative order, and the sheets every rule reads are the DM's.
///
/// This closes the gap C5 noted and left open ("a client's `TokenMoved` is not
/// ownership-gated on the host"). It was never one missing check: intents fell
/// through to a permissive catch-all, so every event nobody had explicitly
/// refused was accepted from anyone. `SheetSet` was the sharp end -- there is no
/// need to forge a 999-damage verdict, already refused, if you can set the
/// boss's hit points to zero directly.
#[test]
fn a_player_commands_its_own_tokens_and_may_not_edit_the_board() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "B"); // token 2 is B's goblin; token 1 is A's knight

    // Its own token moves and turns to face: an ordinary play move.
    sim.client_intent(PeerId(10), mv(2, (5, 6)));
    sim.client_intent(
        PeerId(10),
        GameEvent::Map(SessionEvent::TokenFaced {
            id: TokenId(2),
            facing: Facing::East,
        }),
    );
    let goblin = sim.host.state().map.token(TokenId(2)).unwrap();
    assert_eq!((goblin.at, goblin.facing), ((5, 6), Facing::East));

    let seq = sim.host.seq();
    let hash = sim.host.log_hash();

    // Somebody else's token is not yours to walk around.
    sim.client_intent(PeerId(10), mv(1, (4, 4)));
    assert_eq!(sim.host.state().map.token(TokenId(1)).unwrap().at, (1, 1));

    // The board is the DM's: no painting terrain, no raising ground, no
    // spawning (a placed token carries its own `owner`, so an unchecked spawn
    // hands out ownership of anything), and no deleting what you dislike.
    let grass = sim.host.state().map.tile_kinds.len() as u16 - 1;
    sim.client_intent(
        PeerId(10),
        GameEvent::Map(SessionEvent::TilePlaced {
            layer: isometry_core::Layer::Ground,
            at: (0, 0),
            kind: isometry_core::TileKindId(grass),
        }),
    );
    sim.client_intent(
        PeerId(10),
        GameEvent::Map(SessionEvent::ElevationSet {
            at: (0, 0),
            height: 3,
        }),
    );
    sim.client_intent(
        PeerId(10),
        GameEvent::Map(SessionEvent::TokenPlaced(Token {
            id: TokenId(50),
            at: (3, 3),
            facing: Facing::South,
            sprite: "dragon".to_owned(),
            owner: None, // claiming a DM-owned token outright
        })),
    );
    sim.client_intent(
        PeerId(10),
        GameEvent::Map(SessionEvent::TokenRemoved { id: TokenId(1) }),
    );
    assert!(
        sim.host.state().map.token(TokenId(1)).is_some(),
        "a player deleted another player's token"
    );
    assert!(
        sim.host.state().map.token(TokenId(50)).is_none(),
        "a player spawned a token onto the board"
    );

    // The initiative order is the table's.
    sim.client_intent(PeerId(10), GameEvent::TurnAdd(TokenId(2)));
    sim.client_intent(PeerId(10), GameEvent::TurnSetOrder(vec![TokenId(2)]));
    assert!(sim.host.state().turns.active().is_none());

    // And a sheet is where every number a rule reads lives.
    sim.client_intent(
        PeerId(10),
        GameEvent::SheetSet {
            token: TokenId(1),
            sheet: sheet("Knight", 0, 1), // the boss, at zero hit points
        },
    );
    assert!(
        sim.host.state().map.sheets.get(&TokenId(1)).is_none(),
        "a player wrote another token's sheet"
    );

    assert_eq!(
        sim.host.seq(),
        seq,
        "a refused intent entered the replicated log"
    );
    assert_eq!(sim.host.log_hash(), hash);
    assert_converged(&sim);
}

#[test]
fn a_player_may_emote_for_itself_without_the_host_adjudicating() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    // Token 2 belongs to player B; token 1 to player A.
    sim.client_hello(PeerId(10), "B");

    // Unlike an attack, a client's own emote is accepted: there is no verdict to
    // forge and no state to change, so the worst a liar can do is wave.
    sim.client_intent(
        PeerId(10),
        GameEvent::Emoted {
            token: TokenId(2),
            beat: "cheer".to_owned(),
        },
    );

    let beats = &sim.host.state().last_beats;
    assert_eq!(beats, &[Beat::new(TokenId(2), "cheer")]);
    assert_eq!(
        &sim.clients[&PeerId(10)].state().unwrap().last_beats,
        beats,
        "everyone at the table sees the cheer"
    );
    // It is a flourish, not a fact: no roll, no delta, nothing to undo.
    assert!(sim.host.state().roll_log.is_empty());
    assert_converged(&sim);

    // But only your own: a wave is harmless, and puppeteering someone else's
    // token (or the DM's monsters) is not.
    let seq = sim.host.seq();
    sim.client_intent(
        PeerId(10),
        GameEvent::Emoted {
            token: TokenId(1), // player A's knight
            beat: "taunt".to_owned(),
        },
    );
    assert_eq!(sim.host.seq(), seq, "B puppeteered A's knight");

    // And an emote for a token that does not exist is still refused.
    sim.client_intent(
        PeerId(10),
        GameEvent::Emoted {
            token: TokenId(99),
            beat: "cheer".to_owned(),
        },
    );
    assert_eq!(sim.host.seq(), seq);
    assert_converged(&sim);
}

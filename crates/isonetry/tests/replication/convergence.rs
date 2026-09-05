//! Ordering, joining, and refusal: the base the rest of the suite stands on.
//!
//! The host numbers events, every client replays them in that order, and a
//! peer that joins mid-session catches up from a snapshot plus the tail.
//! Whatever else a test proves, it proves it on top of these.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn from_start_clients_converge_on_host_ordering() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.connect(PeerId(11));
    // Each peer plays the token it owns: token 1 is A's knight, token 2 is B's.
    sim.client_hello(PeerId(11), "A");
    sim.client_hello(PeerId(10), "B");

    // Host and both clients each propose moves; the host orders them.
    sim.host_event(mv(1, (2, 1)));
    sim.client_intent(PeerId(10), mv(2, (5, 6)));
    sim.client_intent(PeerId(11), mv(1, (2, 2)));
    sim.host_event(GameEvent::TurnAdd(TokenId(1)));

    assert_eq!(sim.host.seq(), 4);
    assert_eq!(sim.host.state().map.token(TokenId(1)).unwrap().at, (2, 2));
    assert_eq!(sim.host.state().map.token(TokenId(2)).unwrap().at, (5, 6));
    assert_converged(&sim);
}

#[test]
fn late_joiner_gets_snapshot_plus_tail_and_converges() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "B"); // token 2 is B's goblin

    // Play happens before the second player joins.
    sim.host_event(mv(1, (3, 1)));
    sim.client_intent(PeerId(10), mv(2, (4, 6)));
    sim.host_event(GameEvent::TurnAdd(TokenId(1)));
    let hash_at_join = sim.host.log_hash();
    let seq_at_join = sim.host.seq();

    // Mid-session join: snapshot carries the 3 prior events' state+hash.
    sim.connect(PeerId(20));
    sim.client_hello(PeerId(20), "A"); // the joiner plays A's knight
    let joiner = &sim.clients[&PeerId(20)];
    assert_eq!(joiner.applied(), seq_at_join);
    assert_eq!(joiner.log_hash(), hash_at_join);
    assert_eq!(joiner.state(), Some(sim.host.state()));

    // More play; the late joiner replays the tail and stays converged
    // with the peer that saw everything.
    sim.client_intent(PeerId(20), mv(1, (3, 2)));
    sim.host_event(GameEvent::TurnAdvance);
    assert_eq!(sim.host.seq(), 5);
    assert_converged(&sim);
}

#[test]
fn invalid_intent_is_rejected_without_divergence() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "A"); // token 1 is A's knight
    sim.host_event(mv(1, (2, 1)));
    let seq = sim.host.seq();
    let hash = sim.host.log_hash();

    // Its own token, off the edge of the board: this one clears the ownership
    // gate and must then fail the substrate's own validation. (Before the gate
    // existed every case below was refused for bounds; now only this one is, so
    // it is the case that still proves validation runs.)
    sim.client_intent(PeerId(10), mv(1, (99, 0)));
    // A token that does not exist is nobody's, so it is refused as unowned.
    sim.client_intent(PeerId(10), mv(7, (2, 2)));
    // And the turn order is the DM's, whatever token is named.
    sim.client_intent(PeerId(10), GameEvent::TurnAdd(TokenId(7)));

    assert_eq!(sim.host.seq(), seq, "rejected intents changed the log");
    assert_eq!(sim.host.log_hash(), hash);
    assert_converged(&sim);
}

#[test]
fn forced_movement_is_truth_and_lands_on_the_same_tile_everywhere() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(2),
        sheet: sheet("Goblin", 7, 15),
    });
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(1),
        sheet: sheet("Knight", 12, 16),
    });

    // A shove: the goblin genuinely relocates. Unlike a stagger beat, which
    // peers may render however they like, this changes what the goblin can
    // reach and see, so every peer must land it on exactly the same tile.
    let mut shove = attack_hit(0);
    if let GameEvent::ActionResolved(res) = &mut shove {
        res.action_key = "shove".to_owned();
        res.deltas.clear();
        res.displaced = vec![(TokenId(2), (7, 6))];
        res.beats[1] = Beat::new(TokenId(2), "shoved-e");
    }
    sim.host_event(shove);

    let at = |s: &GameSnapshot| s.map.token(TokenId(2)).unwrap().at;
    assert_eq!(at(sim.host.state()), (7, 6), "the goblin was pushed");
    assert_eq!(
        at(sim.clients[&PeerId(10)].state().unwrap()),
        (7, 6),
        "forced movement is game truth, so it cannot be left to each peer"
    );
    assert_converged(&sim);
}

#[test]
fn turn_order_replicates() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "A"); // token 1 is A's knight, and goes first
    sim.host_event(GameEvent::TurnAdd(TokenId(1)));
    sim.host_event(GameEvent::TurnAdd(TokenId(2)));
    // A ends its own turn, which is the one turn it may end.
    sim.client_intent(PeerId(10), GameEvent::TurnAdvance);

    assert_eq!(sim.host.state().turns.active(), Some(TokenId(2)));
    // Now it is B's turn, so A may not end it: a player cannot skip past
    // somebody else, nor spam the order forward through an enemy's turn.
    sim.client_intent(PeerId(10), GameEvent::TurnAdvance);
    assert_eq!(
        sim.host.state().turns.active(),
        Some(TokenId(2)),
        "a player ended a turn that was not its own"
    );
    sim.host_event(GameEvent::TurnRemove(TokenId(2)));
    assert_eq!(sim.host.state().turns.active(), Some(TokenId(1)));
    assert_converged(&sim);
}

#[test]
fn whisper_reaches_only_the_named_player() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.connect(PeerId(20));
    sim.client_hello(PeerId(10), "alice");
    sim.client_hello(PeerId(20), "bob");

    sim.host_whisper("dm", "alice", "the door is trapped");
    assert_eq!(
        sim.clients[&PeerId(10)].inbox(),
        &[("dm".to_owned(), "the door is trapped".to_owned())]
    );
    assert!(
        sim.clients[&PeerId(20)].inbox().is_empty(),
        "bob does not see alice's whisper"
    );

    // A whisper to a name nobody announced goes nowhere.
    sim.host_whisper("dm", "carol", "hello?");
    assert_eq!(sim.clients[&PeerId(10)].inbox().len(), 1);
    // Whispers are directed, so they never touch the replicated log.
    assert_converged(&sim);
}

#[test]
fn move_and_facing_batch_orders_atomically() {
    // A play move is two events (move + face); interleaving from two
    // clients must not split a pair, because the host applies each
    // intent fully before the next.
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "A"); // token 1 is A's knight
    sim.client_intent(PeerId(10), mv(1, (2, 1)));
    sim.client_intent(
        PeerId(10),
        GameEvent::Map(SessionEvent::TokenFaced {
            id: TokenId(1),
            facing: Facing::East,
        }),
    );
    let t = sim.host.state().map.token(TokenId(1)).unwrap();
    assert_eq!((t.at, t.facing), ((2, 1), Facing::East));
    assert_converged(&sim);
}

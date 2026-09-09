//! Asking, ruling, and the verdict landing on every peer.
//!
//! The net crate never decides an outcome: a client asks, the host's rules
//! resolve it, and the resolution replicates as ordinary logged events. These
//! cover the ask, the refusals around it, and the conditions and counters a
//! verdict writes.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn a_resolved_attack_replicates_and_lands_on_every_peer() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(1),
        sheet: sheet("Knight", 12, 16),
    });
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(2),
        sheet: sheet("Goblin", 7, 15),
    });

    sim.host_event(attack_hit(5));

    // The goblin took the hit on the host and on the client, identically.
    let hp = |s: &GameSnapshot| s.map.sheet(TokenId(2)).unwrap().int("hp_current");
    assert_eq!(hp(sim.host.state()), Some(2), "7 hp less 5 damage");
    assert_eq!(hp(sim.clients[&PeerId(10)].state().unwrap()), Some(2));
    // The attacker is untouched: a resolution changes only what it addresses.
    assert_eq!(
        sim.host
            .state()
            .map
            .sheet(TokenId(1))
            .unwrap()
            .int("hp_current"),
        Some(12)
    );
    // Both rolls reached the shared log, and the beats reached the client so it
    // can play the exchange rather than merely read about it.
    assert_eq!(sim.host.state().roll_log.len(), 2);
    let beats = &sim.clients[&PeerId(10)].state().unwrap().last_beats;
    assert_eq!(
        beats.len(),
        2,
        "the client must see the exchange to play it"
    );
    assert_eq!(beats[1], Beat::new(TokenId(2), "recoil"));
    assert_converged(&sim);
}

#[test]
fn a_killing_blow_replicates_and_the_fallen_lose_their_turn() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(1),
        sheet: sheet("Knight", 12, 16),
    });
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(2),
        sheet: sheet("Goblin", 7, 15),
    });
    sim.host_event(GameEvent::TurnAdd(TokenId(1)));
    sim.host_event(GameEvent::TurnAdd(TokenId(2)));

    // A blow that drops the goblin. The system judged it; the substrate obeys.
    let mut lethal = attack_hit(7);
    if let GameEvent::ActionResolved(res) = &mut lethal {
        res.defeated = vec![TokenId(2)];
        res.beats[1] = Beat::new(TokenId(2), "fall");
    }
    sim.host_event(lethal);

    let down = |s: &GameSnapshot| s.map.is_defeated(TokenId(2));
    assert!(down(sim.host.state()));
    assert!(
        down(sim.clients[&PeerId(10)].state().unwrap()),
        "the client must know it fell, or it will still let you swing at it"
    );

    // The turn passes from the knight straight back to the knight: the corpse
    // does not get a turn, and every peer computes that skip from state it
    // already has rather than being told about it.
    assert_eq!(sim.host.state().turns.active(), Some(TokenId(1)));
    sim.host_event(GameEvent::TurnAdvance);
    assert_eq!(sim.host.state().turns.active(), Some(TokenId(1)));
    assert_converged(&sim);
}

#[test]
fn allegiance_replicates_and_a_convinced_creature_joins_your_side() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));

    // Token 2 (the goblin) belongs to player B. A convince, ruled by the host,
    // hands it to player A. Owner changes are truth, so every peer applies it.
    let mut won = attack_hit(0);
    if let GameEvent::ActionResolved(res) = &mut won {
        res.action_key = "convince".to_owned();
        res.deltas.clear();
        res.owner_changes = vec![(TokenId(2), Some("A".to_owned()))];
        res.beats[1] = Beat::new(TokenId(2), "cheer");
    }
    sim.host_event(won);

    let owner = |s: &GameSnapshot| s.map.token(TokenId(2)).unwrap().owner.clone();
    assert_eq!(
        owner(sim.host.state()).as_deref(),
        Some("A"),
        "the goblin joined A"
    );
    assert_eq!(
        owner(sim.clients[&PeerId(10)].state().unwrap()).as_deref(),
        Some("A"),
        "allegiance is game truth, so the client holds it too"
    );
    // It did no damage: convince changes sides, not hit points.
    assert_eq!(
        sim.host
            .state()
            .map
            .sheet(TokenId(2))
            .and_then(|s| s.int("hp_current")),
        None,
        "no sheet was bound, and none was needed to change owner"
    );
    assert_converged(&sim);
}

#[test]
fn a_condition_and_its_numbers_replicate_and_standing_up_restores_them() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(2),
        sheet: sheet("Goblin", 7, 15),
    });

    // A trip lands: prone plus the rules' recomputed numbers, one event.
    let mut trip = attack_hit(0);
    if let GameEvent::ActionResolved(res) = &mut trip {
        res.action_key = "trip".to_owned();
        res.deltas.clear();
        res.conditions = vec![(TokenId(2), "prone".to_owned(), 1)];
        res.mobility = vec![(TokenId(2), Some((2, 6)))];
    }
    sim.host_event(trip);

    let check = |s: &GameSnapshot| {
        (
            s.map.has_condition(TokenId(2), "prone"),
            s.map.effective_mobility(TokenId(2), (5, 6)),
        )
    };
    assert_eq!(check(sim.host.state()), (true, (2, 6)));
    assert_eq!(
        check(sim.clients[&PeerId(10)].state().unwrap()),
        (true, (2, 6)),
        "the client computes fog and reach locally, so it must hold the numbers"
    );

    // Standing up: the condition clears and the override clears with it, so the
    // sheet's base values stand again.
    sim.host_event(GameEvent::ConditionSet {
        token: TokenId(2),
        condition: "prone".to_owned(),
        value: 0,
        mobility: None,
    });
    assert_eq!(check(sim.host.state()), (false, (5, 6)));
    assert_eq!(
        check(sim.clients[&PeerId(10)].state().unwrap()),
        (false, (5, 6))
    );
    assert_converged(&sim);

    // A client may not pronounce a condition: that is a rules ruling.
    let seq = sim.host.seq();
    sim.client_intent(
        PeerId(10),
        GameEvent::ConditionSet {
            token: TokenId(2),
            condition: "blinded".to_owned(),
            value: 1,
            mobility: Some((5, 0)),
        },
    );
    assert_eq!(sim.host.seq(), seq, "a client ruled on a condition");
    assert_converged(&sim);
}

#[test]
fn a_client_asks_and_the_host_adjudicates() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "A"); // token 1 is A's knight

    // The player swings. What crosses the wire is a *request*: no roll, no
    // damage, no verdict. The host holds the rules; the client holds none.
    sim.client_action(
        PeerId(10),
        ActionIntent::new(TokenId(1), TokenId(2), "attack"),
    );

    // It changes nothing by itself. It is not an event and never enters the log;
    // it waits for the host's rules system to answer it.
    assert_eq!(sim.host.seq(), 0, "an ask is not a fact");
    let queued = sim.host.take_action_intents();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].actor, TokenId(1));
    assert_eq!(queued[0].action_key, "attack");
    // Drained exactly once: the host app resolves it, and it does not linger to
    // be resolved twice.
    assert!(sim.host.take_action_intents().is_empty());
    assert_converged(&sim);
}

#[test]
fn a_client_may_only_act_with_its_own_tokens() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "B"); // B owns token 2, not token 1

    // Swinging *someone else's* sword is refused before the rules are consulted:
    // ownership is one of the two things the rules-blind session can check.
    sim.client_action(
        PeerId(10),
        // Player A's knight, which B does not command.
        ActionIntent::new(TokenId(1), TokenId(2), "attack"),
    );
    assert!(
        sim.host.take_action_intents().is_empty(),
        "B queued an action for A's knight"
    );
    assert_converged(&sim);
}

#[test]
fn a_client_cannot_pronounce_its_own_verdict() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(1),
        sheet: sheet("Knight", 12, 16),
    });
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(2),
        sheet: sheet("Goblin", 7, 15),
    });
    let seq = sim.host.seq();
    let hash = sim.host.log_hash();

    // A client proposing a resolution is proposing that it hit and for how
    // much. The rules run on the sequencer; a client asks, it never decides.
    sim.client_intent(PeerId(10), attack_hit(999));

    assert_eq!(sim.host.seq(), seq, "a forged verdict entered the log");
    assert_eq!(sim.host.log_hash(), hash);
    assert_eq!(
        sim.host
            .state()
            .map
            .sheet(TokenId(2))
            .unwrap()
            .int("hp_current"),
        Some(7),
        "the goblin took damage from an unadjudicated claim"
    );
    assert_converged(&sim);
}

#[test]
fn a_resolution_addressing_an_unsheeted_token_is_refused_whole() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    // Only the attacker is statted; the goblin was never bound a sheet.
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(1),
        sheet: sheet("Knight", 12, 16),
    });
    let seq = sim.host.seq();

    sim.host_event(attack_hit(5));

    assert_eq!(
        sim.host.seq(),
        seq,
        "a half-appliable resolution entered the log"
    );
    assert_converged(&sim);
}

#[test]
fn a_graded_condition_replicates_at_its_magnitude() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(2),
        sheet: sheet("Goblin", 7, 15),
    });

    // A Demoralize critical: frightened 2, no damage. The magnitude is truth,
    // so every peer must hold the same number -- "frightened 1" and
    // "frightened 2" are different states and only one of them is real here.
    let mut fear = attack_hit(0);
    if let GameEvent::ActionResolved(res) = &mut fear {
        res.action_key = "demoralize".to_owned();
        res.deltas.clear();
        res.conditions = vec![(TokenId(2), "frightened".to_owned(), 2)];
    }
    sim.host_event(fear);

    let value = |s: &GameSnapshot| s.map.condition_value(TokenId(2), "frightened");
    assert_eq!(value(sim.host.state()), 2);
    assert_eq!(
        value(sim.clients[&PeerId(10)].state().unwrap()),
        2,
        "the magnitude is game truth, so the client holds 2, not merely 'frightened'"
    );
    assert_converged(&sim);
}

#[test]
fn per_turn_counters_replicate_and_reset_when_the_turn_comes_round() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(1),
        sheet: sheet("Knight", 12, 16),
    });
    sim.host_event(GameEvent::SheetSet {
        token: TokenId(2),
        sheet: sheet("Goblin", 7, 15),
    });
    sim.host_event(GameEvent::TurnAdd(TokenId(1)));
    sim.host_event(GameEvent::TurnAdd(TokenId(2)));

    // Two strikes in the knight's turn, each spending an action and adding to
    // the multiple-attack tally. The net layer carries the integers the rules
    // decided; it never learns they mean "actions" or "attacks".
    let strike = || {
        let mut e = attack_hit(3);
        if let GameEvent::ActionResolved(res) = &mut e {
            res.turn_counters = vec![
                (TokenId(1), "actions_spent".to_owned(), 1),
                (TokenId(1), "strikes".to_owned(), 1),
            ];
        }
        e
    };
    sim.host_event(strike());
    sim.host_event(strike());

    // The ledger accumulated, and the client holds exactly the same count: a
    // per-turn resource is truth, applied verbatim like any sheet delta.
    let spent = |s: &GameSnapshot| s.map.turn_counter(TokenId(1), "actions_spent");
    assert_eq!(spent(sim.host.state()), 2);
    assert_eq!(spent(sim.clients[&PeerId(10)].state().unwrap()), 2);
    assert_eq!(sim.host.state().map.turn_counter(TokenId(1), "strikes"), 2);

    // The knight's turn ends and the goblin's begins. A turn-start wipes the
    // *incoming* token's counters, so the goblin's clear (they were empty), but
    // the knight keeps its spend while someone else acts.
    sim.host_event(GameEvent::TurnAdvance);
    assert_eq!(sim.host.state().turns.active(), Some(TokenId(2)));
    assert_eq!(
        spent(sim.host.state()),
        2,
        "another token's turn does not refill yours"
    );

    // The turn comes back to the knight: now its counters wipe, so it has its
    // whole action economy again. Every peer computes the identical reset from
    // the same TurnAdvance -- nobody is told the counts separately.
    sim.host_event(GameEvent::TurnAdvance);
    assert_eq!(sim.host.state().turns.active(), Some(TokenId(1)));
    assert_eq!(
        spent(sim.host.state()),
        0,
        "the knight's own turn refilled its actions"
    );
    assert_eq!(spent(sim.clients[&PeerId(10)].state().unwrap()), 0);
    assert_converged(&sim);
}

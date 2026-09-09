// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::{Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef, Yaw};
use paredros_identity::{SubjectId, Tick};
use paredros_world::timed_action::{Direction, TimedActionRules, TimedActionSession};
use paredros_world::{GameIntent, GameState, ItemLocation, Name, Session, World, WorldConfig};
use std::collections::BTreeSet;
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};

fn prepared() -> TimedActionSession {
    let subject = SubjectId(1);
    let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at) => Some(at),
            _ => None,
        })
        .unwrap();
    game.apply(GameIntent::Generate {
        tick: game.next_tick(),
        subject,
        body_seed: 1,
        at,
    })
    .unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(),
        subject,
        name: Name::new("Tester").unwrap(),
    })
    .unwrap();
    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [2; 3]);
    for offset in [[3, 0, 0], [-3, 0, 0]] {
        body.attach(
            VolumeRef::from_tag(2),
            20,
            [1; 3],
            Attachment {
                parent: PartId(0),
                offset,
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    }
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(),
        subject,
        revision: game.bodies().get(subject).unwrap().revision,
        document: Box::new(body),
    })
    .unwrap();
    let network = FunctionalNetwork::new(
        vec![
            Node {
                id: NodeId(0),
                kind: NodeKind::Source {
                    part: PartRef {
                        subject: 1,
                        part: 0,
                    },
                    capacity: 50,
                    charge: 50,
                },
            },
            Node {
                id: NodeId(1),
                kind: NodeKind::Effect {
                    part: PartRef {
                        subject: 1,
                        part: 1,
                    },
                },
            },
            Node {
                id: NodeId(2),
                kind: NodeKind::Effect {
                    part: PartRef {
                        subject: 1,
                        part: 2,
                    },
                },
            },
        ],
        vec![
            Edge {
                from: NodeId(0),
                to: NodeId(1),
                capacity: 50,
            },
            Edge {
                from: NodeId(0),
                to: NodeId(2),
                capacity: 50,
            },
        ],
    )
    .unwrap();
    let rules = TimedActionRules {
        max_contributors: 2,
        max_elapsed_ticks: 100,
        charge_per_tick: 2,
        max_charge_per_limb: 10,
        evaluation: WorldRules {
            allowed_operators: BTreeSet::from([Operator::Strengthen]),
            allowed_costs: BTreeSet::from([2]),
            max_range: 0,
            max_hops: 3,
        },
    };
    let mut state =
        TimedActionSession::begin(Session::begin(game, subject).unwrap(), network, rules).unwrap();
    state.prepare(Direction::Left).unwrap();
    state
}

#[test]
fn saved_contribution_cannot_be_rebound_to_another_real_limb() {
    let state = prepared();
    let mut save = state.save_record().unwrap();
    let action = save.action.as_mut().unwrap();
    let contribution = action.contributors.values_mut().next().unwrap();
    contribution.binding.part = PartId(2);
    assert!(TimedActionSession::restore_record(save).is_err());
}

#[test]
fn wrong_tick_does_not_remove_preparation_or_spend_supply() {
    let mut state = prepared();
    let before = state.clone();
    let same_tick = state.action().unwrap().last_tick;
    assert!(state.charge(same_tick).is_err());
    assert_eq!(state, before);
    state.charge(Tick(same_tick.0 + 1)).unwrap();
    let charged = state.clone();
    assert!(state.release(same_tick).is_err());
    assert_eq!(state, charged);
}

#[test]
fn unknown_network_schema_is_refused_before_continuation() {
    let state = prepared();
    let mut save = state.save_record().unwrap();
    save.network.schema_version += 1;
    assert!(TimedActionSession::restore_record(save).is_err());
}

#[test]
fn partial_loss_round_trip_preserves_only_the_surviving_strike() {
    let mut state = prepared();
    state.join(PartId(2)).unwrap();
    let charged_at = Tick(state.action().unwrap().last_tick.0 + 1);
    state.charge(charged_at).unwrap();
    let subject = state.session().control().played();
    let revision = state
        .session()
        .game()
        .bodies()
        .get(subject)
        .unwrap()
        .revision;
    let tick = state.session().game().next_tick();
    state
        .apply_game_batch(&[
            GameIntent::Fall {
                tick,
                subject,
                distance: 5,
            },
            GameIntent::ReconcileAnatomy {
                tick: Tick(tick.0 + 1),
                subject,
                from_revision: revision,
                revision: paredros_identity::BodyRevisionId(revision.0 + 1),
                severed_parts: vec![PartId(1)],
            },
        ])
        .unwrap();
    let mut restored = TimedActionSession::restore(&state.save().unwrap()).unwrap();
    assert_eq!(restored, state);
    let receipts = restored.release(charged_at).unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].binding.part, PartId(2));
    assert_eq!(receipts[0].charge, 2);
}

#[test]
fn death_and_unreconciled_injury_can_be_saved_without_reviving_an_action() {
    for distance in [5, 20] {
        let mut state = prepared();
        let tick = Tick(state.action().unwrap().last_tick.0 + 1);
        state.charge(tick).unwrap();
        state
            .apply_game_batch(&[GameIntent::Fall {
                tick: state.session().game().next_tick(),
                subject: state.session().control().played(),
                distance,
            }])
            .unwrap();
        let mut restored = TimedActionSession::restore(&state.save().unwrap()).unwrap();
        assert_eq!(restored, state);
        assert!(restored.release(tick).is_err());
        assert_eq!(restored, state);
    }
}

#[test]
fn exhaustion_does_not_erase_paid_charge() {
    let state = prepared();
    let mut save = state.save_record().unwrap();
    if let NodeKind::Source { charge, .. } =
        &mut save.network.network.nodes.get_mut(&NodeId(0)).unwrap().kind
    {
        *charge = 2;
    }
    let mut state = TimedActionSession::restore_record(save).unwrap();
    let tick = state.action().unwrap().last_tick;
    state.charge(Tick(tick.0 + 1)).unwrap();
    state.charge(Tick(tick.0 + 2)).unwrap();
    let strikes = state.release(Tick(tick.0 + 2)).unwrap();
    assert_eq!(strikes.len(), 1);
    assert_eq!(strikes[0].charge, 2);
}

#[test]
fn rejected_injury_batch_leaves_body_and_charge_unchanged() {
    let mut state = prepared();
    let before = state.clone();
    let tick = state.session().game().next_tick();
    let subject = state.session().control().played();
    assert!(
        state
            .apply_game_batch(&[
                GameIntent::Fall {
                    tick,
                    subject,
                    distance: 5
                },
                GameIntent::ReconcileAnatomy {
                    tick: Tick(tick.0 + 1),
                    subject,
                    from_revision: paredros_identity::BodyRevisionId(99),
                    revision: paredros_identity::BodyRevisionId(100),
                    severed_parts: vec![PartId(1)],
                },
            ])
            .is_err()
    );
    assert_eq!(state, before);
}

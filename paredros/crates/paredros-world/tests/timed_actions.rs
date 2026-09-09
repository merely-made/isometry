// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use mesocosm_core::{Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef, Yaw};
use paredros_identity::{SubjectId, Tick};
use paredros_world::timed_action::{Direction, TimedActionRules, TimedActionSession};
use paredros_world::{GameIntent, GameState, ItemLocation, Name, Session, World, WorldConfig};
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};

fn action() -> TimedActionSession {
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
                    capacity: 20,
                    charge: 20,
                },
            },
            Node {
                id: NodeId(1),
                kind: NodeKind::Gate {
                    part: PartRef {
                        subject: 1,
                        part: 0,
                    },
                    open: true,
                },
            },
            Node {
                id: NodeId(2),
                kind: NodeKind::Effect {
                    part: PartRef {
                        subject: 1,
                        part: 1,
                    },
                },
            },
            Node {
                id: NodeId(3),
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
                capacity: 20,
            },
            Edge {
                from: NodeId(1),
                to: NodeId(2),
                capacity: 20,
            },
            Edge {
                from: NodeId(1),
                to: NodeId(3),
                capacity: 20,
            },
        ],
    )
    .unwrap();
    let rules = TimedActionRules {
        max_contributors: 2,
        max_elapsed_ticks: 5,
        charge_per_tick: 2,
        max_charge_per_limb: 4,
        evaluation: WorldRules {
            allowed_operators: BTreeSet::from([Operator::Strengthen]),
            allowed_costs: BTreeSet::from([2]),
            max_range: 0,
            max_hops: 3,
        },
    };
    let mut action =
        TimedActionSession::begin(Session::begin(game, subject).unwrap(), network, rules).unwrap();
    action.prepare(Direction::Left).unwrap();
    action
}

#[test]
fn explicit_join_charges_each_limb_and_release_does_not_debit_again() {
    let mut action = action();
    action.join(PartId(2)).unwrap();
    let at = action.action().unwrap().last_tick;
    action.charge(Tick(at.0 + 1)).unwrap();
    let before_release = action.network().clone();
    let strikes = action.release(Tick(at.0 + 1)).unwrap();
    assert_eq!(strikes.len(), 2);
    assert!(
        strikes
            .iter()
            .all(|strike| strike.direction == Direction::Left && strike.charge == 2)
    );
    assert_eq!(action.network(), &before_release);
}

#[test]
fn a_closed_shared_gate_prevents_already_charged_limb_receipts_without_refund() {
    let mut action = action();
    let at = action.action().unwrap().last_tick;
    action.charge(Tick(at.0 + 1)).unwrap();
    let spent = action.network().clone();
    action.set_gate(NodeId(1), false).unwrap();
    assert!(action.release(Tick(at.0 + 1)).unwrap().is_empty());
    assert_ne!(
        action.network(),
        &FunctionalNetwork::new(
            vec![Node {
                id: NodeId(0),
                kind: NodeKind::Source {
                    part: PartRef {
                        subject: 1,
                        part: 0
                    },
                    capacity: 20,
                    charge: 20
                }
            }],
            vec![]
        )
        .unwrap()
    );
    assert_eq!(
        action.network().nodes.get(&NodeId(0)),
        spent.nodes.get(&NodeId(0))
    );
}

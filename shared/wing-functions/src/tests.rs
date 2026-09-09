// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::BTreeSet;

fn part(id: u32) -> PartRef {
    PartRef {
        subject: 7,
        part: id,
    }
}
fn rules() -> WorldRules {
    WorldRules {
        allowed_operators: [Operator::Strengthen, Operator::Project, Operator::Store]
            .into_iter()
            .collect(),
        allowed_costs: [1, 3, 5, 8].into_iter().collect(),
        max_range: 12,
        max_hops: 6,
    }
}
fn request(operator: Operator, target: u32, cost: u64) -> EvaluationRequest {
    EvaluationRequest {
        operator,
        target: NodeId(target),
        cost,
        range: 4,
        max_hops: 6,
    }
}
fn live() -> BTreeSet<PartRef> {
    [part(1), part(2), part(3), part(4), part(5)]
        .into_iter()
        .collect()
}
fn network(gate_open: bool) -> FunctionalNetwork {
    FunctionalNetwork::new(
        vec![
            Node {
                id: NodeId(1),
                kind: NodeKind::Source {
                    part: part(1),
                    capacity: 8,
                    charge: 8,
                },
            },
            Node {
                id: NodeId(2),
                kind: NodeKind::Gate {
                    part: part(2),
                    open: gate_open,
                },
            },
            Node {
                id: NodeId(3),
                kind: NodeKind::Effect { part: part(3) },
            },
            Node {
                id: NodeId(4),
                kind: NodeKind::Source {
                    part: part(4),
                    capacity: 5,
                    charge: 5,
                },
            },
            Node {
                id: NodeId(5),
                kind: NodeKind::Store {
                    part: part(5),
                    capacity: 8,
                    charge: 0,
                },
            },
        ],
        vec![
            Edge {
                from: NodeId(1),
                to: NodeId(2),
                capacity: 8,
            },
            Edge {
                from: NodeId(2),
                to: NodeId(3),
                capacity: 8,
            },
            Edge {
                from: NodeId(4),
                to: NodeId(3),
                capacity: 5,
            },
            Edge {
                from: NodeId(1),
                to: NodeId(5),
                capacity: 8,
            },
        ],
    )
    .unwrap()
}

#[test]
fn redundancy_survives_a_severed_gate() {
    let mut network = network(false);
    let receipt = network
        .evaluate(&request(Operator::Strengthen, 3, 5), &live(), &rules())
        .unwrap();
    assert_eq!(
        receipt.supplied_by,
        vec![SupplyDebit {
            source: NodeId(4),
            amount: 5
        }]
    );
}

#[test]
fn severed_only_route_is_atomic() {
    let mut network = network(false);
    network.edges.retain(|edge| edge.from != NodeId(4));
    let before = network.clone();
    assert!(matches!(
        network.evaluate(&request(Operator::Strengthen, 3, 3), &live(), &rules()),
        Err(EvaluationError::ClosedGate { .. })
    ));
    assert_eq!(network, before);
}

#[test]
fn store_conserves_charge_and_rejects_overflow() {
    let mut network = network(true);
    let receipt = network
        .evaluate(&request(Operator::Store, 5, 5), &live(), &rules())
        .unwrap();
    assert_eq!(
        receipt.effect,
        EffectReport::Stored {
            store: part(5),
            amount: 5,
            charge_after: 5
        }
    );
    let before = network.clone();
    assert!(matches!(
        network.evaluate(&request(Operator::Store, 5, 5), &live(), &rules()),
        Err(EvaluationError::StoreOverflow { .. })
    ));
    assert_eq!(network, before);
}

#[test]
fn cycles_are_bounded_and_cannot_create_charge() {
    let mut network = network(true);
    network.edges.push(Edge {
        from: NodeId(2),
        to: NodeId(1),
        capacity: 8,
    });
    let receipt = network
        .evaluate(&request(Operator::Project, 3, 8), &live(), &rules())
        .unwrap();
    assert_eq!(
        receipt
            .supplied_by
            .iter()
            .map(|debit| debit.amount)
            .sum::<u64>(),
        8
    );
    assert!(matches!(
        network.evaluate(&request(Operator::Project, 3, 8), &live(), &rules()),
        Err(EvaluationError::InsufficientCharge { .. })
    ));
}

#[test]
fn range_cost_and_part_rules_fail_before_debit() {
    let mut network = network(true);
    let before = network.clone();
    let mut bad = request(Operator::Project, 3, 3);
    bad.range = 13;
    assert!(matches!(
        network.evaluate(&bad, &live(), &rules()),
        Err(EvaluationError::RangeExceeded { .. })
    ));
    let mut missing = live();
    missing.remove(&part(3));
    assert!(matches!(
        network.evaluate(&request(Operator::Project, 3, 3), &missing, &rules()),
        Err(EvaluationError::MissingPart(_))
    ));
    assert_eq!(network, before);
}

#[test]
fn persistence_and_interrupted_chain_preserve_state() {
    let mut network = network(true);
    let encoded = serde_json::to_string(&NetworkSnapshot::new(network.clone())).unwrap();
    let restored: NetworkSnapshot = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored.network, network);
    let before = network.clone();
    let chain = [
        request(Operator::Store, 5, 5),
        request(Operator::Store, 5, 5),
    ];
    assert!(matches!(
        network.evaluate_chain(&chain, &live(), &rules()),
        Err(EvaluationError::StoreOverflow { .. })
    ));
    assert_eq!(network, before);
}

#[test]
fn stored_charge_can_supply_later_after_original_source_disconnects() {
    let mut network = network(true);
    network
        .evaluate(&request(Operator::Store, 5, 5), &live(), &rules())
        .unwrap();
    network
        .edges
        .retain(|edge| (edge.from != NodeId(1) || edge.to != NodeId(2)) && edge.from != NodeId(4));
    network.edges.push(Edge {
        from: NodeId(5),
        to: NodeId(2),
        capacity: 5,
    });
    let receipt = network
        .evaluate(&request(Operator::Project, 3, 5), &live(), &rules())
        .unwrap();
    assert_eq!(
        receipt.supplied_by,
        vec![SupplyDebit {
            source: NodeId(5),
            amount: 5
        }]
    );
}

#[test]
fn malformed_networks_and_empty_chains_are_rejected() {
    let source = Node {
        id: NodeId(1),
        kind: NodeKind::Source {
            part: part(1),
            capacity: 1,
            charge: 1,
        },
    };
    assert!(matches!(
        FunctionalNetwork::new(vec![source.clone(), source], vec![]),
        Err(NetworkError::DuplicateNode(_))
    ));
    let source = Node {
        id: NodeId(1),
        kind: NodeKind::Source {
            part: part(1),
            capacity: 1,
            charge: 1,
        },
    };
    assert!(matches!(
        FunctionalNetwork::new(
            vec![source],
            vec![Edge {
                from: NodeId(1),
                to: NodeId(99),
                capacity: 1
            }]
        ),
        Err(NetworkError::DanglingEdge(_))
    ));
    let mut corrupt = network(true);
    corrupt.edges[0].capacity = 0;
    assert!(matches!(
        corrupt.evaluate_chain(&[], &live(), &rules()),
        Err(EvaluationError::InvalidNetwork(
            NetworkError::InvalidEdgeCapacity(_)
        ))
    ));
}

#[test]
fn persisted_map_and_schema_are_checked() {
    let mut altered = network(true);
    altered.nodes.get_mut(&NodeId(1)).unwrap().id = NodeId(9);
    assert!(matches!(
        altered.validate(),
        Err(NetworkError::NodeKeyMismatch { .. })
    ));
    let snapshot = NetworkSnapshot {
        schema_version: NETWORK_SCHEMA_VERSION + 1,
        network: network(true),
    };
    assert!(matches!(
        snapshot.validate(),
        Err(NetworkError::UnsupportedSchema(_))
    ));
}

#[test]
fn world_bounds_zero_cost_and_chain_cap_reject_without_mutation() {
    let mut network = network(true);
    let before = network.clone();
    assert!(matches!(
        network.evaluate(&request(Operator::Project, 3, 0), &live(), &rules()),
        Err(EvaluationError::ZeroCost)
    ));
    let mut limited = rules();
    limited.max_range = 0;
    assert!(matches!(
        network.evaluate(&request(Operator::Project, 3, 3), &live(), &limited),
        Err(EvaluationError::RangeExceeded { .. })
    ));
    let mut limited = rules();
    limited.max_hops = 0;
    assert!(matches!(
        network.evaluate(&request(Operator::Project, 3, 3), &live(), &limited),
        Err(EvaluationError::HopsExceeded { .. })
    ));
    let chain = vec![request(Operator::Project, 3, 3); MAX_CHAIN + 1];
    assert!(matches!(
        network.evaluate_chain(&chain, &live(), &rules()),
        Err(EvaluationError::ChainTooLong { .. })
    ));
    assert_eq!(network, before);
}

#[test]
fn first_fit_routing_does_not_reroute_for_global_maximum_flow() {
    let nodes = vec![
        Node {
            id: NodeId(1),
            kind: NodeKind::Source {
                part: part(1),
                capacity: 5,
                charge: 5,
            },
        },
        Node {
            id: NodeId(2),
            kind: NodeKind::Source {
                part: part(2),
                capacity: 5,
                charge: 5,
            },
        },
        Node {
            id: NodeId(3),
            kind: NodeKind::Gate {
                part: part(3),
                open: true,
            },
        },
        Node {
            id: NodeId(4),
            kind: NodeKind::Gate {
                part: part(4),
                open: true,
            },
        },
        Node {
            id: NodeId(5),
            kind: NodeKind::Effect { part: part(5) },
        },
    ];
    let mut network = FunctionalNetwork::new(
        nodes,
        vec![
            Edge {
                from: NodeId(1),
                to: NodeId(3),
                capacity: 5,
            },
            Edge {
                from: NodeId(1),
                to: NodeId(4),
                capacity: 5,
            },
            Edge {
                from: NodeId(2),
                to: NodeId(3),
                capacity: 5,
            },
            Edge {
                from: NodeId(3),
                to: NodeId(5),
                capacity: 5,
            },
            Edge {
                from: NodeId(4),
                to: NodeId(5),
                capacity: 5,
            },
        ],
    )
    .unwrap();
    let before = network.clone();
    assert!(matches!(
        network.evaluate(&request(Operator::Project, 5, 8), &live(), &rules()),
        Err(EvaluationError::InsufficientCharge {
            needed: 8,
            available: 5
        })
    ));
    assert_eq!(network, before);
}

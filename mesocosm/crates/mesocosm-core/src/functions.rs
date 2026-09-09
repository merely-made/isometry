// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Current body membership for the shared functional evaluator.
//!
//! The caller owns subject identity and durable network state. Rebuild this
//! reading after each accepted body mutation; never save it as another body.
//! Eligibility is membership, not a claim that every living part can perform
//! every process. Construction admission must assign appropriate functions.

use crate::BodyDocument;
use std::collections::BTreeSet;
pub use wing_functions::PartRef;

/// Read stable, non-severed part identities from the authoritative body.
pub fn live_parts(subject: u64, body: &BodyDocument) -> BTreeSet<PartRef> {
    body.living()
        .map(|part| PartRef {
            subject,
            part: part.id.0,
        })
        .collect()
}

/// Bind a generated network to admitted sites of the current body. Site roles
/// come from an authored construction rule, not guesses based on part order.
pub fn generate_for_body(
    subject: u64,
    body: &BodyDocument,
    seed: u64,
    settings: &wing_functions::generation::GeneratorSettings,
    sites: &[wing_functions::generation::BodySite],
) -> Result<wing_functions::generation::GenerationBatch, PartRef> {
    let live = live_parts(subject, body);
    if let Some(site) = sites.iter().find(|site| !live.contains(&site.part)) {
        return Err(site.part);
    }
    Ok(wing_functions::generation::generate(seed, settings, sites))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Attachment, PartId, Provenance, SpeciesId, VolumeRef, Yaw};
    use wing_functions::{
        Edge, EvaluationRequest, FunctionalNetwork, Node, NodeId, NodeKind, Operator, WorldRules,
    };

    #[test]
    fn body_loss_interrupts_a_function_without_mutating_saved_network() {
        let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [2; 3]);
        let arm = body
            .attach(
                VolumeRef::from_tag(2),
                20,
                [1; 3],
                Attachment {
                    parent: PartId(0),
                    offset: [3, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let mut network = FunctionalNetwork::new(
            vec![
                Node {
                    id: NodeId(0),
                    kind: NodeKind::Source {
                        part: PartRef {
                            subject: 1,
                            part: 0,
                        },
                        capacity: 10,
                        charge: 10,
                    },
                },
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Effect {
                        part: PartRef {
                            subject: 1,
                            part: arm.0,
                        },
                    },
                },
            ],
            vec![Edge {
                from: NodeId(0),
                to: NodeId(1),
                capacity: 10,
            }],
        )
        .unwrap();
        let request = EvaluationRequest {
            operator: Operator::Strengthen,
            target: NodeId(1),
            cost: 3,
            range: 0,
            max_hops: 1,
        };
        let rules = WorldRules {
            allowed_operators: BTreeSet::from([Operator::Strengthen]),
            allowed_costs: BTreeSet::from([3]),
            max_range: 0,
            max_hops: 1,
        };
        network
            .evaluate(&request, &live_parts(1, &body), &rules)
            .unwrap();
        let before = network.clone();
        body.sever(arm);
        assert!(
            network
                .evaluate(&request, &live_parts(1, &body), &rules)
                .is_err()
        );
        assert_eq!(before, network);
        let bytes = postcard::to_allocvec(&(body.clone(), network.clone())).unwrap();
        let (restored_body, mut restored_network): (BodyDocument, FunctionalNetwork) =
            postcard::from_bytes(&bytes).unwrap();
        assert!(
            restored_network
                .evaluate(&request, &live_parts(1, &restored_body), &rules)
                .is_err()
        );
        assert_eq!(restored_network, network);
    }

    #[test]
    fn severing_parent_removes_descendant_bindings_and_replacement_gets_new_identity() {
        let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [2; 3]);
        let arm = body
            .attach(
                VolumeRef::from_tag(2),
                20,
                [1; 3],
                Attachment {
                    parent: PartId(0),
                    offset: [3, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let hand = body
            .attach(
                VolumeRef::from_tag(3),
                10,
                [1; 3],
                Attachment {
                    parent: arm,
                    offset: [2, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        assert!(live_parts(7, &body).contains(&PartRef {
            subject: 7,
            part: hand.0
        }));
        body.sever(arm);
        assert_eq!(
            live_parts(7, &body),
            BTreeSet::from([PartRef {
                subject: 7,
                part: 0
            }])
        );
        let replacement = body
            .attach(
                VolumeRef::from_tag(4),
                20,
                [1; 3],
                Attachment {
                    parent: PartId(0),
                    offset: [3, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        assert_ne!(replacement, arm);
        assert!(!live_parts(7, &body).contains(&PartRef {
            subject: 7,
            part: arm.0
        }));
        assert!(live_parts(7, &body).contains(&PartRef {
            subject: 7,
            part: replacement.0
        }));
    }
}

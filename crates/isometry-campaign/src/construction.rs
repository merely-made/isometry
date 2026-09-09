// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Versioned construction values inside the existing pack generator carrier.
//! This is a proposal, never a campaign/body mutation. Admission uses current
//! product-supplied parts and rules; successful inspection does not reserve charge.

use crate::GenValue;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wing_functions::{
    Evaluation, EvaluationError, EvaluationRequest, FunctionalNetwork, NETWORK_SCHEMA_VERSION,
    PartRef, WorldRules,
};

pub const CONSTRUCTION_SCHEMA: u32 = 1;
const MAX_PAYLOAD_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionProposal {
    pub schema: u32,
    pub network_schema: u16,
    /// Pack/generator identity chosen by the host, preserved with the result.
    pub generator: String,
    pub seed: u64,
    pub network: FunctionalNetwork,
    /// A coordinated batch. Each operation competes for the same finite supply.
    pub operations: Vec<EvaluationRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstructionError {
    Envelope,
    PayloadTooLarge,
    Decode(String),
    Schema(u32),
    NetworkSchema(u16),
    MissingGenerator,
    Execution(EvaluationError),
}

impl ConstructionProposal {
    /// Carry an accepted sampled blueprint forward, without reconstructing it
    /// from its seed on a peer or after a generator update.
    pub fn from_blueprint(
        generator: String,
        blueprint: &wing_functions::generation::GeneratedBlueprint,
        operations: Vec<EvaluationRequest>,
    ) -> Self {
        Self {
            schema: CONSTRUCTION_SCHEMA,
            network_schema: NETWORK_SCHEMA_VERSION,
            generator,
            seed: blueprint.seed,
            network: blueprint.network.clone(),
            operations,
        }
    }
    /// Keep the public generator enum and its existing postcard variant ordering.
    pub fn to_gen_value(&self) -> Result<GenValue, ConstructionError> {
        self.check_header()?;
        let value =
            serde_json::to_string(self).map_err(|e| ConstructionError::Decode(e.to_string()))?;
        if value.len() > MAX_PAYLOAD_BYTES {
            return Err(ConstructionError::PayloadTooLarge);
        }
        Ok(GenValue::Object {
            fields: BTreeMap::from([
                (
                    "kind".into(),
                    GenValue::Text {
                        value: "wing_construction".into(),
                    },
                ),
                ("payload".into(), GenValue::Text { value }),
            ]),
        })
    }

    pub fn from_gen_value(value: &GenValue) -> Result<Self, ConstructionError> {
        let GenValue::Object { fields } = value else {
            return Err(ConstructionError::Envelope);
        };
        if fields.len() != 2
            || fields.get("kind")
                != Some(&GenValue::Text {
                    value: "wing_construction".into(),
                })
        {
            return Err(ConstructionError::Envelope);
        }
        let Some(GenValue::Text { value }) = fields.get("payload") else {
            return Err(ConstructionError::Envelope);
        };
        if value.len() > MAX_PAYLOAD_BYTES {
            return Err(ConstructionError::PayloadTooLarge);
        }
        let proposal: Self =
            serde_json::from_str(value).map_err(|e| ConstructionError::Decode(e.to_string()))?;
        proposal.check_header()?;
        Ok(proposal)
    }

    fn check_header(&self) -> Result<(), ConstructionError> {
        if self.schema != CONSTRUCTION_SCHEMA {
            return Err(ConstructionError::Schema(self.schema));
        }
        if self.network_schema != NETWORK_SCHEMA_VERSION {
            return Err(ConstructionError::NetworkSchema(self.network_schema));
        }
        if self.generator.trim().is_empty() {
            return Err(ConstructionError::MissingGenerator);
        }
        Ok(())
    }

    /// Inspect against today's body and world rules, including cumulative costs.
    /// Deserialization alone is never admission or authorization to execute.
    pub fn inspect(
        &self,
        live: &BTreeSet<PartRef>,
        rules: &WorldRules,
    ) -> Result<Vec<Evaluation>, ConstructionError> {
        self.check_header()?;
        self.network
            .clone()
            .evaluate_chain(&self.operations, live, rules)
            .map_err(ConstructionError::Execution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GenerationRecord, GeneratorRequest};
    use wing_functions::{Edge, Node, NodeId, NodeKind, Operator};

    fn proposal() -> ConstructionProposal {
        ConstructionProposal {
            schema: CONSTRUCTION_SCHEMA,
            network_schema: NETWORK_SCHEMA_VERSION,
            generator: "test/charged-arm".into(),
            seed: 9,
            network: FunctionalNetwork::new(
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
                                part: 1,
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
            .unwrap(),
            operations: vec![EvaluationRequest {
                operator: Operator::Strengthen,
                target: NodeId(1),
                cost: 3,
                range: 0,
                max_hops: 1,
            }],
        }
    }

    #[test]
    fn generation_record_preserves_construction_without_rerunning_generator() {
        let p = proposal();
        let record = GenerationRecord {
            id: "construction-9".into(),
            request: GeneratorRequest {
                generator: p.generator.clone(),
                args: GenValue::Text {
                    value: "creature".into(),
                },
                locks: BTreeMap::new(),
            },
            entropy: p.seed,
            proposal: p.to_gen_value().unwrap(),
        };
        record.validate(8).unwrap();
        let bytes = postcard::to_allocvec(&record).unwrap();
        let restored: GenerationRecord = postcard::from_bytes(&bytes).unwrap();
        let restored = ConstructionProposal::from_gen_value(&restored.proposal).unwrap();
        assert_eq!(restored, p);
        let live = BTreeSet::from([
            PartRef {
                subject: 1,
                part: 0,
            },
            PartRef {
                subject: 1,
                part: 1,
            },
        ]);
        let rules = WorldRules {
            allowed_operators: BTreeSet::from([Operator::Strengthen]),
            allowed_costs: BTreeSet::from([3]),
            max_range: 0,
            max_hops: 1,
        };
        assert!(restored.inspect(&live, &rules).is_ok());
        assert!(
            restored
                .inspect(
                    &BTreeSet::from([PartRef {
                        subject: 1,
                        part: 0
                    }]),
                    &rules
                )
                .is_err()
        );
        assert_eq!(restored, p, "preview must not spend charge");
    }

    #[test]
    fn foreign_network_version_is_refused_on_restore_and_inspection() {
        let mut p = proposal();
        p.network_schema += 1;
        let value = GenValue::Object {
            fields: BTreeMap::from([
                (
                    "kind".into(),
                    GenValue::Text {
                        value: "wing_construction".into(),
                    },
                ),
                (
                    "payload".into(),
                    GenValue::Text {
                        value: serde_json::to_string(&p).unwrap(),
                    },
                ),
            ]),
        };
        assert_eq!(
            ConstructionProposal::from_gen_value(&value),
            Err(ConstructionError::NetworkSchema(p.network_schema))
        );
        let rules = WorldRules {
            allowed_operators: BTreeSet::new(),
            allowed_costs: BTreeSet::new(),
            max_range: 0,
            max_hops: 0,
        };
        assert_eq!(
            p.inspect(&BTreeSet::new(), &rules),
            Err(ConstructionError::NetworkSchema(p.network_schema))
        );
    }

    #[test]
    fn version_and_size_are_checked_before_admission() {
        let mut p = proposal();
        p.schema += 1;
        assert!(matches!(
            p.to_gen_value(),
            Err(ConstructionError::Schema(_))
        ));
        p.schema = CONSTRUCTION_SCHEMA;
        p.generator = "x".repeat(MAX_PAYLOAD_BYTES);
        assert_eq!(p.to_gen_value(), Err(ConstructionError::PayloadTooLarge));
    }
}

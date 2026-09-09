// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Seeded functional blueprints. The caller supplies the body's part sites.
use crate::{
    Edge, Evaluation, EvaluationRequest, FunctionalNetwork, Node, NodeId, NodeKind, Operator,
    PartRef, WorldRules,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const GENERATOR_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Form {
    Creature,
    Staff,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SiteRole {
    Source,
    Store,
    Gate,
    Actuator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BodySite {
    pub part: PartRef,
    pub role: SiteRole,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneratorSettings {
    pub version: u32,
    pub forms: Vec<Form>,
    pub min_actuators: u32,
    pub max_actuators: u32,
    pub source_capacity: u64,
    pub store_capacity: u64,
    pub edge_capacity: u64,
    pub gate_open: bool,
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            version: GENERATOR_VERSION,
            forms: vec![Form::Creature, Form::Staff],
            min_actuators: 1,
            max_actuators: 4,
            source_capacity: 100,
            store_capacity: 100,
            edge_capacity: 50,
            gate_open: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratedBlueprint {
    pub version: u32,
    pub seed: u64,
    pub form: Form,
    pub actuators: Vec<PartRef>,
    pub network: FunctionalNetwork,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rejection {
    pub form: Form,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub blueprint: GeneratedBlueprint,
    pub rejections: Vec<Rejection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GenerationBatch {
    pub settings: GeneratorSettings,
    pub seed: u64,
    pub candidates: Vec<Candidate>,
    pub rejected: Vec<Rejection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TechniquePreview {
    pub request: EvaluationRequest,
    pub result: Result<Evaluation, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreviewSettings {
    pub cost: u64,
    pub range: u32,
    pub max_hops: u32,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            cost: 10,
            range: 1,
            max_hops: 3,
        }
    }
}

/// Generate one deterministic candidate for each admitted form. Part identities
/// are opaque and remain owned by the body/document adapter.
pub fn generate(seed: u64, settings: &GeneratorSettings, sites: &[BodySite]) -> GenerationBatch {
    let mut batch = GenerationBatch {
        settings: settings.clone(),
        seed,
        candidates: Vec::new(),
        rejected: Vec::new(),
    };
    if settings.version != GENERATOR_VERSION {
        batch.rejected.push(Rejection {
            form: Form::Creature,
            reason: format!("unsupported generator version {}", settings.version),
        });
        return batch;
    }
    if settings.forms.is_empty() || settings.forms.len() > 16 {
        batch.rejected.push(Rejection {
            form: Form::Creature,
            reason: "form selection must contain 1..16 forms".into(),
        });
        return batch;
    }
    if sites.len() > 256 || settings.max_actuators > 61 {
        batch.rejected.push(Rejection {
            form: Form::Creature,
            reason: "body site or actuator bound exceeds supported network size".into(),
        });
        return batch;
    }
    let mut seen_forms = BTreeSet::new();
    for form in settings.forms.iter().copied() {
        if !seen_forms.insert(form) {
            batch.rejected.push(Rejection {
                form,
                reason: "duplicate form selection".into(),
            });
            continue;
        }
        let candidate_seed = splitmix(seed ^ form_tag(form));
        match build(candidate_seed, form, settings, sites) {
            Ok(blueprint) => batch.candidates.push(Candidate {
                blueprint,
                rejections: Vec::new(),
            }),
            Err(reason) => batch.rejected.push(Rejection { form, reason }),
        }
    }
    batch
}

fn build(
    seed: u64,
    form: Form,
    s: &GeneratorSettings,
    sites: &[BodySite],
) -> Result<GeneratedBlueprint, String> {
    if s.min_actuators == 0 || s.min_actuators > s.max_actuators {
        return Err("actuator range is unordered".into());
    }
    if s.edge_capacity == 0 || s.source_capacity == 0 {
        return Err("capacities must be positive".into());
    }
    if s.max_actuators > 61 {
        return Err("actuator maximum exceeds network node bound".into());
    }
    let mut seen_sites = BTreeSet::new();
    for site in sites {
        if !seen_sites.insert((site.part, site.role as u8)) {
            return Err("duplicate body site".into());
        }
    }
    let mut sources = sites
        .iter()
        .filter(|x| x.role == SiteRole::Source)
        .map(|x| x.part);
    let source = sources.next().ok_or("missing source site")?;
    let store = sites
        .iter()
        .find(|x| x.role == SiteRole::Store)
        .map(|x| x.part);
    let gate = sites
        .iter()
        .find(|x| x.role == SiteRole::Gate)
        .map(|x| x.part);
    let mut actuators: Vec<_> = sites
        .iter()
        .filter(|x| x.role == SiteRole::Actuator)
        .map(|x| x.part)
        .collect();
    let want = match form {
        Form::Staff => 1,
        Form::Creature => {
            s.min_actuators + (splitmix(seed) as u32 % (s.max_actuators - s.min_actuators + 1))
        },
    };
    if want < s.min_actuators || want > s.max_actuators {
        return Err("actuator range cannot be realized by supplied sites".into());
    }
    actuators.truncate(want as usize);
    if actuators.len() != want as usize {
        return Err(format!(
            "need {want} actuator sites, found {}",
            actuators.len()
        ));
    }
    let store = store.ok_or("missing store site")?;
    let gate = gate.ok_or("missing gate site")?;
    let mut nodes = vec![
        Node {
            id: NodeId(0),
            kind: NodeKind::Source {
                part: source,
                capacity: s.source_capacity,
                charge: s.source_capacity,
            },
        },
        Node {
            id: NodeId(1),
            kind: NodeKind::Store {
                part: store,
                capacity: s.store_capacity,
                charge: 0,
            },
        },
        Node {
            id: NodeId(2),
            kind: NodeKind::Gate {
                part: gate,
                open: s.gate_open,
            },
        },
    ];
    for (i, part) in actuators.iter().copied().enumerate() {
        nodes.push(Node {
            id: NodeId(3 + i as u32),
            kind: NodeKind::Effect { part },
        });
    }
    let mut edges = vec![
        Edge {
            from: NodeId(0),
            to: NodeId(1),
            capacity: s.edge_capacity,
        },
        Edge {
            from: NodeId(1),
            to: NodeId(2),
            capacity: s.edge_capacity,
        },
    ];
    for i in 0..actuators.len() {
        edges.push(Edge {
            from: NodeId(2),
            to: NodeId(3 + i as u32),
            capacity: s.edge_capacity,
        });
    }
    let network = FunctionalNetwork::new(nodes, edges).map_err(|e| e.to_string())?;
    Ok(GeneratedBlueprint {
        version: GENERATOR_VERSION,
        seed,
        form,
        actuators,
        network,
    })
}

/// Evaluate every supported operator against every node, retaining refusals.
pub fn preview_techniques(
    blueprint: &GeneratedBlueprint,
    live: &BTreeSet<PartRef>,
    rules: &WorldRules,
    preview: &PreviewSettings,
) -> Vec<TechniquePreview> {
    let mut out = Vec::new();
    for (&id, node) in &blueprint.network.nodes {
        let operators: &[Operator] = match node.kind {
            NodeKind::Store { .. } => &[Operator::Store],
            NodeKind::Effect { .. } => &[Operator::Strengthen, Operator::Project],
            _ => &[],
        };
        for &operator in operators {
            let range = if operator == Operator::Project {
                preview.range
            } else {
                0
            };
            let request = EvaluationRequest {
                operator,
                target: id,
                cost: preview.cost,
                range,
                max_hops: preview.max_hops,
            };
            let result = blueprint
                .network
                .preview(&request, live, rules)
                .map_err(|e| e.to_string());
            out.push(TechniquePreview { request, result });
        }
    }
    out
}

fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn form_tag(form: Form) -> u64 {
    match form {
        Form::Creature => 0x4352_4541_5455_5245,
        Form::Staff => 0x5354_4146_4600_0001,
    }
}

#[cfg(test)]
mod tests;

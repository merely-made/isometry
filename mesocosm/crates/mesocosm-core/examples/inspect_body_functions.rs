// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Join the existing body generator to functional construction and body loss.
//! Run with `cargo run -p mesocosm-core --example inspect_body_functions`.
use mesocosm_core::{PartPalette, functions, world::generation::Request};
use std::collections::BTreeSet;
use wing_functions::generation::{BodySite, Form, GeneratorSettings, SiteRole};
use wing_functions::{Operator, PartRef, WorldRules};

fn main() {
    let request = Request {
        candidates: 1,
        ..Request::default()
    };
    let draft = request
        .preview(PartPalette::default())
        .expect("body generation");
    let mut body = draft
        .candidates
        .first()
        .expect("admitted creature")
        .body
        .clone();
    let parts: Vec<_> = body.living().map(|p| p.id).collect();
    assert!(
        parts.len() >= 4,
        "this inspection profile requires four sites"
    );
    // An explicit inspection profile. Material/tissue admission is subsequent
    // construction work; the sampler must not infer magical tissue from shape.
    let sites: Vec<_> = parts
        .iter()
        .enumerate()
        .map(|(i, p)| BodySite {
            part: PartRef {
                subject: 1,
                part: p.0,
            },
            role: match i {
                0 => SiteRole::Source,
                1 => SiteRole::Store,
                2 => SiteRole::Gate,
                _ => SiteRole::Actuator,
            },
        })
        .collect();
    let settings = GeneratorSettings {
        forms: vec![Form::Creature],
        max_actuators: 4.min(parts.len() as u32 - 3),
        ..GeneratorSettings::default()
    };
    let batch = functions::generate_for_body(1, &body, 7, &settings, &sites).unwrap();
    let candidate = &batch
        .candidates
        .first()
        .expect("functional candidate")
        .blueprint;
    let rules = WorldRules {
        allowed_operators: BTreeSet::from([
            Operator::Strengthen,
            Operator::Project,
            Operator::Store,
        ]),
        allowed_costs: BTreeSet::from([10]),
        max_range: 5,
        max_hops: 8,
    };
    let commands: Vec<_> = candidate
        .network
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            matches!(node.kind, wing_functions::NodeKind::Effect { .. }).then_some(
                wing_functions::EvaluationRequest {
                    operator: Operator::Strengthen,
                    target: *id,
                    cost: 10,
                    range: 0,
                    max_hops: 8,
                },
            )
        })
        .collect();
    let before: Vec<_> = commands
        .iter()
        .map(|command| {
            candidate
                .network
                .preview(command, &functions::live_parts(1, &body), &rules)
                .map_err(|e| e.to_string())
        })
        .collect();
    let removed = candidate.actuators[0];
    body.sever(mesocosm_core::PartId(removed.part));
    let after: Vec<_> = commands
        .iter()
        .map(|command| {
            candidate
                .network
                .preview(command, &functions::live_parts(1, &body), &rules)
                .map_err(|e| e.to_string())
        })
        .collect();
    assert!(before.iter().any(Result::is_ok));
    assert!(after.iter().any(Result::is_err));
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "body_seed": draft.candidates[0].seed, "functional_seed": candidate.seed,
            "removed": removed, "before": before, "after": after,
            "body": body, "construction": candidate,
        }))
        .unwrap()
    );
}

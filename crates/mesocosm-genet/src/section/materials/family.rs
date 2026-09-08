// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Joined family evidence: one runtime, recorded intents and admitted content.

use super::Capture;
use crate::played::{BodyLayout, PlayedTrace, SceneMode};
use mesocosm_core::{Intent, OrganismId};
use mesocosm_mesh::content::ContentPack;
use mesocosm_runtime::Runtime;

fn apply(runtime: &mut Runtime, intent: Intent) -> mesocosm_core::Outcome {
    runtime.queue(intent);
    assert_eq!(
        runtime.step(1),
        1,
        "the recorded choice answers the checkpoint"
    );
    runtime.last_outcomes()[0]
}

fn record_stage(
    runtime: &Runtime,
    pack: &ContentPack,
    capture: &mut Capture,
    label: &str,
    subjects: &[OrganismId],
) {
    let hash = runtime.state_hash();
    let directory = std::env::var("MESOCOSM_FAMILY_RECEIPTS").ok();
    let mut bodies = Vec::new();
    for &subject in subjects {
        let organism = runtime
            .world()
            .organisms
            .iter()
            .find(|o| o.id == subject)
            .unwrap();
        let materials = super::super::project(&organism.phenotype, runtime.world().ruleset());
        let (pixels, _) = capture.body(runtime.world(), subject, false);
        if let Some(directory) = &directory {
            let path = std::path::Path::new(directory).join(format!("{label}-{}.png", subject.0));
            let mut encoder = png::Encoder::new(std::fs::File::create(path).unwrap(), 512, 512);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&pixels)
                .unwrap();
        }
        bodies.push(serde_json::json!({
            "subject": subject.0,
            "body": organism.body(),
            "species": organism.species,
            "alive": organism.is_alive(),
            "reserve_mg": organism.energy_mg,
            "phenotype_digest": organism.phenotype.digest(),
            "secretory_parts": materials.iter().filter(|m| m.process == mesocosm_core::process::Process::Secrete).count(),
        }));
    }
    assert_eq!(runtime.state_hash(), hash, "rendering is read-only");
    if let Some(directory) = directory {
        let directory = std::path::Path::new(&directory);
        let trace = PlayedTrace {
            start: None,
            trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
            scene: SceneMode::FamilyPractice,
            body_layout: BodyLayout::Spaced,
            seed: 7,
            organisms: runtime.receipt().organisms,
            steps: runtime.trace().len() as u64,
            state_hash: hash,
            intents: runtime.trace().to_vec(),
            content: Some(pack.clone()),
        };
        std::fs::write(
            directory.join(format!("{label}.trace.json")),
            serde_json::to_vec_pretty(&trace).unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("{label}.json")),
            serde_json::to_vec_pretty(&serde_json::json!({
                "stage": label, "tick": runtime.world().tick,
                "state_hash": format!("{hash:016x}"), "bodies": bodies,
                "history": runtime.history(), "assisted_origin": true,
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn family_scene_replays_intake_expression_and_a_natural_descendant() {
    use mesocosm_core::{Founding, Outcome, history::Event, process::Process};
    let pack = ContentPack::generate(Founding::SpacedRoster.palette()).unwrap();
    let mut runtime =
        Runtime::family_practice(7, 10, Founding::SpacedRoster, pack.palette).unwrap();
    let ids = runtime.world().family_practice_ids().unwrap();
    let opening = runtime.world().family_practice_opening().unwrap();
    let original_program = runtime
        .world()
        .lineages()
        .get(runtime.world().controlled().unwrap().species)
        .unwrap()
        .program()
        .digest();
    let initial_matter = runtime.world().total_matter_mg();
    let Some(mut capture) = Capture::with_volumes(pack.resolve().unwrap()) else {
        return;
    };
    let mut refused = runtime.world().clone();
    let mut elapsed = refused.clone();
    assert!(matches!(
        refused.apply(Intent::Consume {
            organism: ids.donor,
            part: mesocosm_core::PartId(u32::MAX),
        }),
        Outcome::Rejected(_)
    ));
    elapsed.apply(Intent::Resume);
    assert_eq!(
        mesocosm_core::state_hash(&refused),
        mesocosm_core::state_hash(&elapsed)
    );
    let (refused_pixels, _) = capture.body(&refused, ids.parent, false);
    let (elapsed_pixels, uploads) = capture.body(&elapsed, ids.parent, false);
    assert_eq!(refused_pixels, elapsed_pixels);
    assert_eq!(uploads, 0, "refusal introduces no geometry upload");
    record_stage(
        &runtime,
        &pack,
        &mut capture,
        "origin",
        &[ids.parent, ids.relative],
    );
    let names = ["intake", "graft", "expression", "boundary", "revision"];
    for (index, intent) in opening.into_iter().enumerate() {
        let outcome = apply(&mut runtime, intent);
        assert!(
            match index {
                0 => matches!(outcome, Outcome::Consumed { .. }),
                1 => matches!(outcome, Outcome::Grafted { .. }),
                2 => matches!(outcome, Outcome::Expressed { .. }),
                3 => matches!(outcome, Outcome::EpochEnded { .. }),
                4 => matches!(outcome, Outcome::Revised { .. }),
                _ => false,
            },
            "{}: {outcome:?}",
            names[index]
        );
        if index < 3 {
            assert_eq!(
                original_program,
                runtime
                    .world()
                    .lineages()
                    .get(runtime.world().controlled().unwrap().species)
                    .unwrap()
                    .program()
                    .digest()
            );
        }
        record_stage(
            &runtime,
            &pack,
            &mut capture,
            names[index],
            &[ids.parent, ids.relative],
        );
    }
    let Outcome::Revised { revision, .. } = runtime.last_outcomes()[0] else {
        panic!("the opening ends with an accepted lineage revision");
    };
    let mut child = None;
    for intent in runtime.world().family_practice_birth_run() {
        apply(&mut runtime, intent);
        child = runtime
            .history()
            .log()
            .entries()
            .iter()
            .find_map(|entry| match entry.record {
                Event::Born {
                    organism,
                    parent: Some(parent),
                    ..
                } if parent == ids.parent && organism != ids.relative && organism != ids.donor => {
                    Some(organism)
                },
                _ => None,
            });
        if child.is_some() {
            break;
        }
    }
    let child = child.expect("recorded parent bears through ordinary ecology");
    assert!(runtime.history().log().entries().iter().any(|entry| matches!(entry.record,
        Event::Born { organism, parent: Some(parent), .. } if organism == ids.relative && parent == ids.parent)));
    assert!(
        runtime
            .history()
            .log()
            .entries()
            .iter()
            .any(|entry| matches!(entry.record,
        Event::Inherited { organism, revision: inherited, .. } if organism == child && inherited == revision))
    );
    assert!(!runtime.trace().iter().any(|intent| matches!(
        intent,
        Intent::ForceBirth { .. } | Intent::Kill { .. } | Intent::PlaceMatter { .. }
    )));
    assert_eq!(runtime.world().total_matter_mg(), initial_matter);
    let relative = runtime
        .world()
        .organisms
        .iter()
        .find(|o| o.id == ids.relative)
        .unwrap();
    assert!(relative.is_alive());
    assert!(
        !super::super::project(&relative.phenotype, runtime.world().ruleset())
            .iter()
            .any(|m| m.process == Process::Secrete)
    );
    assert!(!runtime.history().log().entries().iter().any(|entry| matches!(entry.record,
        Event::Expressed { organism, .. } | Event::Grafted { organism, .. } if organism == ids.relative)));
    let offspring = runtime
        .world()
        .organisms
        .iter()
        .find(|o| o.id == child)
        .unwrap();
    assert!(
        offspring
            .body()
            .living()
            .all(|part| { matches!(part.provenance.origin, mesocosm_core::Origin::Founding) }),
        "a descendant develops its own tissue rather than copying the acquired branch"
    );
    assert!(
        super::super::project(&offspring.phenotype, runtime.world().ruleset())
            .iter()
            .any(|m| m.process == Process::Secrete)
    );
    record_stage(
        &runtime,
        &pack,
        &mut capture,
        "descendant",
        &[ids.parent, ids.relative, child],
    );
    let mut replay = Runtime::family_practice(7, 10, Founding::SpacedRoster, pack.palette).unwrap();
    for intent in runtime.trace() {
        apply(&mut replay, intent.clone());
    }
    assert_eq!(replay.state_hash(), runtime.state_hash());
    assert_eq!(replay.history(), runtime.history());
}

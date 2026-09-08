// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Host, played::PlayedTrace};
use mesocosm_core::{Intent, state_hash};
use mesocosm_mesh::mesh_body;

#[test]
fn incompatible_recording_is_refused_before_founding_or_content_resolution() {
    let legacy: PlayedTrace =
        serde_json::from_str(r#"{"seed":7,"organisms":3,"steps":0,"state_hash":0,"intents":[]}"#)
            .unwrap();
    assert_eq!(legacy.trophic_grammar, 0);
    for revision in [0, mesocosm_core::TROPHIC_GRAMMAR_REVISION + 1] {
        let trace = PlayedTrace {
            start: None,
            trophic_grammar: revision,
            ..legacy.clone()
        };
        let result = start(&HostConfig {
            replay: Some(trace),
            ..HostConfig::default()
        });
        assert!(matches!(result, Err(why) if why.contains("trophic grammar revision")));
    }
}

#[test]
fn family_practice_scene_roundtrips_and_overrides_the_host_scene() {
    let trace: PlayedTrace = serde_json::from_str(
        r#"{"scene":"family_practice","seed":7,"organisms":1,"steps":0,"state_hash":0,"intents":[]}"#,
    )
    .unwrap();
    assert_eq!(trace.scene, crate::played::SceneMode::FamilyPractice);
    let saved = serde_json::to_vec(&trace).unwrap();
    let restored: PlayedTrace = serde_json::from_slice(&saved).unwrap();
    let config = HostConfig {
        scene: crate::played::SceneMode::Ecology,
        replay: Some(restored),
        ..HostConfig::default()
    };
    assert_eq!(
        config.effective_scene(),
        crate::played::SceneMode::FamilyPractice
    );
}

fn same_mesh(a: mesocosm_mesh::BodyMesh, b: mesocosm_mesh::BodyMesh) {
    assert_eq!(a.placements, b.placements);
    assert_eq!(a.mesh_count(), b.mesh_count());
    for placement in &a.placements {
        assert_eq!(a.mesh_for(placement.volume), b.mesh_for(placement.volume));
    }
}

#[test]
fn generated_selection_replays_with_saved_content_and_ignores_current_start() {
    use mesocosm_core::world::generation::{Request, Selection};
    assert!(serde_json::from_str::<Request>(r#"{"criteria":{"wings":true}}"#).is_err());
    assert!(serde_json::from_str::<Request>(r#"{"temperature":20}"#).is_err());
    let selection = Selection {
        request: Request::default(),
        candidate: 1,
    };
    let config = HostConfig {
        start: Some(selection.clone()),
        ..HostConfig::default()
    };
    let (mut live, pack, volumes) = start(&config).unwrap();
    let expected = selection
        .request
        .preview(pack.as_ref().unwrap().palette)
        .unwrap();
    assert_eq!(live.world().body().unwrap(), &expected.candidates[1].body);
    for _ in 0..20 {
        live.queue(Intent::Idle);
        live.advance(100_000);
    }
    let trace = PlayedTrace {
        start: Some(selection.clone()),
        trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        scene: crate::played::SceneMode::Ecology,
        body_layout: config.body_layout,
        seed: selection.request.seed,
        organisms: selection.request.organisms,
        steps: live.trace().len() as u64,
        state_hash: live.state_hash(),
        intents: live.trace().to_vec(),
        content: pack,
    };
    let saved: PlayedTrace = serde_json::from_slice(&serde_json::to_vec(&trace).unwrap()).unwrap();
    let mut host = Host::new(HostConfig {
        start: Some(Selection {
            request: Request {
                version: 99,
                ..Request::default()
            },
            candidate: 99,
        }),
        generated_content: false,
        replay: Some(saved),
        ..HostConfig::default()
    });
    while !host.advance() {}
    assert_eq!(host.runtime.state_hash(), trace.state_hash);
    assert_eq!(host.config.organisms, selection.request.organisms);
    same_mesh(
        mesh_body(live.world().body().unwrap(), &volumes).unwrap(),
        mesh_body(host.runtime.world().body().unwrap(), &host.volumes).unwrap(),
    );
    let mut bad = trace;
    bad.seed += 1;
    assert!(bad.validate_rules().is_err());
}

#[test]
fn recipe_json_accepts_legacy_fields_and_rejects_duplicate_authority() {
    use mesocosm_core::axis::Recipe;
    let legacy = r#"{"tagmata":[{"segments":3,"appendage":"None","per_segment":0,"segment_shape":0,"appendage_shape":0}],"variance":1,"lexicon":["None","Mouth"]}"#;
    let recipe: Recipe = serde_json::from_str(legacy).unwrap();
    assert_eq!(recipe, Recipe::founding(3));
    let duplicate = legacy.replace("\"variance\":1", "\"variance\":1,\"variance\":2");
    assert!(serde_json::from_str::<Recipe>(&duplicate).is_err());
    let branching = mesocosm_core::axis::archetype::branching::producer_shrub();
    let decoded: Recipe = serde_json::from_slice(&serde_json::to_vec(&branching).unwrap()).unwrap();
    assert_eq!(decoded, branching);
    let jointed = mesocosm_core::axis::archetype::jointed::consumer_browser();
    let json = serde_json::to_string(&jointed).unwrap();
    assert_eq!(serde_json::from_str::<Recipe>(&json).unwrap(), jointed);
    let duplicate = json.replace(
        "\"appendage_chains\":",
        "\"appendage_chains\":[],\"appendage_chains\":",
    );
    assert!(serde_json::from_str::<Recipe>(&duplicate).is_err());
}

#[test]
fn recorded_content_replays_without_the_current_generation_setting() {
    let config = HostConfig {
        seed: 7,
        organisms: 40,
        ..HostConfig::default()
    };
    let (mut live, pack, volumes) = start(&config).unwrap();
    for _ in 0..24 {
        live.queue(Intent::Idle);
        live.advance(100_000);
    }
    let trace = PlayedTrace {
        start: None,
        trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        scene: crate::played::SceneMode::Ecology,
        body_layout: config.body_layout,
        seed: config.seed,
        organisms: config.organisms,
        steps: live.trace().len() as u64,
        state_hash: live.state_hash(),
        intents: live.trace().to_vec(),
        content: pack,
    };
    let saved = serde_json::to_vec(&trace).unwrap();
    let restored: PlayedTrace = serde_json::from_slice(&saved).unwrap();
    let mut host = Host::new(HostConfig {
        generated_content: false,
        body_layout: crate::played::BodyLayout::Axial,
        replay: Some(restored),
        ..config
    });
    while !host.advance() {}
    assert_eq!(host.runtime.state_hash(), trace.state_hash);
    assert!(host.content.is_some());
    for organism in &live.world().organisms {
        let other = host
            .runtime
            .world()
            .organisms
            .iter()
            .find(|o| o.id == organism.id)
            .unwrap();
        same_mesh(
            mesh_body(organism.body(), &volumes).unwrap(),
            mesh_body(other.body(), &host.volumes).unwrap(),
        );
    }
}

#[test]
fn historical_recordings_keep_their_original_palette() {
    let trace = crate::played::record_demo(7, 40, 10, 24);
    let bytes = serde_json::to_vec(&trace).unwrap();
    let trace: PlayedTrace = serde_json::from_slice(&bytes).unwrap();
    let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
    let mut host = Host::new(HostConfig {
        seed: 7,
        organisms: 40,
        replay: Some(trace),
        ..HostConfig::default()
    });
    while !host.advance() {}
    assert!(host.content.is_none());
    assert_eq!(host.runtime.state_hash(), state_hash(&world));
}

#[test]
fn a_bad_saved_pack_is_refused_without_falling_back_to_fixtures() {
    let mut trace = crate::played::record_demo(7, 40, 10, 0);
    let mut pack = ContentPack::generate(mesocosm_core::Founding::Roster.palette()).unwrap();
    pack.version = u16::MAX;
    trace.content = Some(pack);
    let result = start(&HostConfig {
        seed: 7,
        organisms: 40,
        generated_content: false,
        replay: Some(trace),
        ..HostConfig::default()
    });
    assert!(matches!(result, Err(why) if why.contains("UnknownVersion")));
}

#[test]
fn snapshot_and_saved_content_recover_the_same_anatomy_and_voxels() {
    let (runtime, pack, volumes) = start(&HostConfig {
        organisms: 40,
        ..HostConfig::default()
    })
    .unwrap();
    let snapshot = mesocosm_core::snapshot(runtime.world()).unwrap();
    let world = mesocosm_core::restore(&snapshot).unwrap();
    assert_eq!(state_hash(&world), runtime.state_hash());
    let bytes = serde_json::to_vec(&pack.unwrap()).unwrap();
    let saved: ContentPack = serde_json::from_slice(&bytes).unwrap();
    let restored = saved.resolve_for(world.development_palette()).unwrap();
    assert!(
        saved
            .resolve_for(mesocosm_core::Founding::Roster.palette())
            .is_err()
    );
    assert_eq!(world.development_palette(), saved.palette);
    for line in world.lineages().all() {
        assert_eq!(
            line.recipe,
            runtime.world().lineages().get(line.id).unwrap().recipe
        );
    }
    for organism in &world.organisms {
        same_mesh(
            mesh_body(organism.body(), &volumes).unwrap(),
            mesh_body(organism.body(), &restored).unwrap(),
        );
    }
}

#[test]
fn changing_surface_content_preserves_all_other_world_facts_through_play() {
    let recorded = crate::played::record_demo(7, 40, 10, 120);
    let config = HostConfig {
        body_layout: crate::played::BodyLayout::Axial,
        seed: 7,
        organisms: 40,
        ..HostConfig::default()
    };
    let (mut generated, pack, _) = start(&config).unwrap();
    for intent in &recorded.intents {
        generated.queue(intent.clone());
        generated.advance(100_000);
    }
    let (baseline, _) = Runtime::replay(7, 40, &recorded.intents);
    let base_palette = baseline.development_palette();
    let mapping: Vec<_> = pack
        .unwrap()
        .entries
        .iter()
        .map(|entry| {
            (
                entry.reference.0,
                base_palette.template_at(entry.role, entry.slot).volume.0,
            )
        })
        .collect();
    // Postcard stores VolumeRef's fixed [u8;32] verbatim. Normalize only these
    // known cryptographic addresses, leaving every other snapshot byte intact.
    // JSON cannot encode the ground's coordinate-array map keys.
    let mut generated_facts = mesocosm_core::snapshot(generated.world()).unwrap();
    let mut offset = 0;
    let mut replacements = 0;
    while offset + 32 <= generated_facts.len() {
        if let Some((_, replacement)) = mapping
            .iter()
            .find(|(original, _)| generated_facts[offset..offset + 32] == original[..])
        {
            generated_facts[offset..offset + 32].copy_from_slice(replacement);
            offset += 32;
            replacements += 1;
        } else {
            offset += 1;
        }
    }
    assert!(replacements >= mapping.len());
    let baseline_facts = mesocosm_core::snapshot(&baseline).unwrap();
    assert_eq!(generated_facts.len(), baseline_facts.len());
    let difference = generated_facts
        .iter()
        .zip(&baseline_facts)
        .position(|(a, b)| a != b);
    assert_eq!(
        difference, None,
        "world facts changed beyond admitted volume addresses"
    );
}

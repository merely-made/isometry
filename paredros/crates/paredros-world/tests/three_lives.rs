// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::{Origin, PartId};
use paredros_world::fixtures::three_lives as fixture;
use paredros_world::{
    ActionBlocker, BindingBlocker, PartFunction, SourceQuery, SubjectBody, TechniqueId, arrest_fall,
};

#[test]
fn shared_line_keeps_lost_addresses_and_identifies_symbiont_donor() {
    let lives = fixture::three_lives();
    for life in &lives {
        assert_eq!(life.body.species, fixture::WETLAND_LINE);
        assert!(life.body.is_living(fixture::MANTLE));
        assert!(life.body.is_living(fixture::FIBRE_GLAND));
        assert!(fixture::facts_have_sources(life) && fixture::source_order_is_valid(life));
    }
    assert_eq!(lives[0].body, lives[1].body);
    assert_ne!(lives[0].subject, lives[1].subject);
    assert_ne!(lives[0].revision, lives[2].revision);
    assert!(!lives[2].body.is_living(PartId(1)));
    assert!(lives[2].body.is_living(PartId(2)));
    let donor = fixture::symbiont_donor();
    assert_eq!(
        lives[2]
            .body
            .part(fixture::ADHESIVE_SYMBIONT)
            .unwrap()
            .provenance
            .origin,
        Origin::Incorporated {
            from_species: donor.species,
            from_part: donor.root
        }
    );
    assert!(donor.is_living(donor.root));
}

#[test]
fn chronology_rejects_missing_future_early_and_wrong_place_sources() {
    let life = &fixture::three_lives()[0];
    let fact = life.facts[1];
    for changed in [
        fixture::StartingFact {
            source_id: "missing",
            ..fact
        },
        fixture::StartingFact {
            acquired_at: fixture::NOW + 1,
            ..fact
        },
        fixture::StartingFact {
            acquired_at: 1,
            ..fact
        },
        fixture::StartingFact {
            acquired_place: "elsewhere",
            ..fact
        },
    ] {
        assert!(!fixture::fact_has_source(life, &changed));
    }
}

#[test]
fn example_inputs_explain_different_lives_without_mutation() {
    let lives = fixture::three_lives();
    for (index, life) in lives.iter().enumerate() {
        let inputs = fixture::inputs(life);
        let knowledge = fixture::knowledge(life);
        let before = (inputs.clone(), knowledge.clone(), life.body.clone());
        let query = arrest_fall(
            &knowledge,
            SubjectBody {
                subject: life.subject,
                revision: life.revision,
                body: &life.body,
            },
            life.revision,
            &inputs,
        );
        assert_eq!((inputs, knowledge, life.body.clone()), before);
        match index {
            0 => {
                assert!(query.bindings[0].available() && query.bindings[2].available());
                assert!(
                    query.bindings[0]
                        .sources
                        .contains(&SourceQuery::Part(PartId(2)))
                );
            },
            1 => {
                assert!(
                    query
                        .blockers
                        .contains(&ActionBlocker::NotLearned(TechniqueId::ArrestFall))
                );
                assert!(query.bindings.iter().all(|b| !b.available()));
            },
            _ => {
                assert!(query.bindings[0].available() && query.bindings[1].available());
                assert!(
                    query.bindings[1]
                        .sources
                        .contains(&SourceQuery::Part(fixture::ADHESIVE_SYMBIONT))
                );
            },
        }
    }
}

#[test]
fn missing_grip_does_not_erase_knowledge_or_disable_adhesion() {
    let life = &fixture::three_lives()[2];
    let mut inputs = fixture::inputs(life);
    inputs
        .part_capabilities
        .retain(|p| p.function != PartFunction::Grip || p.part == PartId(1));
    let query = arrest_fall(
        &fixture::knowledge(life),
        SubjectBody {
            subject: life.subject,
            revision: life.revision,
            body: &life.body,
        },
        life.revision,
        &inputs,
    );
    assert!(
        query.bindings[0]
            .blockers
            .contains(&BindingBlocker::SeveredPart(PartId(1)))
    );
    assert!(query.bindings[1].available());
    assert!(
        fixture::knowledge(life)
            .learned
            .contains(&TechniqueId::ArrestFall)
    );
}

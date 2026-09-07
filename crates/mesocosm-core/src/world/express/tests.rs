// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::body::{Attachment, Provenance, VolumeRef, Yaw};
use crate::discovery::{ConditionId, Evidence, HUNGER_TICKS, Stress};
use crate::phenotype::Refusal;
use crate::process::DefinitionDigest;
use crate::{Outcome, Rejection};

use super::super::World;

fn ready_world() -> (World, ConditionId) {
    let condition = crate::discovery::conditions()
        .into_iter()
        .find(|condition| condition.name == "mesocosm:endured-hunger")
        .expect("the native rules name hunger")
        .id();
    let mut world = World::new(4_242, 0);
    world.observe(Evidence::Endured {
        stress: Stress::Hunger,
        ticks: HUNGER_TICKS,
    });
    let recipient = world.controlled_id().expect("world starts embodied");
    let organism = world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == recipient)
        .expect("the recipient is in the roster");
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            [6, 4, 1],
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("the fixture's plate attaches");
    (world, condition)
}

fn assert_preview_kept_world(
    world: &World,
    before_hash: u64,
    before_flows: Vec<crate::RecordedFlow>,
) {
    assert_eq!(
        crate::state_hash(world),
        before_hash,
        "a preview moved world state"
    );
    assert_eq!(
        world.flows().to_vec(),
        before_flows,
        "a preview published a flow"
    );
}

#[test]
fn preview_is_pure_and_commit_lands_its_checked_candidate() {
    let (mut world, condition) = ready_world();
    let before_hash = crate::state_hash(&world);
    let before_flows = world.flows().to_vec();
    let preview = world
        .preview_expression(condition)
        .expect("the gland is viable");
    assert_preview_kept_world(&world, before_hash, before_flows);

    assert_eq!(preview.recipient, world.controlled_id().expect("embodied"));
    assert_eq!(preview.condition, condition);
    let outcome = world.express(condition);
    assert_eq!(
        outcome,
        Outcome::Expressed {
            part: preview.part,
            cost_mg: preview.cost_mg,
            revision: preview.revision,
        }
    );
    assert_eq!(
        world.phenotype().expect("still embodied"),
        &preview.phenotype,
        "commit must publish the preview's checked phenotype"
    );
}

#[test]
fn unavailable_and_rejected_previews_do_not_mutate_the_world_or_lineage() {
    let (mut world, condition) = ready_world();
    let species = world.controlled().expect("embodied").species;
    let program = world
        .lineages
        .get(species)
        .expect("line exists")
        .program()
        .clone();

    let before_hash = crate::state_hash(&world);
    let before_flows = world.flows().to_vec();
    assert_eq!(
        world.preview_expression(ConditionId(u64::MAX)),
        Err(Rejection::Undiscovered(ConditionId(u64::MAX)))
    );
    assert_preview_kept_world(&world, before_hash, before_flows);

    world
        .discoveries
        .iter_mut()
        .find(|discovery| discovery.condition == condition)
        .expect("the fixture discovered hunger")
        .candidate
        .process
        .definition = DefinitionDigest(0);
    let before_hash = crate::state_hash(&world);
    let before_flows = world.flows().to_vec();
    assert!(matches!(
        world.preview_expression(condition),
        Err(Rejection::Refused(Refusal::UnknownProcess(_)))
    ));
    assert_preview_kept_world(&world, before_hash, before_flows);
    assert_eq!(
        world.lineages.get(species).expect("line exists").program(),
        &program,
        "an expression preview must not alter descendant programming"
    );
}

#[test]
fn an_unaffordable_preview_leaves_everything_unpublished() {
    let (mut world, condition) = ready_world();
    let recipient = world.controlled_id().expect("embodied");
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == recipient)
        .expect("recipient remains")
        .energy_mg = 0;
    let species = world.controlled().expect("embodied").species;
    let program = world
        .lineages
        .get(species)
        .expect("line exists")
        .program()
        .clone();
    let before_hash = crate::state_hash(&world);
    let before_flows = world.flows().to_vec();

    assert_eq!(
        world.preview_expression(condition),
        Err(Rejection::InsufficientMass)
    );
    assert_preview_kept_world(&world, before_hash, before_flows);
    assert_eq!(
        world.lineages.get(species).expect("line exists").program(),
        &program
    );
}

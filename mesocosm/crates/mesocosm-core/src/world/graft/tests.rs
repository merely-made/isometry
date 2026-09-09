// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Attachment, Crossing, Intent, Kingdom, Organism, OrganismId, Outcome, PartId, Provenance,
    Rejection, SpeciesId, Stage, VolumeRef, World, Yaw, state_hash,
};

use super::GraftPreview;

fn fixture() -> (World, OrganismId, PartId, PartId) {
    let mut world = World::new(4_242, 24);
    let recipient = world.controlled_id().expect("embodied");
    let (species, position) = {
        let organism = world
            .organisms
            .iter()
            .find(|organism| organism.id == recipient)
            .expect("controlled organism");
        (organism.species, organism.position)
    };
    *world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == recipient)
        .expect("controlled organism") = Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            recipient,
            species,
            Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    let donor = OrganismId(9_700);
    let mut carcass = Organism {
        stage: Stage::Carrion,
        ..Organism::founding(
            donor,
            SpeciesId(9_700),
            Kingdom::Producer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            [position[0] + 1, position[1], position[2]],
            1_200,
        )
    };
    let root = carcass.body().root;
    let branch = carcass
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            400,
            [6, 4, 1],
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("branch attaches");
    let tip = carcass
        .phenotype
        .attach(
            VolumeRef::from_tag(9),
            150,
            [7, 1, 1],
            Attachment {
                parent: branch,
                offset: [13, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("tip attaches");
    world.organisms.push(carcass);
    (world, donor, branch, tip)
}

#[test]
fn preview_is_read_only_and_matches_the_published_graft() {
    let (mut world, donor, part, _) = fixture();
    assert!(matches!(
        world.apply(Intent::Deposit { mass_mg: 1 }),
        Outcome::Deposited { .. }
    ));
    let flows = world.clone().drain_flows();
    assert!(!flows.is_empty(), "the fixture has a flow to preserve");
    let before = state_hash(&world);
    let preview = world
        .preview_graft(donor, part, Crossing::Regrow)
        .expect("fixture accepts its branch");
    assert_eq!(
        state_hash(&world),
        before,
        "preview changes no saved state or RNG"
    );
    assert_eq!(
        world.drain_flows(),
        flows,
        "preview leaves the current flow buffer alone"
    );

    let outcome = world.apply(Intent::Graft {
        organism: donor,
        part,
        crossing: Crossing::Regrow,
    });
    assert!(matches!(
        outcome,
        Outcome::Grafted {
            root,
            parts,
            mass_mg,
            crossing: Crossing::Regrow,
            verdict,
            ..
        } if root == preview.root
            && parts == preview.parts.len() as u32
            && mass_mg == preview.mass_mg
            && verdict == preview.verdict
    ));
    assert_matches_published(&world, &preview);
}

#[test]
fn graft_sources_are_reachable_nonroot_parts_in_stable_order() {
    let (world, donor, branch, tip) = fixture();
    // A producer founder already has a frond before the authored branch.
    assert_eq!(
        world.graft_sources(),
        vec![(donor, PartId(1)), (donor, branch), (donor, tip)]
    );
}

#[test]
fn preview_refusal_matches_the_actual_request_and_the_request_rechecks() {
    let (mut world, donor, part, _) = fixture();
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == donor)
        .unwrap()
        .stage = Stage::Mature;
    let expected = world
        .preview_graft(donor, part, Crossing::Regrow)
        .expect_err("a living donor is not on offer");
    let outcome = world.apply(Intent::Graft {
        organism: donor,
        part,
        crossing: Crossing::Regrow,
    });
    assert_eq!(outcome, Outcome::Rejected(expected));

    let (mut world, donor, part, _) = fixture();
    let preview = world
        .preview_graft(donor, part, Crossing::Regrow)
        .expect("fixture accepts its branch");
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == donor)
        .unwrap()
        .stage = Stage::Mature;
    assert_eq!(
        world.apply(Intent::Graft {
            organism: donor,
            part,
            crossing: Crossing::Regrow,
        }),
        Outcome::Rejected(Rejection::StillLiving(donor)),
        "the action rechecks rather than publishing an earlier preview"
    );
    assert_ne!(
        world.controlled().expect("still embodied").phenotype,
        preview.phenotype,
        "the stale candidate was not published"
    );
}

fn assert_matches_published(world: &World, preview: &GraftPreview) {
    let graft = world.last_graft().expect("graft recorded");
    assert_eq!(graft.recipient, preview.recipient);
    assert_eq!(graft.donor, preview.donor);
    assert_eq!(graft.donor_line, preview.donor_line);
    assert_eq!(graft.donor_part, preview.donor_part);
    assert_eq!(graft.root, preview.root);
    assert_eq!(graft.parts, preview.parts);
    assert_eq!(graft.mass_mg, preview.mass_mg);
    assert_eq!(graft.crossing, preview.crossing);
    assert_eq!(graft.verdict, preview.verdict);
    assert_eq!(graft.cost_mg, preview.cost_mg);
    assert_eq!(graft.revision, preview.revision);
    assert_eq!(
        world.controlled().expect("still embodied").phenotype,
        preview.phenotype
    );
}

#[test]
fn graft_flow_stays_at_the_recipient_when_the_donor_is_in_another_place() {
    let (mut world, donor, part, _) = fixture();
    world.places = crate::places::Places::scatter(&mut crate::rng::Rng::from_seed(7), 2, 1);
    let recipient = world.controlled_id().unwrap();
    let at = [-1, 20, -1];
    let donor_at = [0, 20, -1];
    for organism in &mut world.organisms {
        if organism.id == recipient {
            organism.position = at;
        }
        if organism.id == donor {
            organism.position = donor_at;
        }
    }
    world.ground.author_walker_stance(at, 8, 16, [0, 0]);
    let place = world.places.at(at);
    assert_ne!(place, world.places.at(donor_at));
    world.drain_flows();
    assert!(matches!(
        world.apply(Intent::Graft {
            organism: donor,
            part,
            crossing: Crossing::Regrow
        }),
        Outcome::Grafted { .. }
    ));
    let flows: Vec<_> = world
        .drain_flows()
        .into_iter()
        .filter(|flow| flow.record.process == crate::flow::Process::Graft)
        .collect();
    assert!(!flows.is_empty());
    assert!(
        flows.iter().all(|flow| flow.place == place),
        "the graft transfer occurs at the recipient"
    );
}

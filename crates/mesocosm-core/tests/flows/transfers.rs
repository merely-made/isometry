// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tissue crossing from one body to another, read against the ledger.
//!
//! Split out of `flows.rs` at the 600-line ceiling when P3's branch transfer
//! added a second one. Both verbs here move substance **between two bodies**
//! rather than between a body and the ground, which is the one shape the
//! compartment reconciliation has to be shown separately: an organ off a
//! carcass (PE2) and a whole branch off one (P3).
//!
//! The harness is next door and shared, because two files reconciling ticks
//! two different ways is how they would come to disagree.

use mesocosm_core::flow::{Account, Process};
use mesocosm_core::{FeedingMode, Intent, OrganismId, World};

use super::stepped;

#[test]
fn a_consumed_part_moves_exactly_its_own_milligrams_and_says_so() {
    // PE2's part-level meal, through the same instrument. One organ off a
    // carcass is one transfer of one number, and "settles that part's exact
    // matter" is the reconciliation passing on that tick rather than a claim
    // in a comment.
    let mut world = World::new(11, 40);
    let scavenger = world
        .organisms
        .iter()
        .find(|organism| organism.is_alive() && organism.feeding_mode() == FeedingMode::Scavenger)
        .map(|organism| organism.id)
        .expect("the founding roster has a live scavenger");
    assert!(matches!(
        world.apply(Intent::TakeControl { organism: scavenger }),
        mesocosm_core::Outcome::Inhabited { organism } if organism == scavenger
    ));
    world.apply(Intent::Idle);
    let here = world.position().expect("a played critter");
    let id = OrganismId(9_600);
    let mut corpse = mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Carrion,
        ..mesocosm_core::Organism::founding(
            id,
            mesocosm_core::SpeciesId(6),
            mesocosm_core::Kingdom::Producer,
            mesocosm_core::VolumeRef::from_tag(1),
            [2, 2, 2],
            [here[0] + 1, here[1], here[2]],
            900,
        )
    };
    let root = corpse.body().root;
    let plate = corpse
        .phenotype
        .attach(
            mesocosm_core::VolumeRef::from_tag(7),
            400,
            [6, 4, 1],
            mesocosm_core::Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: mesocosm_core::Yaw::Zero,
            },
            mesocosm_core::Provenance::founding(),
        )
        .expect("a plate attaches");
    world.organisms.push(corpse);

    let flows = stepped(
        &mut world,
        Intent::Consume {
            organism: id,
            part: plate,
        },
        "after taking one organ off a carcass",
    );
    // The carcass's own side of the ledger. The enclosure is eating around it
    // on the same tick, which is why this asks about the donor rather than
    // about every meal that happened.
    let taken: Vec<_> = flows
        .iter()
        .map(|recorded| recorded.record)
        .filter(|flow| {
            flow.process == Process::Feeding
                && flow.from.map(|from| from.organism) == Some(id)
                && flow.source == Account::Substance
                && flow.destination == Account::Substance
        })
        .collect();
    assert_eq!(taken.len(), 1, "one organ, one transfer: {taken:?}");
    assert_eq!(taken[0].amount_mg, 400);
    assert_eq!(taken[0].source, Account::Substance);
    assert_eq!(
        taken[0].destination,
        Account::Substance,
        "an organ becomes body, never budget"
    );
}

/// A carcass in reach carrying a two-part branch, with a gland arranged on the
/// frond so a carried arrangement actually costs something.
///
/// Returns the branch root. The two files that need one build it the same way
/// on purpose: a graft that moved no allocation would exercise one of the two
/// records this test is about.
fn branched_carcass(
    world: &mut World,
    id: OrganismId,
    line: mesocosm_core::SpeciesId,
) -> mesocosm_core::PartId {
    let here = world.position().expect("a played critter");
    let mut corpse = mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Carrion,
        ..mesocosm_core::Organism::founding(
            id,
            line,
            mesocosm_core::Kingdom::Producer,
            mesocosm_core::VolumeRef::from_tag(1),
            [2, 2, 2],
            [here[0] + 1, here[1], here[2]],
            900,
        )
    };
    let root = corpse.body().root;
    let frond = corpse
        .phenotype
        .attach(
            mesocosm_core::VolumeRef::from_tag(7),
            400,
            [6, 4, 1],
            mesocosm_core::Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: mesocosm_core::Yaw::Zero,
            },
            mesocosm_core::Provenance::founding(),
        )
        .expect("a frond attaches");
    corpse
        .phenotype
        .attach(
            mesocosm_core::VolumeRef::from_tag(9),
            150,
            [7, 1, 1],
            mesocosm_core::Attachment {
                parent: frond,
                offset: [13, 0, 0],
                yaw: mesocosm_core::Yaw::Zero,
            },
            mesocosm_core::Provenance::founding(),
        )
        .expect("and a limb hangs off it");
    let capacity = corpse.phenotype.mosaic(frond).expect("a mosaic").capacity();
    let proposal = mesocosm_core::AllocationProposal {
        expect: corpse.phenotype.digest(),
        source: mesocosm_core::Arrangement::Direct,
        parts: vec![frond],
        sites: vec![mesocosm_core::ProposedSite {
            part: frond,
            process: mesocosm_core::Registry::native()
                .of_native(mesocosm_core::Process::Secrete)
                .reference(),
            cells: (0..capacity)
                .map(|cell| mesocosm_core::CellId(cell as u16))
                .collect(),
        }],
    };
    corpse
        .phenotype
        .develop(mesocosm_core::Registry::native(), &proposal)
        .expect("valid on the donor");
    world.organisms.push(corpse);
    // The two lines share a domain, so the carry is native and the branch
    // arrives with the arrangement it had. Set rather than drawn: this test is
    // about the ledger, not about what the world's stream happened to pick.
    let mine = world.controlled().expect("embodied").species;
    let domain = world.lineages().domain(mine);
    let lineages = world.lineages_mut();
    lineages.found(line);
    lineages.set_domain(line, domain);
    frond
}

#[test]
fn a_grafted_branch_moves_exactly_its_own_milligrams_and_says_so() {
    // P3's branch transfer, through the same instrument. A graft is two
    // movements and no more: the branch's exact substance from one body to the
    // other, and the development's price out of the reserve into the ground.
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    let donor = OrganismId(9_700);
    let frond = branched_carcass(&mut world, donor, mesocosm_core::SpeciesId(6));
    let branch_mg = {
        let body = world
            .organisms
            .iter()
            .find(|o| o.id == donor)
            .expect("on the roster")
            .body();
        body.descendants(frond)
            .into_iter()
            .filter_map(|part| body.part(part).map(|found| found.mass_mg))
            .sum::<u64>()
    };

    let flows = stepped(
        &mut world,
        Intent::Graft {
            organism: donor,
            part: frond,
            crossing: mesocosm_core::Crossing::Carry,
        },
        "after taking a branch off a carcass",
    );
    let carried: Vec<_> = flows
        .iter()
        .map(|recorded| recorded.record)
        .filter(|flow| flow.process == Process::Graft)
        .collect();
    assert_eq!(carried.len(), 1, "one branch, one transfer: {carried:?}");
    assert_eq!(carried[0].amount_mg, branch_mg);
    assert_eq!(carried[0].from.map(|from| from.organism), Some(donor));
    assert_eq!(
        carried[0].to.map(|to| to.organism),
        world.controlled_id(),
        "both subjects, because a loss and an acquisition are one fact"
    );
    assert_eq!(carried[0].source, Account::Substance);
    assert_eq!(
        carried[0].destination,
        Account::Substance,
        "a branch becomes body, never budget"
    );
    assert!(
        flows
            .iter()
            .any(|recorded| recorded.record.process == Process::Develop),
        "and the arrangement it carried was paid for: {flows:?}"
    );
}

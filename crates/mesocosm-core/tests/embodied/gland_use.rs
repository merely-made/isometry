// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD2's other two states: what a gland is worth, and what losing it costs.
//!
//! Split out of `gland.rs` at the 600-line ceiling when PD3 re-expressed the
//! suite through `Intent::Express`. The four claims are one gate and the
//! fixtures are shared, so this file takes the two that are about the world
//! around the body rather than about the development itself:
//!
//! 3. **world conditions can make it useful or dormant** — a gland makes its
//!    toxin out of the ground under the body, and goes dry where that ground
//!    cannot supply what it holds, without losing a cell or a milligram of
//!    rent;
//! 4. **severing its dependency removes the consequence** — and the branch can
//!    still say what it used to do.

use mesocosm_core::{
    AllocationProposal, Arrangement, Attachment, CellId, Intent, Organism, OrganismId, Outcome,
    PartId, Placement, ProposedSite, Provenance, Refusal, Stage, VolumeRef, World, Yaw,
};

use super::develop_played;
use super::discovery::hunger;
use super::gland::{FROND, express, frond_of, fronded_world, gland, gland_cells, ready_world};

// ---------------------------------------------------------------------------
// 3. Useful, and dormant
// ---------------------------------------------------------------------------

/// A neighbour the played critter can reach, with a gland of `cells` cells on
/// a frond of its own.
fn armed_neighbour(world: &mut World, cells: u32) -> OrganismId {
    let here = world.position().unwrap();
    let at = [here[0] + 1, here[1], here[2]];
    let id = OrganismId(9_200);
    let mut prey = Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            id,
            world.controlled().unwrap().species,
            // The played fronded body admits producer meals; this neighbour
            // is the producer carrying the charged gland.
            mesocosm_core::Kingdom::Producer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            at,
            1_200,
        )
    };
    let root = prey.body().root;
    let part = prey
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches");
    if cells > 0 {
        let capacity = prey.phenotype.mosaic(part).unwrap().capacity();
        let proposal = AllocationProposal {
            expect: prey.phenotype.digest(),
            source: Arrangement::Direct,
            parts: vec![part],
            sites: vec![ProposedSite {
                part,
                process: gland(),
                cells: (capacity - cells..capacity)
                    .map(|i| CellId(i as u16))
                    .collect(),
            }],
        };
        prey.phenotype
            .develop(mesocosm_core::Registry::native(), &proposal)
            .expect("a plate admits a gland");
    }
    world.organisms.push(prey);
    id
}

#[test]
fn a_charged_gland_costs_whatever_eats_the_body_that_carries_it() {
    // Useful: the same meal, twice, differing only in whether the prey had
    // turned some of a frond into poison. What the eater loses is exactly what
    // the gland held.
    let mut plain = fronded_world(4_242);
    let mut armed = plain.clone();
    let prey = armed_neighbour(&mut plain, 0);
    assert_eq!(armed_neighbour(&mut armed, 4), prey);
    let potency = armed
        .organisms
        .iter()
        .find(|o| o.id == prey)
        .unwrap()
        .phenotype
        .secretory_mg();
    assert!(potency > 0);

    let intent = Intent::Metabolize {
        organism: prey,
        placement: Placement::Planned,
    };
    plain.apply(intent.clone());
    armed.apply(intent);

    assert_eq!(
        plain.energy_mg().unwrap() - armed.energy_mg().unwrap(),
        potency,
        "the bite cost exactly what the gland held"
    );
    // And it went into the ground under the prey rather than nowhere.
    assert_eq!(plain.total_matter_mg(), armed.total_matter_mg());
}

/// Builds a gland of `cells` cells and then walks one column off the ground it
/// was built on.
///
/// **The step matters, and it is a finding rather than a workaround.** The
/// development's price is paid into the column under the body, and building a
/// gland from scratch costs exactly what the gland comes to hold — so a fresh
/// gland is always charged where it was made, by its own spoil. Dormancy is
/// therefore something a body walks into, which is the right shape for it:
/// carrying a big gland means keeping to rich ground.
fn armed_and_moved_on(world: &mut World) -> PartId {
    let part = frond_of(world);
    express(world, hunger());
    // **Two columns, not one.** The first is still holding this body's own
    // spoil, and a column the line has been standing on through a hunger
    // window holds more besides.
    world.apply(Intent::Move { delta: [2, 0, 0] });
    world.apply(Intent::Move { delta: [2, 0, 0] });
    part
}

#[test]
fn a_gland_bigger_than_the_ground_is_dry_and_still_costs_its_rent() {
    // Dormant, and the whole of the rule: a gland makes its toxin out of the
    // column under the body, so it works only where that ground could replace
    // what the gland holds. Nothing about the allocation changes — the plan
    // forbids a changing environment from rewriting the mosaic — so the tissue
    // is still there, still committed, and still charged for.
    let (mut world, _part) = ready_world(4_242);
    armed_and_moved_on(&mut world);

    let reading = world.gland().expect("it has one");
    assert!(
        reading.potency_mg > reading.ground_mg,
        "the fixture wants a gland the fresh column cannot supply: \
         {} mg against {} mg",
        reading.potency_mg,
        reading.ground_mg
    );
    assert!(!reading.charged, "so it is dry");
    assert_eq!(
        world.controlled().unwrap().bite_mg(reading.ground_mg),
        world.controlled().unwrap().venom_mg,
        "and a bite costs only what the line was born with"
    );
    assert!(reading.rent_mg > 0, "which it is paying for regardless");
    assert_eq!(reading.cells, gland_cells(), "and has lost no tissue");
}

#[test]
fn enriching_the_ground_charges_the_gland_the_body_already_had() {
    // The same claim from the other side, through a verb a player already has:
    // depositing into the column under you is what turns a dry gland on. The
    // allocation does not move, the revision does not move, and the body's
    // capability changes — which is exactly what "world conditions can make it
    // useful or dormant" has to mean if it is not to be a second biology.
    let (mut world, _part) = ready_world(4_242);
    armed_and_moved_on(&mut world);
    let dry = world.gland().expect("it has one");
    assert!(!dry.charged);
    let revision = world.phenotype().unwrap().revision();

    // A deposit is matter out of the reserve, and a body fresh out of a hunger
    // window has little of it. Top it up the same way the stress fixture does,
    // so the claim under test is the ground rather than the budget.
    let me = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .energy_mg = 8_000;
    let owed = dry.potency_mg - dry.ground_mg;
    let deposited = world.apply(Intent::Deposit { mass_mg: owed + 8 });
    assert!(
        matches!(deposited, Outcome::Deposited { .. }),
        "{deposited:?}"
    );

    let charged = world.gland().expect("still has one");
    assert!(
        charged.charged,
        "{} mg of ground against {} mg of gland",
        charged.ground_mg, charged.potency_mg
    );
    assert_eq!(charged.cells, dry.cells, "no tissue moved");
    assert_eq!(charged.potency_mg, dry.potency_mg);
    assert_eq!(
        world.phenotype().unwrap().revision(),
        revision,
        "and no development happened"
    );
}

// ---------------------------------------------------------------------------
// 4. Severed, and gone
// ---------------------------------------------------------------------------

#[test]
fn severing_the_frond_takes_the_bite_and_the_rent_with_it() {
    let (mut world, part) = ready_world(4_242);
    // A control that never had one, severed the same way: rent falls when a
    // part is lost whatever that part was doing, so the claim here is that
    // what is *left* is the same, not that nothing changed.
    let mut control = world.clone();
    express(&mut world, hunger());
    assert!(world.gland().unwrap().charged);

    for at in [&mut world, &mut control] {
        let me = at.controlled_id().unwrap();
        let organism = at.organisms.iter_mut().find(|o| o.id == me).unwrap();
        organism.phenotype.sever(part);
    }

    let after = world.gland().expect("the loss is still readable");
    assert!(after.sites.is_empty(), "nothing expresses it any more");
    assert_eq!(after.cells, 0);
    assert_eq!(after.potency_mg, 0);
    assert_eq!(after.rent_mg, 0, "and it stopped costing rent");
    assert_eq!(
        world.controlled().unwrap().upkeep_mg(),
        control.controlled().unwrap().upkeep_mg(),
        "a body that lost its gland pays what a body that never had one pays"
    );
    assert_eq!(
        world.controlled().unwrap().bite_mg(u64::MAX),
        world.controlled().unwrap().venom_mg,
        "the bite went with the branch"
    );

    // But the injury is still explainable: PD1b keeps a severed part's mosaic
    // addressable precisely so a player can be told what that branch used to
    // do, and PD2 is the first thing with something worth saying.
    assert_eq!(after.lost, vec![part]);
    let explanation = world
        .phenotype()
        .unwrap()
        .explain(part)
        .expect("the part is still addressable");
    assert!(!explanation.living);
    assert!(
        explanation
            .sites
            .iter()
            .any(|site| site.named.as_ref().map(|id| id.name.as_str()) == Some("secrete")),
        "the lost branch can still name what it did: {explanation:?}"
    );
}

#[test]
fn a_severed_frond_is_not_somewhere_a_development_can_put_anything() {
    // The other half of severing, at the boundary that decides it: a branch
    // that is gone is not somewhere a development can put anything, and the
    // door above it never even offers — `candidate_intent` finds no living
    // part of the shape and answers `None`.
    let (mut world, part) = ready_world(4_242);
    let me = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .phenotype
        .sever(part);

    assert!(
        world.candidate_intent(hunger()).is_none(),
        "the door offers nothing once the only plate is gone"
    );
    // Authored against the body as it now is, so the answer is the severing
    // rather than the staleness that would otherwise be reported first.
    let proposal = AllocationProposal {
        expect: world.phenotype().unwrap().digest(),
        source: Arrangement::Direct,
        parts: vec![part],
        sites: vec![ProposedSite {
            part,
            process: gland(),
            cells: vec![CellId(0)],
        }],
    };
    assert!(matches!(
        develop_played(&mut world, &proposal),
        Err(Refusal::SeveredPart(_))
    ));
}

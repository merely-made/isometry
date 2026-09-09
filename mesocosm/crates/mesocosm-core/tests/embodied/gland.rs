// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD2: one native played process, in its four states.
//!
//! The gate's whole specification is four claims, and there is one test for
//! each of them here:
//!
//! 1. **acquiring or expressing it creates a readable choice** — nothing grows
//!    a gland, so having one means a development took tissue off what the part
//!    was already doing, and converting a whole frond costs a body its living;
//! 2. **allocation locates it on a part and charges its cost** — a named part,
//!    a counted number of cells, a milligram out of the reserve and into the
//!    ground, and a standing rent from then on;
//! 3. **world conditions can make it useful or dormant** and
//! 4. **severing its dependency removes the consequence** are next door in
//!    `gland_use.rs`, split off at the 600-line ceiling. The fixtures below
//!    are shared with it.
//!
//! # Two routes, since PD3 deleted the editor operation
//!
//! PD2 proved these through `Intent::Rearrange`, which carried a complete
//! hand-authored allocation. That door is gone: `Intent::Express` names a
//! discovered condition, and the arrangement comes from the admitted ruleset
//! and the discovery record. So each claim here goes through whichever
//! boundary actually decides it — the world door for what a development costs
//! and records, [`develop_played`](super::develop_played) for what a body a
//! host could never author *is*. One validator underneath both, unchanged.

use mesocosm_core::{
    AllocationProposal, Arrangement, Attachment, CellId, ConditionId, Intent, Outcome, PartId,
    Process, ProcessRef, ProposedSite, Provenance, Refusal, Registry, VolumeRef, World, Yaw,
};

use super::discovery::{endure, hunger};
use super::{bulk_world, develop_played};

/// The acquired definition, addressed the way a phenotype addresses it.
pub(crate) fn gland() -> ProcessRef {
    Registry::native().of_native(Process::Secrete).reference()
}

pub(crate) fn fixing() -> ProcessRef {
    Registry::native().of_native(Process::Fix).reference()
}

/// A plate wide enough that its gland can outgrow the ground.
///
/// `[6, 4, 1]` classifies as `Role::Plate` and lattices to 4x3x1 = twelve
/// cells, each worth 23 mg of the part's own adult mass. That matters for the
/// dormancy test and nowhere else: a gland on more than four of those cells
/// holds more than a fresh soil column does, which is what lets one fixture
/// show a charged gland and a dry one without waiting for the enclosure to
/// draw itself down.
pub(crate) const FROND: [i32; 3] = [6, 4, 1];

/// A world whose played critter carries one frond held up in a canopy
/// position — so it reads as a producer, and carries the only shape that
/// admits a gland.
pub(crate) fn fronded_world(seed: u64) -> World {
    let mut world = bulk_world(seed, 24);
    frond_on(&mut world);
    world
}

pub(crate) fn frond_on(world: &mut World) -> PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                // Held up and clear of everything else on the body: the DC4
                // canopy position, which is what makes this a frond rather
                // than a shell.
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root")
}

/// The played critter's only frond.
pub(crate) fn frond_of(world: &World) -> PartId {
    let body = world.body().expect("embodied");
    body.living()
        .map(|part| part.id)
        .find(|part| {
            mesocosm_core::classify(body.part(*part).unwrap().half_extent)
                == mesocosm_core::Role::Plate
        })
        .expect("the fixture attached one")
}

/// A frond split between fixing and a gland of `cells` cells, taken off the
/// high end of the lattice.
///
/// A proposal rather than an intent since PD3: no host can say this, so it is
/// stated to the validator directly. The candidate the game actually grants
/// takes [`GLAND_CELLS`] and comes through the door.
fn split(world: &World, part: PartId, cells: u32) -> AllocationProposal {
    let phenotype = world.phenotype().expect("embodied");
    let capacity = phenotype
        .mosaic(part)
        .expect("a living part carries a mosaic")
        .capacity();
    let kept: Vec<CellId> = (0..capacity - cells).map(|i| CellId(i as u16)).collect();
    let taken: Vec<CellId> = (capacity - cells..capacity)
        .map(|i| CellId(i as u16))
        .collect();
    let mut sites = Vec::new();
    if !kept.is_empty() {
        sites.push(ProposedSite {
            part,
            process: fixing(),
            cells: kept,
        });
    }
    sites.push(ProposedSite {
        part,
        process: gland(),
        cells: taken,
    });
    AllocationProposal {
        expect: phenotype.digest(),
        source: Arrangement::Direct,
        parts: vec![part],
        sites,
    }
}

/// Cells the granted candidate takes. The discovery table's number, read
/// rather than repeated, so a fixture cannot drift from the rule.
pub(crate) fn gland_cells() -> u32 {
    mesocosm_core::discovery::resolve(hunger())
        .expect("the table holds it")
        .grants
        .cells
}

/// A world whose played critter carries a frond **and** whose line has come
/// through hunger, so the gland is available to express.
///
/// The frond goes on after the stress, exactly as `discovery.rs` does it: a
/// canopy earns while the body is meant to be going without.
pub(crate) fn ready_world(seed: u64) -> (World, PartId) {
    let mut world = bulk_world(seed, 24);
    endure(&mut world, mesocosm_core::discovery::HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the line came through hunger");
    let part = frond_on(&mut world);
    (world, part)
}

/// Expresses the discovered gland through the one door a host has. (PD3)
pub(crate) fn express(world: &mut World, condition: ConditionId) -> Outcome {
    let intent = world
        .candidate_intent(condition)
        .expect("the body is somewhere to put it");
    assert_eq!(intent, Intent::Express { condition });
    world.apply(intent)
}

// ---------------------------------------------------------------------------
// 1. A readable choice
// ---------------------------------------------------------------------------

#[test]
fn no_body_a_world_founds_has_one() {
    // The property the whole gate rests on, asserted over a founded roster
    // rather than hoped for: geometry seeds four definitions and admits five,
    // so a gland is only ever somewhere a development put it. If this fails,
    // every ecology number below has moved for a reason nobody chose.
    let world = World::new(4_242, 24);
    for organism in &world.organisms {
        assert_eq!(
            organism.phenotype.secretory_mg(),
            0,
            "{:?} was founded with a gland",
            organism.id
        );
        assert!(organism.phenotype.glands().is_empty());
    }
    assert_eq!(World::new(4_242, 24).gland(), None);
}

#[test]
fn the_tissue_has_to_come_off_something() {
    // A seeded part arrives fully committed, so the first development that
    // wants a second process takes tissue from the first. That is the choice,
    // and it is visible as an exchange of cells rather than as an addition.
    let (mut world, part) = ready_world(4_242);
    let before = world.phenotype().unwrap().mosaic(part).unwrap().free();
    assert_eq!(before, 0, "a seeded frond has nothing spare");

    assert!(matches!(
        express(&mut world, hunger()),
        Outcome::Expressed { .. }
    ));
    let mosaic = world.phenotype().unwrap().mosaic(part).unwrap();
    assert_eq!(mosaic.free(), 0, "and it still has nothing spare");
    assert_eq!(mosaic.sites().len(), 2, "it does two things now");
    assert_eq!(world.gland().unwrap().cells, gland_cells());
}

#[test]
fn converting_the_whole_frond_costs_the_body_its_living() {
    // The downside, and the reason PD1b left the canopy reading for this gate:
    // a plate held up in the light that is not allocated to fixing is not a
    // canopy, so a body that turned its whole frond into poison stops being a
    // producer and has to eat like everything else.
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    assert_eq!(
        world.controlled().unwrap().kingdom(),
        mesocosm_core::Kingdom::Producer,
        "the frond is what makes it one"
    );

    // A whole frond of poison is more than any candidate grants, so it is
    // stated to the validator: the claim is what the body then *is*.
    let capacity = world.phenotype().unwrap().mosaic(part).unwrap().capacity();
    let proposal = split(&world, part, capacity);
    develop_played(&mut world, &proposal).expect("a plate admits a gland");

    assert!(!world.phenotype().unwrap().canopy());
    assert_ne!(
        world.controlled().unwrap().kingdom(),
        mesocosm_core::Kingdom::Producer,
        "it is still held up, and it is not fixing"
    );
    // The anatomy is untouched: the plate is still a plate, still in a canopy
    // position. What changed is what its tissue is doing.
    assert_eq!(world.body().unwrap().canopy_parts().count(), 1);
}

#[test]
fn a_part_still_cannot_acquire_a_capability_by_editing_a_number() {
    // The anti-Spore gate, over the acquired definition: admission is by
    // shape, so a gland cannot be put on bulk. To make a body secrete you
    // still have to give it the shape that secretes.
    let mut world = fronded_world(4_242);
    let root = world.body().unwrap().root;
    let proposal = AllocationProposal {
        expect: world.phenotype().unwrap().digest(),
        source: Arrangement::Direct,
        parts: vec![root],
        sites: vec![ProposedSite {
            part: root,
            process: gland(),
            cells: vec![CellId(0)],
        }],
    };
    assert!(
        matches!(
            develop_played(&mut world, &proposal),
            Err(Refusal::SiteMismatch { .. })
        ),
        "bulk is not a shape that secretes"
    );
    assert_eq!(world.gland(), None, "and nothing was allocated");
}

#[test]
fn a_refusal_names_its_boundary_and_moves_nothing() {
    // PD1b made the boundary the answer rather than a single word, and both of
    // these are proposals no door could carry now: a site that is not one
    // piece of tissue, and a definition this world's ruleset does not hold.
    // The refusal still names which.
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    let before = world.phenotype().unwrap().clone();
    let one = |process, cells| AllocationProposal {
        expect: before.digest(),
        source: Arrangement::Direct,
        parts: vec![part],
        sites: vec![ProposedSite {
            part,
            process,
            cells,
        }],
    };

    // Not a connected region: an organ is a piece of tissue.
    let scattered = one(gland(), vec![CellId(0), CellId(2)]);
    assert!(matches!(
        develop_played(&mut world, &scattered),
        Err(Refusal::Disconnected(_))
    ));
    // Never substituted for the nearest thing this world does hold.
    let unknown = one(
        ProcessRef {
            definition: mesocosm_core::DefinitionDigest(0xDEAD),
        },
        vec![CellId(0)],
    );
    assert!(matches!(
        develop_played(&mut world, &unknown),
        Err(Refusal::UnknownProcess(_))
    ));
    assert_eq!(
        world.phenotype().unwrap().revision(),
        before.revision(),
        "a refused development moves no allocation"
    );
}

// ---------------------------------------------------------------------------
// 2. Located, and charged
// ---------------------------------------------------------------------------

#[test]
fn the_development_is_located_paid_for_and_on_the_record() {
    let (mut world, part) = ready_world(4_242);
    let cell_mg = world.phenotype().unwrap().cell_mg(part);
    let matter_before = world.total_matter_mg();

    let outcome = express(&mut world, hunger());
    let Outcome::Expressed {
        part: on,
        cost_mg,
        revision,
    } = outcome
    else {
        panic!("{outcome:?}");
    };

    // Located: on a named part, and the reading says which.
    assert_eq!(on, part);
    assert_eq!(world.gland().unwrap().sites, vec![(part, gland_cells())]);
    // Charged: the cells whose expression changed, priced in that part's own
    // tissue. The candidate's cells changed hands, and nothing else did.
    assert_eq!(cost_mg, u64::from(gland_cells()) * cell_mg);
    assert_eq!(revision, 1, "and the development is ordered");

    // Paid: out of the reserve, on this tick, for exactly that. Read off the
    // flow record rather than off the reserve, because a tick that develops
    // also earns and spends rent, and the claim here is what the development
    // itself moved.
    let paid: Vec<_> = world
        .flows()
        .iter()
        .map(|recorded| recorded.record)
        .filter(|flow| flow.process == mesocosm_core::flow::Process::Develop)
        .collect();
    assert_eq!(paid.len(), 1, "one development, one payment: {paid:?}");
    assert_eq!(paid[0].amount_mg, cost_mg);
    assert_eq!(paid[0].source, mesocosm_core::Account::Reserve);

    // The milligram went into the ground, it did not evaporate. Ecology moves
    // matter between accounts on the same tick; the sum is what must not move.
    assert_eq!(
        world.total_matter_mg(),
        matter_before,
        "the development conserved matter"
    );
}

#[test]
fn carrying_a_gland_costs_rent_every_tick() {
    // The standing cost, and the reason a gland is a decision rather than a
    // free upgrade. Same body, same mass, same ceiling; the only difference is
    // what its tissue is doing.
    let (mut world, _part) = ready_world(4_242);
    let plain = world.controlled().unwrap().upkeep_mg();

    express(&mut world, hunger());
    let armed = world.controlled().unwrap().upkeep_mg();
    assert!(
        armed > plain,
        "rent {armed} should exceed the {plain} the same body paid without a gland"
    );
    assert_eq!(world.gland().unwrap().rent_mg, armed - plain);
}

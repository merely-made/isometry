// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P2: one embodied consequence.
//!
//! Reach used to be `const REACH: i32 = 8`. A twelve-limbed creature and a
//! single cube touched exactly as far, `Role::Limb` was classified and never
//! consulted, and anatomy was decoration.
//!
//! These tests pin the claim that **what a critter can do is read off what it
//! is made of**: two bodies reach differently because of their shapes, severing
//! the part responsible takes the ability with it, and a refusal says which
//! embodied requirement went unmet. Nothing here edits a capability number,
//! because there is no capability number to edit.

use mesocosm_core::{
    Attachment, Capability, IntakePort, Intent, NisKind, OrganismId, Outcome, Placement, Process,
    ProcessRef, Provenance, Registry, Rejection, Unmet, VolumeRef, World, Yaw,
};

// An integration test's crate root resolves `mod` against `tests/`, and a
// bare `tests/allocation.rs` would become a second test binary. The explicit
// path keeps the split file beside the suite it belongs to.
#[path = "embodied/allocation.rs"]
mod allocation;
#[path = "embodied/discovery.rs"]
mod discovery;
#[path = "embodied/gland.rs"]
mod gland;
#[path = "embodied/gland_use.rs"]
mod gland_use;
#[path = "embodied/graft.rs"]
mod graft;
#[path = "embodied/lineage.rs"]
mod lineage;
#[path = "embodied/part_meal.rs"]
mod part_meal;
#[path = "embodied/review.rs"]
mod review;
#[path = "embodied/round.rs"]
mod round;

/// The definition a contracting part expresses. Identity is the registry's
/// now, not the enum's: this is the reference a phenotype actually stores.
fn contract() -> ProcessRef {
    Registry::native().of_native(Process::Contract).reference()
}

/// Runs a proposal through the one validator on the played body, with none of
/// the world's price, payment or record.
///
/// **The validator is what survived PD3's deletion.** `Intent::Rearrange` is
/// gone and `Intent::Express` names a discovered condition rather than an
/// arrangement, so a claim about a body a host could never author — a whole
/// frond turned to poison, a gland asked for on bulk, a site that is not one
/// connected region — is made here, at the boundary that actually decides it.
/// A claim about what a development *costs* goes through the door.
fn develop_played(
    world: &mut World,
    proposal: &mesocosm_core::AllocationProposal,
) -> Result<mesocosm_core::Development, mesocosm_core::Refusal> {
    let me = world.controlled_id().expect("embodied");
    // The world's own admitted ruleset (PD4), which is what the door hands
    // the validator. Taken first, so the shared handle outlives the mutable
    // borrow of the roster.
    let ruleset = world.admitted();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("still in the roster")
        .phenotype
        .develop(&ruleset, proposal)
}

/// Empties the played critter's budget. Since TD4 the body routes its own
/// meals, so this is how a test asks for the burning half of the verb.
fn starve(world: &mut World) {
    let me = world.controlled_id().expect("embodied");
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("still in the roster")
        .energy_mg = 0;
}

/// A world whose played critter is a plain bulk root with a cropping mouth.
///
/// **DC4 moved the starting body.** The played critter founds from an archetype
/// now — six legs, a crop and two eyes — so `World::new`'s critter is already an
/// actuator and cannot show what growing one does. These tests are about
/// anatomy deciding capability, so they put the body the claim is about in
/// front of it rather than relying on what worldgen happens to hand them.
fn bulk_world(seed: u64, founders: u32) -> World {
    let mut world = World::new(seed, founders);
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
            me,
            species,
            mesocosm_core::Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    let mouth = organism.body().mouth_part().expect("bulk body has a mouth");
    let support = organism
        .phenotype
        .part_port(mouth)
        .expect("bulk mouth port is active")
        .support()
        .expect("bulk mouth port has support");
    assert!(
        organism.phenotype.declare_port(
            mouth,
            IntakePort::live(NisKind::Producer)
                .with_deadstock()
                .supported_by(support),
        )
    );
    world
}

/// Gives the played critter a long limb along `+x`, and returns its id.
fn grow_a_limb(world: &mut World) -> mesocosm_core::PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(9),
            200,
            // Long in one axis only, which is what `classify` reads as a limb.
            [7, 1, 1],
            Attachment {
                parent: root,
                offset: [9, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches")
}

#[test]
fn a_bare_critter_reaches_less_than_a_limbed_one() {
    // The headline. Same world, same seed, different anatomy, different reach.
    let bare = bulk_world(4_242, 24);
    let mut limbed = bare.clone();
    grow_a_limb(&mut limbed);

    assert!(
        limbed.reach() > bare.reach(),
        "limbed {} should out-reach bare {}",
        limbed.reach(),
        bare.reach()
    );
}

#[test]
fn reach_is_not_a_constant_any_more() {
    // The specific thing replaced. Eight was the old answer for everybody.
    let world = World::new(4_242, 24);
    assert_ne!(
        world.reach(),
        8,
        "a starting critter no longer inherits the old constant"
    );
    assert!(
        world.reach() > 0,
        "but it can still touch what is against it"
    );
}

#[test]
fn a_limb_makes_something_edible_that_was_not() {
    // The consequence in gameplay rather than in a fold: a meal just out of
    // range becomes reachable because the body changed, and for no other
    // reason.
    let mut bare = bulk_world(4_242, 24);
    let here = bare.position().unwrap();

    // A morsel placed just beyond bulk reach and within a limb's.
    let bare_reach = bare.reach();
    let at = [here[0] + bare_reach + 3, here[1], here[2]];
    let id = OrganismId(9_100);
    bare.organisms.push(mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
            id,
            mesocosm_core::SpeciesId(3),
            mesocosm_core::Kingdom::Producer,
            VolumeRef::from_tag(2),
            [1, 1, 1],
            at,
            100,
        )
    });

    let mut limbed = bare.clone();
    grow_a_limb(&mut limbed);

    assert!(!bare.in_reach(at), "out of reach for a stubby body");
    assert!(limbed.in_reach(at), "and in reach once it grew an arm");

    // Both empty, so the meal that lands lands the same way for both and the
    // only difference between them stays the one under test: the arm.
    starve(&mut bare);
    starve(&mut limbed);

    assert!(matches!(
        bare.apply(Intent::Metabolize {
            organism: id,
            placement: Placement::Planned,
        }),
        Outcome::Rejected(Rejection::OutOfReach(_))
    ));
    assert!(matches!(
        limbed.apply(Intent::Metabolize {
            organism: id,
            placement: Placement::Planned,
        }),
        Outcome::Burned { .. }
    ));
}

#[test]
fn severing_the_limb_removes_the_ability_it_provided() {
    // The other direction, and the reason the cascade ruling matters: a
    // capability that came from anatomy leaves with it.
    let mut world = bulk_world(4_242, 24);
    let limb = grow_a_limb(&mut world);
    let reached = world.reach();

    let me = world.controlled_id().unwrap();
    let lost = world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .phenotype
        .sever(limb);

    assert_eq!(lost, vec![limb]);
    assert!(world.reach() < reached, "the reach went with the arm");

    // **And the allocation went with it, in the same commit.** Anatomy and
    // phenotype cannot be split: there is no ordering in which the arm is gone
    // and the tissue allocated to contraction is still expressing.
    let me = world.controlled().unwrap();
    assert!(
        !me.phenotype.expresses(contract()),
        "no living allocation contracts any more"
    );
    assert!(
        me.phenotype
            .allocations()
            .all(|(part, _)| me.body().is_living(part)),
        "and every living allocation still names a living part"
    );
    assert!(me.phenotype.conserves());
}

#[test]
fn a_refusal_says_which_embodied_requirement_failed() {
    // "Too far" and "you have no arm" are different problems, and a receipt
    // that only says OutOfReach cannot tell a player which one they have.
    let mut world = bulk_world(4_242, 24);
    let here = world.position().unwrap();
    let far = [here[0] + 500, here[1], here[2]];

    let id = OrganismId(9_200);
    world.organisms.push(mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
            id,
            mesocosm_core::SpeciesId(3),
            mesocosm_core::Kingdom::Producer,
            VolumeRef::from_tag(2),
            [1, 1, 1],
            far,
            100,
        )
    });

    // No actuator at all: the body is a bulk root.
    let outcome = world.apply(Intent::Metabolize {
        organism: id,
        placement: Placement::Planned,
    });
    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::OutOfReach(Unmet::NoProcess {
            capability: Capability::Reach,
            needs: Process::Contract,
        })),
        "a critter with no actuator is told so"
    );

    // With an arm, the same refusal reports a distance instead.
    grow_a_limb(&mut world);
    let reach = world.reach();
    let outcome = world.apply(Intent::Metabolize {
        organism: id,
        placement: Placement::Planned,
    });
    assert!(
        matches!(outcome, Outcome::Rejected(Rejection::OutOfReach(Unmet::TooFar { reach: r, .. })) if r == reach),
        "an armed critter is told how far it can actually touch, got {outcome:?}"
    );
}

#[test]
fn processes_are_read_from_shape_and_not_stored() {
    // The anti-Spore property. There is no field granting a part an ability,
    // so a part cannot have one its shape does not imply.
    let mut world = bulk_world(4_242, 24);
    let body = world.body().unwrap();

    assert!(
        !body.performs(Process::Contract),
        "a bulk root is not an actuator"
    );
    assert!(body.performs(Process::Intake), "but it does admit material");

    let limb = grow_a_limb(&mut world);
    let body = world.body().unwrap();
    assert!(body.performs(Process::Contract), "a long part acts");
    assert_eq!(body.processes(limb), &[Process::Contract]);
}

#[test]
fn every_organism_answers_the_same_way() {
    // Capability is a property of a body, not of being played. An unplayed
    // critter's reach is computed by the same fold.
    let world = World::new(4_242, 24);
    for organism in &world.organisms {
        assert!(
            organism.body().reach() > 0,
            "{:?} can touch something",
            organism.id
        );
        assert_eq!(
            organism.body().can_reach(organism.body().reach()),
            Ok(()),
            "and can do what its own fold says it can"
        );
    }
}

#[test]
fn growing_raises_the_rent_and_burning_does_not() {
    // **The reconciliation, and the reason P0's choice was not yet a choice.**
    // Upkeep used to be a flat milligram, so a forty-part critter cost exactly
    // what a single cell cost and incorporating had no downside to weigh
    // burning against.
    let base = World::new(4_242, 24);
    let before = base.controlled().unwrap().upkeep_mg();

    // Two critters, same meal, different destinations.
    let mut grown = base.clone();
    let me = grown.controlled_id().unwrap();
    {
        let organism = grown.organisms.iter_mut().find(|o| o.id == me).unwrap();
        let root = organism.body().root;
        organism
            .phenotype
            .attach(
                VolumeRef::from_tag(9),
                4_000,
                [7, 1, 1],
                Attachment {
                    parent: root,
                    offset: [9, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
    }

    let mut burnt = base;
    burnt.controlled_id().unwrap();
    {
        let id = burnt.controlled_id().unwrap();
        burnt
            .organisms
            .iter_mut()
            .find(|o| o.id == id)
            .unwrap()
            .energy_mg += 4_000;
    }

    assert!(
        grown.controlled().unwrap().upkeep_mg() > before,
        "the grown critter pays more rent forever"
    );
    assert_eq!(
        burnt.controlled().unwrap().upkeep_mg(),
        before,
        "the one that burned it pays exactly what it did before"
    );
}

#[test]
fn a_body_and_its_weight_are_one_account() {
    // There is no scalar mass beside the anatomy any more. Adding substance
    // shows up in the body, and the body is what everything reads.
    let mut world = World::new(4_242, 24);
    let me = world.controlled_id().unwrap();
    let before = world.controlled().unwrap().biomass_mg();
    // Growth is determinate, so the gain is taken out of the body's own
    // headroom rather than a literal: an archetype-bodied critter has an adult
    // mass and a test that asks for more than it gets a capped answer.
    let room = (world.controlled().unwrap().mass_ceiling_mg() - before).min(500);
    assert!(room > 0, "the starting body has room to grow");

    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .gain_mass(room);

    let after = world.controlled().unwrap();
    assert_eq!(after.biomass_mg(), before + room);
    assert_eq!(
        after.biomass_mg(),
        after.body().total_mass_mg(),
        "one account, not two"
    );
}

#[test]
fn a_carve_is_recorded_ground_truth() {
    // G1 complete: the ground lives inside the world, a carve is an ordered
    // intent, and both survive snapshot and replay identically.
    let mut world = mesocosm_core::World::new(4_242, 24);
    let here = world.position().expect("embodied");

    // A carvable voxel in reach: solid, and above the bedrock floor the
    // ground protects (y >= 1). The scan is the receipt's honesty: the
    // fixture takes what the world affords instead of assuming a column.
    let reach = world.reach().max(1);
    let mut site = None;
    'scan: for dy in -reach..=reach {
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let at = [here[0] + dx, here[1] + dy, here[2] + dz];
                if at[1] >= 1 && world.ground().solid(at) {
                    site = Some(at);
                    break 'scan;
                }
            }
        }
    }
    let under = site.expect("something solid within reach of a grounded world");

    let outcome = world.apply(Intent::Carve {
        at: under,
        radius: 1,
    });
    let Outcome::Carved { at, removed } = outcome else {
        panic!("carving in reach was refused: {outcome:?}");
    };
    assert_eq!(at, under);
    assert!(removed > 0, "solid ground yielded nothing");
    assert!(!world.ground().solid(under), "the voxel is air now");

    // The carve is inside the replay contract: a twin applying the same
    // intents reaches the same bytes, ground included.
    let mut twin = mesocosm_core::World::new(4_242, 24);
    twin.apply(Intent::Carve {
        at: under,
        radius: 1,
    });
    assert_eq!(
        mesocosm_core::state_hash(&world),
        mesocosm_core::state_hash(&twin)
    );

    // And it survives the snapshot.
    let resumed = mesocosm_core::restore(&mesocosm_core::snapshot(&world).unwrap()).unwrap();
    assert_eq!(resumed.ground(), world.ground());
}

#[test]
fn a_carve_beyond_reach_is_refused() {
    let mut world = mesocosm_core::World::new(4_242, 24);
    let here = world.position().expect("embodied");
    let far = [here[0] + 50, here[1], here[2]];
    assert!(matches!(
        world.apply(Intent::Carve { at: far, radius: 1 }),
        Outcome::Rejected(Rejection::OutOfReach(Unmet::TooFar { .. }))
    ));
}

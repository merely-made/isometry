// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD1b: allocation is the phenotype's, and it cannot drift from anatomy.
//!
//! Split out of `embodied.rs` at the 600-line ceiling. The P2 claims next door
//! are what these extend: capability is still read off what a body is made of,
//! and now the tissue that reads that way is named, counted and conserved.

use mesocosm_core::{
    Aim, AllocationProposal, Arrangement, Intent, OrganismId, Outcome, PartPalette, PartTemplate,
    Placement, Process, Registry, RoleShapes, VolumeRef, World, arrange, snapshot,
};

use super::{bulk_world, grow_a_limb};

#[test]
fn every_founder_carries_one_mosaic_per_living_part() {
    // The invariant over a whole founded roster rather than a fixture: a body
    // arrives with an allocation, every living allocation names a living part,
    // and every part's mosaic conserves its capacity.
    let world = World::new(4_242, 24);
    for organism in &world.organisms {
        let phenotype = &organism.phenotype;
        assert!(phenotype.conserves(), "{:?} lost count", organism.id);
        assert_eq!(
            phenotype.allocations().count(),
            organism.body().living().count(),
            "{:?} has parts and allocations in different numbers",
            organism.id
        );
        for (part, mosaic) in phenotype.allocations() {
            assert!(organism.body().is_living(part));
            assert_eq!(mosaic.occupied() + mosaic.free(), mosaic.capacity());
        }
    }
}

#[test]
fn the_allocation_and_the_anatomy_reading_agree_everywhere() {
    // Geometry seeds allocation, so the two answers are the same answer. This
    // is what stops PD1b from quietly becoming an ecology change: `performs`
    // still decides capability, and the mosaic beside it says the same thing.
    let world = World::new(4_242, 24);
    let registry = Registry::native();
    for organism in &world.organisms {
        for process in Process::ALL {
            let by_definition = registry.of_native(process).reference();
            assert_eq!(
                organism.phenotype.expresses(by_definition),
                organism.body().performs(process),
                "{:?} disagrees about {process:?}",
                organism.id
            );
        }
    }
}

#[test]
fn incorporating_a_part_seeds_its_allocation_in_the_same_meal() {
    // A meal that lands as anatomy lands as phenotype. There is no window in
    // which the body carries a part with no mosaic behind it, because the
    // wrapper does both or neither.
    let mut world = bulk_world(4_242, 24);
    grow_a_limb(&mut world);
    // Well fed, so the body builds with the meal rather than burning it: TD4
    // routes by budget, and this test is about the building half.
    let me = world.controlled_id().expect("embodied");
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("in the roster")
        .energy_mg = 1_000_000;
    let here = world.position().unwrap();
    let id = OrganismId(9_300);
    world.organisms.push(
        mesocosm_core::Organism::founding(
            id,
            mesocosm_core::SpeciesId(3),
            mesocosm_core::Kingdom::Producer,
            VolumeRef::from_tag(2),
            [1, 1, 1],
            [here[0] + 2, here[1], here[2]],
            100,
        )
        .matured(),
    );
    let before = world.controlled().unwrap().body().living().count();

    let outcome = world.apply(Intent::Metabolize {
        organism: id,
        placement: Placement::Planned,
    });
    assert!(
        matches!(
            outcome,
            Outcome::Incorporated { .. } | Outcome::IncorporatedPair { .. }
        ),
        "the meal was meant to land as a part: {outcome:?}"
    );

    let me = world.controlled().unwrap();
    assert!(me.body().living().count() > before);
    assert_eq!(
        me.phenotype.allocations().count(),
        me.body().living().count(),
        "the incorporated part arrived with a mosaic"
    );
    assert!(me.phenotype.conserves());
}

#[test]
fn one_validator_serves_the_player_and_the_game() {
    // **The done-condition, over a live body.** The same candidate submitted
    // as a hand arrangement and as an automatic one lowers to the same
    // instruction and the same bytes; the same invalid candidate earns the
    // same refusal. Proposal source is diagnostic and cannot alter validation.
    let world = bulk_world(4_242, 24);
    let mut automatic = world.controlled().unwrap().phenotype.clone();
    let mut direct = automatic.clone();

    let by_game = arrange(&automatic, Aim::Spare);
    let by_hand = AllocationProposal {
        source: Arrangement::Direct,
        ..by_game.clone()
    };

    let there = automatic
        .develop(mesocosm_core::Registry::native(), &by_game)
        .expect("valid");
    let here = direct
        .develop(mesocosm_core::Registry::native(), &by_hand)
        .expect("valid");
    assert_eq!(there.instruction, here.instruction);
    assert_eq!(
        snapshot::encode(&automatic).unwrap(),
        snapshot::encode(&direct).unwrap()
    );

    // And the invalid half, from both sources.
    let mut game_refused = world.controlled().unwrap().phenotype.clone();
    let mut hand_refused = game_refused.clone();
    let mut bad_game = arrange(&game_refused, Aim::Spare);
    bad_game.sites[0].cells.clear();
    let bad_hand = AllocationProposal {
        source: Arrangement::Direct,
        ..bad_game.clone()
    };
    assert_eq!(
        game_refused
            .develop(mesocosm_core::Registry::native(), &bad_game)
            .unwrap_err(),
        hand_refused
            .develop(mesocosm_core::Registry::native(), &bad_hand)
            .unwrap_err(),
        "one refusal, whoever asked"
    );
}

#[test]
fn re_realizing_under_changed_conditions_keeps_identity_and_provenance() {
    // Phenotypic plasticity is intended: the same developmental program under
    // different declared inputs may grow a different body. What must not move
    // with it is *which processes* it expresses and *why* — the definitions it
    // cites and the cause that put them there.
    //
    // The changed condition here is the **world's admitted materials**, which
    // is the regrow-here case: a lineage carries which shape, a world carries
    // what that shape is. Bulkier bulk and broader plates mean literally more
    // tissue to allocate, so the mosaics are different graphs and not just
    // heavier ones.
    let world = World::new(4_242, 24);
    let organism = world.controlled().expect("embodied");
    let lineage = world.lineages().get(organism.species).expect("a lineage");
    let here = world.development_palette();
    let elsewhere = PartPalette {
        mass: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(1),
            half_extent: [5, 5, 5],
        }),
        plate: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(3),
            half_extent: [6, 6, 1],
        }),
        ..here
    };

    let seed = organism.development_seed;
    let native =
        mesocosm_core::BodyPhenotype::seed(lineage.realize(seed, 4_000, here).expect("realizes"));
    let foreign = mesocosm_core::BodyPhenotype::seed(
        lineage
            .realize(seed, 4_000, elsewhere)
            .expect("realizes elsewhere"),
    );

    assert_ne!(native, foreign, "different conditions, different phenotype");
    let capacity =
        |p: &mesocosm_core::BodyPhenotype| p.allocations().map(|(_, m)| m.capacity()).sum::<u32>();
    assert_ne!(
        capacity(&native),
        capacity(&foreign),
        "and there is genuinely a different amount of tissue to allocate"
    );
    assert_eq!(
        native.expressed(),
        foreign.expressed(),
        "but the same program expresses the same definitions, for the same reason"
    );
    assert!(native.conserves() && foreign.conserves());

    // Identical inputs still produce an identical phenotype: plasticity is not
    // nondeterminism.
    assert_eq!(
        native,
        mesocosm_core::BodyPhenotype::seed(lineage.realize(seed, 4_000, here).expect("realizes"))
    );
}

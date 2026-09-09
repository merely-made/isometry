// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PE2's part-level meal: what one organ off a carcass does and does not do.
//!
//! Split out of `discovery.rs` at the 600-line ceiling. Two of the gate's
//! named claims live here:
//!
//! - **one meal supplies evidence without unlocking an incompatible
//!   candidate** — the evidence is recorded, the condition that never declared
//!   the meal lane cannot be reached by it, and the donor's other recipe words
//!   are not taught;
//! - **one consumed part settles only its own matter and provenance** — the
//!   organ's exact milligrams move, its children keep theirs, and `from_part`
//!   finally names the part it came off.
//!
//! The fixtures are next door, because the two files are two halves of one
//! claim and duplicating a carcass builder is how they would drift.

use std::collections::BTreeSet;

use mesocosm_core::discovery::Input;
use mesocosm_core::{
    Appendage, Intent, Organism, OrganismId, Origin, Outcome, PartId, Placement, Rejection, Stage,
    VolumeRef, World,
};

use super::bulk_world;
use super::discovery::{Miss, carcass, hunger, plate_eaten};

// ---------------------------------------------------------------------------
// 2. One meal supplies evidence without unlocking an incompatible candidate
// ---------------------------------------------------------------------------

#[test]
fn a_meal_supplies_evidence_and_cannot_reach_a_condition_that_never_asked_for_one() {
    let mut world = bulk_world(4_242, 24);
    let (donor, plate, _) = carcass(&mut world);

    let outcome = world.apply(Intent::Consume {
        organism: donor,
        part: plate,
    });
    assert!(matches!(outcome, Outcome::Consumed { .. }), "{outcome:?}");

    // The evidence is on the record whether or not anything took it.
    let observation = world.last_observation().expect("a meal is an observation");
    assert_eq!(observation.route, Input::Meal);
    assert_eq!(
        observation.matched,
        Some(plate_eaten().id()),
        "the organ that was eaten is what the condition reads"
    );

    // **The incompatible candidate.** The gland condition declared only the
    // endurance lane, so this evidence could not reach its rule at all — and
    // the record says that rather than saying the rule went unmet.
    assert_eq!(
        observation
            .missed
            .iter()
            .find(|(id, _)| *id == hunger())
            .map(|(_, why)| *why),
        Some(Miss::UndeclaredInput)
    );
    assert!(
        !world.discovered(hunger()),
        "a meal did not unlock the gland however good it was"
    );
}

#[test]
fn a_meal_no_longer_teaches_the_donors_whole_recipe() {
    // **What `learn_from` did, and does not any more.** It read the eaten
    // *lineage's recipe* and taught the eater's line every non-innate appendage
    // in it, on every meal — a food category mapped straight onto a reward
    // category. Now the only thing a meal says is what organ was in your mouth.
    let mut world = World::new(4_242, 40);
    let mine = world.controlled().unwrap().species;
    let held: BTreeSet<Appendage> = world
        .lineages()
        .get(mine)
        .unwrap()
        .recipe
        .lexicon()
        .collect();

    // A founded line whose recipe holds words ours does not, and one of its
    // bodies moved into reach — so the meal is guaranteed rather than fished
    // for, and the claim is tested rather than skipped.
    let (rich, unknown) = world
        .lineages()
        .all()
        .map(|line| {
            let words: Vec<Appendage> = line
                .recipe
                .lexicon()
                .filter(|word| !held.contains(word))
                .collect();
            (line.id, words)
        })
        .find(|(_, words)| !words.is_empty())
        .expect("a founded enclosure holds vocabulary the played line lacks");

    let id = OrganismId(9_500);
    let here = world.position().expect("embodied");
    // Give the selected line a fresh producer body. This keeps its authored
    // recipe, guarantees a live producer meal for the played crop, and avoids
    // depending on whether worldgen happened to leave that line alive.
    world.organisms.push(Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            id,
            rich,
            mesocosm_core::Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            [here[0] + 1, here[1], here[2]],
            300,
        )
    });

    let outcome = world.apply(Intent::Metabolize {
        organism: id,
        placement: Placement::Planned,
    });
    assert!(
        !matches!(outcome, Outcome::Rejected(_)),
        "the meal has to land for the claim to mean anything: {outcome:?}"
    );
    let after: BTreeSet<Appendage> = world
        .lineages()
        .get(mine)
        .unwrap()
        .recipe
        .lexicon()
        .collect();
    assert_eq!(
        held, after,
        "the meal taught the eater's line the donor's vocabulary: {unknown:?}"
    );
    assert!(
        world.last_observation().is_some(),
        "but it was observed, and the observation is the record that says so"
    );
}

#[test]
fn the_organ_that_teaches_also_teaches_the_word_for_it() {
    // Inheritance, and the narrowed remains of the old lesson: eating a plate
    // teaches the line to grow plates. Eating anything else does not.
    let mut world = bulk_world(4_242, 24);
    let mine = world.controlled().unwrap().species;
    assert!(
        !world
            .lineages()
            .get(mine)
            .unwrap()
            .recipe
            .can_express(Appendage::Plate),
        "a bulk consumer's line starts without the word"
    );
    let (donor, plate, _) = carcass(&mut world);
    world.apply(Intent::Consume {
        organism: donor,
        part: plate,
    });
    assert!(
        world
            .lineages()
            .get(mine)
            .unwrap()
            .recipe
            .can_express(Appendage::Plate),
        "and holds it afterwards"
    );
}

// ---------------------------------------------------------------------------
// 3. One consumed part settles only its own matter and provenance
// ---------------------------------------------------------------------------

#[test]
fn a_consumed_part_settles_its_own_matter_and_nothing_elses() {
    let mut world = bulk_world(4_242, 24);
    let (donor, plate, under) = carcass(&mut world);
    let matter_before = world.total_matter_mg();
    let corpse = |world: &World, id: OrganismId| {
        world
            .organisms
            .iter()
            .find(|o| o.id == id)
            .expect("still there")
            .clone()
    };
    let before = corpse(&world, donor);
    let plate_mg = before.body().part(plate).unwrap().mass_mg;
    let under_mg = before.body().part(under).unwrap().mass_mg;
    let root_mg = before.body().part(before.body().root).unwrap().mass_mg;
    let mine_before = world.controlled().unwrap().biomass_mg();

    let outcome = world.apply(Intent::Consume {
        organism: donor,
        part: plate,
    });
    let Outcome::Consumed {
        part,
        from,
        from_part,
        mass_mg,
    } = outcome
    else {
        panic!("{outcome:?}");
    };

    // Exactly that part's milligrams, and no others.
    assert_eq!(mass_mg, plate_mg);
    assert_eq!(from, donor);
    assert_eq!(from_part, plate);
    let after = corpse(&world, donor);
    assert_eq!(after.body().part(plate).unwrap().mass_mg, 0, "it is empty");
    assert_eq!(
        after.body().part(under).unwrap().mass_mg,
        under_mg,
        "what hung off it kept its own substance: no branch came away"
    );
    assert_eq!(
        after.body().part(after.body().root).unwrap().mass_mg,
        root_mg
    );
    assert_eq!(
        world.controlled().unwrap().biomass_mg(),
        mine_before + plate_mg,
        "and the eater gained exactly that and nothing more"
    );
    assert_eq!(
        world.total_matter_mg(),
        matter_before,
        "the enclosure's matter did not move"
    );

    // **Provenance, and `from_part` finally naming something.** Before PE2 this
    // field was written `PartId(0)` at every call site.
    let grown = world.body().unwrap().part(part).expect("it attached");
    assert_eq!(
        grown.provenance.origin,
        Origin::Incorporated {
            from_species: before.species,
            from_part: plate,
        }
    );
    assert_eq!(grown.mass_mg, plate_mg);
}

#[test]
fn an_organ_can_only_be_taken_once_and_only_off_something_that_has_stopped() {
    let mut world = bulk_world(4_242, 24);
    let (donor, plate, _) = carcass(&mut world);

    // A living body's organs are not on offer: that is live dismemberment,
    // which this proof deliberately does not open.
    let living = OrganismId(9_401);
    let mut alive = world
        .organisms
        .iter()
        .find(|o| o.id == donor)
        .unwrap()
        .clone();
    alive.id = living;
    alive.stage = Stage::Mature;
    world.organisms.push(alive);
    assert!(matches!(
        world.apply(Intent::Consume {
            organism: living,
            part: plate,
        }),
        Outcome::Rejected(Rejection::StillLiving(_))
    ));

    assert!(matches!(
        world.apply(Intent::Consume {
            organism: donor,
            part: PartId(200),
        }),
        Outcome::Rejected(Rejection::NoSuchPart(_))
    ));

    assert!(matches!(
        world.apply(Intent::Consume {
            organism: donor,
            part: plate,
        }),
        Outcome::Consumed { .. }
    ));
    assert!(
        matches!(
            world.apply(Intent::Consume {
                organism: donor,
                part: plate,
            }),
            Outcome::Rejected(Rejection::NothingLeft(_))
        ),
        "an emptied part has already been settled"
    );
}

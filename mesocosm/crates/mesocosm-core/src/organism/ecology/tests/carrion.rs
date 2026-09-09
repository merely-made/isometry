// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the dead do, and who lives on them.
//!
//! Split out of `ecology/tests.rs` on 2026-08-29 when TD8's carrion ruling
//! pushed that file further past the six-hundred-line ceiling — the same
//! split-before-adding move the module's own source has made three times.

use super::*;

#[test]
fn the_dead_become_carrion_then_return() {
    let mut world = vec![organism(Kingdom::Consumer, 30)];
    assert!(
        until(&mut world, |w| w
            .first()
            .is_some_and(|o| o.stage == Stage::Carrion)),
        "starving leaves a body"
    );
    assert!(
        until(&mut world, |w| w.is_empty()),
        "carrion returns to the world"
    );
}

#[test]
fn a_decomposer_feeds_on_the_dead_beside_it() {
    let mut world = vec![
        organism(Kingdom::Decomposer, 200),
        Organism {
            id: OrganismId(1),
            stage: Stage::Carrion,
            ..organism(Kingdom::Consumer, 300)
        },
    ];
    let before = world[0].biomass_mg();
    run(&mut world, 10);
    assert!(
        world[0].biomass_mg() > before,
        "a decomposer earns beside a corpse"
    );
}

#[test]
fn a_decomposer_alone_declines() {
    let mut world = vec![organism(Kingdom::Decomposer, 200)];
    let (body, budget) = (world[0].biomass_mg(), world[0].energy_mg);
    run(&mut world, 10);
    assert!(
        world[0].energy_mg < budget || world[0].biomass_mg() < body,
        "no dead, no living: something has to be draining"
    );
    assert!(
        until(&mut world, |w| w.first().is_none_or(|o| !o.is_alive())),
        "an unfed decomposer does not last forever"
    );
}

/// TD8's carrion ruling, measured as duration: a corpse nobody eats stands
/// `CARRION_DECAY_TICKS` times as long as it used to, and every milligram it
/// sheds is still in the ground underneath it.
#[test]
fn a_corpse_stands_longer_and_still_returns_every_milligram() {
    let mut world = vec![organism(Kingdom::Producer, 300)];
    world[0].stage = Stage::Carrion;
    let corpse = world[0].biomass_mg();
    let mut ground = soil();
    let held = ground.total_mg();
    let mut rng = Rng::from_seed(5);
    let mut next = 50;
    let lineages = registry(&world);
    let mut ticks = 0u32;
    while world.iter().any(|o| o.stage == Stage::Carrion) && ticks < 10_000 {
        step(
            &mut world,
            &mut next,
            &mut rng,
            &mut Sink::default().stream(),
            &lineages,
            PartPalette::primitive(),
            &mut ground,
        );
        ticks += 1;
    }
    assert!(
        ticks >= corpse as u32 * CARRION_DECAY_TICKS,
        "a corpse returned in {ticks} ticks, faster than the ruled duration"
    );
    assert_eq!(
        ground.total_mg(),
        held + corpse,
        "the corpse's matter did not all reach the ground it lay on"
    );
}

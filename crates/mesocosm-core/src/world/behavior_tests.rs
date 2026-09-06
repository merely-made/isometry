// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::body::{SpeciesId, VolumeRef};
use crate::organism::{FaunaDrive, Kingdom, Organism, OrganismId};
use crate::places::{WALKER_HEIGHT, spot, step};
use crate::process::FeedingMode;

#[test]
fn bounded_fauna_policy_names_its_decision_and_replays() {
    let mut world = World::new(4_242, 0);
    let mut stances = Vec::new();
    for z in -ENCLOSURE..=ENCLOSURE {
        for x in -ENCLOSURE..=ENCLOSURE {
            let Some(top) = world.ground().surface(x, z) else {
                continue;
            };
            let at = [x, top + 1, z];
            if world.ground().stands(at, WALKER_HEIGHT) {
                stances.push(at);
            }
        }
    }
    let encounter = stances.iter().find_map(|from| {
        stances.iter().find_map(|prey| {
            let distance = (0..3)
                .map(|axis| (from[axis] - prey[axis]).abs())
                .max()
                .unwrap_or(0);
            ((2..=8).contains(&distance)
                && spot(world.ground(), *from, *prey, 8)
                && step(world.ground(), *from, *prey) != *from)
                .then_some((*from, *prey, distance))
        })
    });
    let (from, prey_at, distance) =
        encounter.expect("the seeded enclosure has a visible grounded encounter");

    let predator_id = world.controlled_id().expect("the fixture has a founder");
    let predator = Organism::founding(
        predator_id,
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [3, 1, 1],
        from,
        300,
    );
    let prey_id = OrganismId(900);
    let prey = Organism::founding(
        prey_id,
        SpeciesId(3),
        // The policy receipt measures pursuit, not the former predicate that
        // let every predator graze. Give the jaw a live consumer target.
        Kingdom::Consumer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        prey_at,
        300,
    );
    world.organisms = vec![predator, prey];
    // An uncommanded predator: TD4 spares a *held* critter its instincts, and
    // this one is being watched, not driven.
    world.idle_run = INSTINCT_IDLE_TICKS;
    let mut twin = world.clone();

    assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
    twin.apply(Intent::Idle);

    let predator = world.controlled().expect("the predator remains controlled");
    let trace = predator
        .last_fauna_decision
        .expect("near fauna records its bounded target proposal");
    assert_eq!(trace.traits.feeding_mode, FeedingMode::Predator);
    assert_eq!(trace.traits.reach, predator.body().reach());
    assert_eq!(trace.traits.locomotion, predator.locomotion());
    assert_eq!(trace.senses.target_distance, distance);
    assert_eq!(trace.senses.target_mass_mg, 300);
    assert_eq!(trace.selected_target, Some(prey_id));
    assert_eq!(trace.selected_drive, FaunaDrive::Pursue);
    assert!(trace.scores.pursue > trace.scores.avoid);
    assert!(trace.scores.pursue > trace.scores.hold);
    assert_ne!(predator.fauna_policy.state, [0; 3]);
    assert_ne!(predator.position, from);
    assert!(world.ground().stands(predator.position, WALKER_HEIGHT));
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );

    let snapshot = crate::snapshot::snapshot(&world).expect("the policy state snapshots");
    let restored = crate::snapshot::restore(&snapshot).expect("the policy state restores");
    assert_eq!(world, restored);
}

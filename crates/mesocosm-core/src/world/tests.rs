// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::body::{Attachment, Origin, PartId, Provenance, VolumeRef, Yaw};
use crate::history::History;
use crate::organism::{Kingdom, LastSeen, Stage};
use crate::places::{
    Places, Tier, WALKER_HEIGHT, WalkerShape, route_step, spot, spot_for, step, step_for,
    surface_stance_for,
};
use crate::process::{IntakePort, NisKind};

/// A hand-authored hunter must say that its active mouth admits producer NIS.
/// Geometry still supplies the actuator and movement shape; this declaration
/// only keeps these navigation fixtures valid under typed intake.
fn producer_hunter(id: OrganismId, position: [i32; 3], half_extent: [i32; 3]) -> Organism {
    let mut hunter = Organism::founding(
        id,
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        half_extent,
        position,
        300,
    );
    admit_producers(&mut hunter);
    hunter
}

fn admit_producers(hunter: &mut Organism) {
    let mouth = hunter.body().mouth_part().expect("hunter has a mouth");
    let support = hunter
        .phenotype
        .part_port(mouth)
        .expect("seeded hunter mouth is active")
        .support()
        .expect("seeded hunter mouth has support");
    assert!(hunter.phenotype.declare_port(
        mouth,
        IntakePort::live(NisKind::Producer).supported_by(support)
    ));
}

/// Walks the critter to its nearest neighbour and returns it.
/// Lets the hand go, so the next tick's ecology drives the controlled body
/// like any other. TD4 spares a *held* critter its instincts, and the tests
/// below are about an uncommanded one; this is how they say so without
/// spending thirty ticks idling to get there.
fn let_go(world: &mut World) {
    world.idle_run = INSTINCT_IDLE_TICKS;
}

fn near_organism(world: &mut World) -> OrganismId {
    for _ in 0..400 {
        let here = world.position().expect("embodied");
        let Some((id, at)) = world
            .organisms
            .iter()
            .filter(|m| {
                Some(m.id) != world.controlled_id()
                    && m.is_alive()
                    && world
                        .controlled()
                        .is_some_and(|eater| eater.admits(m.kingdom().nis_kind(), false))
            })
            .map(|m| (m.id, m.position))
            .min_by_key(|(_, at): &(_, [i32; 3])| {
                (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
            })
        else {
            break;
        };
        if world.in_reach(at) {
            return id;
        }
        let step = [0, 1, 2].map(|a| (at[a] - here[a]).signum());
        world.apply(Intent::Move { delta: step });
    }
    panic!("nothing came within reach")
}

fn assert_grounded_near(world: &World) {
    for organism in world
        .organisms
        .iter()
        .filter(|organism| organism.is_alive() && organism.tier == Tier::Near)
    {
        assert!(
            organism
                .walker_shape()
                .stands(world.ground(), organism.position),
            "near organism {:?} left footing at {:?}",
            organism.id,
            organism.position
        );
    }
}

/// The generated entrance route for the first nest, shared with generation
/// rather than reconstructed as a second terrain fixture.
fn generated_nest_entry(seed: u64) -> Vec<[i32; 3]> {
    let grown = Places::grown(seed ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let nest = grown.nests.first().expect("seed grows a nest");
    crate::places::nest_entry(&grown, ENCLOSURE, *nest)
        .expect("seed's nest has a generated entry")
        .route
}

fn generated_sight_split(
    ground: &crate::places::Ground,
    compact: WalkerShape,
    tall: WalkerShape,
    target_shape: WalkerShape,
) -> ([i32; 3], [i32; 3]) {
    const DIRECTIONS: [[i32; 2]; 8] = [
        [1, 0],
        [-1, 0],
        [0, 1],
        [0, -1],
        [1, 1],
        [1, -1],
        [-1, 1],
        [-1, -1],
    ];
    const RANGE: i32 = 8;
    for z in -40..40 {
        for x in -40..40 {
            let Some(observer) = surface_stance_for(ground, compact, [x, 0, z]) else {
                continue;
            };
            if !tall.stands(ground, observer) {
                continue;
            }
            for distance in 3..=RANGE {
                for [dx, dz] in DIRECTIONS {
                    let Some(target) = surface_stance_for(
                        ground,
                        target_shape,
                        [x + dx * distance, 0, z + dz * distance],
                    ) else {
                        continue;
                    };
                    if !spot_for(ground, compact, observer, target_shape, target, RANGE)
                        && spot_for(ground, tall, observer, target_shape, target, RANGE)
                    {
                        return (observer, target);
                    }
                }
            }
        }
    }
    panic!("generated Ground offered no compact/tall terrain sight split");
}

#[path = "tests_feeding.rs"]
mod tests_feeding;
#[path = "tests_invariants.rs"]
mod tests_invariants;
#[path = "tests_movement_a.rs"]
mod tests_movement_a;
#[path = "tests_movement_b.rs"]
mod tests_movement_b;

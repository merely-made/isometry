// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shows one live anatomical change lifting terrain sight and therefore
//! changing an ordinary fauna decision in the same generated world.

use mesocosm_core::body::{Attachment, Provenance, Yaw};
use mesocosm_core::organism::FaunaDrive;
use mesocosm_core::places::{Ground, WalkerShape, spot_for, surface_stance_for};
use mesocosm_core::{
    Intent, Kingdom, Organism, OrganismId, SpeciesId, VolumeRef, World, state_hash,
};

const SEED: u64 = 0;
const PREY: OrganismId = OrganismId(0);
const HUNTER: OrganismId = OrganismId(900);
const RANGE: i32 = 8;

fn main() {
    let ground_world = World::new(SEED, 0);
    let compact_shape = hunter(false, [0; 3]).walker_shape();
    let tall_shape = hunter(true, [0; 3]).walker_shape();
    let prey_shape = prey([0; 3]).walker_shape();
    let (observer, target) =
        sight_split(ground_world.ground(), compact_shape, tall_shape, prey_shape);

    let mut compact = setup(observer, target, false);
    let mut tall = setup(observer, target, true);
    let mut compact_twin = compact.clone();
    let mut tall_twin = tall.clone();

    assert!(!spot_for(
        compact.ground(),
        compact_shape,
        observer,
        prey_shape,
        target,
        RANGE,
    ));
    assert!(spot_for(
        tall.ground(),
        tall_shape,
        observer,
        prey_shape,
        target,
        RANGE,
    ));

    compact.apply(Intent::Idle);
    tall.apply(Intent::Idle);
    compact_twin.apply(Intent::Idle);
    tall_twin.apply(Intent::Idle);

    assert_eq!(drive(&compact), None);
    assert_eq!(drive(&tall), Some(FaunaDrive::Pursue));
    assert_eq!(state_hash(&compact), state_hash(&compact_twin));
    assert_eq!(state_hash(&tall), state_hash(&tall_twin));

    println!(
        "body-sight receipt: seed={SEED}, observer={observer:?}, target={target:?}, \
         compact_shape={compact_shape:?}, compact_eye={:?}, compact_drive={:?}, \
         tall_shape={tall_shape:?}, tall_eye={:?}, tall_drive={:?}, \
         compact_hash={}, tall_hash={}",
        compact_shape.sight_point(observer),
        drive(&compact),
        tall_shape.sight_point(observer),
        drive(&tall),
        state_hash(&compact),
        state_hash(&tall),
    );
}

fn sight_split(
    ground: &Ground,
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
    panic!("seed {SEED} offered no terrain sight split between the two bodies");
}

fn setup(observer: [i32; 3], target: [i32; 3], tall: bool) -> World {
    let mut world = World::new(SEED, 0);
    world.organisms = vec![prey(target), hunter(tall, observer)];
    world
}

fn prey(position: [i32; 3]) -> Organism {
    Organism::founding(
        PREY,
        SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        position,
        300,
    )
}

fn hunter(tall: bool, position: [i32; 3]) -> Organism {
    let mut hunter = Organism::founding(
        HUNTER,
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [3, 1, 1],
        position,
        300,
    );
    if tall {
        hunter
            .phenotype
            .attach(
                VolumeRef::from_tag(17),
                1,
                [1, 5, 1],
                Attachment {
                    parent: hunter.body().root,
                    offset: [0, 6, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .expect("the sight stalk attaches above the root");
    }
    hunter
}

fn drive(world: &World) -> Option<FaunaDrive> {
    world
        .organisms
        .iter()
        .find(|organism| organism.id == HUNTER)
        .and_then(|organism| organism.last_fauna_decision.as_ref())
        .map(|decision| decision.selected_drive)
}

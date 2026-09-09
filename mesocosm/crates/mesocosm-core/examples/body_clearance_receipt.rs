// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shows the same ecological pursuit meeting two different bodies at one
//! generated burrow threshold.

use mesocosm_core::organism::FaunaDrive;
use mesocosm_core::places::{PlaceId, Places};
use mesocosm_core::world::{ENCLOSURE, PLACE_SALT, PLACE_SIDE};
use mesocosm_core::{
    Intent, Kingdom, Organism, OrganismId, SpeciesId, VolumeRef, World, state_hash,
};

const SEED: u64 = 0;
const PREY: OrganismId = OrganismId(0);
const HUNTER: OrganismId = OrganismId(900);

fn main() {
    let grown = Places::grown(SEED ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let route = grown
        .nest_entries(ENCLOSURE)
        .next()
        .expect("seed 0 grows a burrow entry")
        .1
        .route;
    let mut compact = setup(&route, [1, 1, 1]);
    let mut broad = setup(&route, [3, 1, 3]);
    let mut compact_twin = compact.clone();
    let mut broad_twin = broad.clone();

    compact.apply(Intent::Idle);
    broad.apply(Intent::Idle);
    compact_twin.apply(Intent::Idle);
    broad_twin.apply(Intent::Idle);

    assert_eq!(drive(&compact), Some(FaunaDrive::Pursue));
    assert_eq!(drive(&broad), Some(FaunaDrive::Pursue));
    assert_eq!(position(&compact), route[1]);
    assert_ne!(position(&broad), route[1]);
    assert_eq!(state_hash(&compact), state_hash(&compact_twin));
    assert_eq!(state_hash(&broad), state_hash(&broad_twin));

    println!(
        "body-clearance receipt: seed={SEED}, threshold={:?}->{:?}, place={:?}->{:?}, \
         compact_shape={:?}, compact_after={:?}, broad_shape={:?}, broad_after={:?}, \
         drive={:?}, compact_hash={}, broad_hash={}",
        route[0],
        route[1],
        grown.places.at(route[0]).unwrap_or(PlaceId(u16::MAX)),
        grown.places.at(route[1]).unwrap_or(PlaceId(u16::MAX)),
        hunter(&compact).walker_shape(),
        position(&compact),
        hunter(&broad).walker_shape(),
        position(&broad),
        drive(&compact),
        state_hash(&compact),
        state_hash(&broad),
    );
}

fn setup(route: &[[i32; 3]], hunter_extent: [i32; 3]) -> World {
    let mut world = World::new(SEED, 0);
    world.organisms = vec![
        Organism::founding(
            PREY,
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            route[2],
            300,
        ),
        Organism::founding(
            HUNTER,
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            hunter_extent,
            route[0],
            300,
        ),
    ];
    world
}

fn hunter(world: &World) -> &Organism {
    world
        .organisms
        .iter()
        .find(|organism| organism.id == HUNTER)
        .expect("the fixture retains its hunter")
}

fn position(world: &World) -> [i32; 3] {
    hunter(world).position
}

fn drive(world: &World) -> Option<FaunaDrive> {
    hunter(world)
        .last_fauna_decision
        .as_ref()
        .map(|decision| decision.selected_drive)
}

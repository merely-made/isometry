// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The generated threshold fixture shared by G4's native and browser hosts.
//! Selection is pinned here so both hosts present the same run.

use mesocosm_core::places::{PlaceId, Places, WALKER_HEIGHT, step};
use mesocosm_core::world::{ENCLOSURE, PLACE_SALT, PLACE_SIDE};
use mesocosm_core::{Intent, Kingdom, Organism, OrganismId, SpeciesId, VolumeRef, World};

pub const SEED: u64 = 0;
pub const PLAYER_ID: OrganismId = OrganismId(0);
pub const HUNTER_ID: OrganismId = OrganismId(900);

pub struct CrossingFixture {
    pub world: World,
    pub route: Vec<[i32; 3]>,
    pub boundary_step: usize,
    pub from_place: PlaceId,
    pub to_place: PlaceId,
    pub trace: Vec<Intent>,
}

pub fn setup() -> CrossingFixture {
    let grown = Places::grown(SEED ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let (nest, entry, boundary_step) = grown
        .nest_entries(ENCLOSURE)
        .find_map(|(nest, entry)| {
            let boundary_step = entry
                .route
                .windows(2)
                .position(|pair| grown.places.at(pair[0]) != grown.places.at(pair[1]))?;
            Some((nest, entry, boundary_step))
        })
        .expect("seed 0 has a generated nest entry crossing a place boundary");
    let route = entry.route;
    let from_place = grown
        .places
        .at(route[boundary_step])
        .expect("the threshold begins in a place");
    let to_place = grown
        .places
        .at(route[boundary_step + 1])
        .expect("the threshold ends in a place");

    assert_eq!(nest.host, PlaceId(3), "the pinned nest host moved");
    assert_eq!(boundary_step, 2, "the pinned place edge moved");
    assert_eq!(from_place, PlaceId(3));
    assert_eq!(to_place, PlaceId(0));
    assert_eq!(
        route,
        vec![
            [-8, 17, -10],
            [-8, 16, -11],
            [-8, 15, -12],
            [-8, 14, -13],
            [-8, 13, -14],
        ],
        "seed 0's G4 generated entry moved"
    );

    let mut world = World::new(SEED, 0);
    for pair in route.windows(2) {
        assert!(world.ground().stands(pair[0], WALKER_HEIGHT));
        assert_eq!(
            step(world.ground(), pair[0], pair[1]),
            pair[1],
            "generated route contains a step the owned mover refuses"
        );
    }
    assert!(world.ground().stands(*route.last().unwrap(), WALKER_HEIGHT));
    let inside = *route.last().unwrap();
    assert!(
        world.ground().solid([inside[0], inside[1] + 2, inside[2]]),
        "the inner stance lost its generated roof"
    );
    world.organisms = vec![
        Organism::founding(
            PLAYER_ID,
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            route[1],
            300,
        ),
        Organism::founding(
            HUNTER_ID,
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            [3, 1, 1],
            route[0],
            300,
        ),
    ];
    let trace = route
        .windows(2)
        .skip(1)
        .map(|pair| Intent::Move {
            delta: [0, 1, 2].map(|axis| pair[1][axis] - pair[0][axis]),
        })
        .collect();
    CrossingFixture {
        world,
        route,
        boundary_step,
        from_place,
        to_place,
        trace,
    }
}

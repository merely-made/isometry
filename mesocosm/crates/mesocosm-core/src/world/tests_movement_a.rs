// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn hunter_and_player_cross_one_generated_entry_and_place_boundary() {
    // Re-pinned 0 -> 129 by S1, and the reason is the finding: `PLACE_SIDE`
    // stayed 3 while the enclosure went 16 -> 64, so a region is now 43 voxels
    // across and a generated nest entry is 5 to 8 voxels long. An entry that
    // crosses a place boundary went from ordinary to rare — 30 of the first
    // thousand seeds still have one, and this is the first whose crossing sits
    // at the same step the pinned run always asserted **and** joins two places
    // the grown graph actually links — an unlinked pair is two hops apart, which
    // is `demote_hops`, so the hunter would answer by teleporting a region
    // rather than following a stance.
    const SEED: u64 = 172;
    let grown = Places::grown(SEED ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let (route, boundary_step, from_place, to_place) = grown
        .nest_entries(ENCLOSURE)
        .find_map(|(_, entry)| {
            let route = entry.route;
            let boundary_step = route
                .windows(2)
                .position(|step| grown.places.at(step[0]) != grown.places.at(step[1]))?;
            Some((
                route.clone(),
                boundary_step,
                grown.places.at(route[boundary_step])?,
                grown.places.at(route[boundary_step + 1])?,
            ))
        })
        .expect("seed has a generated entry crossing a place boundary");
    assert_ne!(from_place, to_place);
    assert_eq!(boundary_step, 2, "the pinned run moved its place edge");
    assert!(route.len() > boundary_step + 2);

    let mut world = World::new(SEED, 0);
    world.organisms = vec![
        Organism::founding(
            OrganismId(0),
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            route[1],
            300,
        ),
        producer_hunter(OrganismId(900), route[0], [3, 1, 1]),
    ];
    let mut twin = world.clone();
    let mut history = History::new();
    let mut twin_history = History::new();
    let mut player_positions = vec![route[1]];
    let mut hunter_positions = vec![route[0]];
    let trace = route
        .windows(2)
        .skip(1)
        .map(|step| Intent::Move {
            delta: [0, 1, 2].map(|axis| step[1][axis] - step[0][axis]),
        })
        .collect::<Vec<_>>();

    for (index, intent) in trace.iter().enumerate() {
        let target = route[index + 2];
        let player = world.position().unwrap();
        assert_eq!(step(world.ground(), player, target), target);
        let outcome = world.apply(intent.clone());
        let twin_outcome = twin.apply(intent.clone());
        history.record_all(world.drain_events());
        twin_history.record_all(twin.drain_events());
        assert_eq!(outcome, twin_outcome, "replay changed an outcome");
        assert!(matches!(outcome, Outcome::Moved));
        assert_eq!(
            world.position(),
            Some(target),
            "player stuttered on the entry"
        );
        let hunter = world
            .organisms
            .iter()
            .find(|organism| organism.id == OrganismId(900))
            .unwrap();
        assert_eq!(
            hunter.position,
            route[index + 1],
            "hunter did not follow one generated stance behind"
        );
        assert_eq!(
            hunter
                .last_fauna_decision
                .as_ref()
                .map(|decision| decision.selected_drive),
            Some(crate::organism::FaunaDrive::Pursue)
        );
        assert!(world.ground().stands(hunter.position, WALKER_HEIGHT));
        player_positions.push(target);
        hunter_positions.push(hunter.position);
    }

    assert_eq!(grown.places.at(player_positions[0]), Some(from_place));
    assert_eq!(
        grown.places.at(*player_positions.last().unwrap()),
        Some(to_place)
    );
    assert_eq!(grown.places.at(hunter_positions[0]), Some(from_place));
    assert_eq!(
        grown.places.at(*hunter_positions.last().unwrap()),
        Some(to_place)
    );
    assert_eq!(hunter_positions[1], route[1], "hunter missed the threshold");
    assert_eq!(
        player_positions.iter().collect::<BTreeSet<_>>().len(),
        player_positions.len(),
        "player stuttered"
    );
    assert_eq!(
        hunter_positions.iter().collect::<BTreeSet<_>>().len(),
        hunter_positions.len(),
        "hunter stuttered"
    );
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
    assert_eq!(history, twin_history);
}

#[test]
fn movement_spends_the_budget() {
    let mut world = World::new(3, 0);
    let from = world.position().unwrap();
    let before = world.energy_mg().unwrap();
    let upkeep = world.controlled().unwrap().upkeep_mg();
    world.apply(Intent::Move { delta: [3, 0, -2] });
    let after = world.position().unwrap();
    // A Move is now one legal kinematic step, even if its caller supplied a
    // larger offset. The ecology's later upkeep is still charged for the
    // whole tick.
    assert!(
        (after[0] - from[0]).abs() <= 1 && (after[2] - from[2]).abs() <= 1,
        "movement teleported to {after:?}"
    );
    let distance = u64::from((after[0] - from[0]).unsigned_abs())
        + u64::from((after[2] - from[2]).unsigned_abs());
    assert_eq!(world.energy_mg().unwrap(), before - distance - upkeep);
}

#[test]
fn founders_begin_on_footing() {
    let world = World::new(4_242, 24);
    for organism in &world.organisms {
        assert!(
            organism
                .walker_shape()
                .stands(world.ground(), organism.position),
            "founder {:?} begins without footing at {:?}",
            organism.id,
            organism.position
        );
    }
}

#[test]
fn a_near_consumer_descends_a_generated_roofed_nest_entry() {
    const SEED: u64 = 4_242;
    let route = generated_nest_entry(SEED);
    let from = route[0];
    let inside = *route.last().unwrap();
    let mut world = World::new(SEED, 0);
    assert!(world.ground().stands(from, WALKER_HEIGHT));
    assert!(world.ground().stands(inside, WALKER_HEIGHT));
    assert!(world.ground().solid([inside[0], inside[1] + 2, inside[2]]));
    assert!(spot(world.ground(), from, inside, 8));

    world.organisms = vec![
        Organism::founding(
            OrganismId(0),
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            inside,
            300,
        ),
        producer_hunter(OrganismId(900), from, [3, 1, 1]),
    ];
    let mut twin = world.clone();
    assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
    twin.apply(Intent::Idle);

    let hunter = world
        .organisms
        .iter()
        .find(|organism| organism.id == OrganismId(900))
        .unwrap()
        .position;
    assert!(
        route.contains(&hunter),
        "hunter left the generated entry: {hunter:?}"
    );
    assert!(
        hunter[1] < from[1],
        "hunter did not descend: {from:?} -> {hunter:?}"
    );
    assert!(world.ground().stands(hunter, WALKER_HEIGHT));
    assert_eq!(
        world
            .organisms
            .iter()
            .find(|organism| organism.id == OrganismId(900))
            .and_then(|organism| organism.last_seen),
        Some(LastSeen {
            target: OrganismId(0),
            position: inside,
            ticks_left: 8,
        }),
        "direct near-tier perception must become replayable organism state"
    );
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn the_same_pursuit_selects_between_bodies_at_a_generated_threshold() {
    const SEED: u64 = 0;
    const PREY: OrganismId = OrganismId(0);
    const HUNTER: OrganismId = OrganismId(900);
    let route = generated_nest_entry(SEED);
    let setup = |hunter_extent| {
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
            producer_hunter(HUNTER, route[0], hunter_extent),
        ];
        world
    };
    // Both bodies are actuators, and were single voxels of bulk until TD8: a
    // body with no contractile part cannot travel at all now, so a hunter with
    // none could not have demonstrated the threshold either way. The long axis
    // is vertical in both, so what still separates them is the footprint the
    // one-voxel gap has to admit.
    let mut compact = setup([1, 4, 1]);
    let mut broad = setup([3, 7, 3]);
    let mut compact_twin = compact.clone();
    let mut broad_twin = broad.clone();

    compact.apply(Intent::Idle);
    broad.apply(Intent::Idle);
    compact_twin.apply(Intent::Idle);
    broad_twin.apply(Intent::Idle);

    fn hunter(world: &World, id: OrganismId) -> &Organism {
        world
            .organisms
            .iter()
            .find(|organism| organism.id == id)
            .unwrap()
    }
    for world in [&compact, &broad] {
        assert_eq!(
            hunter(world, HUNTER)
                .last_fauna_decision
                .as_ref()
                .map(|decision| decision.selected_drive),
            Some(crate::organism::FaunaDrive::Pursue),
            "body shape must constrain the shared pursuit rather than select it"
        );
        assert!(
            hunter(world, HUNTER)
                .walker_shape()
                .stands(world.ground(), hunter(world, HUNTER).position)
        );
    }
    assert_eq!(hunter(&compact, HUNTER).walker_shape().radius(), 0);
    assert_eq!(hunter(&broad, HUNTER).walker_shape().radius(), 1);
    assert_eq!(hunter(&compact, HUNTER).position, route[1]);
    assert_ne!(
        hunter(&broad, HUNTER).position,
        route[1],
        "the broad body entered the one-voxel generated threshold"
    );
    assert_eq!(
        crate::snapshot::state_hash(&compact),
        crate::snapshot::state_hash(&compact_twin)
    );
    assert_eq!(
        crate::snapshot::state_hash(&broad),
        crate::snapshot::state_hash(&broad_twin)
    );
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn live_height_changes_perception_and_behavior_over_the_same_ground() {
    const SEED: u64 = 0;
    const PREY: OrganismId = OrganismId(0);
    const HUNTER: OrganismId = OrganismId(900);
    const RANGE: i32 = 8;
    let make_hunter = |tall: bool, position| {
        let mut hunter = producer_hunter(HUNTER, position, [3, 1, 1]);
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
                .unwrap();
        }
        hunter
    };
    let make_prey = |position| {
        Organism::founding(
            PREY,
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            position,
            300,
        )
    };
    let compact_hunter = make_hunter(false, [0; 3]);
    let tall_hunter = make_hunter(true, [0; 3]);
    let prey_shape = make_prey([0; 3]).walker_shape();
    let compact_shape = compact_hunter.walker_shape();
    let tall_shape = tall_hunter.walker_shape();
    assert_eq!(compact_hunter.feeding_mode(), tall_hunter.feeding_mode());
    assert_eq!((compact_shape.radius(), compact_shape.height()), (0, 1));
    // Four rather than three since DC1.5: a founding consumer carries the jaw
    // that makes it one, and a jaw hangs below the head. The point of the test
    // is the *difference* between the two shapes over one ground, which is
    // unchanged.
    assert_eq!((tall_shape.radius(), tall_shape.height()), (0, 4));

    let terrain = World::new(SEED, 0);
    let (observer, target) =
        generated_sight_split(terrain.ground(), compact_shape, tall_shape, prey_shape);
    let setup = |hunter: Organism| {
        let mut world = World::new(SEED, 0);
        let mut hunter = hunter;
        hunter.position = observer;
        world.organisms = vec![make_prey(target), hunter];
        world
    };
    let mut compact = setup(compact_hunter);
    let mut tall = setup(tall_hunter);
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

    let decision = |world: &World| {
        world
            .organisms
            .iter()
            .find(|organism| organism.id == HUNTER)
            .and_then(|organism| organism.last_fauna_decision.as_ref())
            .map(|decision| decision.selected_drive)
    };
    assert_eq!(decision(&compact), None);
    assert_eq!(decision(&tall), Some(crate::organism::FaunaDrive::Pursue));
    assert_eq!(
        crate::snapshot::state_hash(&compact),
        crate::snapshot::state_hash(&compact_twin)
    );
    assert_eq!(
        crate::snapshot::state_hash(&tall),
        crate::snapshot::state_hash(&tall_twin)
    );
}

#[test]
fn a_near_consumer_routes_to_a_recently_seen_target_around_a_turn() {
    // This is a player-carvable L bore, not a parallel collision fixture. The
    // prey is occluded at the turn, so the only target this tick is the
    // predator's replayed LastSeen state.
    let mut world = World::new(4_242, 0);
    for [x, z] in [[0, 0], [4, 0], [4, 4]] {
        let top = world
            .ground()
            .surface(x, z)
            .expect("the generated enclosure has surface terrain");
        world.ground.carve([x, top + 1, z], 1);
    }

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
        stances.iter().find_map(|target| {
            let direct = step(world.ground(), *from, *target);
            let routed = route_step(world.ground(), *from, *target, 8)?;
            (routed != direct
                && !spot(world.ground(), *from, *target, 8)
                && step(world.ground(), routed, *target) != routed)
                .then_some((*from, *target, routed))
        })
    });
    let (from, last_seen_at, expected_next) =
        encounter.expect("the carved bore contains an occluded local turn");
    assert!(world.ground().stands(from, WALKER_HEIGHT));
    assert!(world.ground().stands(last_seen_at, WALKER_HEIGHT));

    let predator_id = world.controlled_id().expect("the fixture has a founder");
    let mut predator = producer_hunter(predator_id, from, [3, 1, 1]);
    predator.last_seen = Some(LastSeen {
        target: OrganismId(900),
        position: last_seen_at,
        ticks_left: 2,
    });
    let prey = Organism::founding(
        OrganismId(900),
        SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        last_seen_at,
        300,
    );
    world.organisms = vec![predator, prey];
    let_go(&mut world);
    let mut twin = world.clone();

    assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
    twin.apply(Intent::Idle);
    let predator = world.controlled().expect("predator remains controlled");
    assert_eq!(
        predator.position, expected_next,
        "lost sight should take the bounded legal detour, not greedily stall"
    );
    assert_eq!(
        predator.last_seen,
        Some(LastSeen {
            target: OrganismId(900),
            position: last_seen_at,
            ticks_left: 1,
        })
    );
    assert!(world.ground().stands(predator.position, WALKER_HEIGHT));
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn a_moving_player_can_be_lost_and_reacquired_through_a_carved_turn() {
    // Keep the player and hunter in one replayed World. The test searches the
    // generated enclosure after a normal L carve, rather than giving either
    // actor a hand-authored navigation mesh or a scripted state transition.
    let mut terrain = World::new(4_242, 0);
    for [x, z] in [[0, 0], [4, 0], [4, 4]] {
        let top = terrain
            .ground()
            .surface(x, z)
            .expect("the generated enclosure has surface terrain");
        terrain.ground.carve([x, top + 1, z], 1);
    }
    let mut stances = Vec::new();
    for z in -ENCLOSURE..=ENCLOSURE {
        for x in -ENCLOSURE..=ENCLOSURE {
            let Some(top) = terrain.ground().surface(x, z) else {
                continue;
            };
            let at = [x, top + 1, z];
            if terrain.ground().stands(at, WALKER_HEIGHT) {
                stances.push(at);
            }
        }
    }

    let candidate = stances.iter().find_map(|from| {
        stances.iter().find_map(|seen| {
            if !spot(terrain.ground(), *from, *seen, 8) || seen == from {
                return None;
            }
            [[1, 0], [-1, 0], [0, 1], [0, -1]]
                .into_iter()
                .find_map(|[dx, dz]| {
                    let hidden = step(
                        terrain.ground(),
                        *seen,
                        [seen[0] + dx, seen[1], seen[2] + dz],
                    );
                    if hidden == *seen || !terrain.ground().stands(hidden, WALKER_HEIGHT) {
                        return None;
                    }

                    let mut initial = terrain.clone();
                    initial.organisms = vec![
                        Organism::founding(
                            OrganismId(0),
                            SpeciesId(3),
                            Kingdom::Producer,
                            VolumeRef::from_tag(18),
                            [1, 1, 1],
                            *seen,
                            300,
                        ),
                        producer_hunter(OrganismId(900), *from, [3, 1, 1]),
                    ];
                    let mut probe = initial.clone();
                    let mut trace = vec![Intent::Idle];
                    probe.apply(Intent::Idle);
                    let hunter_after_sight = probe
                        .organisms
                        .iter()
                        .find(|organism| organism.id == OrganismId(900))
                        .expect("the short pursuit preserves its hunter")
                        .position;
                    let saw = probe
                        .organisms
                        .iter()
                        .find(|organism| organism.id == OrganismId(900))
                        .and_then(|organism| organism.last_seen)
                        .is_some_and(|memory| memory.position == *seen);
                    if !saw || spot(probe.ground(), hunter_after_sight, hidden, 8) {
                        return None;
                    }

                    let moved = probe.apply(Intent::Move {
                        delta: [
                            hidden[0] - seen[0],
                            hidden[1] - seen[1],
                            hidden[2] - seen[2],
                        ],
                    });
                    trace.push(Intent::Move {
                        delta: [
                            hidden[0] - seen[0],
                            hidden[1] - seen[1],
                            hidden[2] - seen[2],
                        ],
                    });
                    if !matches!(moved, Outcome::Moved) || probe.position() != Some(hidden) {
                        return None;
                    }
                    let lost = probe
                        .organisms
                        .iter()
                        .find(|organism| organism.id == OrganismId(900))
                        .and_then(|organism| organism.last_seen)
                        .is_some_and(|memory| memory.position == *seen && memory.ticks_left < 8);
                    if !lost {
                        return None;
                    }

                    for _ in 0..8 {
                        probe.apply(Intent::Idle);
                        trace.push(Intent::Idle);
                        let reacquired = probe
                            .organisms
                            .iter()
                            .find(|organism| organism.id == OrganismId(900))
                            .and_then(|organism| organism.last_seen)
                            .is_some_and(|memory| {
                                memory.position == hidden && memory.ticks_left == 8
                            });
                        if reacquired {
                            return Some((initial, trace, hidden));
                        }
                    }
                    None
                })
        })
    });
    let (mut world, trace, hidden) =
        candidate.expect("the carved turn admits a lost-sight reacquisition run");
    let mut twin = world.clone();
    for intent in trace {
        world.apply(intent.clone());
        twin.apply(intent);
        assert_grounded_near(&world);
    }
    let memory = world
        .organisms
        .iter()
        .find(|organism| organism.id == OrganismId(900))
        .and_then(|organism| organism.last_seen)
        .expect("the hunter reacquired the player");
    assert_eq!(memory.position, hidden);
    assert_eq!(memory.ticks_left, 8);
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn a_carve_opens_a_grounded_step_and_replays() {
    // Find a two-high face that owned locomotion cannot cross, then carve a
    // one-voxel doorway through it. The fixture searches real generated
    // ground rather than constructing a second terrain authority for a test.
    let mut world = World::new(4_242, 0);
    // **A compact body, put there on purpose** (DC4). The played critter founds
    // from an archetype now and its legs make it a five-column walker, which
    // cannot stand beside a two-high face at all — so it can never be the body
    // that carves a one-voxel doorway. That is a real consequence of the roster
    // and it is recorded as a finding; this test is about carving and replay,
    // so it drives a standard walker the way the rest of the near-tier fixtures
    // do.
    world.organisms[0] = Organism::founding(
        OrganismId(0),
        SpeciesId(1),
        Kingdom::Consumer,
        VolumeRef::from_tag(1),
        [1, 1, 1],
        [0, 0, 0],
        300,
    );
    let shape = world.organisms[0].walker_shape();
    let directions = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let doorway = (-ENCLOSURE..=ENCLOSURE).find_map(|z| {
        (-ENCLOSURE..=ENCLOSURE).find_map(|x| {
            let top = world.ground().surface(x, z)?;
            let from = [x, top + 1, z];
            if !shape.stands(world.ground(), from) {
                return None;
            }
            directions.into_iter().find_map(|[dx, dz]| {
                let target = [from[0] + dx, from[1], from[2] + dz];
                let blocked = step_for(world.ground(), shape, from, target) == from
                    && world.ground().solid(target)
                    && world.ground().solid([target[0], target[1] + 1, target[2]])
                    && world.ground().solid([target[0], target[1] - 1, target[2]]);
                blocked.then_some((from, target))
            })
        })
    });
    let (from, target) = doorway.expect("the seeded terrain contains a climb-blocking face");
    let me = world.controlled_id().expect("embodied");
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == me)
        .expect("the controlled organism exists")
        .position = from;

    let mut twin = world.clone();
    let opening = [target[0], target[1] + 1, target[2]];
    let trace = [
        Intent::Carve {
            at: opening,
            radius: shape.radius().max(1),
        },
        Intent::Move {
            delta: [target[0] - from[0], 0, target[2] - from[2]],
        },
    ];
    let outcomes = world.apply_all(&trace);
    twin.apply_all(&trace);

    assert!(matches!(outcomes[0], Outcome::Carved { removed, .. } if removed > 0));
    assert_eq!(
        world.position(),
        Some(target),
        "doorway did not admit the step"
    );
    assert!(shape.stands(world.ground(), target));
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn autonomous_near_bodies_need_sight_and_take_grounded_steps() {
    // Use the same generated wall as the player receipt, but put an
    // uncommanded predator behind it. Before the opening it cannot acquire
    // the producer; the carve makes the prey visible and lets ecology cross
    // one legal voxel into the doorway on its own tick.
    let mut world = World::new(4_242, 0);
    let directions = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let encounter = (-ENCLOSURE..=ENCLOSURE).find_map(|z| {
        (-ENCLOSURE..=ENCLOSURE).find_map(|x| {
            let top = world.ground().surface(x, z)?;
            let from = [x, top + 1, z];
            if !world.ground().stands(from, WALKER_HEIGHT) {
                return None;
            }
            directions.into_iter().find_map(|[dx, dz]| {
                let doorway = [from[0] + dx, from[1], from[2] + dz];
                let prey_top = world.ground().surface(doorway[0] + dx, doorway[2] + dz)?;
                let prey = [doorway[0] + dx, prey_top + 1, doorway[2] + dz];
                let blocked = step(world.ground(), from, doorway) == from
                    && world.ground().solid(doorway)
                    && world
                        .ground()
                        .solid([doorway[0], doorway[1] + 1, doorway[2]])
                    && world
                        .ground()
                        .solid([doorway[0], doorway[1] - 1, doorway[2]]);
                let close = (0..3).all(|axis| (prey[axis] - from[axis]).abs() <= 8);
                (blocked
                    && close
                    && world.ground().stands(prey, WALKER_HEIGHT)
                    && !spot(world.ground(), from, prey, 8))
                .then_some((from, doorway, prey))
            })
        })
    });
    let (from, doorway, prey_at) =
        encounter.expect("the seeded terrain contains a nearby occluded doorway");
    let predator_id = world.controlled_id().expect("the fixture has a founder");
    let predator = producer_hunter(predator_id, from, [3, 1, 1]);
    let prey = Organism::founding(
        OrganismId(900),
        SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        prey_at,
        300,
    );
    world.organisms = vec![predator, prey];
    let_go(&mut world);

    assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
    assert_eq!(
        world.controlled().map(|organism| organism.position),
        Some(from),
        "an occluded prey must not become an abstract steering target"
    );

    let mut twin = world.clone();
    let opening = [doorway[0], doorway[1] + 1, doorway[2]];
    let outcome = world.apply(Intent::Carve {
        at: opening,
        radius: 1,
    });
    twin.apply(Intent::Carve {
        at: opening,
        radius: 1,
    });
    assert!(matches!(outcome, Outcome::Carved { removed, .. } if removed > 0));

    // The carve was the hand's, so that tick belongs to the hand and this body
    // holds still through it (TD4). Let go again and the ecology takes the
    // opening on the next tick, which is the claim under test.
    assert_eq!(
        world.controlled().map(|organism| organism.position),
        Some(from),
        "the tick a player acts on is the player's"
    );
    for world in [&mut world, &mut twin] {
        let_go(world);
        world.apply(Intent::Idle);
    }

    assert_eq!(
        world.controlled().map(|organism| organism.position),
        Some(doorway),
        "the embodied ecology did not enter the opened doorway"
    );
    assert!(world.ground().stands(doorway, WALKER_HEIGHT));
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

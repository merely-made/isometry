// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use paredros_room::crossing::{
    self, BodyId, BodyKind, ContactWorld, HeldInput, Input, TriggeredInput,
};

fn walk(world: &mut ContactWorld, id: BodyId, x: f32, z: f32, ticks: usize) {
    for _ in 0..ticks {
        world
            .step(&[(
                id,
                Input {
                    held: HeldInput {
                        move_x: x,
                        move_z: z,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )])
            .unwrap();
    }
}

#[test]
fn loose_board_supports_the_actual_crossing() {
    let (mut world, id) = crossing::new_world(BodyKind::Crawler);
    world
        .step(&[(
            id,
            Input {
                triggered: TriggeredInput {
                    interact: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )])
        .unwrap();
    assert_eq!(world.board_holder(), Some(id));
    walk(&mut world, id, 1., 0., 80);
    world
        .step(&[(
            id,
            Input {
                triggered: TriggeredInput {
                    interact: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )])
        .unwrap();
    assert_eq!(world.board_holder(), None);
    let mut touched_board = false;
    for _ in 0..100 {
        walk(&mut world, id, 1., 0., 1);
        let body = world.body(id).unwrap();
        assert!(
            body.position[1] >= -0.01,
            "the board route must not drop into the channel: {body:?}"
        );
        touched_board |= body.position[1] > 0.15;
    }
    assert!(touched_board);
    assert!(world.body(id).unwrap().position[0] > 2.);
    assert_eq!(
        ContactWorld::restore(&world.save().unwrap()).unwrap(),
        world
    );
}

#[test]
fn channel_is_a_recoverable_route_for_both_bodies() {
    for kind in [BodyKind::Crawler, BodyKind::Climber] {
        let (mut world, id) = crossing::new_world(kind);
        // Avoid the board and the overhead support. Stop near the channel centre.
        walk(
            &mut world,
            id,
            0.,
            1.,
            if kind == BodyKind::Crawler { 36 } else { 27 },
        );
        walk(
            &mut world,
            id,
            1.,
            0.,
            if kind == BodyKind::Crawler { 120 } else { 90 },
        );
        walk(&mut world, id, 0., 0., 60);
        assert!(world.body(id).unwrap().position[1] < -1.);
        walk(&mut world, id, 0., 1., 140);
        walk(&mut world, id, 1., 0., 70);
        let body = world.body(id).unwrap();
        assert!(
            body.position[0] > 2. && body.position[1] >= -0.01,
            "exit steps should reach the bank: {body:?}"
        );
    }
}

#[test]
fn fixture_camera_and_visible_geometry_are_finite() {
    let (world, id) = crossing::new_world(BodyKind::Climber);
    for yaw in [0., 1.5, 3., 4.5] {
        let camera = crossing::camera(&world, id, yaw, 12.);
        assert!(camera.eye.is_finite() && camera.view.is_finite());
        assert!(camera.eye.distance(camera.target) >= 0.5);
    }
    for vertex in crossing::geometry(&world)
        .into_iter()
        .chain(crossing::body_geometry(&world))
    {
        assert!(
            vertex
                .position
                .into_iter()
                .chain(vertex.color)
                .all(f32::is_finite)
        );
    }
    assert!(crossing::geometry(&world).len() >= world.solids().len() * 36);
}

#[test]
fn release_escapes_the_held_plank_trap_and_replays() {
    let (mut world, id) = crossing::new_world(BodyKind::Crawler);
    let action = |release, interact| Input {
        triggered: TriggeredInput {
            release,
            interact,
            ..Default::default()
        },
        ..Default::default()
    };
    world.step(&[(id, action(false, true))]).unwrap();
    walk(&mut world, id, 1., 0., 130);
    walk(&mut world, id, 0., 0., 60);
    let trapped = *world.body(id).unwrap();
    assert_eq!(world.board_holder(), Some(id));
    assert!(!trapped.grounded && trapped.vertical_velocity < 0.);
    walk(&mut world, id, 0., 0., 10);
    assert_eq!(world.body(id).unwrap().position, trapped.position);
    world.step(&[(id, action(false, true))]).unwrap();
    assert_eq!(world.board_holder(), Some(id), "placement is obstructed");
    let board_before = world.board().unwrap().position;
    // Release wins over a simultaneous placement request.
    world.step(&[(id, action(true, true))]).unwrap();
    assert_eq!(world.board_holder(), None);
    assert!(world.body(id).unwrap().position[1] < trapped.position[1]);
    assert!(world.board().unwrap().position[1] <= board_before[1]);
    assert_eq!(
        ContactWorld::restore(&world.save().unwrap()).unwrap(),
        world
    );
    walk(&mut world, id, 0., 1., 160);
    walk(&mut world, id, 1., 0., 70);
    let body = world.body(id).unwrap();
    assert!(
        body.position[0] > 2. && body.position[1] >= -0.01,
        "{body:?}"
    );
    assert_eq!(
        ContactWorld::restore(&world.save().unwrap()).unwrap(),
        world
    );
}

use paredros_world::{
    BodyKind, BoxCollider, ContactEffect, ContactSave, ContactWorld, HeldInput, Input, InputFrame,
    MovableBoard, TriggeredInput,
};

fn box_(min: [f32; 3], max: [f32; 3]) -> BoxCollider {
    BoxCollider { min, max }
}
fn floor() -> BoxCollider {
    box_([-20.0, -1.0, -20.0], [20.0, 0.0, 20.0])
}
fn input(move_x: f32, move_z: f32) -> Input {
    Input {
        held: HeldInput {
            move_x,
            move_z,
            ..HeldInput::default()
        },
        ..Input::default()
    }
}

#[test]
fn fixed_steps_land_and_do_not_pass_an_authored_wall() {
    let mut world = ContactWorld::new(vec![floor(), box_([1.0, 0.0, -2.0], [2.0, 3.0, 2.0])]);
    let crawler = world.add_body(BodyKind::Crawler, [0.0, 2.0, 0.0]).unwrap();
    for _ in 0..90 {
        world.step(&[(crawler, input(1.0, 0.0))]).unwrap();
    }
    let state = world.body(crawler).unwrap();
    assert!(state.grounded);
    assert!(
        state.position[1].abs() < 0.004,
        "shared controller clearance"
    );
    assert!(state.position[0] < 0.46, "body must remain before wall");
    assert!(
        world
            .effects()
            .iter()
            .any(|effect| matches!(effect, ContactEffect::Blocked { .. }))
    );
}

#[test]
fn restore_rejects_bad_initial_configuration_and_frame_order() {
    let invalid_profile = paredros_world::ContactBodyProfile {
        half_extents: [0.0, 0.4, 0.4],
        ..paredros_world::ContactBodyProfile::crawler()
    };
    let malformed = ContactSave {
        version: 2,
        solids: vec![floor()],
        bodies: vec![(BodyKind::Crawler, invalid_profile, [0.0; 3])],
        board: None,
        frames: vec![],
    };
    assert!(ContactWorld::restore(&malformed.to_bytes().unwrap()).is_err());
    let nan = ContactSave {
        version: 2,
        solids: vec![floor()],
        bodies: vec![(
            BodyKind::Crawler,
            paredros_world::ContactBodyProfile::crawler(),
            [f32::NAN, 0.0, 0.0],
        )],
        board: None,
        frames: vec![],
    };
    assert!(ContactWorld::restore(&nan.to_bytes().unwrap()).is_err());
    let wrong_version = ContactSave {
        version: 9,
        solids: vec![floor()],
        bodies: vec![],
        board: None,
        frames: vec![],
    };
    assert!(matches!(
        ContactWorld::restore(&wrong_version.to_bytes().unwrap()),
        Err(paredros_world::ContactError::UnsupportedVersion(9))
    ));
    let bad_tick = ContactSave {
        version: 2,
        solids: vec![floor()],
        bodies: vec![],
        board: None,
        frames: vec![InputFrame {
            tick: 3,
            inputs: vec![],
        }],
    };
    assert!(matches!(
        ContactWorld::restore(&bad_tick.to_bytes().unwrap()),
        Err(paredros_world::ContactError::InvalidFrameTick { .. })
    ));
}

#[test]
fn configuration_locks_after_first_tick_and_rejects_out_of_range_input() {
    let mut world = ContactWorld::new(vec![floor()]);
    let crawler = world.add_body(BodyKind::Crawler, [0.0; 3]).unwrap();
    assert!(world.step(&[(crawler, input(1.01, 0.0))]).is_err());
    assert_eq!(world.tick(), 0);
    world.step(&[]).unwrap();
    assert!(matches!(
        world.add_body(BodyKind::Climber, [1.0; 3]),
        Err(paredros_world::ContactError::ConfigurationLocked)
    ));
    assert!(matches!(
        world.set_board(MovableBoard {
            collider: box_([-1.0, 0.0, -1.0], [1.0, 0.2, 1.0]),
            position: [0.0; 3],
            mass: 1.0
        }),
        Err(paredros_world::ContactError::ConfigurationLocked)
    ));
}

#[test]
fn crawler_carries_and_places_a_board_that_becomes_floor_support() {
    let mut world = ContactWorld::new(vec![
        box_([-10.0, -1.0, -10.0], [-1.3, 0.0, 10.0]),
        box_([1.3, -1.0, -10.0], [10.0, 0.0, 10.0]),
    ]);
    let crawler = world.add_body(BodyKind::Crawler, [-5.8, 0.0, 0.0]).unwrap();
    world
        .set_board(MovableBoard {
            collider: box_([-1.8, 0.0, -0.6], [1.8, 0.2, 0.6]),
            position: [-4.0, 0.0, 0.0],
            mass: 1.0,
        })
        .unwrap();
    world
        .step(&[(
            crawler,
            Input {
                triggered: TriggeredInput {
                    interact: true,
                    ..TriggeredInput::default()
                },
                ..Input::default()
            },
        )])
        .unwrap();
    assert!(
        world
            .effects()
            .iter()
            .any(|effect| matches!(effect, ContactEffect::BoardHeld { .. }))
    );
    for _ in 0..70 {
        world.step(&[(crawler, input(1.0, 0.0))]).unwrap();
    }
    world
        .step(&[(
            crawler,
            Input {
                held: HeldInput {
                    look_x: 1.0,
                    ..HeldInput::default()
                },
                triggered: TriggeredInput {
                    interact: true,
                    ..TriggeredInput::default()
                },
            },
        )])
        .unwrap();
    let board = world.board().unwrap();
    assert!(board.position[0] > -1.5 && board.position[0] < 1.5);
    for _ in 0..120 {
        world.step(&[(crawler, input(1.0, 0.0))]).unwrap();
    }
    assert!(
        world.body(crawler).unwrap().position[0] > 1.3,
        "placed board supports a crossing"
    );
}

#[test]
fn contact_impairment_recovers_and_saved_inputs_replay() {
    let mut world = ContactWorld::new(vec![floor(), box_([-1.0, 3.0, -1.0], [1.0, 3.2, 1.0])]);
    let crawler = world.add_body(BodyKind::Crawler, [0.0, 0.0, 0.0]).unwrap();
    let climber = world.add_body(BodyKind::Climber, [1.0, 0.0, 0.0]).unwrap();
    world
        .step(&[(
            crawler,
            Input {
                held: HeldInput {
                    look_x: 1.0,
                    ..HeldInput::default()
                },
                triggered: TriggeredInput {
                    attack: true,
                    ..TriggeredInput::default()
                },
            },
        )])
        .unwrap();
    for _ in 0..8 {
        world.step(&[]).unwrap();
    }
    assert!(world.body(climber).unwrap().impairment.is_some());
    world
        .step(&[(
            climber,
            Input {
                triggered: TriggeredInput {
                    recover: true,
                    ..TriggeredInput::default()
                },
                ..Input::default()
            },
        )])
        .unwrap();
    for _ in 0..31 {
        world.step(&[]).unwrap();
    }
    assert_eq!(world.body(climber).unwrap().impairment, None);
    let restored = ContactWorld::restore(&world.save().unwrap()).unwrap();
    assert_eq!(restored, world);
}

#[test]
fn climber_anchor_catches_fall_and_nonfinite_input_does_not_record() {
    let mut world = ContactWorld::new(vec![
        box_([-20., -6., -20.], [20., -5., 20.]),
        box_([-1.0, 3.0, -1.0], [1.0, 3.2, 1.0]),
    ]);
    let climber = world.add_body(BodyKind::Climber, [0.0, 1.2, 0.0]).unwrap();
    world
        .step(&[(
            climber,
            Input {
                triggered: TriggeredInput {
                    anchor: true,
                    ..TriggeredInput::default()
                },
                ..Input::default()
            },
        )])
        .unwrap();
    for _ in 0..60 {
        world.step(&[]).unwrap();
    }
    assert!(world.body(climber).unwrap().anchored);
    let body = world.body(climber).unwrap();
    assert!(
        body.position[1] >= -0.001 && !body.grounded,
        "a short tether permits slack before arresting a fall"
    );
    let anchor = body.anchor.unwrap();
    let distance = ((body.position[0] - anchor[0]).powi(2)
        + (body.position[1] - anchor[1]).powi(2)
        + (body.position[2] - anchor[2]).powi(2))
    .sqrt();
    assert!(distance <= body.profile.tether_length + 0.001);
    let count = world.recorded_inputs().len();
    assert!(world.step(&[(climber, input(f32::NAN, 0.0))]).is_err());
    assert_eq!(world.recorded_inputs().len(), count);
}

#[test]
fn authored_low_step_is_walkable_but_a_wall_is_not() {
    let mut world = ContactWorld::new(vec![
        floor(),
        box_([0.0, 0.0, -2.0], [0.8, 0.2, 2.0]),
        box_([2.0, 0.0, -2.0], [2.8, 2.0, 2.0]),
    ]);
    let crawler = world.add_body(BodyKind::Crawler, [-0.8, 0.0, 0.0]).unwrap();
    for _ in 0..40 {
        world.step(&[(crawler, input(1.0, 0.0))]).unwrap();
    }
    let body = world.body(crawler).unwrap();
    assert!(body.position[0] > 0.8 && body.position[0] < 1.5);
    assert!(
        (body.position[1] - 0.2).abs() < 0.004,
        "shared controller clearance: {body:?}"
    );
}

#[test]
fn grip_impairment_prevents_reanchor_and_attack_cooldown_applies_on_hit() {
    let mut world = ContactWorld::new(vec![floor(), box_([-1.0, 3.0, 0.8], [1.0, 3.2, 1.2])]);
    let climber = world.add_body(BodyKind::Climber, [0.0, 1.2, 0.0]).unwrap();
    let crawler = world.add_body(BodyKind::Crawler, [1.0, 1.2, 0.0]).unwrap();
    world
        .step(&[
            (
                climber,
                Input {
                    triggered: TriggeredInput {
                        anchor: true,
                        ..TriggeredInput::default()
                    },
                    ..Input::default()
                },
            ),
            (
                crawler,
                Input {
                    held: HeldInput {
                        look_x: -1.0,
                        ..HeldInput::default()
                    },
                    triggered: TriggeredInput {
                        attack: true,
                        ..TriggeredInput::default()
                    },
                },
            ),
        ])
        .unwrap();
    for _ in 0..8 {
        world.step(&[]).unwrap();
    }
    assert_eq!(
        world.body(climber).unwrap().impairment,
        Some(paredros_world::Impairment::Grip)
    );
    assert!(!world.body(climber).unwrap().anchored);
    world
        .step(&[(
            climber,
            Input {
                triggered: TriggeredInput {
                    anchor: true,
                    ..TriggeredInput::default()
                },
                ..Input::default()
            },
        )])
        .unwrap();
    assert!(!world.body(climber).unwrap().anchored);
    assert!(world.body(crawler).unwrap().attack_cooldown > 0);
}

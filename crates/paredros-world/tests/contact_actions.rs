// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use paredros_world::{BodyKind, BoxCollider, ContactWorld, HeldInput, Input, TriggeredInput};

fn attack(look_x: f32) -> Input {
    Input {
        held: HeldInput {
            look_x,
            ..Default::default()
        },
        triggered: TriggeredInput {
            attack: true,
            ..Default::default()
        },
    }
}

#[test]
fn facing_and_a_thin_wall_each_prevent_a_hit_but_not_recovery_cost() {
    for (facing, wall) in [(-1., false), (1., true)] {
        let mut solids = vec![BoxCollider {
            min: [-10., -1., -10.],
            max: [10., 0., 10.],
        }];
        if wall {
            solids.push(BoxCollider {
                min: [0.56, 0., -1.],
                max: [0.60, 2., 1.],
            });
        }
        let mut world = ContactWorld::new(solids);
        let source = world.add_body(BodyKind::Crawler, [0., 0., 0.]).unwrap();
        let target = world.add_body(BodyKind::Climber, [1., 0., 0.]).unwrap();
        world.step(&[(source, attack(facing))]).unwrap();
        for _ in 0..8 {
            world.step(&[]).unwrap();
        }
        assert_eq!(world.body(target).unwrap().integrity, 3.);
        assert_eq!(world.body(target).unwrap().impairment, None);
        assert!(world.body(source).unwrap().attack_cooldown > 0);
        world.step(&[(source, attack(1.))]).unwrap();
        assert_eq!(world.body(source).unwrap().attack_remaining, 0);
    }
}

#[test]
fn grounded_brace_preserves_capability_and_halves_damage() {
    for braced in [false, true] {
        let mut world = ContactWorld::new(vec![BoxCollider {
            min: [-10., -1., -10.],
            max: [10., 0., 10.],
        }]);
        let source = world.add_body(BodyKind::Crawler, [0., 0., 0.]).unwrap();
        let target = world.add_body(BodyKind::Climber, [1., 0., 0.]).unwrap();
        let guard = Input {
            held: HeldInput {
                brace: braced,
                ..Default::default()
            },
            ..Default::default()
        };
        world
            .step(&[(source, attack(1.)), (target, guard)])
            .unwrap();
        for _ in 0..8 {
            world.step(&[(target, guard)]).unwrap();
        }
        let result = world.body(target).unwrap();
        assert_eq!(result.integrity, if braced { 2.5 } else { 2. });
        assert_eq!(result.impairment.is_none(), braced);
    }
}

#[test]
fn airborne_release_needs_no_support_and_lands_without_tunnelling() {
    use paredros_world::{ContactEffect, MovableBoard};
    let mut world = ContactWorld::new(vec![BoxCollider {
        min: [-10., -0.01, -10.],
        max: [10., 0., 10.],
    }]);
    let holder = world.add_body(BodyKind::Crawler, [0., 3., 0.]).unwrap();
    let other = world.add_body(BodyKind::Crawler, [5., 0., 0.]).unwrap();
    world
        .set_board(MovableBoard {
            collider: BoxCollider {
                min: [-0.2, 0., -0.2],
                max: [0.2, 0.1, 0.2],
            },
            position: [0., 3., 0.8],
            mass: 1.,
        })
        .unwrap();
    let release = Input {
        triggered: TriggeredInput {
            release: true,
            ..Default::default()
        },
        ..Default::default()
    };
    world
        .step(&[(
            holder,
            Input {
                triggered: TriggeredInput {
                    interact: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )])
        .unwrap();
    world.step(&[(other, release)]).unwrap();
    assert_eq!(world.board_holder(), Some(holder));
    let before = world.board().unwrap().position;
    world.step(&[(holder, release)]).unwrap();
    assert_eq!(world.board_holder(), None);
    assert!(world.board().unwrap().position[1] < before[1]);
    assert!(world.effects().contains(&ContactEffect::BoardReleased {
        by: holder,
        position: before
    }));
    assert_eq!(
        ContactWorld::restore(&world.save().unwrap()).unwrap(),
        world
    );
    for _ in 0..120 {
        world.step(&[]).unwrap();
    }
    // A released board may rest on its former holder. Walking out from under
    // it must remove that support and let it settle onto the thin floor.
    assert!(world.board().unwrap().position[1] > 0.8);
    for _ in 0..90 {
        world
            .step(&[(
                holder,
                Input {
                    held: HeldInput {
                        move_x: -1.,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )])
            .unwrap();
    }
    assert_eq!(world.board().unwrap().position[1], 0.);
    world.step(&[(holder, release)]).unwrap();
    assert!(
        !world
            .effects()
            .iter()
            .any(|e| matches!(e, ContactEffect::BoardReleased { .. }))
    );
    assert_eq!(
        ContactWorld::restore(&world.save().unwrap()).unwrap(),
        world
    );
}

#[test]
fn old_contact_input_grammar_is_explicitly_rejected() {
    assert_eq!(
        ContactWorld::restore(&[1]),
        Err(paredros_world::ContactError::UnsupportedVersion(1))
    );
}

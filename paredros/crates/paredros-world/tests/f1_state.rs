// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use paredros_identity::{SubjectId, Tick};
use paredros_world::{
    BodyError, DeathCause, GameError, GameEvent, GameIntent, GameState, ItemId, ItemKind,
    ItemLocation, MOBILITY_WOUND, Name, Navigation, SiteKind, World, WorldConfig,
};

const SEED: u64 = 4_242;
const NAMED: SubjectId = SubjectId(1);
const PEER: SubjectId = SubjectId(44);

fn world() -> World {
    World::generate(SEED, WorldConfig::default()).unwrap()
}

fn settlement_start(world: &World) -> [i32; 3] {
    let settlement = world
        .map()
        .slots_of_kind(SiteKind::Settlement)
        .next()
        .unwrap();
    Navigation::default()
        .surface_stance(world, settlement)
        .unwrap()
}

fn ruin_route(state: &GameState, subject: SubjectId) -> Vec<[i32; 3]> {
    let ruin = state
        .world()
        .map()
        .slots_of_kind(SiteKind::Ruin)
        .next()
        .unwrap();
    Navigation::default()
        .route_to_slot(
            state.world(),
            state.movement().position(subject).unwrap(),
            ruin,
        )
        .unwrap()
}

fn intent_tick(state: &GameState) -> Tick {
    state.next_tick()
}

fn apply_pair(
    straight: &mut GameState,
    restored: &mut GameState,
    intent: GameIntent,
) -> Vec<GameEvent> {
    assert_eq!(straight.next_tick(), restored.next_tick());
    let first = straight.apply(intent.clone()).unwrap();
    let second = restored.apply(intent).unwrap();
    assert_eq!(first, second);
    first
}

fn sustain_pair(straight: &mut GameState, restored: &mut GameState, subject: SubjectId) {
    let body = straight.bodies().get(subject).unwrap();
    if body.needs.fatigue >= 80 {
        let intent = GameIntent::Rest {
            tick: intent_tick(straight),
            subject,
        };
        apply_pair(straight, restored, intent);
    }
    let body = straight.bodies().get(subject).unwrap();
    if body.needs.hunger >= 35 {
        let food = straight
            .items()
            .carried_by(subject)
            .find(|item| item.kind == ItemKind::Food)
            .map(|item| item.id);
        if let Some(item) = food {
            let intent = GameIntent::Eat {
                tick: intent_tick(straight),
                subject,
                item,
            };
            apply_pair(straight, restored, intent);
        }
    }
}

#[test]
fn one_body_lives_under_composed_system_rules_and_replays_exactly() {
    let world = world();
    let start = settlement_start(&world);
    let mut state = GameState::new(world);

    state
        .apply(GameIntent::Generate {
            tick: intent_tick(&state),
            subject: NAMED,
            body_seed: 17,
            at: start,
        })
        .unwrap();
    assert!(!state.bodies().get(NAMED).unwrap().alive());
    let rejected_at = state.next_tick();
    assert_eq!(
        state.apply(GameIntent::Wait {
            tick: rejected_at,
            subject: NAMED,
        }),
        Err(GameError::Body(BodyError::Unnamed(NAMED)))
    );
    assert_eq!(state.next_tick(), rejected_at, "refusal is not authority");

    state
        .apply(GameIntent::Name {
            tick: intent_tick(&state),
            subject: NAMED,
            name: Name::new("Rill").unwrap(),
        })
        .unwrap();
    let body = state.bodies().get(NAMED).unwrap();
    assert!(body.alive());
    assert_eq!(body.born_at, Some(Tick(1)));

    let sight_range = body.profile.sight_range;
    let near = state
        .apply(GameIntent::Observe {
            tick: intent_tick(&state),
            subject: NAMED,
            target: start,
        })
        .unwrap();
    assert!(matches!(
        near.as_slice(),
        [GameEvent::Observed { visible: true, .. }]
    ));
    let far = state
        .apply(GameIntent::Observe {
            tick: intent_tick(&state),
            subject: NAMED,
            target: [start[0] + sight_range + 1, start[1], start[2]],
        })
        .unwrap();
    assert!(matches!(
        far.as_slice(),
        [GameEvent::Observed { visible: false, .. }]
    ));

    let supplies = state
        .items()
        .at(start)
        .filter(|item| matches!(item.kind, ItemKind::Food | ItemKind::Dressing))
        .map(|item| item.id)
        .collect::<Vec<_>>();
    assert_eq!(supplies.len(), 5);
    for item in supplies {
        state
            .apply(GameIntent::Take {
                tick: intent_tick(&state),
                subject: NAMED,
                item,
            })
            .unwrap();
    }
    assert_eq!(state.items().carried_by(NAMED).count(), 5);
    assert!(state.items().carried_mass_mg(NAMED) > 0);
    assert!(state.bodies().get(NAMED).unwrap().needs.hunger > 0);

    let route = ruin_route(&state, NAMED);
    let midpoint = route.len() / 2;
    for toward in route.iter().copied().take(midpoint).skip(1) {
        let needs = state.bodies().get(NAMED).unwrap().needs;
        if needs.fatigue >= 80 {
            state
                .apply(GameIntent::Rest {
                    tick: intent_tick(&state),
                    subject: NAMED,
                })
                .unwrap();
        }
        if state.bodies().get(NAMED).unwrap().needs.hunger >= 35 {
            let food = {
                state
                    .items()
                    .carried_by(NAMED)
                    .find(|item| item.kind == ItemKind::Food)
                    .map(|item| item.id)
            };
            if let Some(item) = food {
                state
                    .apply(GameIntent::Eat {
                        tick: intent_tick(&state),
                        subject: NAMED,
                        item,
                    })
                    .unwrap();
            }
        }
        state
            .apply(GameIntent::Move {
                tick: intent_tick(&state),
                subject: NAMED,
                toward,
            })
            .unwrap();
    }

    let bytes = state.save().unwrap();
    let mut straight = state;
    let mut restored = GameState::restore(&bytes).unwrap();
    assert_eq!(restored, straight);

    for toward in route.into_iter().skip(midpoint) {
        sustain_pair(&mut straight, &mut restored, NAMED);
        let intent = GameIntent::Move {
            tick: intent_tick(&straight),
            subject: NAMED,
            toward,
        };
        apply_pair(&mut straight, &mut restored, intent);
    }

    let fall = GameIntent::Fall {
        tick: intent_tick(&straight),
        subject: NAMED,
        distance: 9,
    };
    let injury = apply_pair(&mut straight, &mut restored, fall);
    assert!(matches!(
        injury.as_slice(),
        [GameEvent::Injured {
            harm: MOBILITY_WOUND,
            ..
        }]
    ));
    assert!(!straight.bodies().get(NAMED).unwrap().mobile());

    let refused_at = straight.next_tick();
    let position = straight.movement().position(NAMED).unwrap();
    let refused = GameIntent::Move {
        tick: refused_at,
        subject: NAMED,
        toward: position,
    };
    assert_eq!(
        straight.apply(refused.clone()),
        Err(GameError::Body(BodyError::Immobile(NAMED)))
    );
    assert_eq!(
        restored.apply(refused),
        Err(GameError::Body(BodyError::Immobile(NAMED)))
    );
    assert_eq!(straight.next_tick(), refused_at);

    let dressing_count = straight
        .items()
        .carried_by(NAMED)
        .filter(|item| item.kind == ItemKind::Dressing)
        .count();
    let rest = GameIntent::Rest {
        tick: intent_tick(&straight),
        subject: NAMED,
    };
    let recovery = apply_pair(&mut straight, &mut restored, rest);
    assert!(matches!(
        recovery.as_slice(),
        [GameEvent::Rested { recovered, .. }] if *recovered > 0
    ));
    assert!(straight.bodies().get(NAMED).unwrap().mobile());
    assert_eq!(
        straight
            .items()
            .carried_by(NAMED)
            .filter(|item| item.kind == ItemKind::Dressing)
            .count(),
        dressing_count - 1
    );

    let food = {
        straight
            .items()
            .carried_by(NAMED)
            .find(|item| item.kind == ItemKind::Food)
            .map(|item| item.id)
    };
    if let Some(item) = food {
        let eat = GameIntent::Eat {
            tick: intent_tick(&straight),
            subject: NAMED,
            item,
        };
        apply_pair(&mut straight, &mut restored, eat);
    }

    let at = straight.movement().position(NAMED).unwrap();
    let scrap = straight
        .items()
        .at(at)
        .find(|item| item.kind == ItemKind::Scrap)
        .map(|item| item.id)
        .unwrap();
    let take = GameIntent::Take {
        tick: intent_tick(&straight),
        subject: NAMED,
        item: scrap,
    };
    apply_pair(&mut straight, &mut restored, take);
    assert_eq!(
        straight.items().get(scrap).unwrap().location,
        ItemLocation::Carried(NAMED)
    );
    assert!(
        straight.items().carried_mass_mg(NAMED)
            <= straight
                .bodies()
                .get(NAMED)
                .unwrap()
                .profile
                .carry_capacity_mg
    );

    let lethal = GameIntent::Fall {
        tick: intent_tick(&straight),
        subject: NAMED,
        distance: 20,
    };
    let death = apply_pair(&mut straight, &mut restored, lethal);
    assert!(matches!(
        death.as_slice(),
        [
            GameEvent::Injured { .. },
            GameEvent::Died {
                cause: DeathCause::Fall,
                ..
            }
        ]
    ));
    assert!(!straight.bodies().get(NAMED).unwrap().alive());
    let dead_at = straight.next_tick();
    assert_eq!(
        straight.apply(GameIntent::Wait {
            tick: dead_at,
            subject: NAMED,
        }),
        Err(GameError::Body(BodyError::Dead(NAMED)))
    );
    assert_eq!(straight.next_tick(), dead_at);

    assert_eq!(straight, restored);
    assert_eq!(
        straight.state_hash().unwrap(),
        restored.state_hash().unwrap()
    );
    assert_eq!(
        GameState::restore(&straight.save().unwrap()).unwrap(),
        straight
    );
}

#[test]
fn every_subject_uses_the_same_transition_grammar() {
    let world = world();
    let start = settlement_start(&world);
    let mut state = GameState::new(world);

    for (subject, seed, name) in [(NAMED, 17, "Rill"), (PEER, 93, "Moss")] {
        state
            .apply(GameIntent::Generate {
                tick: intent_tick(&state),
                subject,
                body_seed: seed,
                at: start,
            })
            .unwrap();
        state
            .apply(GameIntent::Name {
                tick: intent_tick(&state),
                subject,
                name: Name::new(name).unwrap(),
            })
            .unwrap();
        state
            .apply(GameIntent::Wait {
                tick: intent_tick(&state),
                subject,
            })
            .unwrap();
        state
            .apply(GameIntent::Fall {
                tick: intent_tick(&state),
                subject,
                distance: 5,
            })
            .unwrap();
    }

    let named = state.bodies().get(NAMED).unwrap();
    let peer = state.bodies().get(PEER).unwrap();
    assert_eq!(named.needs, peer.needs);
    assert_eq!(named.wound, peer.wound);
    assert_eq!(named.revision, peer.revision);
    assert_eq!(
        state.movement().position(NAMED),
        state.movement().position(PEER)
    );
    assert_eq!(state.bodies().all().count(), 2);
}

#[test]
fn unmet_needs_can_end_a_life_without_a_special_death_path() {
    let world = world();
    let start = settlement_start(&world);
    let mut state = GameState::new(world);
    state
        .apply(GameIntent::Generate {
            tick: Tick(0),
            subject: NAMED,
            body_seed: 17,
            at: start,
        })
        .unwrap();
    state
        .apply(GameIntent::Name {
            tick: Tick(1),
            subject: NAMED,
            name: Name::new("Rill").unwrap(),
        })
        .unwrap();

    let mut death = None;
    while state.bodies().get(NAMED).unwrap().alive() {
        let events = state
            .apply(GameIntent::Wait {
                tick: intent_tick(&state),
                subject: NAMED,
            })
            .unwrap();
        death = events
            .into_iter()
            .find(|event| matches!(event, GameEvent::Died { .. }))
            .or(death);
        assert!(state.intents().len() < 100);
    }

    assert!(matches!(
        death,
        Some(GameEvent::Died {
            cause: DeathCause::Starvation,
            ..
        })
    ));
    assert_eq!(state.bodies().get(NAMED).unwrap().vitality, 0);
}

#[test]
fn item_identity_and_body_generation_are_deterministic() {
    let first = GameState::new(world());
    let second = GameState::new(world());
    assert_eq!(
        first.items().all().collect::<Vec<_>>(),
        second.items().all().collect::<Vec<_>>()
    );

    let food_ids = first
        .items()
        .all()
        .filter(|item| item.kind == ItemKind::Food)
        .map(|item| item.id)
        .collect::<Vec<ItemId>>();
    assert!(!food_ids.is_empty());

    let start = settlement_start(first.world());
    let mut first = first;
    let mut second = second;
    for state in [&mut first, &mut second] {
        state
            .apply(GameIntent::Generate {
                tick: Tick(0),
                subject: NAMED,
                body_seed: 17,
                at: start,
            })
            .unwrap();
    }
    assert_eq!(first.bodies().get(NAMED), second.bodies().get(NAMED));
}

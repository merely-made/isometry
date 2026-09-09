// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The coordinator over separately owned world, movement, body, and item state.

use mesocosm_core::places::spot;
use mesocosm_core::snapshot::{self, hash_bytes};
use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::bodies::{Bodies, BodyError};
use crate::items::{ItemError, ItemKind, ItemLocation, Items};
use crate::{
    DeathCause, GAME_STATE_VERSION, GameError, GameEvent, GameIntent, GameSave, Movement,
    MovementError, MovementEvent, World,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    world: World,
    movement: Movement,
    bodies: Bodies,
    items: Items,
    intents: Vec<GameIntent>,
    events: Vec<GameEvent>,
}

impl GameState {
    pub fn new(world: World) -> Self {
        let items = Items::generate(&world);
        Self {
            world,
            movement: Movement::new(),
            bodies: Bodies::new(),
            items,
            intents: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn movement(&self) -> &Movement {
        &self.movement
    }

    pub fn bodies(&self) -> &Bodies {
        &self.bodies
    }

    pub fn items(&self) -> &Items {
        &self.items
    }

    pub fn intents(&self) -> &[GameIntent] {
        &self.intents
    }

    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    pub fn next_tick(&self) -> Tick {
        Tick(self.intents.len() as u64)
    }

    pub fn state_hash(&self) -> Result<u64, GameError> {
        snapshot::encode(self)
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| GameError::Encode)
    }

    fn living(&self, subject: SubjectId) -> Result<(), GameError> {
        let body = self
            .bodies
            .get(subject)
            .ok_or(BodyError::MissingSubject(subject))?;
        if !body.named() {
            return Err(BodyError::Unnamed(subject).into());
        }
        if !body.alive() {
            return Err(BodyError::Dead(subject).into());
        }
        Ok(())
    }

    fn exert(
        &mut self,
        subject: SubjectId,
        hunger: u16,
        fatigue: u16,
        tick: Tick,
        events: &mut Vec<GameEvent>,
    ) {
        let died = self
            .bodies
            .get_mut(subject)
            .expect("a validated body exists")
            .exert(hunger, fatigue, tick);
        if died {
            events.push(GameEvent::Died {
                tick,
                subject,
                cause: DeathCause::Starvation,
            });
        }
    }

    pub fn apply(&mut self, intent: GameIntent) -> Result<Vec<GameEvent>, GameError> {
        let expected = self.next_tick();
        if intent.tick() != expected {
            return Err(GameError::WrongTick {
                expected,
                actual: intent.tick(),
            });
        }
        let tick = intent.tick();
        let subject = intent.subject();
        let mut events = match &intent {
            GameIntent::Generate { body_seed, at, .. } => {
                if self.bodies.get(subject).is_some() {
                    return Err(BodyError::SubjectExists(subject).into());
                }
                if self.movement.position(subject).is_some() {
                    return Err(MovementError::SubjectExists(subject).into());
                }
                if !self
                    .world
                    .ground()
                    .stands(*at, mesocosm_core::places::WALKER_HEIGHT)
                {
                    return Err(MovementError::InvalidStart(*at).into());
                }
                let revision = self
                    .bodies
                    .generate(self.world.seed(), *body_seed, subject)?
                    .revision;
                self.movement.spawn(&self.world, subject, *at)?;
                vec![GameEvent::Generated {
                    tick,
                    subject,
                    revision,
                }]
            },
            GameIntent::Name { name, .. } => {
                name.validate()?;
                let body = self
                    .bodies
                    .get_mut(subject)
                    .ok_or(BodyError::MissingSubject(subject))?;
                body.name(name.clone(), tick)?;
                vec![GameEvent::Named {
                    tick,
                    subject,
                    name: name.clone(),
                }]
            },
            GameIntent::Move { toward, .. } => {
                self.living(subject)?;
                if !self.bodies.get(subject).unwrap().mobile() {
                    return Err(BodyError::Immobile(subject).into());
                }
                let moved = self.movement.step(&self.world, subject, *toward)?;
                let event = match moved {
                    MovementEvent::Moved { from, to, .. } => GameEvent::Moved {
                        tick,
                        subject,
                        from,
                        to,
                    },
                    MovementEvent::Held { at, .. } => GameEvent::Held { tick, subject, at },
                    MovementEvent::Spawned { .. } => unreachable!("step cannot spawn"),
                };
                let mut events = vec![event];
                self.exert(subject, 1, 2, tick, &mut events);
                events
            },
            GameIntent::Observe { target, .. } => {
                self.living(subject)?;
                let body = self.bodies.get(subject).unwrap();
                let at = self
                    .movement
                    .position(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                let visible = spot(self.world.ground(), at, *target, body.profile.sight_range);
                let mut events = vec![GameEvent::Observed {
                    tick,
                    subject,
                    target: *target,
                    visible,
                }];
                self.exert(subject, 1, 1, tick, &mut events);
                events
            },
            GameIntent::Take { item, .. } => {
                self.living(subject)?;
                let at = self
                    .movement
                    .position(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                let found = *self.items.get(*item).ok_or(ItemError::Missing(*item))?;
                if found.location != ItemLocation::At(at) {
                    return Err(ItemError::NotHere(*item).into());
                }
                let body = self.bodies.get(subject).unwrap();
                let carried = self.items.carried_mass_mg(subject);
                let attempted = carried.saturating_add(found.kind.mass_mg());
                if !body.can_carry(carried, found.kind.mass_mg()) {
                    return Err(ItemError::OverCapacity {
                        capacity_mg: body.profile.carry_capacity_mg,
                        attempted_mg: attempted,
                    }
                    .into());
                }
                self.items.take(*item, subject)?;
                let mut events = vec![GameEvent::Took {
                    tick,
                    subject,
                    item: *item,
                }];
                self.exert(subject, 1, 1, tick, &mut events);
                events
            },
            GameIntent::Eat { item, .. } => {
                self.living(subject)?;
                let food = *self.items.get(*item).ok_or(ItemError::Missing(*item))?;
                if food.location != ItemLocation::Carried(subject) {
                    return Err(ItemError::NotCarried(*item, subject).into());
                }
                if food.kind != ItemKind::Food {
                    return Err(ItemError::WrongKind(*item, food.kind).into());
                }
                self.items.consume(*item)?;
                let body = self.bodies.get_mut(subject).unwrap();
                body.eat();
                let mut events = vec![GameEvent::Ate {
                    tick,
                    subject,
                    item: *item,
                    hunger: body.needs.hunger,
                }];
                self.exert(subject, 0, 1, tick, &mut events);
                events
            },
            GameIntent::Rest { .. } => {
                self.living(subject)?;
                let dressing = (self.bodies.get(subject).unwrap().wound > 0)
                    .then(|| {
                        self.items
                            .carried_by(subject)
                            .find(|item| item.kind == ItemKind::Dressing)
                            .map(|item| item.id)
                    })
                    .flatten();
                if let Some(item) = dressing {
                    self.items.consume(item)?;
                }
                let body = self.bodies.get_mut(subject).unwrap();
                let (recovered, revision) = body.rest(dressing.is_some());
                let mut events = vec![GameEvent::Rested {
                    tick,
                    subject,
                    recovered,
                    fatigue: body.needs.fatigue,
                    revision,
                }];
                self.exert(subject, 1, 0, tick, &mut events);
                events
            },
            GameIntent::Fall { distance, .. } => {
                self.living(subject)?;
                let body = self.bodies.get_mut(subject).unwrap();
                let (harm, revision, died) = body.fall(*distance, tick);
                let mut events = vec![GameEvent::Injured {
                    tick,
                    subject,
                    distance: *distance,
                    harm,
                    revision,
                }];
                if died {
                    events.push(GameEvent::Died {
                        tick,
                        subject,
                        cause: DeathCause::Fall,
                    });
                } else {
                    self.exert(subject, 1, 1, tick, &mut events);
                }
                events
            },
            GameIntent::Wait { .. } => {
                self.living(subject)?;
                let mut events = vec![GameEvent::Waited { tick, subject }];
                self.exert(subject, 2, 1, tick, &mut events);
                events
            },
        };
        let returned = events.clone();
        self.intents.push(intent);
        self.events.append(&mut events);
        Ok(returned)
    }

    pub fn save_record(&self) -> Result<GameSave, GameError> {
        Ok(GameSave {
            version: GAME_STATE_VERSION,
            world: self.world.save_record(),
            expected_hash: self.state_hash()?,
            intents: self.intents.clone(),
        })
    }

    pub fn save(&self) -> Result<Vec<u8>, GameError> {
        snapshot::encode(&self.save_record()?).map_err(|_| GameError::Encode)
    }

    pub fn restore(bytes: &[u8]) -> Result<Self, GameError> {
        let save: GameSave = snapshot::decode(bytes).map_err(|_| GameError::Decode)?;
        Self::restore_record(save)
    }

    pub fn restore_record(save: GameSave) -> Result<Self, GameError> {
        if save.version != GAME_STATE_VERSION {
            return Err(GameError::VersionDiverged {
                saved: save.version,
                current: GAME_STATE_VERSION,
            });
        }
        let world = World::restore_record(save.world)?;
        let mut state = Self::new(world);
        for intent in save.intents {
            state.apply(intent)?;
        }
        let restored = state.state_hash()?;
        if restored != save.expected_hash {
            return Err(GameError::StateDiverged {
                saved: save.expected_hash,
                restored,
            });
        }
        Ok(state)
    }
}

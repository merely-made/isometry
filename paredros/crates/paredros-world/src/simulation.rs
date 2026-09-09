// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Deterministic autonomous scheduling over the shared game transitions.

use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};

use crate::{
    GameError, GameEvent, GameIntent, GameState, ItemId, ItemKind, Life, Name, Navigation,
    NavigationError, Needs, Population, PopulationConfig, PopulationError, PopulationOrigin,
    ProjectError, ProjectGoal, ProjectId, ProjectIntent, ProjectStatus, Projects, SlotId, World,
};

const NEED_THRESHOLD: u16 = 35;
const REST_THRESHOLD: u16 = 80;
const CURIOSITY_PERIOD: u64 = 5;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pursuit {
    Hunger,
    Safety,
    Work(ItemId),
    Curiosity(SubjectId),
    Travel(SlotId),
    Project(ProjectId),
    Routine,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub round: u64,
    pub subject: SubjectId,
    pub pursuit: Pursuit,
    pub intent_index: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifeReport {
    pub subject: SubjectId,
    pub name: Name,
    pub origin: PopulationOrigin,
    pub position: [i32; 3],
    pub needs: Needs,
    pub vitality: u16,
    pub wound: u16,
    pub carried: Vec<ItemId>,
    pub autonomous_actions: u64,
    pub last_pursuit: Option<Pursuit>,
    pub active_project: Option<ProjectId>,
    pub completed_projects: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Simulation {
    pub(crate) game: GameState,
    pub(crate) population: Population,
    pub(crate) projects: Projects,
    pub(crate) round: u64,
    pub(crate) decisions: Vec<Decision>,
}

impl Simulation {
    pub fn generate(world: World, config: PopulationConfig) -> Result<Self, SimulationError> {
        let population = Population::generate(&world, config)?;
        let projects = Projects::generate(&world, &population)?;
        let mut game = GameState::new(world);
        for life in population.all() {
            game.apply(GameIntent::Generate {
                tick: game.next_tick(),
                subject: life.subject,
                body_seed: life.body_seed,
                at: life.origin_at,
            })?;
            game.apply(GameIntent::Name {
                tick: game.next_tick(),
                subject: life.subject,
                name: life.name.clone(),
            })?;
        }
        Ok(Self {
            game,
            population,
            projects,
            round: 0,
            decisions: Vec::new(),
        })
    }

    pub fn game(&self) -> &GameState {
        &self.game
    }

    pub fn population(&self) -> &Population {
        &self.population
    }

    pub fn projects(&self) -> &Projects {
        &self.projects
    }

    pub const fn round(&self) -> u64 {
        self.round
    }

    pub fn decisions(&self) -> &[Decision] {
        &self.decisions
    }

    /// Apply an intent from any external source. There is deliberately no
    /// controlled-subject argument or special transition path here.
    pub fn apply(&mut self, intent: GameIntent) -> Result<Vec<GameEvent>, SimulationError> {
        self.game.apply(intent).map_err(Into::into)
    }

    /// Advance every living member once per round. Observation is not an input
    /// to scheduling, so callers may leave the simulation entirely unattended.
    pub fn advance(&mut self, rounds: u64) -> Result<Vec<Decision>, SimulationError> {
        let first = self.decisions.len();
        for _ in 0..rounds {
            self.advance_round()?;
        }
        Ok(self.decisions[first..].to_vec())
    }

    fn advance_round(&mut self) -> Result<(), SimulationError> {
        let next_round = self
            .round
            .checked_add(1)
            .ok_or(SimulationError::RoundOverflow)?;
        let subjects = self
            .population
            .all()
            .map(|life| life.subject)
            .collect::<Vec<_>>();
        for subject in subjects {
            let life = self
                .population
                .get(subject)
                .expect("population subjects remain stable");
            let Some((pursuit, intent, completed_project)) = self.decide(life)? else {
                continue;
            };
            let intent_index = self.game.intents().len() as u64;
            self.game.apply(intent.clone())?;
            if let Some(project) = completed_project {
                self.projects.apply(ProjectIntent::Complete {
                    round: self.round,
                    project,
                    subject,
                })?;
            }
            self.decisions.push(Decision {
                round: self.round,
                subject,
                pursuit,
                intent_index,
            });
        }
        self.round = next_round;
        Ok(())
    }

    fn decide(
        &self,
        life: &Life,
    ) -> Result<Option<(Pursuit, GameIntent, Option<ProjectId>)>, SimulationError> {
        let Some(body) = self.game.bodies().get(life.subject) else {
            return Err(SimulationError::MissingSubject(life.subject));
        };
        if !body.alive() {
            return Ok(None);
        }
        let at = self
            .game
            .movement()
            .position(life.subject)
            .ok_or(SimulationError::MissingSubject(life.subject))?;
        let tick = self.game.next_tick();

        if body.wound > 0 || body.needs.fatigue >= REST_THRESHOLD {
            return Ok(Some((
                Pursuit::Safety,
                GameIntent::Rest {
                    tick,
                    subject: life.subject,
                },
                None,
            )));
        }

        if body.needs.hunger >= NEED_THRESHOLD {
            if let Some(item) = self
                .game
                .items()
                .carried_by(life.subject)
                .find(|item| item.kind == ItemKind::Food)
            {
                return Ok(Some((
                    Pursuit::Hunger,
                    GameIntent::Eat {
                        tick,
                        subject: life.subject,
                        item: item.id,
                    },
                    None,
                )));
            }
            if let Some(item) = self.game.items().at(at).find(|item| {
                item.kind == ItemKind::Food
                    && body.can_carry(
                        self.game.items().carried_mass_mg(life.subject),
                        item.kind.mass_mg(),
                    )
            }) {
                return Ok(Some((
                    Pursuit::Hunger,
                    GameIntent::Take {
                        tick,
                        subject: life.subject,
                        item: item.id,
                    },
                    None,
                )));
            }
        }

        let carried = self.game.items().carried_mass_mg(life.subject);
        if let Some(item) = self
            .game
            .items()
            .at(at)
            .find(|item| body.can_carry(carried, item.kind.mass_mg()))
        {
            return Ok(Some((
                Pursuit::Work(item.id),
                GameIntent::Take {
                    tick,
                    subject: life.subject,
                    item: item.id,
                },
                None,
            )));
        }

        if (self.round + life.subject.0).is_multiple_of(CURIOSITY_PERIOD)
            && let Some((other, target)) = self.nearest_other(life.subject, at)
        {
            return Ok(Some((
                Pursuit::Curiosity(other),
                GameIntent::Observe {
                    tick,
                    subject: life.subject,
                    target,
                },
                None,
            )));
        }

        if matches!(life.origin, PopulationOrigin::Migration { .. }) && at != life.home_at {
            return self.move_toward(
                life,
                life.home,
                life.home_at,
                Pursuit::Travel(life.home),
                None,
            );
        }
        if let Some(project) = self.projects.active_for(life.subject) {
            let ProjectGoal::Visit { slot, at: target } = project.goal;
            return self.move_toward(
                life,
                slot,
                target,
                Pursuit::Project(project.id),
                Some(project.id),
            );
        }
        if at != life.home_at {
            return self.move_toward(
                life,
                life.home,
                life.home_at,
                Pursuit::Travel(life.home),
                None,
            );
        }
        if let Some((other, target)) = self.nearest_other(life.subject, at) {
            return Ok(Some((
                Pursuit::Curiosity(other),
                GameIntent::Observe {
                    tick,
                    subject: life.subject,
                    target,
                },
                None,
            )));
        }
        Ok(Some((
            Pursuit::Routine,
            GameIntent::Wait {
                tick,
                subject: life.subject,
            },
            None,
        )))
    }

    fn move_toward(
        &self,
        life: &Life,
        target_slot: SlotId,
        target: [i32; 3],
        pursuit: Pursuit,
        complete: Option<ProjectId>,
    ) -> Result<Option<(Pursuit, GameIntent, Option<ProjectId>)>, SimulationError> {
        let at = self
            .game
            .movement()
            .position(life.subject)
            .ok_or(SimulationError::MissingSubject(life.subject))?;
        let tick = self.game.next_tick();
        if at == target {
            return Ok(Some((
                pursuit,
                GameIntent::Observe {
                    tick,
                    subject: life.subject,
                    target,
                },
                complete,
            )));
        }
        let route = Navigation::default().route_to_position(self.game.world(), at, target)?;
        let toward = route.get(1).copied().ok_or(SimulationError::NoStep {
            subject: life.subject,
            target: target_slot,
        })?;
        Ok(Some((
            pursuit,
            GameIntent::Move {
                tick,
                subject: life.subject,
                toward,
            },
            (toward == target).then_some(complete).flatten(),
        )))
    }

    fn nearest_other(&self, subject: SubjectId, at: [i32; 3]) -> Option<(SubjectId, [i32; 3])> {
        self.population
            .all()
            .filter(|life| life.subject != subject)
            .filter(|life| {
                self.game
                    .bodies()
                    .get(life.subject)
                    .is_some_and(|body| body.alive())
            })
            .filter_map(|life| {
                self.game
                    .movement()
                    .position(life.subject)
                    .map(|position| (life.subject, position))
            })
            .min_by_key(|(other, position)| (distance(at, *position), *other))
    }

    pub fn report(&self, subject: SubjectId) -> Result<LifeReport, SimulationError> {
        let life = self
            .population
            .get(subject)
            .ok_or(SimulationError::MissingSubject(subject))?;
        let body = self
            .game
            .bodies()
            .get(subject)
            .ok_or(SimulationError::MissingSubject(subject))?;
        let position = self
            .game
            .movement()
            .position(subject)
            .ok_or(SimulationError::MissingSubject(subject))?;
        let relevant = self
            .decisions
            .iter()
            .filter(|decision| decision.subject == subject)
            .collect::<Vec<_>>();
        Ok(LifeReport {
            subject,
            name: life.name.clone(),
            origin: life.origin,
            position,
            needs: body.needs,
            vitality: body.vitality,
            wound: body.wound,
            carried: self
                .game
                .items()
                .carried_by(subject)
                .map(|item| item.id)
                .collect(),
            autonomous_actions: relevant.len() as u64,
            last_pursuit: relevant.last().map(|decision| decision.pursuit.clone()),
            active_project: self.projects.active_for(subject).map(|project| project.id),
            completed_projects: self
                .projects
                .all()
                .filter(|project| {
                    project.subject == subject
                        && matches!(project.status, ProjectStatus::Completed { .. })
                })
                .count() as u64,
        })
    }
}

fn distance(left: [i32; 3], right: [i32; 3]) -> u32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| left.abs_diff(right))
        .max()
        .unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationError {
    Game(GameError),
    Population(PopulationError),
    Project(ProjectError),
    Navigation(NavigationError),
    MissingSubject(SubjectId),
    NameDiverged(SubjectId),
    NoStep { subject: SubjectId, target: SlotId },
    DecisionDiverged(u64),
    PopulationDiverged { saved: u64, regrown: u64 },
    ProjectsDiverged { saved: u64, restored: u64 },
    ProjectDecisionDiverged(ProjectId),
    StateDiverged { saved: u64, restored: u64 },
    VersionDiverged { saved: u32, current: u32 },
    RoundOverflow,
    Encode,
    Decode,
}

impl From<GameError> for SimulationError {
    fn from(error: GameError) -> Self {
        Self::Game(error)
    }
}

impl From<PopulationError> for SimulationError {
    fn from(error: PopulationError) -> Self {
        Self::Population(error)
    }
}

impl From<ProjectError> for SimulationError {
    fn from(error: ProjectError) -> Self {
        Self::Project(error)
    }
}

impl From<NavigationError> for SimulationError {
    fn from(error: NavigationError) -> Self {
        Self::Navigation(error)
    }
}

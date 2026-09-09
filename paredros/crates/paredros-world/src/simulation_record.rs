// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Regrow-and-replay persistence for autonomous simulation state.

use mesocosm_core::snapshot::{self, hash_bytes};
use serde::{Deserialize, Serialize};

use crate::{
    Decision, GameIntent, GameSave, GameState, Population, PopulationConfig, ProjectEvent,
    ProjectSave, Projects, Pursuit, Simulation, SimulationError,
};

pub const SIMULATION_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationSave {
    pub version: u32,
    pub game: GameSave,
    pub population: PopulationConfig,
    pub expected_population_hash: u64,
    pub projects: ProjectSave,
    pub expected_projects_hash: u64,
    pub round: u64,
    pub decisions: Vec<Decision>,
    pub expected_hash: u64,
}

impl Simulation {
    pub fn state_hash(&self) -> Result<u64, SimulationError> {
        snapshot::encode(self)
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| SimulationError::Encode)
    }

    pub fn save_record(&self) -> Result<SimulationSave, SimulationError> {
        Ok(SimulationSave {
            version: SIMULATION_VERSION,
            game: self.game.save_record()?,
            population: self.population.config().clone(),
            expected_population_hash: self.population.state_hash()?,
            projects: self.projects.save_record(),
            expected_projects_hash: self.projects.state_hash()?,
            round: self.round,
            decisions: self.decisions.clone(),
            expected_hash: self.state_hash()?,
        })
    }

    pub fn save(&self) -> Result<Vec<u8>, SimulationError> {
        snapshot::encode(&self.save_record()?).map_err(|_| SimulationError::Encode)
    }

    pub fn restore(bytes: &[u8]) -> Result<Self, SimulationError> {
        let save: SimulationSave = snapshot::decode(bytes).map_err(|_| SimulationError::Decode)?;
        Self::restore_record(save)
    }

    pub fn restore_record(save: SimulationSave) -> Result<Self, SimulationError> {
        if save.version != SIMULATION_VERSION {
            return Err(SimulationError::VersionDiverged {
                saved: save.version,
                current: SIMULATION_VERSION,
            });
        }
        let game = GameState::restore_record(save.game)?;
        let population = Population::generate(game.world(), save.population)?;
        let regrown_population = population.state_hash()?;
        if regrown_population != save.expected_population_hash {
            return Err(SimulationError::PopulationDiverged {
                saved: save.expected_population_hash,
                regrown: regrown_population,
            });
        }
        validate_population(&game, &population)?;
        let projects = Projects::restore_record(game.world(), &population, save.projects)?;
        let restored_projects = projects.state_hash()?;
        if restored_projects != save.expected_projects_hash {
            return Err(SimulationError::ProjectsDiverged {
                saved: save.expected_projects_hash,
                restored: restored_projects,
            });
        }
        validate_decisions(&game, save.round, &save.decisions)?;
        validate_project_events(&projects, &save.decisions)?;
        let simulation = Self {
            game,
            population,
            projects,
            round: save.round,
            decisions: save.decisions,
        };
        let restored = simulation.state_hash()?;
        if restored != save.expected_hash {
            return Err(SimulationError::StateDiverged {
                saved: save.expected_hash,
                restored,
            });
        }
        Ok(simulation)
    }
}

fn validate_population(game: &GameState, population: &Population) -> Result<(), SimulationError> {
    for life in population.all() {
        let body = game
            .bodies()
            .get(life.subject)
            .ok_or(SimulationError::MissingSubject(life.subject))?;
        if body.name.as_ref() != Some(&life.name) {
            return Err(SimulationError::NameDiverged(life.subject));
        }
        if game.movement().position(life.subject).is_none() {
            return Err(SimulationError::MissingSubject(life.subject));
        }
    }
    Ok(())
}

fn validate_decisions(
    game: &GameState,
    round: u64,
    decisions: &[Decision],
) -> Result<(), SimulationError> {
    let mut previous = None;
    for (number, decision) in decisions.iter().enumerate() {
        let intent = game.intents().get(decision.intent_index as usize);
        if decision.round >= round
            || previous.is_some_and(|previous| decision.intent_index <= previous)
            || intent.is_none_or(|intent| {
                intent.subject() != decision.subject || !pursuit_matches(&decision.pursuit, intent)
            })
        {
            return Err(SimulationError::DecisionDiverged(number as u64));
        }
        previous = Some(decision.intent_index);
    }
    Ok(())
}

fn pursuit_matches(pursuit: &Pursuit, intent: &GameIntent) -> bool {
    match (pursuit, intent) {
        (Pursuit::Hunger, GameIntent::Eat { .. } | GameIntent::Take { .. })
        | (Pursuit::Safety, GameIntent::Rest { .. })
        | (Pursuit::Curiosity(_), GameIntent::Observe { .. })
        | (Pursuit::Travel(_) | Pursuit::Project(_), GameIntent::Move { .. })
        | (Pursuit::Travel(_) | Pursuit::Project(_), GameIntent::Observe { .. })
        | (Pursuit::Routine, GameIntent::Wait { .. }) => true,
        (Pursuit::Work(expected), GameIntent::Take { item, .. }) => expected == item,
        _ => false,
    }
}

fn validate_project_events(
    projects: &Projects,
    decisions: &[Decision],
) -> Result<(), SimulationError> {
    for event in projects.events() {
        let ProjectEvent::Completed {
            round,
            project,
            subject,
        } = *event;
        if !decisions.iter().any(|decision| {
            decision.round == round
                && decision.subject == subject
                && decision.pursuit == Pursuit::Project(project)
        }) {
            return Err(SimulationError::ProjectDecisionDiverged(project));
        }
    }
    Ok(())
}

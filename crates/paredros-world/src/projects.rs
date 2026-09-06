// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Durable goals and their recorded completion.

use std::collections::BTreeMap;

use mesocosm_core::snapshot::{self, hash_bytes};
use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};

use crate::{Navigation, NavigationError, Population, SlotId, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectGoal {
    Visit { slot: SlotId, at: [i32; 3] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Completed { round: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub subject: SubjectId,
    pub goal: ProjectGoal,
    pub status: ProjectStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectIntent {
    Complete {
        round: u64,
        project: ProjectId,
        subject: SubjectId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectEvent {
    Completed {
        round: u64,
        project: ProjectId,
        subject: SubjectId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSave {
    pub intents: Vec<ProjectIntent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projects {
    projects: BTreeMap<ProjectId, Project>,
    intents: Vec<ProjectIntent>,
    events: Vec<ProjectEvent>,
    last_round: Option<u64>,
}

impl Projects {
    pub fn generate(world: &World, population: &Population) -> Result<Self, ProjectError> {
        let navigation = Navigation::default();
        let mut projects = BTreeMap::new();
        for (number, life) in population.all().enumerate() {
            let target = choose_target(world, life.home, life.body_seed);
            let at = navigation.stance(world, target)?;
            let id = ProjectId(number as u64);
            projects.insert(
                id,
                Project {
                    id,
                    subject: life.subject,
                    goal: ProjectGoal::Visit { slot: target, at },
                    status: ProjectStatus::Active,
                },
            );
        }
        Ok(Self {
            projects,
            intents: Vec::new(),
            events: Vec::new(),
            last_round: None,
        })
    }

    pub fn get(&self, id: ProjectId) -> Option<&Project> {
        self.projects.get(&id)
    }

    pub fn all(&self) -> impl Iterator<Item = &Project> {
        self.projects.values()
    }

    pub fn active_for(&self, subject: SubjectId) -> Option<&Project> {
        self.projects
            .values()
            .find(|project| project.subject == subject && project.status == ProjectStatus::Active)
    }

    pub fn intents(&self) -> &[ProjectIntent] {
        &self.intents
    }

    pub fn events(&self) -> &[ProjectEvent] {
        &self.events
    }

    pub fn apply(&mut self, intent: ProjectIntent) -> Result<ProjectEvent, ProjectError> {
        let event = match intent {
            ProjectIntent::Complete {
                round,
                project,
                subject,
            } => {
                if let Some(previous) = self.last_round
                    && round < previous
                {
                    return Err(ProjectError::OutOfOrder {
                        previous,
                        next: round,
                    });
                }
                let project_state = self
                    .projects
                    .get_mut(&project)
                    .ok_or(ProjectError::Missing(project))?;
                if project_state.subject != subject {
                    return Err(ProjectError::WrongSubject {
                        project,
                        expected: project_state.subject,
                        actual: subject,
                    });
                }
                if project_state.status != ProjectStatus::Active {
                    return Err(ProjectError::AlreadyCompleted(project));
                }
                project_state.status = ProjectStatus::Completed { round };
                ProjectEvent::Completed {
                    round,
                    project,
                    subject,
                }
            },
        };
        self.intents.push(intent);
        self.events.push(event);
        self.last_round = Some(match event {
            ProjectEvent::Completed { round, .. } => round,
        });
        Ok(event)
    }

    pub fn state_hash(&self) -> Result<u64, ProjectError> {
        snapshot::encode(self)
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| ProjectError::Encode)
    }

    pub fn save_record(&self) -> ProjectSave {
        ProjectSave {
            intents: self.intents.clone(),
        }
    }

    pub fn restore_record(
        world: &World,
        population: &Population,
        save: ProjectSave,
    ) -> Result<Self, ProjectError> {
        let mut projects = Self::generate(world, population)?;
        for intent in save.intents {
            projects.apply(intent)?;
        }
        Ok(projects)
    }
}

fn choose_target(world: &World, home: SlotId, draw: u64) -> SlotId {
    let Some(neighbours) = world.map().neighbours(home) else {
        return home;
    };
    if neighbours.is_empty() {
        return home;
    }
    let index = draw as usize % neighbours.len();
    neighbours.iter().copied().nth(index).unwrap_or(home)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectError {
    Missing(ProjectId),
    WrongSubject {
        project: ProjectId,
        expected: SubjectId,
        actual: SubjectId,
    },
    AlreadyCompleted(ProjectId),
    OutOfOrder {
        previous: u64,
        next: u64,
    },
    Navigation(NavigationError),
    Encode,
}

impl From<NavigationError> for ProjectError {
    fn from(error: NavigationError) -> Self {
        Self::Navigation(error)
    }
}

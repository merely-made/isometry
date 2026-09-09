// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Generated bodies and their durable condition.

use std::collections::BTreeMap;

use mesocosm_core::snapshot::{self, hash_bytes};
use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use serde::{Deserialize, Serialize};

pub const MAX_NEED: u16 = 100;
pub const SAFE_FALL: i32 = 4;
pub const MOBILITY_WOUND: u16 = 50;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(String);

impl Name {
    pub fn new(value: impl Into<String>) -> Result<Self, BodyError> {
        let name = Self(value.into());
        name.validate()?;
        Ok(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn validate(&self) -> Result<(), BodyError> {
        let trimmed = self.0.trim();
        if trimmed.is_empty() {
            return Err(BodyError::EmptyName);
        }
        if trimmed.chars().count() > 64 {
            return Err(BodyError::NameTooLong);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Needs {
    pub hunger: u16,
    pub fatigue: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyProfile {
    pub mass_mg: u32,
    pub carry_capacity_mg: u32,
    pub sight_range: i32,
    pub recovery_per_rest: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Body {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub profile: BodyProfile,
    pub name: Option<Name>,
    pub born_at: Option<Tick>,
    pub died_at: Option<Tick>,
    pub vitality: u16,
    pub wound: u16,
    pub needs: Needs,
}

impl Body {
    fn generate(world_seed: u64, body_seed: u64, subject: SubjectId) -> Self {
        let bytes = snapshot::encode(&(world_seed, body_seed, subject))
            .expect("fixed-width body genesis always encodes");
        let draw = hash_bytes(&bytes);
        Self {
            subject,
            revision: BodyRevisionId(0),
            profile: BodyProfile {
                mass_mg: 48_000 + (draw % 8_000) as u32,
                carry_capacity_mg: 1_600 + ((draw >> 13) % 400) as u32,
                sight_range: 16 + ((draw >> 27) % 9) as i32,
                recovery_per_rest: 40 + ((draw >> 39) % 11) as u16,
            },
            name: None,
            born_at: None,
            died_at: None,
            vitality: 100,
            wound: 0,
            needs: Needs::default(),
        }
    }

    pub fn alive(&self) -> bool {
        self.name.is_some() && self.died_at.is_none() && self.vitality > 0
    }

    pub fn named(&self) -> bool {
        self.name.is_some()
    }

    pub fn mobile(&self) -> bool {
        self.alive() && self.wound < MOBILITY_WOUND && self.needs.fatigue < MAX_NEED
    }

    pub fn can_carry(&self, carried_mg: u32, added_mg: u32) -> bool {
        carried_mg.saturating_add(added_mg) <= self.profile.carry_capacity_mg
    }

    pub(crate) fn name(&mut self, name: Name, at: Tick) -> Result<(), BodyError> {
        name.validate()?;
        if self.name.is_some() {
            return Err(BodyError::AlreadyNamed(self.subject));
        }
        self.name = Some(name);
        self.born_at = Some(at);
        Ok(())
    }

    pub(crate) fn exert(&mut self, hunger: u16, fatigue: u16, at: Tick) -> bool {
        self.needs.hunger = self.needs.hunger.saturating_add(hunger).min(MAX_NEED);
        self.needs.fatigue = self.needs.fatigue.saturating_add(fatigue).min(MAX_NEED);
        if self.needs.hunger == MAX_NEED {
            self.vitality = self.vitality.saturating_sub(5);
        }
        if self.vitality == 0 && self.died_at.is_none() {
            self.died_at = Some(at);
            return true;
        }
        false
    }

    pub(crate) fn eat(&mut self) {
        self.needs.hunger = self.needs.hunger.saturating_sub(40);
    }

    /// Returns how much wound was recovered and the new revision.
    pub(crate) fn rest(&mut self, dressed: bool) -> (u16, BodyRevisionId) {
        self.needs.fatigue = self.needs.fatigue.saturating_sub(50);
        let recovered = if dressed {
            self.wound.min(self.profile.recovery_per_rest)
        } else {
            0
        };
        if recovered > 0 {
            self.wound -= recovered;
            self.vitality = self.vitality.saturating_add(recovered).min(100);
            self.revision = BodyRevisionId(self.revision.0 + 1);
        }
        (recovered, self.revision)
    }

    /// Returns authoritative harm, the resulting revision, and whether the
    /// injury killed the body.
    pub(crate) fn fall(&mut self, distance: i32, at: Tick) -> (u16, BodyRevisionId, bool) {
        let harm = distance
            .saturating_sub(SAFE_FALL)
            .saturating_mul(10)
            .clamp(0, 100) as u16;
        if harm > 0 {
            self.wound = self.wound.saturating_add(harm).min(100);
            self.vitality = self.vitality.saturating_sub(harm);
            self.revision = BodyRevisionId(self.revision.0 + 1);
        }
        let died = self.vitality == 0 && self.died_at.is_none();
        if died {
            self.died_at = Some(at);
        }
        (harm, self.revision, died)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bodies {
    bodies: BTreeMap<SubjectId, Body>,
}

impl Bodies {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, subject: SubjectId) -> Option<&Body> {
        self.bodies.get(&subject)
    }

    pub fn all(&self) -> impl Iterator<Item = &Body> {
        self.bodies.values()
    }

    pub(crate) fn get_mut(&mut self, subject: SubjectId) -> Option<&mut Body> {
        self.bodies.get_mut(&subject)
    }

    pub(crate) fn generate(
        &mut self,
        world_seed: u64,
        body_seed: u64,
        subject: SubjectId,
    ) -> Result<&Body, BodyError> {
        if self.bodies.contains_key(&subject) {
            return Err(BodyError::SubjectExists(subject));
        }
        self.bodies
            .insert(subject, Body::generate(world_seed, body_seed, subject));
        Ok(&self.bodies[&subject])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyError {
    SubjectExists(SubjectId),
    MissingSubject(SubjectId),
    EmptyName,
    NameTooLong,
    AlreadyNamed(SubjectId),
    Unnamed(SubjectId),
    Dead(SubjectId),
    Immobile(SubjectId),
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Deterministic origins and genesis facts for named lives.

use std::collections::BTreeMap;

use mesocosm_core::snapshot::{self, hash_bytes};
use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};

use crate::{Name, Navigation, NavigationError, SiteKind, SlotId, World};

pub const MAX_RESIDENTS_PER_SITE: u16 = 16;
pub const MAX_MIGRANTS_PER_ROUTE: u16 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Migration {
    pub from: SlotId,
    pub toward: SlotId,
    pub count: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub seed: u64,
    pub first_subject: SubjectId,
    pub residents_per_site: u16,
    pub include_wilds: bool,
    pub migrations: Vec<Migration>,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            first_subject: SubjectId(1),
            residents_per_site: 1,
            include_wilds: false,
            migrations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopulationOrigin {
    Site { slot: SlotId, kind: SiteKind },
    Migration { from: SlotId, toward: SlotId },
}

impl PopulationOrigin {
    pub const fn slot(self) -> SlotId {
        match self {
            Self::Site { slot, .. } | Self::Migration { from: slot, .. } => slot,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Life {
    pub subject: SubjectId,
    pub origin: PopulationOrigin,
    pub origin_at: [i32; 3],
    pub home: SlotId,
    pub home_at: [i32; 3],
    pub body_seed: u64,
    pub name: Name,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Population {
    config: PopulationConfig,
    lives: BTreeMap<SubjectId, Life>,
}

impl Population {
    pub fn generate(world: &World, config: PopulationConfig) -> Result<Self, PopulationError> {
        if config.residents_per_site > MAX_RESIDENTS_PER_SITE {
            return Err(PopulationError::TooManyResidents(config.residents_per_site));
        }
        if config
            .migrations
            .iter()
            .any(|migration| migration.count > MAX_MIGRANTS_PER_ROUTE)
        {
            return Err(PopulationError::TooManyMigrants);
        }

        let navigation = Navigation::default();
        let mut lives = BTreeMap::new();
        let mut next = config.first_subject.0;

        for site in world.map().sites() {
            if site.kind == SiteKind::Wilds && !config.include_wilds {
                continue;
            }
            let origin = PopulationOrigin::Site {
                slot: site.slot,
                kind: site.kind,
            };
            for ordinal in 0..config.residents_per_site {
                let subject = SubjectId(next);
                next = next
                    .checked_add(1)
                    .ok_or(PopulationError::SubjectOverflow)?;
                let life =
                    generate_life(world, &navigation, config.seed, subject, origin, ordinal)?;
                lives.insert(subject, life);
            }
        }

        for migration in &config.migrations {
            validate_migration(world, *migration)?;
            let origin = PopulationOrigin::Migration {
                from: migration.from,
                toward: migration.toward,
            };
            for ordinal in 0..migration.count {
                let subject = SubjectId(next);
                next = next
                    .checked_add(1)
                    .ok_or(PopulationError::SubjectOverflow)?;
                let life =
                    generate_life(world, &navigation, config.seed, subject, origin, ordinal)?;
                lives.insert(subject, life);
            }
        }

        if lives.is_empty() {
            return Err(PopulationError::Empty);
        }
        Ok(Self { config, lives })
    }

    pub fn config(&self) -> &PopulationConfig {
        &self.config
    }

    pub fn get(&self, subject: SubjectId) -> Option<&Life> {
        self.lives.get(&subject)
    }

    pub fn all(&self) -> impl Iterator<Item = &Life> {
        self.lives.values()
    }

    pub fn state_hash(&self) -> Result<u64, PopulationError> {
        snapshot::encode(self)
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| PopulationError::Encode)
    }
}

fn validate_migration(world: &World, migration: Migration) -> Result<(), PopulationError> {
    if world.map().site(migration.from).is_none() {
        return Err(PopulationError::MissingSlot(migration.from));
    }
    if world.map().site(migration.toward).is_none() {
        return Err(PopulationError::MissingSlot(migration.toward));
    }
    if world
        .map()
        .route(migration.from, migration.toward)
        .is_none()
    {
        return Err(PopulationError::MissingRoute {
            from: migration.from,
            toward: migration.toward,
        });
    }
    Ok(())
}

fn generate_life(
    world: &World,
    navigation: &Navigation,
    population_seed: u64,
    subject: SubjectId,
    origin: PopulationOrigin,
    ordinal: u16,
) -> Result<Life, PopulationError> {
    let origin_at = navigation.stance(world, origin.slot())?;
    let home = match origin {
        PopulationOrigin::Site { slot, .. } => slot,
        PopulationOrigin::Migration { toward, .. } => toward,
    };
    let home_at = navigation.stance(world, home)?;
    let draw = genesis_draw(world.seed(), population_seed, subject, origin, ordinal)?;
    let body_seed = draw.rotate_left(17) ^ population_seed;
    let name = generated_name(draw, subject)?;
    Ok(Life {
        subject,
        origin,
        origin_at,
        home,
        home_at,
        body_seed,
        name,
    })
}

fn genesis_draw(
    world_seed: u64,
    population_seed: u64,
    subject: SubjectId,
    origin: PopulationOrigin,
    ordinal: u16,
) -> Result<u64, PopulationError> {
    snapshot::encode(&(world_seed, population_seed, subject, origin, ordinal))
        .map(|bytes| hash_bytes(&bytes))
        .map_err(|_| PopulationError::Encode)
}

fn generated_name(draw: u64, subject: SubjectId) -> Result<Name, PopulationError> {
    const STARTS: [&str; 16] = [
        "Ari", "Bram", "Caro", "Dara", "Eli", "Fenn", "Gale", "Holl", "Ira", "Jori", "Kelm", "Lio",
        "Mara", "Neri", "Olan", "Perr",
    ];
    const ENDS: [&str; 8] = ["a", "en", "i", "o", "ra", "ren", "u", "yn"];
    let start = STARTS[draw as usize % STARTS.len()];
    let end = ENDS[(draw.rotate_right(11) as usize) % ENDS.len()];
    Name::new(format!("{start}{end} {}", subject.0)).map_err(|_| PopulationError::Name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopulationError {
    Empty,
    TooManyResidents(u16),
    TooManyMigrants,
    SubjectOverflow,
    MissingSlot(SlotId),
    MissingRoute { from: SlotId, toward: SlotId },
    Navigation(NavigationError),
    Name,
    Encode,
}

impl From<NavigationError> for PopulationError {
    fn from(error: NavigationError) -> Self {
        Self::Navigation(error)
    }
}

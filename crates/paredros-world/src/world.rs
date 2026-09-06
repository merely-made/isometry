// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::places::{Ground, Grown, Places};
use mesocosm_core::snapshot::{self, hash_bytes};
use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::{HistoryFactId, SiteKind, SlotId, WorldMap};

pub const GENERATOR_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldConfig {
    pub side: u16,
    pub extent: i32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            side: 8,
            extent: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldIntent {
    Carve {
        tick: Tick,
        by: SubjectId,
        centre: [i32; 3],
        radius: i32,
    },
    InheritSite {
        tick: Tick,
        slot: SlotId,
        kind: SiteKind,
        fact: HistoryFactId,
    },
}

impl WorldIntent {
    pub const fn tick(self) -> Tick {
        match self {
            Self::Carve { tick, .. } | Self::InheritSite { tick, .. } => tick,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldEvent {
    Carved {
        tick: Tick,
        by: SubjectId,
        centre: [i32; 3],
        radius: i32,
        removed: u32,
        ground_revision: u64,
    },
    SiteInherited {
        tick: Tick,
        slot: SlotId,
        kind: SiteKind,
        fact: HistoryFactId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSave {
    pub generator_version: u32,
    pub seed: u64,
    pub config: WorldConfig,
    pub base_hash: u64,
    pub intents: Vec<WorldIntent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    InvalidSide,
    InvalidExtent,
    InvalidRadius,
    MissingSlot(SlotId),
    OutOfOrder { previous: Tick, next: Tick },
    GeneratorDiverged { saved: u32, current: u32 },
    BaseDiverged { saved: u64, regrown: u64 },
    Encode,
    Decode,
}

/// Authoritative Paredros world state. The base facts are retained for direct
/// consumers, while persistence records genesis plus accepted intents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    seed: u64,
    config: WorldConfig,
    base_hash: u64,
    grown: Grown,
    ground: Ground,
    map: WorldMap,
    intents: Vec<WorldIntent>,
    events: Vec<WorldEvent>,
    last_tick: Option<Tick>,
}

impl World {
    pub fn generate(seed: u64, config: WorldConfig) -> Result<Self, WorldError> {
        if !(2..=256).contains(&config.side) {
            return Err(WorldError::InvalidSide);
        }
        if config.extent < i32::from(config.side) {
            return Err(WorldError::InvalidExtent);
        }
        let grown = Places::grown(seed, config.side, config.extent);
        let ground = Ground::grow(&grown, config.extent);
        let map = WorldMap::generate(&grown);
        let base_hash = Self::hash_base(&grown, &ground, &map)?;
        Ok(Self {
            seed,
            config,
            base_hash,
            grown,
            ground,
            map,
            intents: Vec::new(),
            events: Vec::new(),
            last_tick: None,
        })
    }

    fn hash_base(grown: &Grown, ground: &Ground, map: &WorldMap) -> Result<u64, WorldError> {
        snapshot::encode(&(grown, ground, map))
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| WorldError::Encode)
    }

    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub const fn config(&self) -> WorldConfig {
        self.config
    }

    pub const fn base_hash(&self) -> u64 {
        self.base_hash
    }

    pub fn grown(&self) -> &Grown {
        &self.grown
    }

    pub fn ground(&self) -> &Ground {
        &self.ground
    }

    pub fn map(&self) -> &WorldMap {
        &self.map
    }

    pub fn intents(&self) -> &[WorldIntent] {
        &self.intents
    }

    pub fn events(&self) -> &[WorldEvent] {
        &self.events
    }

    pub fn apply(&mut self, intent: WorldIntent) -> Result<WorldEvent, WorldError> {
        let tick = intent.tick();
        if let Some(previous) = self.last_tick
            && tick < previous
        {
            return Err(WorldError::OutOfOrder {
                previous,
                next: tick,
            });
        }

        let event = match intent {
            WorldIntent::Carve {
                tick,
                by,
                centre,
                radius,
            } => {
                if radius < 0 {
                    return Err(WorldError::InvalidRadius);
                }
                let removed = self.ground.carve(centre, radius);
                WorldEvent::Carved {
                    tick,
                    by,
                    centre,
                    radius,
                    removed,
                    ground_revision: self.ground.revision(),
                }
            },
            WorldIntent::InheritSite {
                tick,
                slot,
                kind,
                fact,
            } => {
                if !self.map.inherit(slot, kind, fact) {
                    return Err(WorldError::MissingSlot(slot));
                }
                WorldEvent::SiteInherited {
                    tick,
                    slot,
                    kind,
                    fact,
                }
            },
        };
        self.intents.push(intent);
        self.events.push(event);
        self.last_tick = Some(tick);
        Ok(event)
    }

    pub fn state_hash(&self) -> Result<u64, WorldError> {
        snapshot::encode(self)
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| WorldError::Encode)
    }

    pub fn save_record(&self) -> WorldSave {
        WorldSave {
            generator_version: GENERATOR_VERSION,
            seed: self.seed,
            config: self.config,
            base_hash: self.base_hash,
            intents: self.intents.clone(),
        }
    }

    pub fn save(&self) -> Result<Vec<u8>, WorldError> {
        snapshot::encode(&self.save_record()).map_err(|_| WorldError::Encode)
    }

    pub fn restore(bytes: &[u8]) -> Result<Self, WorldError> {
        let save: WorldSave = snapshot::decode(bytes).map_err(|_| WorldError::Decode)?;
        Self::restore_record(save)
    }

    pub fn restore_record(save: WorldSave) -> Result<Self, WorldError> {
        if save.generator_version != GENERATOR_VERSION {
            return Err(WorldError::GeneratorDiverged {
                saved: save.generator_version,
                current: GENERATOR_VERSION,
            });
        }
        let mut world = Self::generate(save.seed, save.config)?;
        if save.base_hash != world.base_hash {
            return Err(WorldError::BaseDiverged {
                saved: save.base_hash,
                regrown: world.base_hash,
            });
        }
        for intent in save.intents {
            world.apply(intent)?;
        }
        Ok(world)
    }
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The recorded transition vocabulary shared by every embodied subject.

use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::bodies::{BodyError, Name};
use crate::items::{ItemError, ItemId};
use crate::{AnatomyError, MovementError, WorldError, WorldSave};

pub const GAME_STATE_VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameIntent {
    Generate {
        tick: Tick,
        subject: SubjectId,
        body_seed: u64,
        at: [i32; 3],
    },
    Name {
        tick: Tick,
        subject: SubjectId,
        name: Name,
    },
    Move {
        tick: Tick,
        subject: SubjectId,
        toward: [i32; 3],
    },
    Observe {
        tick: Tick,
        subject: SubjectId,
        target: [i32; 3],
    },
    Take {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
    },
    Eat {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
    },
    Rest {
        tick: Tick,
        subject: SubjectId,
    },
    Fall {
        tick: Tick,
        subject: SubjectId,
        distance: i32,
    },
    Wait {
        tick: Tick,
        subject: SubjectId,
    },
    /// Admit an immutable, validated detailed body snapshot for this revision.
    AdmitAnatomy {
        tick: Tick,
        subject: SubjectId,
        revision: BodyRevisionId,
        document: Box<mesocosm_core::BodyDocument>,
    },
    ReconcileAnatomy {
        tick: Tick,
        subject: SubjectId,
        from_revision: BodyRevisionId,
        revision: BodyRevisionId,
        severed_parts: Vec<mesocosm_core::PartId>,
    },
    AttachItem {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
        part: mesocosm_core::PartId,
        revision: BodyRevisionId,
    },
    DetachItem {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
    },
}

impl GameIntent {
    pub const fn tick(&self) -> Tick {
        match self {
            Self::Generate { tick, .. }
            | Self::Name { tick, .. }
            | Self::Move { tick, .. }
            | Self::Observe { tick, .. }
            | Self::Take { tick, .. }
            | Self::Eat { tick, .. }
            | Self::Rest { tick, .. }
            | Self::Fall { tick, .. }
            | Self::AdmitAnatomy { tick, .. }
            | Self::ReconcileAnatomy { tick, .. }
            | Self::AttachItem { tick, .. }
            | Self::DetachItem { tick, .. }
            | Self::Wait { tick, .. } => *tick,
        }
    }

    pub const fn subject(&self) -> SubjectId {
        match self {
            Self::Generate { subject, .. }
            | Self::Name { subject, .. }
            | Self::Move { subject, .. }
            | Self::Observe { subject, .. }
            | Self::Take { subject, .. }
            | Self::Eat { subject, .. }
            | Self::Rest { subject, .. }
            | Self::Fall { subject, .. }
            | Self::AdmitAnatomy { subject, .. }
            | Self::ReconcileAnatomy { subject, .. }
            | Self::AttachItem { subject, .. }
            | Self::DetachItem { subject, .. }
            | Self::Wait { subject, .. } => *subject,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    Fall,
    Starvation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    AnatomyReconciled {
        tick: Tick,
        subject: SubjectId,
        from_revision: BodyRevisionId,
        revision: BodyRevisionId,
    },
    ItemAttached {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
        part: mesocosm_core::PartId,
    },
    ItemDetached {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
    },
    ItemReleased {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
        at: [i32; 3],
    },
    AnatomyAdmitted {
        tick: Tick,
        subject: SubjectId,
        revision: BodyRevisionId,
    },
    Generated {
        tick: Tick,
        subject: SubjectId,
        revision: BodyRevisionId,
    },
    Named {
        tick: Tick,
        subject: SubjectId,
        name: Name,
    },
    Moved {
        tick: Tick,
        subject: SubjectId,
        from: [i32; 3],
        to: [i32; 3],
    },
    Held {
        tick: Tick,
        subject: SubjectId,
        at: [i32; 3],
    },
    Observed {
        tick: Tick,
        subject: SubjectId,
        target: [i32; 3],
        visible: bool,
    },
    Took {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
    },
    Ate {
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
        hunger: u16,
    },
    Rested {
        tick: Tick,
        subject: SubjectId,
        recovered: u16,
        fatigue: u16,
        revision: BodyRevisionId,
    },
    Injured {
        tick: Tick,
        subject: SubjectId,
        distance: i32,
        harm: u16,
        revision: BodyRevisionId,
    },
    Waited {
        tick: Tick,
        subject: SubjectId,
    },
    Died {
        tick: Tick,
        subject: SubjectId,
        cause: DeathCause,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSave {
    pub version: u32,
    pub world: WorldSave,
    pub expected_hash: u64,
    pub intents: Vec<GameIntent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    World(WorldError),
    Movement(MovementError),
    Body(BodyError),
    Item(ItemError),
    Anatomy(AnatomyError),
    WrongTick { expected: Tick, actual: Tick },
    StateDiverged { saved: u64, restored: u64 },
    VersionDiverged { saved: u32, current: u32 },
    Encode,
    Decode,
}

impl From<WorldError> for GameError {
    fn from(error: WorldError) -> Self {
        Self::World(error)
    }
}

impl From<MovementError> for GameError {
    fn from(error: MovementError) -> Self {
        Self::Movement(error)
    }
}

impl From<BodyError> for GameError {
    fn from(error: BodyError) -> Self {
        Self::Body(error)
    }
}

impl From<ItemError> for GameError {
    fn from(error: ItemError) -> Self {
        Self::Item(error)
    }
}

impl From<AnatomyError> for GameError {
    fn from(error: AnatomyError) -> Self {
        Self::Anatomy(error)
    }
}

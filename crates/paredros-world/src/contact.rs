// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Small deterministic embodied-contact probe.
//!
//! This deliberately takes authored boxes rather than claiming ownership of
//! generated terrain. It is a fixture-scale f32 world-unit simulation; the
//! integer Mesocosm `Aabb` remains the durable anatomical/spatial document.

mod math;
mod mechanics;
mod spatial;
use math::*;

use mesocosm_core::snapshot;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const FIXED_DT_SECONDS: f32 = 1.0 / 60.0;
/// One twenty-minute fixture trace; callers save/rotate longer histories.
pub const MAX_RECORDED_FRAMES: usize = 72_000;
pub type Position = [f32; 3];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoxCollider {
    pub min: Position,
    pub max: Position,
}

impl BoxCollider {
    fn translated(self, at: Position) -> Self {
        Self {
            min: add(self.min, at),
            max: add(self.max, at),
        }
    }

    fn overlaps(self, other: Self) -> bool {
        (0..3).all(|axis| self.min[axis] < other.max[axis] && self.max[axis] > other.min[axis])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BodyId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyKind {
    Crawler,
    Climber,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BodyProfile {
    pub half_extents: Position,
    pub move_speed: f32,
    pub brace_speed: f32,
    pub reach: f32,
    pub anchor_range: f32,
    pub tether_length: f32,
    pub max_integrity: f32,
    pub carry_capacity: f32,
    pub recover_ticks: u16,
    pub attack_windup_ticks: u16,
    pub attack_recovery_ticks: u16,
}

impl BodyProfile {
    pub const fn crawler() -> Self {
        Self {
            half_extents: [0.55, 0.45, 0.7],
            move_speed: 3.0,
            brace_speed: 1.0,
            reach: 1.1,
            anchor_range: 0.0,
            tether_length: 0.0,
            max_integrity: 4.0,
            carry_capacity: 4.0,
            recover_ticks: 45,
            attack_windup_ticks: 8,
            attack_recovery_ticks: 18,
        }
    }
    pub const fn climber() -> Self {
        Self {
            half_extents: [0.35, 0.8, 0.35],
            move_speed: 4.0,
            brace_speed: 1.5,
            reach: 1.7,
            anchor_range: 2.5,
            tether_length: 3.0,
            max_integrity: 3.0,
            carry_capacity: 0.0,
            recover_ticks: 30,
            attack_windup_ticks: 6,
            attack_recovery_ticks: 15,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeldInput {
    pub move_x: f32,
    pub move_z: f32,
    pub look_x: f32,
    pub look_z: f32,
    pub brace: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggeredInput {
    pub interact: bool,
    pub anchor: bool,
    pub attack: bool,
    pub recover: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub held: HeldInput,
    pub triggered: TriggeredInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Impairment {
    Grip,
    Reach,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BodyState {
    pub id: BodyId,
    pub kind: BodyKind,
    pub profile: BodyProfile,
    pub position: Position,
    pub integrity: f32,
    pub impairment: Option<Impairment>,
    pub bracing: bool,
    pub anchored: bool,
    pub anchor: Option<Position>,
    pub heading: Position,
    pub vertical_velocity: f32,
    pub grounded: bool,
    pub recovery_remaining: u16,
    pub attack_remaining: u16,
    pub attack_cooldown: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MovableBoard {
    pub collider: BoxCollider,
    pub position: Position,
    pub mass: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContactEffect {
    Blocked {
        body: BodyId,
    },
    BoardMoved {
        by: BodyId,
        position: Position,
    },
    BoardHeld {
        by: BodyId,
    },
    BoardPlaced {
        by: BodyId,
        position: Position,
    },
    Anchored {
        body: BodyId,
    },
    AnchorLost {
        body: BodyId,
    },
    AttackStarted {
        body: BodyId,
    },
    AttackHit {
        source: BodyId,
        target: BodyId,
        impairment: Impairment,
    },
    AttackBraced {
        source: BodyId,
        target: BodyId,
    },
    RecoveryStarted {
        body: BodyId,
    },
    Recovered {
        body: BodyId,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputFrame {
    pub tick: u64,
    pub inputs: Vec<(BodyId, Input)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContactSave {
    pub version: u8,
    pub solids: Vec<BoxCollider>,
    pub bodies: Vec<(BodyKind, BodyProfile, Position)>,
    pub board: Option<MovableBoard>,
    pub frames: Vec<InputFrame>,
}

impl ContactSave {
    /// Encodes a record for durable storage. [`ContactWorld::restore`] remains
    /// the admission boundary and rejects malformed decoded records.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ContactError> {
        snapshot::encode(self).map_err(|_| ContactError::Decode)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactError {
    NonFinite,
    InvalidConfiguration,
    ConfigurationLocked,
    RecordFull,
    Decode,
    UnsupportedVersion(u8),
    InvalidFrameTick { expected: u64, actual: u64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContactWorld {
    solids: Vec<BoxCollider>,
    bodies: BTreeMap<BodyId, BodyState>,
    board: Option<MovableBoard>,
    board_holder: Option<BodyId>,
    initial_board: Option<MovableBoard>,
    starts: Vec<(BodyKind, BodyProfile, Position)>,
    effects: Vec<ContactEffect>,
    inputs: Vec<InputFrame>,
    tick: u64,
    next_body: u32,
}

impl ContactWorld {
    pub fn new(solids: Vec<BoxCollider>) -> Self {
        Self {
            solids,
            bodies: BTreeMap::new(),
            board: None,
            board_holder: None,
            initial_board: None,
            starts: Vec::new(),
            effects: Vec::new(),
            inputs: Vec::new(),
            tick: 0,
            next_body: 0,
        }
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn body(&self, id: BodyId) -> Option<&BodyState> {
        self.bodies.get(&id)
    }
    pub fn bodies(&self) -> impl Iterator<Item = &BodyState> {
        self.bodies.values()
    }
    pub fn solids(&self) -> &[BoxCollider] {
        &self.solids
    }
    pub fn board(&self) -> Option<&MovableBoard> {
        self.board.as_ref()
    }
    pub fn board_holder(&self) -> Option<BodyId> {
        self.board_holder
    }
    pub fn effects(&self) -> &[ContactEffect] {
        &self.effects
    }
    pub fn recorded_inputs(&self) -> &[InputFrame] {
        &self.inputs
    }

    pub fn add_body(&mut self, kind: BodyKind, position: Position) -> Result<BodyId, ContactError> {
        let profile = match kind {
            BodyKind::Crawler => BodyProfile::crawler(),
            BodyKind::Climber => BodyProfile::climber(),
        };
        self.add_body_with_profile(kind, profile, position)
    }

    pub fn add_body_with_profile(
        &mut self,
        kind: BodyKind,
        profile: BodyProfile,
        position: Position,
    ) -> Result<BodyId, ContactError> {
        self.configuration_open()?;
        if !finite_profile(profile) || !finite_position(position) {
            return Err(ContactError::InvalidConfiguration);
        }
        let id = BodyId(self.next_body);
        self.next_body += 1;
        self.starts.push((kind, profile, position));
        self.bodies.insert(
            id,
            BodyState {
                id,
                kind,
                profile,
                position,
                integrity: profile.max_integrity,
                impairment: None,
                bracing: false,
                anchored: false,
                anchor: None,
                heading: [0.0, 0.0, 1.0],
                vertical_velocity: 0.0,
                grounded: false,
                recovery_remaining: 0,
                attack_remaining: 0,
                attack_cooldown: 0,
            },
        );
        Ok(id)
    }

    pub fn set_board(&mut self, board: MovableBoard) -> Result<(), ContactError> {
        self.configuration_open()?;
        if !finite_board(board) {
            return Err(ContactError::InvalidConfiguration);
        }
        self.board = Some(board);
        self.initial_board = Some(board);
        Ok(())
    }

    /// Advances exactly one 60 Hz tick. Inputs for unknown bodies are ignored,
    /// and accepted inputs are recorded in body-id order for deterministic replay.
    pub fn step(&mut self, inputs: &[(BodyId, Input)]) -> Result<(), ContactError> {
        if !self.solids.iter().all(finite_box)
            || !self.bodies.values().all(|body| finite_body(*body))
            || !self.board.is_none_or(finite_board)
            || !inputs.iter().all(|(_, input)| finite_input(*input))
        {
            return Err(ContactError::NonFinite);
        }
        if self.inputs.len() == MAX_RECORDED_FRAMES {
            return Err(ContactError::RecordFull);
        }
        self.effects.clear();
        let mut accepted = inputs
            .iter()
            .copied()
            .filter(|(id, _)| self.bodies.contains_key(id))
            .collect::<Vec<_>>();
        accepted.sort_by_key(|(id, _)| *id);
        accepted.dedup_by_key(|(id, _)| *id);
        for id in self.bodies.keys().copied() {
            if !accepted.iter().any(|(present, _)| *present == id) {
                accepted.push((id, Input::default()));
            }
        }
        accepted.sort_by_key(|(id, _)| *id);
        self.inputs.push(InputFrame {
            tick: self.tick,
            inputs: accepted.clone(),
        });
        for (id, input) in accepted {
            self.step_body(id, input);
        }
        self.tick += 1;
        Ok(())
    }

    pub fn replay(
        solids: Vec<BoxCollider>,
        bodies: &[(BodyKind, BodyProfile, Position)],
        board: Option<MovableBoard>,
        frames: &[InputFrame],
    ) -> Result<Self, ContactError> {
        validate_initial(&solids, bodies, board)?;
        let mut world = Self::new(solids);
        for (kind, profile, position) in bodies {
            world.add_body_with_profile(*kind, *profile, *position)?;
        }
        if let Some(board) = board {
            world.set_board(board)?;
        }
        for frame in frames {
            if frame.tick != world.tick {
                return Err(ContactError::InvalidFrameTick {
                    expected: world.tick,
                    actual: frame.tick,
                });
            }
            world.step(&frame.inputs)?;
        }
        Ok(world)
    }

    pub fn save_record(&self) -> ContactSave {
        ContactSave {
            version: 1,
            solids: self.solids.clone(),
            bodies: self.starts.clone(),
            board: self.initial_board,
            frames: self.inputs.clone(),
        }
    }
    pub fn save(&self) -> Result<Vec<u8>, ContactError> {
        if !self.solids.iter().all(finite_box)
            || !self.bodies.values().all(|body| finite_body(*body))
            || !self.board.is_none_or(finite_board)
        {
            return Err(ContactError::NonFinite);
        }
        self.save_record().to_bytes()
    }
    pub fn restore(bytes: &[u8]) -> Result<Self, ContactError> {
        let save: ContactSave = snapshot::decode(bytes).map_err(|_| ContactError::Decode)?;
        if save.version != 1 {
            return Err(ContactError::UnsupportedVersion(save.version));
        }
        Self::replay(save.solids, &save.bodies, save.board, &save.frames)
    }
}

fn finite_box(box_: &BoxCollider) -> bool {
    box_.min
        .iter()
        .chain(box_.max.iter())
        .all(|value| value.is_finite())
        && (0..3).all(|axis| box_.min[axis] < box_.max[axis])
}
fn finite_input(input: Input) -> bool {
    [
        input.held.move_x,
        input.held.move_z,
        input.held.look_x,
        input.held.look_z,
    ]
    .into_iter()
    .all(|value| value.is_finite() && (-1.0..=1.0).contains(&value))
}
fn finite_board(board: MovableBoard) -> bool {
    finite_box(&board.collider)
        && board.position.iter().all(|value| value.is_finite())
        && board.mass.is_finite()
        && board.mass > 0.0
}
fn finite_position(position: Position) -> bool {
    position.into_iter().all(f32::is_finite)
}
fn finite_profile(profile: BodyProfile) -> bool {
    profile
        .half_extents
        .into_iter()
        .all(|value| value.is_finite() && value > 0.0)
        && [
            profile.move_speed,
            profile.brace_speed,
            profile.reach,
            profile.anchor_range,
            profile.tether_length,
            profile.max_integrity,
            profile.carry_capacity,
        ]
        .into_iter()
        .all(|value| value.is_finite() && value >= 0.0)
        && profile.move_speed > 0.0
        && profile.brace_speed > 0.0
        && profile.max_integrity > 0.0
}
fn finite_body(body: BodyState) -> bool {
    finite_position(body.position)
        && finite_profile(body.profile)
        && body.integrity.is_finite()
        && body.vertical_velocity.is_finite()
        && finite_position(body.heading)
        && body.anchor.is_none_or(finite_position)
}
fn validate_initial(
    solids: &[BoxCollider],
    bodies: &[(BodyKind, BodyProfile, Position)],
    board: Option<MovableBoard>,
) -> Result<(), ContactError> {
    if solids.iter().all(finite_box)
        && bodies
            .iter()
            .all(|(_, profile, position)| finite_profile(*profile) && finite_position(*position))
        && board.is_none_or(finite_board)
    {
        Ok(())
    } else {
        Err(ContactError::InvalidConfiguration)
    }
}

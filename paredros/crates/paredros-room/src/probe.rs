// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The played body, and the discipline that makes a run repeatable.
//!
//! A tick is one heading from a fixed trace handed to `near::step`. Nothing
//! else moves. The probe keeps the position log because the log, not the
//! final position, is what a replay has to match: two runs can end in the
//! same corner having taken different paths there, and only one of those is
//! determinism.
//!
//! A save names its world by seed and carries a hash of the ground it was
//! taken from, so restoring into a world that regenerated differently is a
//! reported failure rather than a silently divergent replay.

use mesocosm_core::places::{WALKER_HEIGHT, step};
use mesocosm_core::snapshot::{self, hash_bytes};
use serde::{Deserialize, Serialize};

use crate::room::{Room, RoomError};

/// Per-tick move headings, in x and z. Eight runs of eight: the four
/// cardinals, then the four diagonals. Each run is longer than the room is
/// wide, so every leg ends pressed against a wall and the kinematics get
/// asked to refuse a move as well as make one.
#[rustfmt::skip]
pub const TRACE: [[i32; 2]; 64] = [
    [ 1,  0], [ 1,  0], [ 1,  0], [ 1,  0], [ 1,  0], [ 1,  0], [ 1,  0], [ 1,  0],
    [ 0,  1], [ 0,  1], [ 0,  1], [ 0,  1], [ 0,  1], [ 0,  1], [ 0,  1], [ 0,  1],
    [-1,  0], [-1,  0], [-1,  0], [-1,  0], [-1,  0], [-1,  0], [-1,  0], [-1,  0],
    [ 0, -1], [ 0, -1], [ 0, -1], [ 0, -1], [ 0, -1], [ 0, -1], [ 0, -1], [ 0, -1],
    [ 1,  1], [ 1,  1], [ 1,  1], [ 1,  1], [ 1,  1], [ 1,  1], [ 1,  1], [ 1,  1],
    [-1, -1], [-1, -1], [-1, -1], [-1, -1], [-1, -1], [-1, -1], [-1, -1], [-1, -1],
    [-1,  1], [-1,  1], [-1,  1], [-1,  1], [-1,  1], [-1,  1], [-1,  1], [-1,  1],
    [ 1, -1], [ 1, -1], [ 1, -1], [ 1, -1], [ 1, -1], [ 1, -1], [ 1, -1], [ 1, -1],
];

/// How long a full run is. The trace is played once, not looped.
pub const TICKS: usize = TRACE.len();

/// A probe run, captured whole.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Save {
    pub seed: u64,
    /// FNV-1a over the encoded ground, so a restore can prove it is standing
    /// in the world the save was taken from.
    pub ground_hash: u64,
    pub tick: usize,
    pub at: [i32; 3],
    pub log: Vec<[i32; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeError {
    Room(RoomError),
    /// The save's world does not regrow to the ground it was taken from.
    GroundDiverged,
    Encode,
    Decode,
}

/// One body in one room, driven by [`TRACE`].
#[derive(Clone, Debug)]
pub struct Probe {
    room: Room,
    tick: usize,
    at: [i32; 3],
    log: Vec<[i32; 3]>,
}

impl Probe {
    /// A fresh run at tick zero, standing on the room's floor.
    pub fn new(seed: u64) -> Result<Self, ProbeError> {
        let room = Room::grow(seed).map_err(ProbeError::Room)?;
        let at = room.stance;
        Ok(Self {
            room,
            tick: 0,
            at,
            log: vec![at],
        })
    }

    pub fn room(&self) -> &Room {
        &self.room
    }

    pub fn tick_count(&self) -> usize {
        self.tick
    }

    pub fn at(&self) -> [i32; 3] {
        self.at
    }

    pub fn log(&self) -> &[[i32; 3]] {
        &self.log
    }

    /// Whether the trace still has moves left in it.
    pub fn running(&self) -> bool {
        self.tick < TICKS
    }

    /// The heading the body last took, for anything that needs to know which
    /// way it is facing. Before the first tick, the one it is about to take.
    pub fn heading(&self) -> [i32; 2] {
        TRACE[self.tick.saturating_sub(1).min(TICKS - 1)]
    }

    /// One tick: one heading, one `step`, one logged position. Returns false
    /// when the trace is spent.
    pub fn advance(&mut self) -> bool {
        if !self.running() {
            return false;
        }
        let [dx, dz] = TRACE[self.tick];
        let toward = [self.at[0] + dx, self.at[1], self.at[2] + dz];
        self.at = step(&self.room.ground, self.at, toward);
        debug_assert!(
            self.room.ground.stands(self.at, WALKER_HEIGHT),
            "step left the body unstandable at {:?}",
            self.at
        );
        self.log.push(self.at);
        self.tick += 1;
        true
    }

    /// Runs until the trace is spent, or `ticks` have passed.
    pub fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            if !self.advance() {
                break;
            }
        }
    }

    /// FNV-1a over the encoded position log. The equality witness a replay
    /// has to reproduce.
    pub fn hash(&self) -> u64 {
        hash_bytes(&snapshot::encode(&self.log).expect("a position log always encodes"))
    }

    pub fn ground_hash(&self) -> u64 {
        hash_bytes(&snapshot::encode(&self.room.ground).expect("ground always encodes"))
    }

    pub fn save(&self) -> Result<Vec<u8>, ProbeError> {
        let save = Save {
            seed: self.room.seed,
            ground_hash: self.ground_hash(),
            tick: self.tick,
            at: self.at,
            log: self.log.clone(),
        };
        snapshot::encode(&save).map_err(|_| ProbeError::Encode)
    }

    /// Restores a run into a freshly grown world. The world is rebuilt from
    /// the seed rather than carried in the save, and then checked: a ground
    /// that regrew differently is a loud failure, because every later gate
    /// rests on the same regrow-from-seed assumption.
    pub fn restore(bytes: &[u8]) -> Result<Self, ProbeError> {
        let save: Save = snapshot::decode(bytes).map_err(|_| ProbeError::Decode)?;
        let room = Room::grow(save.seed).map_err(ProbeError::Room)?;
        let probe = Self {
            room,
            tick: save.tick,
            at: save.at,
            log: save.log,
        };
        if probe.ground_hash() != save.ground_hash {
            return Err(ProbeError::GroundDiverged);
        }
        Ok(probe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::SEED;

    #[test]
    fn a_run_visits_more_than_one_voxel_and_stays_in_the_room() {
        let mut probe = Probe::new(SEED).unwrap();
        probe.run(TICKS);
        assert_eq!(probe.tick_count(), TICKS);
        assert_eq!(probe.log().len(), TICKS + 1);

        let distinct: std::collections::BTreeSet<_> = probe.log().iter().collect();
        assert!(distinct.len() > 4, "the trace barely moved: {distinct:?}");

        let (low, high) = probe.room().interior();
        for at in probe.log() {
            for axis in 0..3 {
                assert!(
                    at[axis] >= low[axis] && at[axis] <= high[axis],
                    "the body left the room at {at:?}"
                );
            }
        }
    }

    #[test]
    fn the_trace_ends_pressed_against_walls() {
        // Each leg is longer than the room is wide, so the kinematics must
        // refuse moves as well as make them. If every tick moved the body,
        // the room is not enclosing it.
        let mut probe = Probe::new(SEED).unwrap();
        probe.run(TICKS);
        let held = probe
            .log()
            .windows(2)
            .filter(|pair| pair[0] == pair[1])
            .count();
        assert!(held > 0, "no tick was refused; the walls did nothing");
    }
}

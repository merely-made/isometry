// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A small inherited policy behind fauna target choice.
//!
//! The policy is deliberately less powerful than the movement resolver. It
//! can rank one visible target and propose pursuit, avoidance, or holding. It
//! cannot authorize sight, cross terrain, spend matter, or emit an event.

use serde::{Deserialize, Serialize};

use crate::process::{FeedingMode, Process};
use crate::rng::Rng;

use super::{Organism, OrganismId, Signal};

const SENSOR_COUNT: usize = 5;
const DRIVE_COUNT: usize = 3;
const INPUT_LIMIT: i32 = 32;
const WEIGHT_SCALE: i32 = 8;
const STATE_LIMIT: i32 = 64;
const POLICY_SALT: u64 = 0x504F_4C49_4359_0001;

/// The three bounded effects the first fauna controller may propose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FaunaDrive {
    Pursue,
    Avoid,
    Hold,
}

impl FaunaDrive {
    const ALL: [Self; DRIVE_COUNT] = [Self::Pursue, Self::Avoid, Self::Hold];

    const fn index(self) -> usize {
        match self {
            Self::Pursue => 0,
            Self::Avoid => 1,
            Self::Hold => 2,
        }
    }
}

/// Body-derived affordances that gated a fauna decision.
///
/// Every field is a reading of the body, and since DC1.5 that is true of
/// `feeding_mode` too — it used to arrive by way of `body.plan.symmetry`, which
/// no decision could ever change. This trace is serialized inside
/// `last_fauna_decision` and so inside `state_hash`, which is why the unbinding
/// moved every replay hash: the same body now reports a mode read off its
/// mouth. Priced in and accepted by the DC1.5 ruling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaunaTraits {
    pub feeding_mode: FeedingMode,
    pub reach: i32,
    pub locomotion: u32,
    pub sensory_parts: u16,
}

impl FaunaTraits {
    pub(crate) fn read(organism: &Organism) -> Self {
        let sensory_parts = organism
            .body()
            .living()
            .filter(|part| organism.body().processes(part.id).contains(&Process::Sense))
            .count()
            .min(u16::MAX as usize) as u16;
        Self {
            feeding_mode: organism.feeding_mode(),
            reach: organism.body().reach(),
            locomotion: organism.locomotion(),
            sensory_parts,
        }
    }
}

/// The local facts quantized for one candidate target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaunaSenses {
    pub energy_deficit_mg: u64,
    pub target_distance: i32,
    pub target_mass_mg: u64,
    /// A warning is available to the policy only when anatomy supplies a
    /// sensory part. `None` means the signal was not sensed, not that the
    /// target necessarily advertised nothing.
    pub target_signal: Option<Signal>,
    pub remembered_target: bool,
}

impl FaunaSenses {
    pub(crate) fn read(
        organism: &Organism,
        traits: FaunaTraits,
        target: OrganismId,
        distance: i32,
        target_mass_mg: u64,
        signal: Signal,
    ) -> Self {
        Self {
            energy_deficit_mg: organism.biomass_mg().saturating_sub(organism.energy_mg),
            target_distance: distance,
            target_mass_mg,
            target_signal: (traits.sensory_parts > 0).then_some(signal),
            remembered_target: organism
                .last_seen
                .is_some_and(|memory| memory.target == target),
        }
    }

    fn quantized(self, own_mass_mg: u64, sight: i32) -> [i16; SENSOR_COUNT] {
        let own_mass = own_mass_mg.max(1);
        let hunger = scale_u64(self.energy_deficit_mg.min(own_mass), own_mass);
        let sight = sight.max(1);
        let nearness =
            ((sight - self.target_distance).clamp(0, sight) * INPUT_LIMIT / sight) as i16;
        let mass_scale = own_mass.max(self.target_mass_mg).max(1);
        let relative_mass = if self.target_mass_mg >= own_mass {
            scale_u64(self.target_mass_mg - own_mass, mass_scale)
        } else {
            -scale_u64(own_mass - self.target_mass_mg, mass_scale)
        };
        let warning = i16::from(self.target_signal == Some(Signal::Warning)) * INPUT_LIMIT as i16;
        let remembered = i16::from(self.remembered_target) * INPUT_LIMIT as i16;
        [hunger, nearness, relative_mass, warning, remembered]
    }
}

fn scale_u64(value: u64, whole: u64) -> i16 {
    ((u128::from(value) * INPUT_LIMIT as u128 / u128::from(whole.max(1))).min(INPUT_LIMIT as u128))
        as i16
}

fn mutate_i8(value: i8, delta: i8) -> i8 {
    match (value, delta) {
        (i8::MAX, 1) => value - 1,
        (i8::MIN, -1) => value + 1,
        _ => value + delta,
    }
}

fn mutate_i16(value: i16, delta: i8) -> i16 {
    match (value, delta) {
        (i16::MAX, 1) => value - 1,
        (i16::MIN, -1) => value + 1,
        _ => value + i16::from(delta),
    }
}

/// Quantized output scores retained in an inspectable decision trace.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaunaDriveScores {
    pub pursue: i16,
    pub avoid: i16,
    pub hold: i16,
}

impl FaunaDriveScores {
    fn from_array(scores: [i16; DRIVE_COUNT]) -> Self {
        Self {
            pursue: scores[FaunaDrive::Pursue.index()],
            avoid: scores[FaunaDrive::Avoid.index()],
            hold: scores[FaunaDrive::Hold.index()],
        }
    }

    fn as_array(self) -> [i16; DRIVE_COUNT] {
        [self.pursue, self.avoid, self.hold]
    }

    pub(crate) fn selected(self) -> FaunaDrive {
        let scores = self.as_array();
        // Variant order is the deterministic tie-break: useful action before
        // avoidance, avoidance before holding.
        FaunaDrive::ALL
            .into_iter()
            .max_by_key(|drive| (scores[drive.index()], std::cmp::Reverse(drive.index())))
            .unwrap_or(FaunaDrive::Hold)
    }

    pub(crate) fn score(self, drive: FaunaDrive) -> i16 {
        self.as_array()[drive.index()]
    }
}

/// Why the controller proposed its last bounded movement intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaunaDecisionTrace {
    pub traits: FaunaTraits,
    pub senses: FaunaSenses,
    pub selected_drive: FaunaDrive,
    pub selected_target: Option<OrganismId>,
    pub scores: FaunaDriveScores,
}

/// A fixed three-drive recurrent policy with integer weights and state.
///
/// Genotype and recurrent state are both replayed. Offspring inherit the
/// genotype with one bounded deterministic mutation and begin with clear
/// recurrent state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaunaPolicy {
    pub sensor_weights: [[i8; SENSOR_COUNT]; DRIVE_COUNT],
    pub recurrent_weights: [i8; DRIVE_COUNT],
    pub biases: [i16; DRIVE_COUNT],
    pub state: [i16; DRIVE_COUNT],
}

impl Default for FaunaPolicy {
    fn default() -> Self {
        Self {
            // Inputs: hunger, target nearness, relative mass, warning, memory.
            sensor_weights: [[4, 4, 1, -8, 2], [-1, 1, -1, 10, -1], [-4, -2, 0, 1, -1]],
            recurrent_weights: [2, 2, 3],
            biases: [16, -16, 0],
            state: [0; DRIVE_COUNT],
        }
    }
}

impl FaunaPolicy {
    pub(crate) fn score(
        self,
        senses: FaunaSenses,
        own_mass_mg: u64,
        sight: i32,
    ) -> FaunaDriveScores {
        let inputs = senses.quantized(own_mass_mg, sight);
        let scores = std::array::from_fn(|drive| {
            let sensed = self.sensor_weights[drive]
                .iter()
                .zip(inputs)
                .map(|(weight, input)| i32::from(*weight) * i32::from(input))
                .sum::<i32>();
            let recurrent = i32::from(self.recurrent_weights[drive]) * i32::from(self.state[drive]);
            let score = i32::from(self.biases[drive]) + (sensed + recurrent) / WEIGHT_SCALE;
            score.clamp(i16::MIN as i32, i16::MAX as i32) as i16
        });
        FaunaDriveScores::from_array(scores)
    }

    pub(crate) fn remember(&mut self, scores: FaunaDriveScores) {
        self.state = scores
            .as_array()
            .map(|score| i32::from(score).clamp(-STATE_LIMIT, STATE_LIMIT) as i16);
    }

    pub(crate) fn inherited(self, seed: u64) -> Self {
        let mut child = self;
        child.state = [0; DRIVE_COUNT];
        let mut rng = Rng::from_seed(seed ^ POLICY_SALT);
        let sensor_genes = DRIVE_COUNT * SENSOR_COUNT;
        let recurrent_genes = DRIVE_COUNT;
        let gene = rng.below((sensor_genes + recurrent_genes + DRIVE_COUNT) as u64) as usize;
        let delta = if rng.below(2) == 0 { -1 } else { 1 };
        if gene < sensor_genes {
            let drive = gene / SENSOR_COUNT;
            let sensor = gene % SENSOR_COUNT;
            child.sensor_weights[drive][sensor] =
                mutate_i8(child.sensor_weights[drive][sensor], delta);
        } else if gene < sensor_genes + recurrent_genes {
            let drive = gene - sensor_genes;
            child.recurrent_weights[drive] = mutate_i8(child.recurrent_weights[drive], delta);
        } else {
            let drive = gene - sensor_genes - recurrent_genes;
            child.biases[drive] = mutate_i16(child.biases[drive], delta);
        }
        child
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inheritance_is_quantized_deterministic_and_clears_memory() {
        let parent = FaunaPolicy {
            state: [12, -4, 7],
            ..FaunaPolicy::default()
        };
        let a = parent.inherited(91);
        let b = parent.inherited(91);
        assert_eq!(a, b);
        assert_eq!(a.state, [0; DRIVE_COUNT]);
        assert_ne!(a, FaunaPolicy::default());
    }

    #[test]
    fn a_sensed_warning_can_reverse_the_bounded_drive() {
        let policy = FaunaPolicy::default();
        let plain = FaunaSenses {
            target_distance: 2,
            target_mass_mg: 100,
            target_signal: Some(Signal::Plain),
            ..FaunaSenses::default()
        };
        let warned = FaunaSenses {
            target_signal: Some(Signal::Warning),
            ..plain
        };

        assert_eq!(policy.score(plain, 100, 8).selected(), FaunaDrive::Pursue);
        assert_eq!(policy.score(warned, 100, 8).selected(), FaunaDrive::Avoid);
    }
}

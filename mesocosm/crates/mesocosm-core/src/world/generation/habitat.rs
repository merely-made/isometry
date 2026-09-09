// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::places::{Ground, WalkerShape, step_for};

/// Development seed and role. The request's body criteria complete the recipe.
/// This is a local generation input, not a portable subject identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedBody {
    pub seed: u64,
    pub role: Kingdom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoilPattern {
    #[default]
    Patches,
    Uniform,
    Contrasting,
}

impl SoilPattern {
    pub fn label(self) -> &'static str {
        match self {
            Self::Patches => "patches",
            Self::Uniform => "uniform",
            Self::Contrasting => "contrasting",
        }
    }

    pub(super) fn draw(self, rng: &mut Rng, min: u64, max: u64) -> u64 {
        match self {
            Self::Patches => min + rng.below(max - min + 1),
            Self::Uniform => min + (max - min) / 2,
            Self::Contrasting => {
                if rng.below(2) == 0 {
                    min
                } else {
                    max
                }
            },
        }
    }
}

pub(super) fn open_steps(ground: &Ground, shape: WalkerShape, position: [i32; 3]) -> u8 {
    [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]]
        .into_iter()
        .filter(|delta| {
            let toward = [0, 1, 2].map(|i| position[i] + delta[i]);
            step_for(ground, shape, position, toward) != position
        })
        .count() as u8
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub ticks: u32,
    /// Producer, consumer, decomposer living counts, in that order.
    pub before: [u32; 3],
    pub after: [u32; 3],
    pub subject_alive: bool,
    /// Diet admission only, within eight voxels horizontally. Not proof of
    /// reach, sight, a successful bite, or a sustained food supply.
    pub compatible_nearby: u32,
    pub matter_before_mg: u64,
    pub matter_after_mg: u64,
    /// Accepted flow and life-cycle records from this disposable trial.
    pub evidence: TrialEvidence,
}

impl Prepared {
    pub fn habitat_world(&self) -> &World {
        &self.foundation
    }

    /// An unattended trial on a disposable clone. Never advances entry state.
    pub fn observe(&self, index: usize, ticks: u32) -> Result<Observation, Error> {
        if ticks > 128 {
            return Err(Error::Invalid("observation is bounded to 128 ticks"));
        }
        let mut world = self.enter(index)?;
        let subject = world.controlled().expect("admitted body");
        let subject_id = subject.id;
        let compatible_nearby = world
            .organisms
            .iter()
            .filter(|other| {
                let kind = match other.kingdom() {
                    Kingdom::Producer => crate::process::NisKind::Producer,
                    Kingdom::Consumer => crate::process::NisKind::Consumer,
                    Kingdom::Decomposer => crate::process::NisKind::Decomposer,
                };
                other.id != subject_id
                    && other.is_alive()
                    && [0, 2]
                        .into_iter()
                        .all(|i| (other.position[i] - subject.position[i]).abs() <= 8)
                    && subject.admits(kind, false)
            })
            .count() as u32;
        let census = |world: &World| {
            let mut counts = [0; 3];
            for organism in world.organisms.iter().filter(|o| o.is_alive()) {
                counts[match organism.kingdom() {
                    Kingdom::Producer => 0,
                    Kingdom::Consumer => 1,
                    Kingdom::Decomposer => 2,
                }] += 1;
            }
            counts
        };
        let before = census(&world);
        let matter_before_mg = world.total_matter_mg();
        // Founding is not trial activity. These buffers are non-authoritative
        // presentation records, so draining them cannot change entry or replay.
        let _ = world.drain_flows();
        let _ = world.drain_events();
        let mut evidence = TrialEvidence::begin(&world, subject_id, ticks);
        let mut living = world.living().map(|organism| organism.id).collect();
        for _ in 0..ticks {
            evidence.record_controller(&world, subject_id);
            world.apply(crate::Intent::Idle);
            evidence.record_tick(&mut world, subject_id, &mut living);
        }
        evidence.finish(&world, subject_id);
        Ok(Observation {
            ticks,
            before,
            after: census(&world),
            compatible_nearby,
            matter_before_mg,
            matter_after_mg: world.total_matter_mg(),
            evidence,
            subject_alive: world
                .organisms
                .iter()
                .any(|o| o.id == subject_id && o.is_alive()),
        })
    }
}

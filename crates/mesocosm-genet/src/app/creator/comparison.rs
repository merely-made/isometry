// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::world::generation::{Criteria, FixedBody, Observation, Request, SoilPattern};

/// The habitat inputs that can change while a generated body is held.
///
/// Body criteria and variation are deliberately absent. Once a body is held,
/// this is the request identity used to deduplicate completed trials.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HabitatKey {
    pub seed: u64,
    pub place: u16,
    pub soil_min_mg: u64,
    pub soil_max_mg: u64,
    pub organisms: u32,
    pub min_open_steps: u8,
    pub soil_pattern: SoilPattern,
    pub ticks: u32,
}

impl HabitatKey {
    pub fn from_request(request: &Request, ticks: u32) -> Option<Self> {
        request.fixed_body.as_ref()?;
        Some(Self {
            seed: request.seed,
            place: request.place,
            soil_min_mg: request.soil_min_mg,
            soil_max_mg: request.soil_max_mg,
            organisms: request.organisms,
            min_open_steps: request.min_open_steps,
            soil_pattern: request.soil_pattern,
            ticks,
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct Trial {
    pub key: HabitatKey,
    pub observation: Observation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HeldBodyKey {
    pub body: FixedBody,
    pub criteria: Criteria,
}

impl HeldBodyKey {
    pub fn from_request(request: &Request) -> Option<Self> {
        Some(Self {
            body: request.fixed_body.clone()?,
            criteria: request.criteria.clone(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct ComparisonHistory {
    body: Option<HeldBodyKey>,
    trials: Vec<Trial>,
}

impl ComparisonHistory {
    pub fn clear(&mut self) {
        self.body = None;
        self.trials.clear();
    }

    pub fn set_body(&mut self, request: &Request) {
        let body = HeldBodyKey::from_request(request);
        if self.body != body {
            self.trials.clear();
            self.body = body;
        }
    }

    pub fn record(&mut self, request: &Request, ticks: u32, observation: Observation) {
        let Some(key) = HabitatKey::from_request(request, ticks) else {
            return;
        };
        self.set_body(request);
        if let Some(index) = self.trials.iter().position(|trial| trial.key == key) {
            self.trials.remove(index);
        }
        self.trials.push(Trial { key, observation });
        if self.trials.len() > 3 {
            self.trials.remove(0);
        }
    }

    pub fn trials(&self) -> &[Trial] {
        &self.trials
    }

    pub fn len(&self) -> usize {
        self.trials.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::world::generation::{TrialController, TrialEvidence};

    fn request(seed: u64) -> Request {
        Request {
            fixed_body: Some(FixedBody {
                seed: 91,
                role: mesocosm_core::Kingdom::Consumer,
            }),
            seed,
            ..Request::default()
        }
    }

    fn observation(ticks: u32) -> Observation {
        Observation {
            ticks,
            before: [1, 2, 3],
            after: [1, 2, 3],
            subject_alive: true,
            compatible_nearby: 1,
            matter_before_mg: 10,
            matter_after_mg: 9,
            evidence: TrialEvidence {
                controller: TrialController {
                    idle_ticks: ticks,
                    held_ticks: ticks,
                    instinct_ticks: 0,
                    inactive_ticks: 0,
                },
                subject_feeding_events: 0,
                subject_feeding_mg: 0,
                subject_uptake_mg: 0,
                subject_incoming_mg: 0,
                subject_predation_mg: 0,
                subject_reserve_before_mg: 10,
                subject_reserve_after_mg: 9,
                subject_alive: true,
                births: 0,
                deaths: 0,
                subject_death_recorded: false,
            },
        }
    }

    #[test]
    fn history_is_bounded_and_identical_requests_deduplicate() {
        let mut history = ComparisonHistory::default();
        for seed in 1..=3 {
            history.record(&request(seed), 128, observation(seed as u32));
        }
        history.record(&request(2), 128, observation(99));
        assert_eq!(history.len(), 3);
        assert_eq!(history.trials()[2].observation.ticks, 99);
        history.record(&request(4), 128, observation(4));
        assert_eq!(history.len(), 3);
        assert_eq!(history.trials()[0].key.seed, 3);
    }

    #[test]
    fn changing_held_body_resets_history() {
        let mut history = ComparisonHistory::default();
        history.record(&request(1), 128, observation(1));
        let mut other = request(1);
        other.fixed_body.as_mut().unwrap().seed += 1;
        history.record(&other, 128, observation(2));
        assert_eq!(history.len(), 1);
        assert_eq!(history.trials()[0].observation.ticks, 2);
    }

    #[test]
    fn trial_window_is_part_of_comparison_identity() {
        let mut history = ComparisonHistory::default();
        history.record(&request(1), 32, observation(32));
        history.record(&request(1), 128, observation(128));
        assert_eq!(history.len(), 2);
        assert_eq!(history.trials()[0].key.ticks, 32);
    }
}

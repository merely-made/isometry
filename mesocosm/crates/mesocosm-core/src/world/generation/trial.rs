// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reduction of the bounded habitat trial's accepted records.

use crate::World;
use crate::flow::{Process, RecordedFlow};
use crate::history::{Event, MealKind};
use crate::organism::OrganismId;
use serde::Serialize;
use std::collections::BTreeSet;

/// How the trial supplied the played body's controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TrialController {
    /// Every sampled tick receives [`crate::Intent::Idle`].
    pub idle_ticks: u32,
    /// The initial held period. Only instinctive movement is suppressed; the
    /// body still ages, pays upkeep, and may receive income or a reachable meal.
    pub held_ticks: u32,
    /// Ticks after the idle grace period, when ordinary instincts may act.
    pub instinct_ticks: u32,
    /// Ticks after the subject was no longer living. They are sampled but do
    /// not claim that the body's instincts acted.
    pub inactive_ticks: u32,
}

/// Accepted resource and life-cycle evidence from one disposable trial.
///
/// This is a reading of recorded events and flows, not an explanation of why a
/// population changed. In particular, an alive founder with no recorded intake
/// only establishes that the bounded run did not record intake.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrialEvidence {
    pub controller: TrialController,
    /// Number of accepted feeding flow entries, not a count of distinct meals.
    pub subject_feeding_events: u32,
    pub subject_feeding_mg: u64,
    pub subject_uptake_mg: u64,
    /// Recorded predatory matter the subject took from another body. This does
    /// not establish why a later death occurred.
    pub subject_predation_mg: u64,
    /// All accepted flows into the subject, including forms not treated as food.
    pub subject_incoming_mg: u64,
    pub subject_reserve_before_mg: u64,
    pub subject_reserve_after_mg: u64,
    pub subject_alive: bool,
    pub births: u32,
    pub deaths: u32,
    pub subject_death_recorded: bool,
}

impl TrialEvidence {
    pub(super) fn begin(world: &World, subject: OrganismId, ticks: u32) -> Self {
        let reserve = reserve(world, subject);
        Self {
            controller: TrialController {
                idle_ticks: ticks,
                held_ticks: 0,
                instinct_ticks: 0,
                inactive_ticks: 0,
            },
            subject_feeding_events: 0,
            subject_feeding_mg: 0,
            subject_uptake_mg: 0,
            subject_predation_mg: 0,
            subject_incoming_mg: 0,
            subject_reserve_before_mg: reserve,
            subject_reserve_after_mg: reserve,
            subject_alive: world.controlled_id() == Some(subject),
            births: 0,
            deaths: 0,
            subject_death_recorded: false,
        }
    }

    pub(super) fn record_controller(&mut self, world: &World, subject: OrganismId) {
        if !is_alive(world, subject) {
            self.controller.inactive_ticks += 1;
        } else if world.idle_run().saturating_add(1) < crate::world::INSTINCT_IDLE_TICKS {
            self.controller.held_ticks += 1;
        } else {
            self.controller.instinct_ticks += 1;
        }
    }

    pub(super) fn record_tick(
        &mut self,
        world: &mut World,
        subject: OrganismId,
        living: &mut BTreeSet<OrganismId>,
    ) {
        for flow in world.drain_flows() {
            self.record_flow(flow, subject);
        }
        for event in world.drain_events() {
            self.record_event(event.record, subject, living);
        }
    }

    pub(super) fn finish(&mut self, world: &World, subject: OrganismId) {
        self.subject_reserve_after_mg = reserve(world, subject);
        self.subject_alive = is_alive(world, subject);
    }

    fn record_flow(&mut self, flow: RecordedFlow, subject: OrganismId) {
        let record = flow.record;
        if record.is_internal() {
            return;
        }
        if record.to.is_some_and(|to| to.organism == subject) {
            self.subject_incoming_mg = self.subject_incoming_mg.saturating_add(record.amount_mg);
            match record.process {
                Process::Feeding => {
                    self.subject_feeding_events += 1;
                    self.subject_feeding_mg =
                        self.subject_feeding_mg.saturating_add(record.amount_mg);
                },
                Process::Uptake => {
                    self.subject_uptake_mg =
                        self.subject_uptake_mg.saturating_add(record.amount_mg);
                },
                _ => {},
            }
        }
    }

    fn record_event(
        &mut self,
        event: Event,
        subject: OrganismId,
        living: &mut BTreeSet<OrganismId>,
    ) {
        match event {
            Event::Born { organism, .. } => {
                self.births += 1;
                living.insert(organism);
            },
            Event::Fed {
                from,
                kind: MealKind::Predation,
                mass_mg,
                ..
            } if from == subject => {
                self.subject_predation_mg = self.subject_predation_mg.saturating_add(mass_mg);
            },
            Event::Died { organism, .. } | Event::Returned { organism }
                if living.remove(&organism) =>
            {
                self.deaths += 1;
                self.subject_death_recorded |= organism == subject;
            },
            _ => {},
        }
    }

    /// Short comparison-row text based only on this trial's recorded facts.
    pub fn summary(&self) -> String {
        let phase = format!(
            "idle {}t/{} instinct",
            self.controller.idle_ticks, self.controller.instinct_ticks
        );
        let intake = if self.subject_feeding_mg > 0 && self.subject_uptake_mg > 0 {
            "fed+uptake"
        } else if self.subject_feeding_mg > 0 {
            "fed"
        } else if self.subject_uptake_mg > 0 {
            "uptake"
        } else {
            "no intake"
        };
        let reserve = if self.subject_reserve_after_mg >= self.subject_reserve_before_mg {
            format!(
                "reserve +{} mg",
                self.subject_reserve_after_mg - self.subject_reserve_before_mg
            )
        } else {
            format!(
                "reserve -{} mg",
                self.subject_reserve_before_mg - self.subject_reserve_after_mg
            )
        };
        let status = if self.subject_alive { "alive" } else { "died" };
        let predation = (self.subject_predation_mg > 0)
            .then(|| format!("; predated {} mg", self.subject_predation_mg))
            .unwrap_or_default();
        format!(
            "{phase}; {status}, {intake}, {reserve}; births {}, deaths {}{predation}",
            self.births, self.deaths,
        )
    }
}

fn reserve(world: &World, subject: OrganismId) -> u64 {
    world
        .organisms
        .iter()
        .find(|organism| organism.id == subject)
        .map_or(0, |organism| organism.energy_mg)
}

fn is_alive(world: &World, subject: OrganismId) -> bool {
    world
        .organisms
        .iter()
        .any(|organism| organism.id == subject && organism.is_alive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpeciesId;
    use crate::flow::{Account, Envelope, FlowEvent, Subject};
    use crate::organism::{Kingdom, OrganismId};
    use std::collections::BTreeSet;

    fn subject() -> Subject {
        Subject {
            organism: OrganismId(7),
            lineage: SpeciesId(3),
            kingdom: Kingdom::Consumer,
        }
    }

    #[test]
    fn feeding_and_uptake_are_distinct_recorded_intake() {
        let mut evidence = TrialEvidence::begin(&World::new(1, 3), OrganismId(7), 1);
        let eater = subject();
        evidence.record_flow(
            Envelope::new(
                1,
                None,
                FlowEvent::between(
                    Process::Feeding,
                    Subject {
                        organism: OrganismId(8),
                        ..subject()
                    },
                    Account::Substance,
                    eater,
                    Account::Reserve,
                    9,
                ),
            ),
            eater.organism,
        );
        evidence.record_flow(
            Envelope::new(1, None, FlowEvent::uptake(eater, Account::Reserve, 4)),
            eater.organism,
        );
        evidence.record_flow(
            Envelope::new(
                1,
                None,
                FlowEvent::between(
                    Process::Uptake,
                    eater,
                    Account::Substance,
                    eater,
                    Account::Reserve,
                    4,
                ),
            ),
            eater.organism,
        );
        assert_eq!(evidence.subject_feeding_events, 1);
        assert_eq!(evidence.subject_feeding_mg, 9);
        assert_eq!(evidence.subject_uptake_mg, 4);
        assert_eq!(evidence.subject_incoming_mg, 13);
    }

    #[test]
    fn no_intake_summary_does_not_claim_a_cause() {
        let evidence = TrialEvidence {
            controller: TrialController {
                idle_ticks: 32,
                held_ticks: 29,
                instinct_ticks: 3,
                inactive_ticks: 0,
            },
            subject_feeding_events: 0,
            subject_feeding_mg: 0,
            subject_uptake_mg: 0,
            subject_predation_mg: 0,
            subject_incoming_mg: 0,
            subject_reserve_before_mg: 800,
            subject_reserve_after_mg: 770,
            subject_alive: true,
            births: 1,
            deaths: 2,
            subject_death_recorded: false,
        };
        let summary = evidence.summary();
        assert!(summary.contains("no intake"));
        assert!(summary.contains("reserve -30 mg"));
        assert!(summary.len() <= 100);
    }

    #[test]
    fn died_and_returned_consume_one_living_id_once() {
        let subject_id = OrganismId(7);
        let corpse = OrganismId(8);
        let mut evidence = TrialEvidence::begin(&World::new(1, 3), subject_id, 1);
        let mut living = BTreeSet::from([subject_id]);
        evidence.record_event(
            Event::Died {
                organism: subject_id,
                species: SpeciesId(3),
            },
            subject_id,
            &mut living,
        );
        evidence.record_event(
            Event::Returned {
                organism: subject_id,
            },
            subject_id,
            &mut living,
        );
        evidence.record_event(
            Event::Returned { organism: corpse },
            subject_id,
            &mut living,
        );
        assert_eq!(evidence.deaths, 1);
        assert!(evidence.subject_death_recorded);
    }
}

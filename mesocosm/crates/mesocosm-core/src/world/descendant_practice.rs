// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Authored, assisted proof of somatic versus filial expression.

use crate::development::{DevelopmentError, PartPalette};
use crate::discovery::ConditionId;
use crate::history::Event;
use crate::organism::{OrganismId, Stage};
use crate::program::RevisionId;
use crate::rules::{EpochRule, WorldRules};

use super::{Founding, Intent, Outcome, World};

/// A named body snapshot suitable for an inspector or renderer.
#[derive(Clone, Debug)]
pub struct DescendantStage {
    pub label: &'static str,
    pub world: World,
    pub subject: OrganismId,
}

/// Worlds and receipts for the bounded VB4a inheritance proof.
#[derive(Clone, Debug)]
pub struct DescendantPractice {
    pub stages: Vec<DescendantStage>,
    pub parent: OrganismId,
    pub control_child: OrganismId,
    pub condition: ConditionId,
    pub revision: RevisionId,
    pub assisted_setup: bool,
    pub expression: Outcome,
    pub before_revision_birth: Outcome,
    pub revision_commit: Outcome,
    pub after_revision_birth: Outcome,
    pub control_birth: Outcome,
    pub intervention_events: Vec<Event>,
    pub control_events: Vec<Event>,
}

fn plate_recipe() -> crate::Recipe {
    crate::Recipe::of(vec![crate::Tagma::new(1, crate::Appendage::Plate)])
}

fn hunger() -> ConditionId {
    crate::discovery::conditions()
        .into_iter()
        .find(|condition| condition.name == "mesocosm:endured-hunger")
        .expect("the native hunger condition is admitted")
        .id()
}

fn make_ready(world: &mut World, parent: OrganismId) {
    let organism = world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == parent)
        .expect("proof parent exists");
    organism.stage = Stage::Mature;
    organism.since_offspring = u32::MAX;
    organism.energy_mg = 100_000;
}

fn events(world: &mut World) -> Vec<Event> {
    world
        .drain_events()
        .into_iter()
        .map(|record| record.record)
        .collect()
}

fn offspring(outcome: Outcome) -> OrganismId {
    match outcome {
        Outcome::Bore { offspring, .. } => offspring,
        other => panic!("proof birth refused: {other:?}"),
    }
}

impl World {
    /// Builds the bounded VB4a proof from the existing expression practice.
    ///
    /// Expression and revision use the ordinary world intents. `ForceBirth`
    /// is an explicit developer assisted entry to the same reproduction
    /// transaction, used to make this inspection fixture deterministic.
    pub fn descendant_practice(
        seed: u64,
        founding: Founding,
        palette: PartPalette,
    ) -> Result<DescendantPractice, DevelopmentError> {
        let base = Self::expression_practice(seed, founding, palette)?;
        let parent = base.controlled_id().expect("practice body is controlled");
        let condition = hunger();
        let rules = WorldRules::native()
            .ending(EpochRule::Timed { ticks: 1 })
            .scoring_over(2);
        let species = base.controlled().expect("controlled").species;
        let mut base = base.with_rules(rules);
        // The plate is an authored starting shape shared by both
        // counterfactual branches. Only the intervention commits a lineage
        // program; the control remains at the founding revision.
        base.lineages_mut().set_recipe(species, plate_recipe());
        let mut intervention = base.clone();
        let mut relative = intervention.clone();
        let mut intervention_events = Vec::new();
        let mut control_events = Vec::new();
        let initial_parent = intervention.clone();
        let initial_control = relative.clone();

        let expression = intervention.apply(Intent::Express { condition });
        intervention_events.extend(events(&mut intervention));
        let expressed_parent = intervention.clone();

        make_ready(&mut intervention, parent);
        let _ = intervention.apply(Intent::Idle);
        intervention_events.extend(events(&mut intervention));
        let before_revision_birth = intervention.apply(Intent::ForceBirth { organism: parent });
        intervention_events.extend(events(&mut intervention));
        let before_child = offspring(before_revision_birth);
        let before_revision_child = intervention.clone();

        let revision_commit = intervention.apply(Intent::Revise { condition });
        intervention_events.extend(events(&mut intervention));
        let revision = match revision_commit {
            Outcome::Revised { revision, .. } => revision,
            other => panic!("proof revision refused: {other:?}"),
        };

        make_ready(&mut intervention, parent);
        let after_revision_birth = intervention.apply(Intent::ForceBirth { organism: parent });
        intervention_events.extend(events(&mut intervention));
        let after_child = offspring(after_revision_birth);
        let after_revision_child = intervention.clone();

        // Equal elapsed ecology, with no intervention on the control.
        make_ready(&mut relative, parent);
        let _ = relative.apply(Intent::Idle);
        control_events.extend(events(&mut relative));
        let unchanged_relative_birth = relative.apply(Intent::ForceBirth { organism: parent });
        control_events.extend(events(&mut relative));
        while relative.tick < after_revision_child.tick {
            let _ = relative.apply(Intent::Idle);
            control_events.extend(events(&mut relative));
        }
        let relative_child = offspring(unchanged_relative_birth);

        Ok(DescendantPractice {
            stages: vec![
                DescendantStage {
                    label: "initial parent",
                    world: initial_parent,
                    subject: parent,
                },
                DescendantStage {
                    label: "initial control parent",
                    world: initial_control,
                    subject: parent,
                },
                DescendantStage {
                    label: "expressed parent",
                    world: expressed_parent,
                    subject: parent,
                },
                DescendantStage {
                    label: "before revision child",
                    world: before_revision_child,
                    subject: before_child,
                },
                DescendantStage {
                    label: "after revision child",
                    world: after_revision_child,
                    subject: after_child,
                },
                DescendantStage {
                    label: "unchanged control child",
                    world: relative,
                    subject: relative_child,
                },
            ],
            parent,
            control_child: relative_child,
            condition,
            revision,
            assisted_setup: true,
            expression,
            before_revision_birth,
            revision_commit,
            after_revision_birth,
            control_birth: unchanged_relative_birth,
            intervention_events,
            control_events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{Process, Registry};

    #[test]
    fn proof_separates_somatic_acquisition_from_filial_expression() {
        let proof = World::descendant_practice(7, Founding::Roster, PartPalette::primitive())
            .expect("authored proof fixture");
        assert!(proof.assisted_setup);
        assert!(matches!(proof.expression, Outcome::Expressed { .. }));
        assert!(matches!(proof.revision_commit, Outcome::Revised { .. }));
        assert!(matches!(proof.before_revision_birth, Outcome::Bore { .. }));
        assert!(matches!(proof.after_revision_birth, Outcome::Bore { .. }));
        assert!(matches!(proof.control_birth, Outcome::Bore { .. }));

        let gland = Registry::native().of_native(Process::Secrete).reference();
        fn body(stage: &DescendantStage) -> &crate::Organism {
            stage
                .world
                .living()
                .find(|organism| organism.id == stage.subject)
                .expect("stage subject")
        }
        assert_eq!(
            body(&proof.stages[0]).phenotype.digest(),
            body(&proof.stages[1]).phenotype.digest(),
            "the counterfactual parents start identically"
        );
        assert!(body(&proof.stages[2]).phenotype.expresses(gland));
        assert!(!body(&proof.stages[3]).phenotype.expresses(gland));
        assert!(body(&proof.stages[4]).phenotype.expresses(gland));
        assert!(!body(&proof.stages[5]).phenotype.expresses(gland));

        let species = body(&proof.stages[2]).species;
        let founding_digest = proof.stages[0]
            .world
            .lineages()
            .get(species)
            .expect("lineage")
            .program()
            .digest();
        assert_eq!(
            proof.stages[2]
                .world
                .lineages()
                .get(species)
                .expect("lineage")
                .program()
                .digest(),
            founding_digest,
            "somatic expression leaves the lineage program unchanged"
        );
        assert_eq!(
            proof.stages[4].world.tick, proof.stages[5].world.tick,
            "the counterfactual control sees the same elapsed ecology"
        );
        assert_ne!(proof.stages[3].subject, proof.stages[4].subject);
        assert_ne!(proof.stages[4].subject, proof.stages[5].subject);
        let repeated = World::descendant_practice(7, Founding::Roster, PartPalette::primitive())
            .expect("repeat fixture");
        assert_eq!(proof.revision, repeated.revision);
        assert_eq!(proof.intervention_events, repeated.intervention_events);
        assert_eq!(proof.control_events, repeated.control_events);
        assert_eq!(proof.stages.len(), repeated.stages.len());
        for (left, right) in proof.stages.iter().zip(&repeated.stages) {
            assert_eq!(
                crate::snapshot::state_hash(&left.world),
                crate::snapshot::state_hash(&right.world)
            );
            assert_eq!(left.subject, right.subject);
        }
        assert!(proof.intervention_events.iter().any(|event| matches!(
            event,
            Event::Expressed { organism, .. } if *organism == proof.parent
        )));
        assert!(proof.intervention_events.iter().any(|event| matches!(
            event,
            Event::Inherited { organism, revision, .. }
                if *organism == proof.stages[4].subject && *revision == proof.revision
        )));
    }
}

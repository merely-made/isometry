// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A compact enclosure where family and acquired tissue can be inspected together.

use crate::body::{Attachment, PartId, Provenance, Yaw};
use crate::development::{DevelopmentError, PartPalette};
use crate::discovery::{ConditionId, conditions};
use crate::graft::Crossing;
use crate::organism::{OrganismId, Stage};
use crate::rules::{EpochRule, WorldRules};

use super::{Founding, Intent, World};

const BRANCH_MG: u64 = 80;
const TIP_MG: u64 = 40;

/// The resolved handles for the family-practice origin.
///
/// These are read from the realized bodies rather than guessed from allocation
/// order. The birth records remain pending at origin so a runtime can enter
/// them into its `History` before it begins stepping the enclosure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FamilyPracticeIds {
    pub parent: OrganismId,
    pub relative: OrganismId,
    pub donor: OrganismId,
    pub graft_part: PartId,
    pub intake_part: PartId,
    pub condition: ConditionId,
}

fn hunger() -> ConditionId {
    conditions()
        .into_iter()
        .find(|condition| condition.name == "mesocosm:endured-hunger")
        .expect("the native hunger condition is admitted")
        .id()
}

fn plate_recipe() -> crate::Recipe {
    crate::Recipe::of(vec![crate::Tagma::new(1, crate::Appendage::Plate)])
}

impl World {
    /// Founds a bounded same-enclosure family practice origin.
    ///
    /// Setup is explicitly assisted: the parent bears two ordinary recorded
    /// births, then one child is made into nearby carrion carrying a branch.
    /// The other child remains a living, untreated relative in the same world.
    /// After origin, every visible action is an ordinary intent. In particular,
    /// `EndEpoch` opens the normal `Revise` boundary under `PlayerTriggered`.
    pub fn family_practice(
        seed: u64,
        founding: Founding,
        palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let mut world = Self::expression_practice(seed, founding, palette)?
            .with_rules(WorldRules::native().ending(EpochRule::PlayerTriggered));
        let parent = world.controlled_id().expect("practice parent is embodied");
        let condition = hunger();
        let species = world.controlled().expect("practice parent exists").species;
        // The origin's children are ordinary births, so their affordable
        // founding recipe has to hold the plate their parent already carries.
        // The later revision is still the only lineage program that adds the
        // discovered gland.
        world.lineages_mut().set_recipe(species, plate_recipe());

        {
            let parent_body = world
                .organisms
                .iter_mut()
                .find(|organism| organism.id == parent)
                .expect("practice parent exists");
            parent_body.stage = Stage::Mature;
            parent_body.since_offspring = u32::MAX;
            parent_body.energy_mg = 100_000;
        }

        let donor = match world.apply(Intent::ForceBirth { organism: parent }) {
            super::Outcome::Bore { offspring, .. } => offspring,
            outcome => panic!("family donor birth refused: {outcome:?}"),
        };
        let relative = match world.apply(Intent::ForceBirth { organism: parent }) {
            super::Outcome::Bore { offspring, .. } => offspring,
            outcome => panic!("family relative birth refused: {outcome:?}"),
        };

        let parent_at = world.position().expect("practice parent remains embodied");
        let donor_index = world
            .organisms
            .iter()
            .position(|organism| organism.id == donor)
            .expect("born donor joins the roster");
        let donor_at = [parent_at[0] + 1, parent_at[1], parent_at[2]];
        let branch = palette.sensor.default;
        {
            let donor_body = &mut world.organisms[donor_index];
            donor_body.position = donor_at;
            donor_body.energy_mg = 0;
            assert_eq!(donor_body.phenotype.spend_mass(BRANCH_MG + TIP_MG), 0);
            let root = donor_body.body().root;
            let graft_part = donor_body
                .phenotype
                .attach(
                    branch.volume,
                    BRANCH_MG,
                    branch.half_extent,
                    Attachment {
                        parent: root,
                        offset: [0, branch.half_extent[1] + 1, 0],
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )
                .expect("family donor branch attaches");
            donor_body
                .phenotype
                .attach(
                    branch.volume,
                    TIP_MG,
                    branch.half_extent,
                    Attachment {
                        parent: graft_part,
                        offset: [branch.half_extent[0] * 2, 0, 0],
                        yaw: Yaw::Quarter,
                    },
                    Provenance::founding(),
                )
                .expect("family donor branch has an intake tip");
            let shape = donor_body.walker_shape();
            world
                .ground
                .author_walker_stance(donor_at, shape.radius(), shape.height(), [0, 0]);
        }
        assert!(matches!(
            world.apply(Intent::Kill { organism: donor }),
            super::Outcome::Killed { .. }
        ));

        // The practice parent is a mixotroph: its plate does not by itself
        // admit carrion. Author a supported intake port through the same
        // declaration seam as founding, instead of bypassing meal admission.
        let intake = crate::process::Registry::native()
            .of_native(crate::process::Process::Intake)
            .reference();
        let parent_body = world.controlled_mut().expect("practice parent lives");
        let intake_part = parent_body.body().root;
        assert!(parent_body.phenotype.declare_port(
            intake_part,
            crate::process::IntakePort::deadstock().supported_by(intake),
        ));
        assert!(parent_body.phenotype.intake_ports().admits_deadstock());

        // Keep the origin's two Born records and ordinary death available to a
        // runtime history. `family_practice_ids` resolves the actual handles
        // without relying on their numeric allocation values.
        let ids = world
            .family_practice_ids()
            .expect("authored family origin remains recognizable");
        assert_eq!(ids.parent, parent);
        assert_eq!(ids.relative, relative);
        assert_eq!(ids.donor, donor);
        assert_eq!(ids.condition, condition);
        Ok(world)
    }

    /// Resolves the actions offered by an untouched family-practice origin.
    pub fn family_practice_ids(&self) -> Option<FamilyPracticeIds> {
        let parent = self.controlled_id()?;
        let species = self.controlled()?.species;
        let donor = self
            .organisms
            .iter()
            .find(|organism| !organism.is_alive() && organism.species == species)?;
        let (source, graft_part) = self
            .graft_sources()
            .into_iter()
            .find(|(id, part)| *id == donor.id && donor.body().children(*part).next().is_some())?;
        let intake_part = donor
            .body()
            .parts
            .iter()
            .find(|part| {
                part.attachment
                    .is_some_and(|attachment| attachment.parent == graft_part)
            })?
            .id;
        let relative = self
            .living()
            .filter(|organism| organism.id != parent && organism.species == species)
            .min_by_key(|organism| organism.id)?
            .id;
        Some(FamilyPracticeIds {
            parent,
            relative,
            donor: source,
            graft_part,
            intake_part,
            condition: hunger(),
        })
    }

    /// The ordinary opening actions, resolved against this origin's real ids.
    pub fn family_practice_opening(&self) -> Option<Vec<Intent>> {
        let ids = self.family_practice_ids()?;
        Some(vec![
            Intent::Consume {
                organism: ids.donor,
                part: ids.intake_part,
            },
            Intent::Graft {
                organism: ids.donor,
                part: ids.graft_part,
                crossing: Crossing::Carry,
            },
            Intent::Express {
                condition: ids.condition,
            },
            Intent::EndEpoch,
            Intent::Revise {
                condition: ids.condition,
            },
        ])
    }

    /// The bounded ordinary wait for the parent's next ecological birth.
    pub fn family_practice_birth_run(&self) -> std::iter::RepeatN<Intent> {
        let ticks = self.controlled().map_or(0, |parent| {
            crate::organism::ecology::gestation_for_mass(parent.life_history_mass_mg())
                .saturating_sub(parent.since_offspring)
                .saturating_add(1)
        });
        std::iter::repeat_n(Intent::Resume, ticks as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Founding, Outcome, PartPalette, snapshot::state_hash};

    #[test]
    fn family_origin_keeps_recorded_kin_and_ordinary_opening() {
        verify_family(Founding::Roster, PartPalette::primitive());
    }

    #[test]
    fn spaced_family_reaches_the_same_biological_join() {
        verify_family(Founding::SpacedRoster, Founding::SpacedRoster.palette());
    }

    fn verify_family(founding: Founding, palette: PartPalette) {
        let mut world = World::family_practice(7, founding, palette).expect("family fixture");
        let ids = world.family_practice_ids().expect("resolved family ids");
        assert!(world.events().iter().any(|event| matches!(
            event.record,
            crate::history::Event::Born { organism, parent: Some(parent), .. }
                if organism == ids.donor && parent == ids.parent
        )));
        assert!(world.events().iter().any(|event| matches!(
            event.record,
            crate::history::Event::Born { organism, parent: Some(parent), .. }
                if organism == ids.relative && parent == ids.parent
        )));
        let opening = world.family_practice_opening().expect("opening sequence");
        let intake = world.apply(opening[0].clone());
        assert!(matches!(intake, Outcome::Consumed { .. }), "{intake:?}");
        let graft = world.apply(opening[1].clone());
        assert!(matches!(graft, Outcome::Grafted { .. }), "{graft:?}");
        let expression = world.apply(opening[2].clone());
        assert!(
            matches!(expression, Outcome::Expressed { .. }),
            "{expression:?}"
        );
        let boundary = world.apply(opening[3].clone());
        assert!(
            matches!(boundary, Outcome::EpochEnded { .. }),
            "{boundary:?}"
        );
        assert!(world.revision_admitted_now());
        let revision = world.apply(opening[4].clone());
        assert!(matches!(revision, Outcome::Revised { .. }), "{revision:?}");
        for intent in world.family_practice_birth_run() {
            let _ = world.apply(intent);
            if world.events().iter().any(|event| {
                matches!(
                    event.record,
                    crate::history::Event::Born { organism, parent: Some(parent), .. }
                        if parent == ids.parent && organism != ids.donor && organism != ids.relative
                )
            }) {
                break;
            }
        }
        assert!(
            world.events().iter().any(|event| matches!(
                event.record,
                crate::history::Event::Born { organism, parent: Some(parent), .. }
                    if parent == ids.parent && organism != ids.donor && organism != ids.relative
            )),
            "parent did not breed: {:?}",
            world
                .organisms
                .iter()
                .map(|o| (
                    o.id,
                    o.stage,
                    o.age,
                    o.since_offspring,
                    o.biomass_mg(),
                    o.mass_ceiling_mg(),
                    o.energy_mg,
                    o.can_reproduce()
                ))
                .collect::<Vec<_>>()
        );
        assert!(world.living().any(|organism| organism.id == ids.relative));
        assert!(!world.events().iter().any(|event| matches!(
            event.record,
            crate::history::Event::Expressed { organism, .. }
                | crate::history::Event::Grafted { organism, .. }
                if organism == ids.relative
        )));
    }

    #[test]
    fn refused_opening_action_does_not_mutate_the_same_state() {
        let mut world = World::family_practice(7, Founding::Roster, PartPalette::primitive())
            .expect("family fixture");
        let mut unchanged = world.clone();
        let ids = world.family_practice_ids().expect("resolved family ids");
        let refusal = world.apply(Intent::Consume {
            organism: ids.donor,
            part: PartId(999),
        });
        assert!(matches!(refusal, Outcome::Rejected(_)), "{refusal:?}");
        let _ = unchanged.apply(Intent::Resume);
        assert_eq!(
            state_hash(&world),
            state_hash(&unchanged),
            "the refusal adds no body or world mutation to the ordinary elapsed tick"
        );
        assert_eq!(world.flows(), unchanged.flows());
    }
}

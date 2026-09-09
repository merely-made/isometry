//! The authored single-critter fixture for the ordinary expression door.
//!
//! This is a practice starting point, not a natural history claim. Its
//! prehistory uses the ordinary world transition path to record hunger and
//! discover the gland candidate. The plate is attached afterwards so the
//! fixture can show the candidate becoming expressible without letting a
//! canopy's income erase the authored stress.

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::body::{Attachment, Provenance, Yaw};
use crate::development::{DevelopmentError, PartPalette};
use crate::discovery::{HUNGER_TICKS, conditions};
use crate::organism::{Kingdom, Organism};
use crate::plan::{Role, classify};

use super::{Founding, Intent, STARVED_UPKEEP_TICKS, World};

const PRACTICE_MASS_MG: u64 = 1_000;
const PRACTICE_RESERVE_MG: u64 = 10_000;

fn hunger() -> crate::discovery::ConditionId {
    conditions()
        .into_iter()
        .find(|condition| condition.name == "mesocosm:endured-hunger")
        .expect("the native hunger condition is admitted")
        .id()
}

impl World {
    /// Founds a single-critter authored practice scene ready for `Express`.
    ///
    /// The controlled body first endures the starvation horizon through
    /// repeated `Resume` intents. Each tick tops its reserve just below the
    /// starvation line so the body survives while the real `World::apply`
    /// path records the crossing. Only after that authored prehistory does
    /// this fixture attach the caller's admitted plate, leaving the body with
    /// a legal site for the discovered gland.
    pub fn expression_practice(
        seed: u64,
        founding: Founding,
        palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let mut world = Self::terrarium(seed, founding, palette)?;
        let controlled = world.controlled_id().expect("the terrarium is embodied");
        let index = world
            .organisms
            .iter()
            .position(|organism| organism.id == controlled)
            .expect("the controlled organism exists");
        let at = world.organisms[index].position;
        let species = world.organisms[index].species;
        let bulk = palette.mass.default;

        // Keep this practice fixture's prehistory independent of the fauna
        // roster. The compact body and the clearing use the same ordinary
        // constructors and footing checks as graft-practice.
        let compact = Organism::founding(
            controlled,
            species,
            Kingdom::Consumer,
            bulk.volume,
            bulk.half_extent,
            at,
            PRACTICE_MASS_MG,
        )
        .matured();
        world.ground.author_walker_stance(
            at,
            compact.walker_shape().radius(),
            compact.walker_shape().height(),
            [0, 0],
        );
        assert!(compact.walker_shape().stands(&world.ground, at));
        world.organisms.clear();
        world.organisms.push(compact);
        world.controlled = Some(controlled);
        world.pending.clear();

        // Explicit authored prehistory. Resume keeps the hand on the body;
        // unlike Idle it cannot hand control to instincts midway through the
        // hunger horizon. This is still ordinary observation, not injection.
        for _ in 0..=HUNGER_TICKS {
            let upkeep = world
                .controlled()
                .expect("practice body survives")
                .upkeep_mg();
            world
                .organisms
                .first_mut()
                .expect("the single practice body exists")
                .energy_mg = upkeep * (STARVED_UPKEEP_TICKS - 1);
            let _ = world.apply(Intent::Resume);
        }
        assert!(
            world.discovered(hunger()),
            "authored prehistory discovers hunger"
        );

        // The body earns its expression site only after discovery. Pay for the
        // authored plate out of its existing body mass, as graft-practice does
        // for its donor branch, and then re-author the final footing.
        let plate = palette.plate.default;
        let plate_mg = 300;
        {
            let organism = world.organisms.first_mut().expect("single body");
            assert_eq!(organism.phenotype.spend_mass(plate_mg), 0);
            let root = organism.body().root;
            let plate_id = organism
                .phenotype
                .attach(
                    plate.volume,
                    plate_mg,
                    plate.half_extent,
                    Attachment {
                        parent: root,
                        offset: [0, bulk.half_extent[1].abs() + plate.half_extent[1], 0],
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )
                .expect("the admitted plate attaches to the compact root");
            assert_eq!(classify(plate.half_extent), Role::Plate);
            assert!(organism.body().part(plate_id).is_some());
            organism.energy_mg = PRACTICE_RESERVE_MG;
        }
        let shape = world.controlled().expect("single body").walker_shape();
        world
            .ground
            .author_walker_stance(at, shape.radius(), shape.height(), [0, 0]);
        assert!(
            shape.stands(&world.ground, at),
            "the final body has footing"
        );
        world.frontier = world
            .controlled()
            .map(|organism| world.intricacy(organism))
            .unwrap_or(0);
        // The named scene reconstructs prehistory; it must not arrive as new
        // activity in the first player tick. Discovery evidence stays in world.
        world.pending.clear();
        let _ = world.drain_flows();
        let _ = world.ground.drain_dirty();
        Ok(world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arrangement, Outcome};

    fn fixture(seed: u64) -> World {
        World::expression_practice(seed, Founding::Roster, PartPalette::primitive())
            .expect("practice fixture")
    }

    #[test]
    fn seeds_start_with_grounded_critters_and_a_real_discovery() {
        for seed in [7, 41] {
            let world = fixture(seed);
            assert_eq!(world.living().count(), 1);
            assert!(world.living().all(|organism| {
                organism
                    .walker_shape()
                    .stands(&world.ground, organism.position)
            }));
            assert!(world.discovered(hunger()));
            assert!(world.flows().is_empty());
            assert!(
                world
                    .candidate_proposal(hunger(), Arrangement::Direct)
                    .is_some()
            );
        }
    }

    #[test]
    fn expression_is_accepted_and_is_somatic() {
        let mut world = fixture(7);
        let species = world.controlled().expect("embodied").species;
        let recipe = world
            .lineages()
            .get(species)
            .expect("lineage")
            .recipe
            .clone();
        let program = world
            .lineages()
            .get(species)
            .expect("lineage")
            .program()
            .digest();
        let before = world.energy_mg().expect("reserve");
        let outcome = world.apply(Intent::Express {
            condition: hunger(),
        });
        let cost = match outcome {
            Outcome::Expressed { cost_mg, .. } => cost_mg,
            other => panic!("expression refused: {other:?}"),
        };
        assert!(cost > 0);
        assert!(world.energy_mg().expect("reserve") <= before - cost);
        let line = world.lineages().get(species).expect("lineage");
        assert_eq!(line.recipe, recipe);
        assert_eq!(line.program().digest(), program);
    }
}

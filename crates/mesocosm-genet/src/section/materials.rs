// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Surface appearance reads admitted, living process tissue. Allocation cells
//! remain simulation facts; their proportions drive a disposable voxel pattern.

use mesocosm_core::{BodyPhenotype, process::Registry};
use mesocosm_render::PartMaterial;
use std::collections::BTreeMap;

#[cfg(test)]
mod capture;

pub(super) fn project(phenotype: &BodyPhenotype, registry: &Registry) -> Vec<PartMaterial> {
    let mut materials = Vec::new();
    for part in phenotype.body().living() {
        let Some(mosaic) = phenotype.mosaic(part.id) else {
            continue;
        };
        let capacity = mosaic.capacity();
        if capacity == 0 {
            continue;
        }
        let mut counts = BTreeMap::new();
        for site in mosaic.sites() {
            let Some(process) = registry
                .resolve(site.process)
                .and_then(|definition| definition.native)
            else {
                continue;
            };
            let living = site
                .cells
                .iter()
                .filter(|cell| mosaic.is_living(**cell))
                .count() as u32;
            *counts.entry(process).or_insert(0u32) += living;
        }
        materials.extend(counts.into_iter().filter(|(_, cells)| *cells > 0).map(
            |(process, cells)| PartMaterial {
                part: part.id,
                process,
                fraction: cells as f32 / capacity as f32,
            },
        ));
    }
    materials
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::{Founding, World, process::Process};

    #[test]
    fn preview_materials_read_allocated_tissue_without_changing_authority_or_geometry() {
        let founding = Founding::SpacedRoster;
        let world = World::expression_practice(7, founding, founding.palette()).unwrap();
        let hash = mesocosm_core::state_hash(&world);
        let condition = world.discoveries()[0].condition;
        let preview = world.preview_expression(condition).unwrap();
        let actual = &world.controlled().unwrap().phenotype;
        let before = project(actual, world.ruleset());
        let after = project(&preview.phenotype, world.ruleset());
        assert!(!before.iter().any(|m| m.process == Process::Secrete));
        let material = after
            .iter()
            .find(|m| m.process == Process::Secrete)
            .unwrap();
        assert_eq!(material.part, preview.part);
        assert!(material.fraction > 0.0 && material.fraction < 1.0);
        assert_eq!(actual.body(), preview.phenotype.body());
        assert_eq!(mesocosm_core::state_hash(&world), hash);
        assert_eq!(after, project(&preview.phenotype, world.ruleset()));
        let empty_registry = Registry::admit(Vec::new()).unwrap();
        assert!(project(&preview.phenotype, &empty_registry).is_empty());
        let mut severed = preview.phenotype.clone();
        severed.sever(preview.part);
        assert!(
            !project(&severed, world.ruleset())
                .iter()
                .any(|m| m.part == preview.part)
        );
    }
}

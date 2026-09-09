// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Face-connected appendage chains in the ordinary developmental graph.
//!
//! A chain is static anatomy: it gives a leaf a stalk or a leg articulated
//! links. It does not claim animation, inverse kinematics, or per-voxel life.

use crate::axis::{AppendageStep, ChainFacing, Recipe, Tagma};
use crate::body::{Attachment, BodyDocument, PartId, Provenance, Yaw};
use crate::development::{DevelopmentError, PartPalette};
use crate::plan::Role;

const MAX_STEPS: usize = 8;

pub(super) fn validate(recipe: &Recipe) -> Result<(), DevelopmentError> {
    if recipe.appendage_chains.is_empty() {
        return Ok(());
    }
    if recipe.appendage_chains.len() != recipe.tagmata.len() {
        return Err(DevelopmentError::AppendageChainLength {
            tagmata: recipe.tagmata.len(),
            chains: recipe.appendage_chains.len(),
        });
    }
    for (tagma, chain) in recipe.appendage_chains.iter().enumerate() {
        validate_one(tagma, &recipe.tagmata[tagma], chain)?;
    }
    Ok(())
}

fn validate_one(
    tagma_index: usize,
    tagma: &Tagma,
    chain: &[AppendageStep],
) -> Result<(), DevelopmentError> {
    if chain.is_empty() {
        return Ok(());
    }
    if chain.len() > MAX_STEPS {
        return Err(DevelopmentError::AppendageChainTooLong {
            tagma: tagma_index,
            steps: chain.len(),
        });
    }
    if chain[0].distal {
        return Err(DevelopmentError::AppendageChainRootDistal { tagma: tagma_index });
    }
    let Some(endpoint_role) = tagma.appendage.role(tagma.appendage_shape) else {
        return Err(DevelopmentError::AppendageChainTerminal { tagma: tagma_index });
    };
    let endpoint_shape = tagma.appendage.shape_index(tagma.appendage_shape);
    let terminal = chain.last().expect("nonempty checked above");
    if terminal.role != endpoint_role || terminal.shape != endpoint_shape {
        return Err(DevelopmentError::AppendageChainTerminal { tagma: tagma_index });
    }
    for (step, link) in chain[..chain.len() - 1].iter().enumerate() {
        if link.role != Role::Mass && link.role != endpoint_role {
            return Err(DevelopmentError::AppendageChainIntermediate {
                tagma: tagma_index,
                step,
            });
        }
    }
    Ok(())
}

pub(super) fn attach(
    body: &mut BodyDocument,
    segment: PartId,
    tagma: &Tagma,
    chain: &[AppendageStep],
    mass_mg: u64,
    palette: PartPalette,
) -> Result<(), DevelopmentError> {
    let segment_half = body
        .part(segment)
        .expect("development only addresses attached segments")
        .half_extent;
    let covers = tagma.appendage.covers(tagma.appendage_shape);
    for ordinal in 0..tagma.per_segment {
        let handedness: &[i32] = match tagma.appendage {
            crate::axis::Appendage::Limb
            | crate::axis::Appendage::Feeler
            | crate::axis::Appendage::Vane => &[-1, 1],
            crate::axis::Appendage::Plate if covers => &[-1, 1],
            crate::axis::Appendage::None => continue,
            crate::axis::Appendage::Plate | crate::axis::Appendage::Mouth => &[0],
        };
        for &side in handedness {
            let mut parent = segment;
            let mut parent_half = segment_half;
            let mut previous: Option<(usize, i32)> = None;
            for link in chain {
                let template = palette.template_at(link.role, link.shape);
                let direction = resolve(link.facing, side);
                let mut offset = face_offset(parent_half, template.half_extent, direction);
                if parent == segment {
                    // Segment slots are tangent to the joining face. Keeping
                    // the normal component intact is what makes even a
                    // Front/Back chain face-connected rather than spaced off
                    // its parent.
                    let tangent = if direction.0 == 2 { 0 } else { 2 };
                    offset[tangent] = bounded_slot(
                        ordinal,
                        tagma.per_segment,
                        parent_half[tangent],
                        template.half_extent[tangent],
                    );
                } else if link.distal {
                    let (axis, sign) = previous.expect("a non-root link has a previous direction");
                    if axis != direction.0 {
                        offset[axis] = sign
                            * (parent_half[axis].abs() - template.half_extent[axis].abs()).max(0);
                    }
                }
                parent = body.attach(
                    template.volume,
                    mass_mg,
                    template.half_extent,
                    Attachment {
                        parent,
                        offset,
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )?;
                parent_half = template.half_extent;
                previous = Some(direction);
            }
        }
    }
    Ok(())
}

fn resolve(facing: ChainFacing, side: i32) -> (usize, i32) {
    // A midline appendage can still author Outward/Inward. Give that singleton
    // a stable right-hand frame instead of emitting a zero normal offset.
    let side = if side == 0 { 1 } else { side.signum() };
    match facing {
        ChainFacing::Outward => (0, side),
        ChainFacing::Inward => (0, -side),
        ChainFacing::Above => (1, 1),
        ChainFacing::Below => (1, -1),
        ChainFacing::Front => (2, -1),
        ChainFacing::Back => (2, 1),
    }
}

fn face_offset(parent: [i32; 3], child: [i32; 3], direction: (usize, i32)) -> [i32; 3] {
    let mut offset = [0; 3];
    offset[direction.0] = direction.1 * (parent[direction.0].abs() + child[direction.0].abs());
    offset
}

fn bounded_slot(ordinal: u8, count: u8, parent_half: i32, child_half: i32) -> i32 {
    let room = (parent_half.abs() - child_half.abs()).max(0);
    let raw = (i32::from(ordinal) * 2 + 1 - i32::from(count)) * child_half.abs().max(1);
    raw.clamp(-room, room)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::{Appendage, Recipe, Soma, Tagma};
    use crate::body::{PartId, SpeciesId};
    use crate::development::{develop_body, minimum_body_mass_mg};

    fn jointed_leg() -> Recipe {
        Recipe::of(vec![Tagma::new(1, Appendage::Limb).with_shapes(0, 3)]).with_appendage_chains(
            vec![vec![
                AppendageStep {
                    role: Role::Limb,
                    shape: 1,
                    facing: ChainFacing::Outward,
                    distal: false,
                },
                AppendageStep {
                    role: Role::Limb,
                    shape: 2,
                    facing: ChainFacing::Below,
                    distal: true,
                },
                AppendageStep {
                    role: Role::Limb,
                    shape: 3,
                    facing: ChainFacing::Front,
                    distal: true,
                },
            ]],
        )
    }

    #[test]
    fn jointed_links_are_face_connected_at_their_outer_tips_and_sever_as_one_branch() {
        let recipe = jointed_leg();
        let soma = Soma {
            segments: vec![1],
            absent: Vec::new(),
        };
        let mass = u64::from(minimum_body_mass_mg(&recipe, &soma).unwrap()) + 100;
        let mut body = develop_body(
            SpeciesId(1),
            &recipe,
            &soma,
            mass,
            crate::axis::archetype::jointed::palette(),
        )
        .unwrap();
        assert_eq!(body.total_mass_mg(), mass);
        assert_eq!(
            body.len(),
            minimum_body_mass_mg(&recipe, &soma).unwrap() as usize,
            "the floor counts every realized chain link"
        );
        assert_eq!(
            body.part(PartId(1)).unwrap().attachment.unwrap().offset,
            [-5, 0, 0]
        );
        assert_eq!(
            body.part(PartId(2)).unwrap().attachment.unwrap().offset,
            [-2, -4, 0]
        );
        assert_eq!(
            body.part(PartId(3)).unwrap().attachment.unwrap().offset,
            [0, -2, -4]
        );
        assert_eq!(
            body.part(PartId(2)).unwrap().attachment.unwrap().parent,
            PartId(1)
        );
        assert_eq!(
            body.part(PartId(3)).unwrap().attachment.unwrap().parent,
            PartId(2)
        );

        body.sever(PartId(1));
        assert!(body.part(PartId(1)).unwrap().severed);
        assert!(body.part(PartId(2)).unwrap().severed);
        assert!(body.part(PartId(3)).unwrap().severed);
        assert!(
            !body.part(PartId(4)).unwrap().severed,
            "the opposite leg remains"
        );
    }

    #[test]
    fn a_chain_has_a_small_explicit_part_budget() {
        let mut recipe = jointed_leg();
        recipe.appendage_chains[0] = (0..9)
            .map(|_| AppendageStep {
                role: Role::Limb,
                shape: 3,
                facing: ChainFacing::Below,
                distal: false,
            })
            .collect();
        let soma = Soma {
            segments: vec![1],
            absent: Vec::new(),
        };
        assert_eq!(
            minimum_body_mass_mg(&recipe, &soma),
            Err(DevelopmentError::AppendageChainTooLong { tagma: 0, steps: 9 })
        );
    }

    #[test]
    fn malformed_chain_programs_refuse_at_the_development_boundary() {
        let soma = Soma {
            segments: vec![1],
            absent: Vec::new(),
        };

        let wrong_length =
            Recipe::of(vec![Tagma::bare(1)]).with_appendage_chains(vec![vec![], vec![]]);
        assert_eq!(
            minimum_body_mass_mg(&wrong_length, &soma),
            Err(DevelopmentError::AppendageChainLength {
                tagmata: 1,
                chains: 2,
            })
        );

        let mut wrong_terminal = jointed_leg();
        wrong_terminal.appendage_chains[0][2].shape = 2;
        assert_eq!(
            minimum_body_mass_mg(&wrong_terminal, &soma),
            Err(DevelopmentError::AppendageChainTerminal { tagma: 0 })
        );

        let mut sensor_intermediate = jointed_leg();
        sensor_intermediate.appendage_chains[0][1].role = Role::Sensor;
        assert_eq!(
            minimum_body_mass_mg(&sensor_intermediate, &soma),
            Err(DevelopmentError::AppendageChainIntermediate { tagma: 0, step: 1 })
        );

        let mut root_distal = jointed_leg();
        root_distal.appendage_chains[0][0].distal = true;
        assert_eq!(
            minimum_body_mass_mg(&root_distal, &soma),
            Err(DevelopmentError::AppendageChainRootDistal { tagma: 0 })
        );
    }

    #[test]
    fn singleton_outward_has_a_real_face_and_slots_never_move_the_normal() {
        assert_eq!(resolve(ChainFacing::Outward, 0), (0, 1));
        assert_eq!(resolve(ChainFacing::Inward, 0), (0, -1));
        // A front-facing root keeps z at its face join and spaces a repeated
        // socket across x, bounded inside the parent's face.
        let normal = face_offset([2, 1, 3], [1, 1, 2], (2, -1));
        assert_eq!(normal[2], -5);
        assert_eq!(bounded_slot(0, 3, 2, 1), -1);
        assert_eq!(bounded_slot(2, 3, 2, 1), 1);
        assert_eq!(bounded_slot(0, 9, 1, 3), 0);
    }
}

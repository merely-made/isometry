// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A roomier visible-body roster: separated feet and a fuller leaf crown.

use super::*;
use crate::axis::{Anchor, AppendageStep, ChainFacing, Stretch};
use crate::body::VolumeRef;
use crate::development::{PartPalette, PartTemplate};
use crate::plan::{Facing, Role};

const BROAD_LEAF: [i32; 3] = [6, 0, 4];
const SHAPE_FOOT: u8 = 3;
const SHAPE_SLIM: u8 = 1;
const SHAPE_LEAF: u8 = 1;
const LEAF_VOLUME_TAG: u8 = 25;

fn shape(tag: u8, half_extent: [i32; 3]) -> PartTemplate {
    PartTemplate {
        volume: VolumeRef::from_tag(tag),
        half_extent,
    }
}

/// The jointed vocabulary, with only its local blade slot widened. The mat
/// and stalk use their own plate selectors, so this changes only this roster's
/// shrub; feet and their price-bearing extents remain exactly jointed's.
pub fn palette() -> PartPalette {
    let mut palette = super::jointed::palette();
    palette.plate.extra[0] = Some(shape(LEAF_VOLUME_TAG, BROAD_LEAF));
    palette
}

fn layout(placements: &[(Option<u8>, Anchor, Facing, Option<u8>)]) -> Vec<Stretch> {
    placements
        .iter()
        .map(|&(parent, anchor, facing, variance)| Stretch {
            parent,
            anchor,
            facing,
            variance,
        })
        .collect()
}

fn leaf_chain() -> Vec<AppendageStep> {
    vec![
        AppendageStep {
            role: Role::Mass,
            shape: SHAPE_SLIM,
            facing: ChainFacing::Above,
            distal: false,
        },
        AppendageStep {
            role: Role::Plate,
            shape: SHAPE_LEAF,
            facing: ChainFacing::Above,
            distal: false,
        },
    ]
}

/// Five broad leaves sit at distinct, connected branch endpoints. Branches
/// vary only through the trunk; every leaf site remains one fixed segment.
pub fn producer_shrub() -> Recipe {
    let mut recipe = Recipe::of(vec![
        Tagma::bare(5).with_shapes(0, 0),
        Tagma::bare(4).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
        Tagma::bare(4).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
        Tagma::bare(4).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
        Tagma::bare(2).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
        Tagma::bare(4).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
    ])
    .with_layout(layout(&[
        (None, Anchor::Tip, Facing::Above, Some(1)),
        (Some(0), Anchor::Base, Facing::Left, None),
        (Some(1), Anchor::Tip, Facing::Above, Some(0)),
        (Some(0), Anchor::Middle, Facing::Right, None),
        (Some(3), Anchor::Tip, Facing::Above, Some(0)),
        (Some(0), Anchor::Tip, Facing::Front, None),
        (Some(5), Anchor::Tip, Facing::Above, Some(0)),
        (Some(0), Anchor::Tip, Facing::Above, None),
        (Some(7), Anchor::Tip, Facing::Above, Some(0)),
        (Some(0), Anchor::Tip, Facing::Back, None),
        (Some(9), Anchor::Tip, Facing::Above, Some(0)),
    ]))
    .with_appendage_chains(vec![
        vec![],
        vec![],
        leaf_chain(),
        vec![],
        leaf_chain(),
        vec![],
        leaf_chain(),
        vec![],
        leaf_chain(),
        vec![],
        leaf_chain(),
    ]);
    recipe.variance = 0;
    recipe
}

/// The branching browser's head, neck, eyes, and tail remain intact. Three
/// fixed leg sites are separated by real bare backbone runs longer than a foot.
pub fn consumer_browser() -> Recipe {
    let mut recipe = Recipe::of(vec![
        Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_CROP),
        Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
        Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_SPECK),
        Tagma::bare(1).with_shapes(SHAPE_SLIM, 0),
        Tagma::bare(2).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_FOOT),
        Tagma::bare(5).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_FOOT),
        Tagma::bare(5).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_FOOT),
        Tagma::bare(4).with_shapes(SHAPE_SLIM, 0),
    ])
    .with_layout(layout(&[
        (None, Anchor::Tip, Facing::Back, None),
        (Some(0), Anchor::Base, Facing::Above, None),
        (Some(1), Anchor::Tip, Facing::Above, None),
        (Some(0), Anchor::Base, Facing::Back, None),
        (Some(3), Anchor::Tip, Facing::Below, Some(1)),
        (None, Anchor::Tip, Facing::Back, None),
        (None, Anchor::Tip, Facing::Back, None),
        (None, Anchor::Tip, Facing::Back, None),
        (None, Anchor::Tip, Facing::Back, None),
        (None, Anchor::Tip, Facing::Back, None),
        (None, Anchor::Tip, Facing::Back, Some(1)),
    ]))
    .with_appendage_chains(vec![
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        leg_chain(),
        vec![],
        leg_chain(),
        vec![],
        leg_chain(),
        vec![],
    ]);
    recipe.variance = 0;
    recipe
}

fn leg_chain() -> Vec<AppendageStep> {
    // The jointed endpoint shapes and directions are unchanged.
    vec![
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
            shape: SHAPE_FOOT,
            facing: ChainFacing::Front,
            distal: true,
        },
    ]
}

pub fn consumer_armoured() -> Recipe {
    super::jointed::consumer_armoured()
}

pub const PRODUCERS: [fn() -> Recipe; 3] =
    [super::producer_mat, producer_shrub, super::producer_stalk];
pub const CONSUMERS: [fn() -> Recipe; 3] =
    [consumer_browser, super::consumer_pursuit, consumer_armoured];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::Soma;
    use crate::body::{BodyDocument, Part, SpeciesId};
    use crate::development::{develop_body, minimum_body_mass_mg};
    use crate::organism::Kingdom;
    use crate::process::FeedingMode;

    const FOOT_VOLUME_TAG: u8 = 14;

    fn body(recipe: &Recipe, seed: u64) -> BodyDocument {
        let soma = Soma::develop(recipe, seed);
        let mass = u64::from(minimum_body_mass_mg(recipe, &soma).unwrap()) + 10_000;
        let body = develop_body(SpeciesId(91), recipe, &soma, mass, palette()).unwrap();
        assert_eq!(body.total_mass_mg(), mass);
        assert!(body.living().all(|part| part.mass_mg > 0));
        body
    }

    fn has_strict_gap(body: &BodyDocument, left: &Part, right: &Part) -> bool {
        let left_at = body.world_pivot(left.id).unwrap();
        let right_at = body.world_pivot(right.id).unwrap();
        (0..3).any(|axis| {
            let left_min = left_at[axis] - left.half_extent[axis].abs();
            let left_max = left_at[axis] + left.half_extent[axis].abs();
            let right_min = right_at[axis] - right.half_extent[axis].abs();
            let right_max = right_at[axis] + right.half_extent[axis].abs();
            left_max < right_min || right_max < left_min
        })
    }

    #[test]
    fn spaced_families_keep_their_readings_and_conserve_mass() {
        for (name, recipe, kingdom, mode) in [
            (
                "shrub",
                producer_shrub as fn() -> Recipe,
                Kingdom::Producer,
                FeedingMode::Producer,
            ),
            (
                "browser",
                consumer_browser,
                Kingdom::Consumer,
                FeedingMode::Grazer,
            ),
            (
                "armoured",
                consumer_armoured,
                Kingdom::Consumer,
                FeedingMode::Grazer,
            ),
        ] {
            for seed in 0..128 {
                let body = body(&recipe(), seed);
                let phenotype = crate::phenotype::BodyPhenotype::seed(body);
                assert_eq!(Kingdom::of(&phenotype), kingdom, "{name}, seed {seed}");
                assert_eq!(FeedingMode::of(&phenotype), mode, "{name}, seed {seed}");
            }
        }
    }

    #[test]
    fn broad_leaf_and_same_side_foot_aabbs_stay_disjoint() {
        let browser_recipe = consumer_browser();
        let mut saw_complete_browser = false;
        for seed in 0..128 {
            let shrub = body(&producer_shrub(), seed);
            let leaves: Vec<_> = shrub
                .living()
                .filter(|part| part.volume == VolumeRef::from_tag(LEAF_VOLUME_TAG))
                .collect();
            assert_eq!(leaves.len(), 5, "shrub seed {seed}");
            for (index, left) in leaves.iter().enumerate() {
                for right in &leaves[index + 1..] {
                    assert!(has_strict_gap(&shrub, left, right), "shrub seed {seed}");
                }
            }

            let soma = Soma::develop(&browser_recipe, seed);
            let expected_pairs = [5u8, 7, 9]
                .into_iter()
                .filter(|tagma| !soma.absent.iter().any(|&(missing, _)| missing == *tagma))
                .count();
            saw_complete_browser |= expected_pairs == 3;
            let browser = body(&browser_recipe, seed);
            let mut feet: [Vec<&Part>; 2] = [Vec::new(), Vec::new()];
            for part in browser
                .living()
                .filter(|part| part.volume == VolumeRef::from_tag(FOOT_VOLUME_TAG))
            {
                let side = usize::from(browser.world_pivot(part.id).unwrap()[0] > 0);
                feet[side].push(part);
            }
            for side in feet {
                assert_eq!(side.len(), expected_pairs, "browser seed {seed}");
                for (index, left) in side.iter().enumerate() {
                    for right in &side[index + 1..] {
                        assert!(has_strict_gap(&browser, left, right), "browser seed {seed}");
                    }
                }
            }
        }
        assert!(
            saw_complete_browser,
            "the sample includes a complete six-foot body"
        );
    }
}

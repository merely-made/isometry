// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The visible-body roster's second arrangement pass.
//!
//! These recipes keep the historical roster available as the axial set. They
//! use ordinary recipe layout data, so development remains the sole source of
//! their part graphs and a renderer never recognizes a species to make a body.

use super::*;
use crate::axis::{Anchor, Stretch};
use crate::plan::Facing;

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

/// A raised shrub with two lateral, leaf-bearing branches and a crown.
///
/// The trunk is vertical; each branch anchors to a realised trunk segment, so
/// the same program remains recognisable when its individual segment count
/// varies. Every plate is still held above the segment it grows from, leaving
/// the existing canopy and fixation reading intact.
pub fn producer_shrub() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::bare(5).with_shapes(0, 0),
            Tagma::new(3, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_BLADE),
            Tagma::new(3, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_BLADE),
            Tagma::new(2, Appendage::Plate)
                .with_shapes(SHAPE_SLIM, SHAPE_FROND)
                .with_per_segment(2),
        ])
        .with_layout(layout(&[
            (None, Anchor::Tip, Facing::Above, None),
            (Some(0), Anchor::Middle, Facing::Left, None),
            (Some(0), Anchor::Tip, Facing::Right, None),
            (Some(0), Anchor::Tip, Facing::Above, None),
        ])),
        1,
    )
}

/// A browsing consumer with a raised head, a compact chest, and a shorter
/// tail. Its only regional variation is in bare neck and tail segments: leg
/// count remains fixed, while relatives still have visible proportions of
/// their own.
pub fn consumer_browser() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_CROP),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_SPECK),
            Tagma::bare(1).with_shapes(SHAPE_SLIM, 0),
            Tagma::bare(2).with_shapes(SHAPE_SLIM, 0),
            Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
            Tagma::bare(1).with_shapes(0, 0),
            Tagma::new(2, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
            Tagma::bare(4).with_shapes(SHAPE_SLIM, 0),
        ])
        .with_layout(layout(&[
            // Head root. The structural long axis remains Back; this is not
            // a camera-facing convention.
            (None, Anchor::Tip, Facing::Back, None),
            // Eyes and specks use distinct sockets above the head rather than
            // sharing one mass pivot.
            (Some(0), Anchor::Base, Facing::Above, None),
            (Some(1), Anchor::Tip, Facing::Above, None),
            // A neck base first clears the crop, then its bare lower run drops
            // into the chest and is the only neck region that may vary.
            (Some(0), Anchor::Base, Facing::Back, None),
            (Some(3), Anchor::Tip, Facing::Below, Some(1)),
            (None, Anchor::Tip, Facing::Back, None),
            (None, Anchor::Tip, Facing::Back, None),
            (None, Anchor::Tip, Facing::Back, None),
            (None, Anchor::Tip, Facing::Back, Some(1)),
        ])),
        0,
    )
}

/// A low armoured cropper: eyes rise from the head, shells continue its trunk,
/// and the legs branch down from the carapace's base. Plates remain covering,
/// so it keeps the consumer reading.
pub fn consumer_armoured() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_CROP),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
            Tagma::new(5, Appendage::Plate).with_shapes(0, SHAPE_SHELL),
            Tagma::new(2, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
            Tagma::bare(3).with_shapes(SHAPE_SLIM, 0),
        ])
        .with_layout(layout(&[
            (None, Anchor::Tip, Facing::Back, None),
            (Some(0), Anchor::Base, Facing::Above, None),
            // The shell run begins at the head, not at the eye branch.
            (Some(0), Anchor::Base, Facing::Back, None),
            // The undercarriage branches from the trunk rather than becoming
            // a serial tail after the shell run.
            (Some(2), Anchor::Base, Facing::Below, None),
            (Some(2), Anchor::Tip, Facing::Back, Some(1)),
        ])),
        0,
    )
}

/// The V2 producer set preserves the historical mat and stalk while replacing
/// its serial shrub with the raised branching construction.
pub const PRODUCERS: [fn() -> Recipe; 3] =
    [super::producer_mat, producer_shrub, super::producer_stalk];

/// The V2 consumer set preserves the pursuit form while arranging browser and
/// armour in the camera's readable plane.
pub const CONSUMERS: [fn() -> Recipe; 3] =
    [consumer_browser, super::consumer_pursuit, consumer_armoured];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::Soma;
    use crate::body::SpeciesId;
    use crate::development::{develop_body, minimum_body_mass_mg};
    use crate::organism::Kingdom;
    use crate::process::FeedingMode;

    fn grown(recipe: &Recipe, seed: u64) -> crate::phenotype::BodyPhenotype {
        let soma = Soma::develop(recipe, seed);
        let floor = u64::from(minimum_body_mass_mg(recipe, &soma).unwrap());
        let mass = floor + 10_000;
        let body = develop_body(SpeciesId(42), recipe, &soma, mass, palette()).unwrap();
        assert_eq!(body.total_mass_mg(), mass);
        assert!(body.living().all(|part| part.mass_mg > 0));
        crate::phenotype::BodyPhenotype::seed(body)
    }

    #[test]
    fn branching_recipes_keep_their_ecological_readings_and_conserve_mass() {
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
            let recipe = recipe();
            for seed in 0..128 {
                let phenotype = grown(&recipe, seed);
                assert_eq!(Kingdom::of(&phenotype), kingdom, "{name}, seed {seed}");
                assert_eq!(FeedingMode::of(&phenotype), mode, "{name}, seed {seed}");
            }
        }
    }

    #[test]
    fn consumer_variation_is_confined_to_bare_neck_and_tail_regions() {
        for recipe in [consumer_browser(), consumer_armoured()] {
            assert_eq!(recipe.variance, 0);
            for seed in 0..128 {
                let soma = Soma::develop(&recipe, seed);
                for (tagma, stretch) in recipe.layout.iter().enumerate() {
                    let authored = recipe.tagmata[tagma].segments;
                    match stretch.variance {
                        Some(1) => assert!(
                            (authored - 1..=authored + 1).contains(&soma.segments[tagma]),
                            "seed {seed}, tagma {tagma}"
                        ),
                        None => {
                            assert_eq!(soma.segments[tagma], authored, "seed {seed}, tagma {tagma}")
                        },
                        Some(other) => panic!("unexpected regional variance {other}"),
                    }
                }
            }
        }
    }

    #[test]
    fn target_families_keep_mass_segments_at_distinct_non_overlapping_pivots() {
        for (name, recipe) in [
            ("shrub", producer_shrub()),
            ("browser", consumer_browser()),
            ("armoured", consumer_armoured()),
        ] {
            for seed in 0..128 {
                let body = grown(&recipe, seed).body().clone();
                let segments: Vec<_> = body
                    .living()
                    .filter(|part| {
                        crate::plan::classify(part.half_extent) == crate::plan::Role::Mass
                    })
                    .collect();
                for (index, left) in segments.iter().enumerate() {
                    let left_at = body.world_pivot(left.id).unwrap();
                    for right in &segments[index + 1..] {
                        let right_at = body.world_pivot(right.id).unwrap();
                        assert_ne!(
                            left_at, right_at,
                            "{name}, seed {seed}: coincident segments"
                        );
                        let overlap = (0..3).all(|axis| {
                            left_at[axis] - left.half_extent[axis].abs()
                                < right_at[axis] + right.half_extent[axis].abs()
                                && right_at[axis] - right.half_extent[axis].abs()
                                    < left_at[axis] + left.half_extent[axis].abs()
                        });
                        assert!(
                            !overlap,
                            "{name}, seed {seed}: mass segments {:?} and {:?} overlap",
                            left.id, right.id
                        );
                    }
                }
            }
        }
    }
}

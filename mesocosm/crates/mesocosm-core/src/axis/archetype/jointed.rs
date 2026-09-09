// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The third visible-body roster: separated leaves and jointed limbs.
//!
//! These programs remain ordinary recipes. The extra links are realized by the
//! same developmental graph, have their own mass, and sever with their parent.

use super::*;
use crate::axis::{Anchor, AppendageStep, ChainFacing, Stretch};
use crate::body::VolumeRef;
use crate::development::{PartPalette, PartTemplate};
use crate::plan::{Facing, Role};

const UPRIGHT_LEG: [i32; 3] = [1, 3, 1];
const FOOT: [i32; 3] = [1, 1, 3];
const LEAF: [i32; 3] = [3, 0, 2];

const SHAPE_UPPER: u8 = 1;
const SHAPE_LOWER: u8 = 2;
const SHAPE_FOOT: u8 = 3;
const SHAPE_STALK: u8 = 1;
const SHAPE_LEAF: u8 = 1;

fn shape(tag: u8, half_extent: [i32; 3]) -> PartTemplate {
    PartTemplate {
        volume: VolumeRef::from_tag(tag),
        half_extent,
    }
}

/// The branching palette plus two directional limb links. It changes only the
/// new jointed founding route; existing worlds keep their recorded palettes.
pub fn palette() -> PartPalette {
    let base = super::palette();
    let mut plate = base.plate;
    // The jointed shrub uses its own leaf proportion. The slot is local to this
    // admitted palette, so it cannot reinterpret a historical content ref.
    plate.extra[0] = Some(shape(15, LEAF));
    PartPalette {
        limb: base.limb.and(shape(13, UPRIGHT_LEG)).and(shape(14, FOOT)),
        plate,
        ..base
    }
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
            shape: SHAPE_STALK,
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

fn leg_chain() -> Vec<AppendageStep> {
    vec![
        AppendageStep {
            role: Role::Limb,
            shape: SHAPE_UPPER,
            facing: ChainFacing::Outward,
            distal: false,
        },
        AppendageStep {
            role: Role::Limb,
            shape: SHAPE_LOWER,
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

/// A branched producer with each leaf held on a paid, face-connected stalk.
/// The leaves remain sparse on their branch tips, so the silhouette has gaps
/// rather than one merged green surface.
pub fn producer_shrub() -> Recipe {
    Recipe::of(vec![
        Tagma::bare(5).with_shapes(0, 0),
        // The lateral runs are structure. Only their fixed terminal segments
        // bear leaves, keeping each blade apart at its real branch tip.
        Tagma::bare(3).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
        Tagma::bare(3).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
        Tagma::bare(2).with_shapes(SHAPE_SLIM, 0),
        Tagma::new(1, Appendage::Plate).with_shapes(SHAPE_SLIM, SHAPE_LEAF),
    ])
    .with_layout(layout(&[
        (None, Anchor::Tip, Facing::Above, Some(1)),
        (Some(0), Anchor::Middle, Facing::Left, None),
        (Some(1), Anchor::Tip, Facing::Above, Some(0)),
        (Some(0), Anchor::Tip, Facing::Right, None),
        (Some(3), Anchor::Tip, Facing::Above, Some(0)),
        (Some(0), Anchor::Tip, Facing::Above, None),
        (Some(5), Anchor::Tip, Facing::Above, Some(0)),
    ]))
    .with_appendage_chains(vec![
        vec![],
        vec![],
        leaf_chain(),
        vec![],
        leaf_chain(),
        vec![],
        leaf_chain(),
    ])
}

/// A grazer whose three pairs of legs each have upper, lower and foot links.
pub fn consumer_browser() -> Recipe {
    let mut recipe = super::branching::consumer_browser();
    let mut chains = vec![Vec::new(); recipe.tagmata.len()];
    for (index, tagma) in recipe.tagmata.iter_mut().enumerate() {
        if tagma.appendage == Appendage::Limb {
            tagma.appendage_shape = SHAPE_FOOT;
            chains[index] = leg_chain();
        }
    }
    recipe.appendage_chains = chains;
    recipe
}

/// A low armoured grazer. Its two leg-bearing stretches branch from the front
/// and rear of the shell run, rather than stacking under one body location.
pub fn consumer_armoured() -> Recipe {
    let mut recipe = Recipe::of(vec![
        Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_CROP),
        Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
        Tagma::new(5, Appendage::Plate).with_shapes(0, SHAPE_SHELL),
        Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_FOOT),
        Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_FOOT),
        Tagma::bare(3).with_shapes(SHAPE_SLIM, 0),
    ])
    .with_layout(layout(&[
        (None, Anchor::Tip, Facing::Back, None),
        (Some(0), Anchor::Base, Facing::Above, None),
        (Some(0), Anchor::Base, Facing::Back, None),
        (Some(2), Anchor::Base, Facing::Below, None),
        (Some(2), Anchor::Tip, Facing::Below, None),
        (Some(2), Anchor::Tip, Facing::Back, Some(1)),
    ]))
    .with_appendage_chains(vec![
        vec![],
        vec![],
        vec![],
        leg_chain(),
        leg_chain(),
        vec![],
    ]);
    // The leg locations are authored. Only the trailing bare run varies,
    // matching the prior arranged armoured recipe.
    recipe.variance = 0;
    recipe
}

pub const PRODUCERS: [fn() -> Recipe; 3] =
    [super::producer_mat, producer_shrub, super::producer_stalk];
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
        let mass = u64::from(minimum_body_mass_mg(recipe, &soma).unwrap()) + 10_000;
        let body = develop_body(SpeciesId(88), recipe, &soma, mass, palette()).unwrap();
        assert_eq!(body.total_mass_mg(), mass);
        assert!(body.living().all(|part| part.mass_mg > 0));
        crate::phenotype::BodyPhenotype::seed(body)
    }

    #[test]
    fn jointed_families_keep_their_feeding_readings_and_mass() {
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
                let phenotype = grown(&recipe(), seed);
                assert_eq!(Kingdom::of(&phenotype), kingdom, "{name}, seed {seed}");
                assert_eq!(FeedingMode::of(&phenotype), mode, "{name}, seed {seed}");
            }
        }
    }
}

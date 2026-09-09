// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::axis::{Anchor, Stretch};
use crate::plan::Facing;

fn exact_soma(recipe: &Recipe) -> Soma {
    Soma {
        segments: recipe.tagmata.iter().map(|tagma| tagma.segments).collect(),
        absent: Vec::new(),
    }
}

fn laid_out() -> Recipe {
    Recipe::of(vec![Tagma::bare(3), Tagma::bare(2), Tagma::bare(2)]).with_layout(vec![
        Stretch {
            parent: None,
            anchor: Anchor::Tip,
            facing: Facing::Above,
            variance: None,
        },
        Stretch {
            parent: Some(0),
            anchor: Anchor::Middle,
            facing: Facing::Right,
            variance: None,
        },
        Stretch {
            parent: Some(0),
            anchor: Anchor::Tip,
            facing: Facing::Left,
            variance: None,
        },
    ])
}

#[test]
fn a_layout_raises_a_trunk_and_branches_from_its_realised_middle() {
    let recipe = laid_out();
    let soma = exact_soma(&recipe);
    let body = develop_body(SpeciesId(9), &recipe, &soma, 700, PartPalette::primitive()).unwrap();

    // Root, two upright segments, then the first branch from the second trunk
    // segment and the second from its tip. Stable allocation order makes these
    // facts pointable without granting presentation any placement authority.
    assert_eq!(body.world_pivot(crate::body::PartId(1)), Some([0, 4, 0]));
    assert_eq!(body.world_pivot(crate::body::PartId(2)), Some([0, 8, 0]));
    assert_eq!(body.world_pivot(crate::body::PartId(3)), Some([4, 4, 0]));
    assert_eq!(body.world_pivot(crate::body::PartId(5)), Some([-4, 8, 0]));
    assert_eq!(body.total_mass_mg(), 700);
    assert_eq!(body.living().count(), 7);
}

#[test]
fn a_layout_keeps_individual_segment_variation_as_its_branch_anchor() {
    let recipe = laid_out();
    let short = Soma {
        segments: vec![2, 2, 2],
        absent: Vec::new(),
    };
    let tall = Soma {
        segments: vec![4, 2, 2],
        absent: Vec::new(),
    };
    let short_body =
        develop_body(SpeciesId(9), &recipe, &short, 600, PartPalette::primitive()).unwrap();
    let tall_body =
        develop_body(SpeciesId(9), &recipe, &tall, 800, PartPalette::primitive()).unwrap();

    // The middle anchor is recomputed from each individual's realised trunk,
    // rather than copied from the authored count.
    assert_eq!(
        short_body.world_pivot(crate::body::PartId(2)),
        Some([4, 4, 0])
    );
    assert_eq!(
        tall_body.world_pivot(crate::body::PartId(4)),
        Some([4, 8, 0])
    );
}

#[test]
fn a_layout_refuses_partial_and_forward_parentage() {
    let recipe = Recipe::of(vec![Tagma::bare(1), Tagma::bare(1)]).with_layout(vec![Stretch {
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Above,
        variance: None,
    }]);
    let soma = exact_soma(&recipe);
    assert_eq!(
        develop_body(SpeciesId(1), &recipe, &soma, 2, PartPalette::primitive()),
        Err(DevelopmentError::LayoutLength {
            tagmata: 2,
            layout: 1
        })
    );

    let recipe = Recipe::of(vec![Tagma::bare(1), Tagma::bare(1)]).with_layout(vec![
        Stretch {
            parent: None,
            anchor: Anchor::Tip,
            facing: Facing::Above,
            variance: None,
        },
        Stretch {
            parent: Some(1),
            anchor: Anchor::Tip,
            facing: Facing::Right,
            variance: None,
        },
    ]);
    let soma = exact_soma(&recipe);
    assert_eq!(
        develop_body(SpeciesId(1), &recipe, &soma, 2, PartPalette::primitive()),
        Err(DevelopmentError::InvalidLayoutParent {
            tagma: 1,
            parent: 1
        })
    );
}

#[test]
fn a_layout_refuses_a_missing_root_or_branch_parent() {
    let recipe = Recipe::of(vec![Tagma::bare(1)]).with_layout(vec![Stretch {
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Above,
        variance: None,
    }]);
    assert_eq!(
        develop_body(
            SpeciesId(1),
            &recipe,
            &Soma {
                segments: vec![0],
                absent: Vec::new(),
            },
            1,
            PartPalette::primitive(),
        ),
        Err(DevelopmentError::EmptyAxis)
    );

    let recipe = Recipe::of(vec![Tagma::bare(1), Tagma::bare(1)]).with_layout(vec![
        Stretch {
            parent: None,
            anchor: Anchor::Tip,
            facing: Facing::Above,
            variance: None,
        },
        Stretch {
            parent: Some(0),
            anchor: Anchor::Tip,
            facing: Facing::Right,
            variance: None,
        },
    ]);
    assert_eq!(
        develop_body(
            SpeciesId(1),
            &recipe,
            &Soma {
                segments: vec![0, 1],
                absent: Vec::new(),
            },
            1,
            PartPalette::primitive(),
        ),
        Err(DevelopmentError::LayoutEmptyRoot)
    );

    let recipe =
        Recipe::of(vec![Tagma::bare(1), Tagma::bare(1), Tagma::bare(1)]).with_layout(vec![
            Stretch {
                parent: None,
                anchor: Anchor::Tip,
                facing: Facing::Above,
                variance: None,
            },
            Stretch {
                parent: None,
                anchor: Anchor::Tip,
                facing: Facing::Right,
                variance: None,
            },
            Stretch {
                parent: Some(1),
                anchor: Anchor::Tip,
                facing: Facing::Right,
                variance: None,
            },
        ]);
    assert_eq!(
        develop_body(
            SpeciesId(1),
            &recipe,
            &Soma {
                segments: vec![1, 0, 1],
                absent: Vec::new(),
            },
            2,
            PartPalette::primitive(),
        ),
        Err(DevelopmentError::LayoutEmptyParent {
            tagma: 2,
            parent: 1
        })
    );
}

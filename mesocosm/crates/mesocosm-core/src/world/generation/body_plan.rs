// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::axis::{Anchor, Stretch, Tagma};
use crate::plan::Facing;
use crate::{Kingdom, Recipe, Rng};
use serde::{Deserialize, Serialize};

/// Developmental arrangement, independent of feeding role and taxonomy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyPlan {
    #[default]
    Axial,
    Branched,
}

impl BodyPlan {
    pub fn label(self) -> &'static str {
        match self {
            Self::Axial => "axial",
            Self::Branched => "branched",
        }
    }

    pub(super) fn generate(self, rng: &mut Rng, role: Kingdom) -> Recipe {
        let mut recipe = crate::axis::seed(rng, role);
        if self == Self::Axial {
            return recipe; // Preserve the version-1 draw stream exactly.
        }
        // Keep the role-bearing organs; extend the drawn vocabulary into a
        // trunk with at least two real, oppositely directed daughter stretches.
        while recipe.tagmata.len() < 4 {
            recipe.tagmata.push(Tagma::bare(2 + rng.below(4) as u8));
        }
        let upright = role == Kingdom::Producer;
        let mut layout = vec![
            Stretch {
                parent: None,
                anchor: Anchor::Tip,
                facing: Facing::Back,
                variance: None,
            },
            Stretch {
                parent: Some(0),
                anchor: Anchor::Tip,
                facing: if upright { Facing::Above } else { Facing::Back },
                variance: None,
            },
        ];
        for index in 2..recipe.tagmata.len() {
            layout.push(Stretch {
                parent: Some(1),
                anchor: if rng.below(2) == 0 {
                    Anchor::Middle
                } else {
                    Anchor::Tip
                },
                facing: match index {
                    2 => Facing::Left,
                    3 => Facing::Right,
                    _ if upright => Facing::Back,
                    _ => Facing::Above,
                },
                variance: None,
            });
        }
        recipe.with_layout(layout)
    }
}

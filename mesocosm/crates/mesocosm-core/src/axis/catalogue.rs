// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Recipes for real animals, as evidence that the method generates the
//! catalogue rather than one creature.
//!
//! These are reference points, not content: worldgen seeds its own lineages
//! with [`seed`](super::seed) and play mutates them. They exist so a test can
//! assert that the same four rules reach a centipede and a tetrapod without
//! special cases, and so the relationships between real plans are checkable.
//!
//! Split from `axis.rs` at the 600-line ceiling.

/// These are reference points, not content: worldgen seeds lineages and play
/// mutates them. They exist so a test can assert that the same four rules
/// reach a centipede and a tetrapod without special cases.
use super::{Appendage, Recipe, Tagma};

/// Many segments, one trunk, a limb pair on every one.
pub fn centipede(segments: u8) -> Recipe {
    Recipe::of(vec![
        Tagma::new(1, Appendage::Feeler),
        Tagma::new(segments, Appendage::Limb),
    ])
}

/// A centipede with fused segments: two limb pairs per apparent one.
pub fn millipede(segments: u8) -> Recipe {
    Recipe::of(vec![
        Tagma::new(1, Appendage::Feeler),
        Tagma::new(segments, Appendage::Limb).with_per_segment(2),
    ])
}

/// Head, thorax, abdomen: legs on the thorax only, wings on part of it.
pub fn insect() -> Recipe {
    Recipe::of(vec![
        Tagma::new(1, Appendage::Feeler),
        Tagma::new(1, Appendage::Mouth),
        Tagma::new(3, Appendage::Limb),
        Tagma::new(2, Appendage::Vane),
        Tagma::bare(11),
    ])
}

/// Two stretches: four leg pairs forward, nothing behind.
pub fn spider() -> Recipe {
    Recipe::of(vec![
        Tagma::new(1, Appendage::Mouth),
        Tagma::new(4, Appendage::Limb),
        Tagma::bare(10),
    ])
}

/// A long trunk with limbs only at two girdles.
pub fn tetrapod(trunk: u8) -> Recipe {
    Recipe::of(vec![
        Tagma::new(1, Appendage::Feeler),
        Tagma::new(1, Appendage::Limb),
        Tagma::bare(trunk),
        Tagma::new(1, Appendage::Limb),
        Tagma::bare(trunk / 2),
    ])
}

/// A tetrapod with its girdles suppressed and its trunk multiplied: the
/// single clearest demonstration that these are variations on a theme.
pub fn snake(trunk: u8) -> Recipe {
    let mut plan = tetrapod(trunk);
    for tagma in &mut plan.tagmata {
        if tagma.appendage == Appendage::Limb {
            tagma.per_segment = 0;
            tagma.appendage = Appendage::None;
        }
    }
    plan.tagmata[2].segments = trunk;
    plan
}

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Appendage, Recipe, Soma};
use crate::rng::Rng;

impl Soma {
    /// Develops an individual from a lineage's recipe and a seed.
    ///
    /// Pure function of the two, so a creature's body is reproducible from
    /// its identity the way everything else in the core is.
    pub fn develop(lineage: &Recipe, seed: u64) -> Self {
        let mut rng = Rng::from_seed(seed);
        let mut segments = Vec::with_capacity(lineage.tagmata.len());
        for (index, tagma) in lineage.tagmata.iter().enumerate() {
            let spread = lineage
                .layout_for(index)
                .and_then(|s| s.variance)
                .unwrap_or(lineage.variance) as i32;
            let drift = if spread > 0 {
                rng.range_i32(-spread, spread)
            } else {
                0
            };
            segments.push((tagma.segments as i32 + drift).clamp(1, 255) as u8);
        }

        // A rare developmental absence, drawn per tagma so a long stretch is
        // likelier to lose one than a short one.
        //
        // **Never a feeding organ, and never a sense organ** (DC1.5, widened at
        // DC4). An individual missing one limb is variation; an individual
        // missing the organ it feeds with is a stillbirth, and since a kingdom
        // is read off that organ it would also be an individual born into a
        // different kingdom from its own line. A mouth is one such organ and a
        // **lit** plate is the other — a plant realized at one leafing segment
        // could otherwise lose its whole canopy and be born a decomposer.
        //
        // A feeler is spared for the ruling's own reason rather than the
        // reading's: senses are *presumed*, and TD11's finding was a world of
        // blind bodies. A line with one sensory stretch would otherwise bear a
        // blind founder about one birth in twelve, which is the defect this
        // plan exists to close, reintroduced by a lottery.
        //
        // What absence still takes is limbs, vanes and covering — a leg pair,
        // a fin, a missing shell. Those are variation.
        let mut absent = Vec::new();
        for (index, tagma) in lineage.tagmata.iter().enumerate() {
            let presumed = match tagma.appendage {
                Appendage::Mouth | Appendage::Feeler => true,
                Appendage::Plate => !tagma.appendage.covers(tagma.appendage_shape),
                _ => false,
            };
            if tagma.appendage == Appendage::None || presumed || tagma.per_segment == 0 {
                continue;
            }
            let realised = segments[index];
            if rng.below(12) == 0 {
                absent.push((index as u8, rng.below(realised.max(1) as u64) as u8));
            }
        }
        Self { segments, absent }
    }

    pub fn total_segments(&self) -> u32 {
        self.segments.iter().map(|s| *s as u32).sum()
    }
}

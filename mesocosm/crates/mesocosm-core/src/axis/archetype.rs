// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Authored creatures: the bodies a world is meant to open holding.
//!
//! Sibling of [`catalogue`](super::catalogue) and its opposite in intent. The
//! catalogue's own doc says "these are reference points, not content" — it
//! exists to prove four axial rules reach a centipede. **This module is
//! content**: imagined creatures, each a [`Recipe`] plus the palette entries
//! its shapes need, developed by the same `Recipe -> Soma -> develop_body`
//! path every other body takes. Nothing here builds a [`BodyDocument`][b]
//! directly; an archetype that did would be the second simulation authority
//! `CLAUDE.md` forbids.
//!
//! # The roster
//!
//! Eight, per the plan's §4.2: three producers, three consumers, two
//! decomposers. Each is one idea followed through rather than a parameter
//! sweep.
//!
//! | tier | archetype | the idea |
//! | --- | --- | --- |
//! | Producer | [`producer_mat`] | a low mat of flat pads lying over the ground |
//! | Producer | [`producer_shrub`] | two leaf sizes on one branching runner |
//! | Producer | [`producer_stalk`] | a long bare stem carrying one crown of big fronds |
//! | Consumer | [`consumer_browser`] | a long-necked hexapod that crops the stand |
//! | Consumer | [`consumer_pursuit`] | a short-bodied sprinter with a swinging jaw |
//! | Consumer | [`consumer_armoured`] | a small cropper wearing its plates as covering |
//! | Decomposer | [`decomposer_crust`] | a flat creeper whose pads lie in the litter |
//! | Decomposer | [`decomposer_detritivore`] | an eight-legged walker that finds the dead |
//!
//! # What the ruled framing does to them
//!
//! The terrarium section looks along `-z` and [`develop_body`] chains segments
//! along `+z`, so **the game's own camera draws every one of these bodies
//! end-on**. That is presentation rather than anatomy and it is recorded as a
//! finding; the receipts turn the slab a quarter to show the body plans.
//!
//! [`develop_body`]: crate::development::develop_body
//!
//! # What each reads as, and why
//!
//! Kingdom is a reading of feeding anatomy (`organism/kingdom.rs`), so an
//! archetype does not declare a tier — it grows the organs that make one. The
//! producers carry plates **in canopy positions**; the consumers carry a mouth
//! under the head and nothing lit; the decomposers carry neither.
//!
//! **Armour is a position, not a shape** (DC4). [`consumer_armoured`] and
//! [`decomposer_crust`] both wear plates, and neither reads Producer, because
//! their plates hang on the flanks rather than being held up.
//!
//! # Senses
//!
//! **Every fauna archetype carries working eyes and something that contracts**,
//! which is the ruling this plan was founded on. The three producers are
//! sightless and sessile, deliberately: a plant has no eyes, and a body with
//! no `Sense` and no `Contract` part reads a build multiple of exactly 1, which
//! is what makes the stand free at the ecology scale (plan §2.4).
//!
//! # No names
//!
//! The identifiers are role-descriptive. Naming rounds in this repo carry
//! crates.io / game / studio / trademark checks and are Mark's; a founding
//! lineage is also deliberately unnamed (`species.rs`: "they were there before
//! anybody arrived to name them").
//!
//! [b]: crate::body::BodyDocument
//!
//! # The palette is world state and the recipe is the program
//!
//! An archetype is therefore two halves that live in different places. Its
//! *shapes* go in [`palette`], which a world snapshots; its *arrangement* goes
//! in the recipe, which a lineage carries. Every shape below is added in a
//! spare [`RoleShapes`](crate::development::RoleShapes) slot — no default
//! moves — so a tier that still draws from [`seed`](super::seed) develops
//! exactly the body it developed before.

use super::{ARMOUR_SHAPE, Appendage, JAW_SHAPE, Recipe, Tagma};
use crate::body::VolumeRef;
use crate::development::{PartPalette, PartTemplate};

/// The trunk shape a slim segment is made of: 5x3x3 voxels, 36 mg of ceiling.
const SLIM: [i32; 3] = [2, 1, 1];
/// A broad segment, the one a leg pair hangs off: 5x5x3, 60 mg.
const BROAD: [i32; 3] = [2, 2, 1];
/// The cropping mouth: a flat 5x3x1 blade, 12 mg. `Mass`-classified, which is
/// what makes the body that wears it a [`Grazer`](crate::process::FeedingMode).
/// Doubles as the flat tile a crust is tiled from.
const CROP: [i32; 3] = [2, 1, 0];
/// A walking leg: 7x3x3, 50 mg, span 3. Build price 6.00 against the primitive
/// limb's 6.25, so it is admissible under the §2.3 guard; DC1's finding is
/// that at a 3x3 cross-section `[4,1,1]` is the longest leg that would be.
const LEG: [i32; 3] = [3, 1, 1];
/// A working eye, the primitive sensor: 3x3x3, 21 mg, span 1.
const EYE: [i32; 3] = [1, 1, 1];
/// A decorative sense voxel: one voxel, 1 mg, and span **zero**, so it buys
/// detail and no horizon. The plan's §2.2 names this as its own kind of part —
/// "an archetype's decorative eye voxels and its functional sense organs are
/// therefore not the same parts."
const SPECK: [i32; 3] = [0, 0, 0];
/// A small upright leaf: 7x7x1, 39 mg. What a shrub is mostly made of.
const BLADE: [i32; 3] = [3, 3, 0];
/// A flat horizontal pad: 9x1x9, 64 mg. Held above a segment it is a mat's
/// leaf; hung on the flank it is the skirt a crust spreads with. The one shape
/// in the palette the position rule is exercised on both ways.
const PAD: [i32; 3] = [4, 0, 4];
/// A flank shell: 1x9x9, 64 mg. Only ever worn as covering, and deep enough in
/// `y` to stand proud of the segment it wraps.
const SHELL: [i32; 3] = [0, 4, 4];

/// Palette selector for [`SLIM`] within the `Mass` bank.
const SHAPE_SLIM: u8 = 1;
/// Palette selector for [`BROAD`].
const SHAPE_BROAD: u8 = 2;
/// Palette selector for [`CROP`]. A mouth's selector picks its *role* as well
/// as its shape, and everything below [`JAW_SHAPE`] is drawn from the `Mass`
/// bank — which is the whole of why a body wearing this one grazes.
const SHAPE_CROP: u8 = 3;
/// Palette selector for a jaw: the `Limb` bank's default `[4,1,1]`, named from
/// above [`JAW_SHAPE`] so the mouth is drawn as an actuator. A jaw swings, so
/// the body that wears one is a predator.
const SHAPE_JAW: u8 = JAW_SHAPE;
/// Palette selector for [`LEG`] within the `Limb` bank.
const SHAPE_LEG: u8 = 1;
/// Palette selector for [`EYE`]: the `Sensor` bank's default.
const SHAPE_EYE: u8 = 0;
/// Palette selector for [`SPECK`].
const SHAPE_SPECK: u8 = 1;
/// Palette selector for the primitive `Plate` `[4,4,1]`, the big frond a crown
/// is made of.
const SHAPE_FROND: u8 = 0;
/// Palette selector for [`BLADE`].
const SHAPE_BLADE: u8 = 1;
/// Palette selector for [`PAD`], held above its segment.
const SHAPE_PAD: u8 = 2;
/// [`PAD`] again, worn as covering: the same leaf lying in the litter beside a
/// crust instead of being held up. Above [`ARMOUR_SHAPE`], so it fixes nothing.
const SHAPE_SKIRT: u8 = ARMOUR_SHAPE + SHAPE_PAD;
/// [`SHELL`], worn as covering. The archetype palette admits no lit version of
/// it, because armour is the only thing this shape is for.
const SHAPE_SHELL: u8 = ARMOUR_SHAPE + 3;

/// The vocabulary the archetypes are carved from, added to the baseline world
/// palette.
///
/// **Every entry is a spare slot.** `PartPalette::primitive`'s four defaults
/// are untouched, so a recipe that names no shape — which is every recipe
/// [`seed`](super::seed) draws — develops the body it always did, and this
/// palette can be installed on a world whose tiers still draw.
///
/// The `Mass` and `Plate` banks are **full** at `PALETTE_SHAPES = 4`; `Limb`
/// spends two of four and `Sensor` two, of which two are all the build-price
/// guard admits anyway.
pub fn palette() -> PartPalette {
    let baseline = PartPalette::primitive();
    // The working eye is the baseline's own sensor, taken rather than added:
    // selector zero already names it, and asserting that keeps the archetype's
    // `SHAPE_EYE` honest if the primitive palette ever moves.
    debug_assert_eq!(baseline.sensor.default.half_extent, EYE);
    PartPalette {
        mass: baseline
            .mass
            .and(shape(5, SLIM))
            .and(shape(6, BROAD))
            .and(shape(7, CROP)),
        limb: baseline.limb.and(shape(8, LEG)),
        sensor: baseline.sensor.and(shape(9, SPECK)),
        plate: baseline
            .plate
            .and(shape(10, BLADE))
            .and(shape(11, PAD))
            .and(shape(12, SHELL)),
    }
}

/// A content address and an exact shape. The addresses are fixture tags while
/// the project has no pack loader, continuing the primitive palette's 1-4.
fn shape(tag: u8, half_extent: [i32; 3]) -> PartTemplate {
    PartTemplate {
        volume: VolumeRef::from_tag(tag),
        half_extent,
    }
}

fn with_variance(mut recipe: Recipe, variance: u8) -> Recipe {
    recipe.variance = variance;
    recipe
}

/// A low ground mat: a run of blocky tiles each holding two flat pads up.
///
/// Forty-one parts. The plainest producer there is, and the one the terrarium
/// floor is mostly made of. Its pads sit two voxels over the tiles they grow
/// from, which is a canopy position by a small margin — the rule asks where a
/// plate hangs, not how far.
///
/// Variance 1: a mat with one tile more or less is the same mat.
pub fn producer_mat() -> Recipe {
    with_variance(
        Recipe::of(vec![
            // A rooting foot, so the mat has somewhere to start.
            Tagma::bare(2).with_shapes(0, 0),
            Tagma::new(13, Appendage::Plate)
                .with_shapes(0, SHAPE_PAD)
                .with_per_segment(2),
        ]),
        1,
    )
}

/// A many-fronded shrub: eight big fronds on a stout stem, then sixteen small
/// blades on a finer branch.
///
/// Forty-one parts in two leafing stretches. Two leaf sizes on one plant is
/// most of what makes it read as a shrub rather than a stack, and it is also
/// the only producer here that reads as foliage at the ruled framing.
///
/// Variance 1, for the same reason the mat has it.
pub fn producer_shrub() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::bare(1).with_shapes(0, 0),
            // A stout main stem carrying big fronds, then a finer branch of
            // small blades: two leaf sizes on one plant, which is most of what
            // makes it read as a shrub rather than a stack.
            Tagma::new(8, Appendage::Plate).with_shapes(0, SHAPE_FROND),
            Tagma::new(8, Appendage::Plate)
                .with_shapes(SHAPE_SLIM, SHAPE_BLADE)
                .with_per_segment(2),
        ]),
        1,
    )
}

/// A tall single stalk with a crown: seventeen bare segments and six big
/// fronds.
///
/// Twenty-six parts, and the one archetype the axial rules fight. **A body's
/// segments chain along `z` and nothing stacks along `y`**, so "tall" is a
/// long bare stem rather than a raised one, and the height in the silhouette
/// comes from the crown's `[4,4,1]` fronds standing eight voxels over it.
/// Recorded as a finding rather than worked around: raising a stalk needs a
/// second growth axis, which is not this slice's to add.
///
/// Variance 1: the stem's length varies and the crown's does not matter, since
/// the guard below keeps a fronded stretch from losing its fronds.
pub fn producer_stalk() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::bare(1).with_shapes(0, 0),
            Tagma::bare(16).with_shapes(0, 0),
            Tagma::new(3, Appendage::Plate)
                .with_shapes(0, SHAPE_FROND)
                .with_per_segment(2),
        ]),
        1,
    )
}

/// A browsing hexapod: the consumer tier's first authored body. (Plan §2.4,
/// carving B)
///
/// Thirty-three parts — a slim head carrying a broad cropping mouth, two eyes
/// and two decorative specks, a five-segment neck, three broad segments bearing
/// six legs around a deep chest, and a ten-segment tail. It reads
/// [`Kingdom::Consumer`](crate::organism::Kingdom) because the head bears a
/// mouth and nothing on it is lit, and [`FeedingMode::Grazer`] because that
/// mouth is bulk rather than a jaw: it crops what stands still.
///
/// # Variance is zero, deliberately
///
/// The plan asks an archetype to set [`Recipe::variance`] rather than inherit
/// the draw's `1 + rng.below(2)`: "a hexapod with a varying leg count is a
/// different creature, while a frond with a varying frond count is the same
/// plant." Kin still are not clones — [`Soma::develop`](super::Soma::develop)
/// keeps its developmental absence, so an individual may be born without a leg
/// pair or without its eyes. What absence may never take is a feeding organ,
/// which is guarded, so no individual can develop out of its line's kingdom.
///
/// [`FeedingMode::Grazer`]: crate::process::FeedingMode::Grazer
pub fn consumer_browser() -> Recipe {
    with_variance(
        Recipe::of(vec![
            // The head, and the crop borne under it: this one attachment is
            // what the world reads the whole body's living off.
            Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_CROP),
            // Eyes, then specks. Two tagmata because a stretch grows one kind
            // of appendage from one shape, and these are two different parts
            // doing two different jobs.
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_SPECK),
            // A browser's reach.
            Tagma::bare(5).with_shapes(SHAPE_SLIM, 0),
            // Six legs on three broad segments, around a deep chest.
            Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
            Tagma::bare(1).with_shapes(0, 0),
            Tagma::new(2, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
            Tagma::bare(10).with_shapes(SHAPE_SLIM, 0),
        ]),
        0,
    )
}

/// A low pursuit form: short body, long tail, six legs, and a jaw.
///
/// Thirty parts. The mouth is drawn from the `Limb` bank
/// (`SHAPE_JAW`), so it classifies as an actuator and the body reads
/// [`FeedingMode::Predator`](crate::process::FeedingMode::Predator) — the
/// ruling of 2026-08-30, in one selector.
///
/// **This is the archetype DC2's finding asked for.** A consumer tier is one
/// interbreeding species per lineage, so founding the browser alone founded a
/// world of nothing but mobile grazers with nothing eating them back; the
/// pursuit form founds beside it.
///
/// Variance 0, for the same reason the browser's is.
pub fn consumer_pursuit() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_JAW),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
            Tagma::new(2, Appendage::Limb).with_shapes(0, SHAPE_LEG),
            Tagma::bare(4).with_shapes(0, 0),
            Tagma::new(1, Appendage::Limb).with_shapes(0, SHAPE_LEG),
            // A long tail behind a short body: what a sprinter steers with.
            Tagma::bare(12).with_shapes(SHAPE_SLIM, 0),
        ]),
        0,
    )
}

/// A small armoured opportunist: four blocky segments wearing eight flank
/// shells, over a cropping mouth.
///
/// Twenty-nine parts, and **the archetype the position rule exists for**. Its
/// ten shells are `Role::Plate`, so every one performs `Process::Fix`, and
/// the body still reads [`Kingdom::Consumer`](crate::organism::Kingdom):
/// covering is not a canopy, so nothing it wears is lit. Before DC4 no founding
/// consumer could be armoured at all — DC1.5 recorded that as a standing
/// constraint on this roster, and this is it lifted.
///
/// Variance 0.
pub fn consumer_armoured() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_CROP),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
            // The carapace, on the deepest segments the palette has.
            Tagma::new(5, Appendage::Plate).with_shapes(0, SHAPE_SHELL),
            // Four legs on one stretch rather than two stretches of one:
            // absence is drawn per stretch, and two single-segment leg
            // stretches can both come up short and leave the body immobile.
            Tagma::new(2, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
            Tagma::bare(3).with_shapes(SHAPE_SLIM, 0),
        ]),
        0,
    )
}

/// A spreading crust: flat tiles with pads lying out to either side, two eyes,
/// and four holdfast threads.
///
/// Thirty-six parts. It reads
/// [`Kingdom::Decomposer`](crate::organism::Kingdom) by carrying neither organ
/// — its head bears no mouth, and its pads are worn as covering rather than
/// held up — which is the residual reading DC1.5 found, arrived at on purpose
/// for once rather than by subtraction.
///
/// It is not sightless: a crust that could not tell where the dead were would
/// be scenery. The holdfasts are `Limb`-classified threads, so it contracts,
/// which is also what the drawn decomposer tier has always done.
///
/// Variance 1: a crust one tile wider is the same crust.
pub fn decomposer_crust() -> Recipe {
    with_variance(
        Recipe::of(vec![
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_BROAD, SHAPE_EYE),
            // Three holdfast segments rather than two: at variance 1 a stretch
            // of two can realize as one and then lose it to absence, and a
            // crust that cannot hold on is a body the ruling does not admit.
            Tagma::new(3, Appendage::Limb).with_shapes(SHAPE_CROP, SHAPE_LEG),
            Tagma::new(8, Appendage::Plate).with_shapes(SHAPE_CROP, SHAPE_SKIRT),
        ]),
        1,
    )
}

/// A mobile detritivore: eight legs, two eyes, two specks, and no mouth at all.
///
/// Thirty-one parts. The one archetype whose reading turns on an **absence**:
/// nothing hangs under its head, so it takes nothing in through an organ and
/// absorbs across its bulk instead, which is what a saprotroph does.
///
/// Variance 0: an eight-legged walker with a varying leg count is a different
/// animal.
pub fn decomposer_detritivore() -> Recipe {
    with_variance(
        Recipe::of(vec![
            // Bare, and that is the whole reading: a head with no mouth.
            Tagma::bare(1).with_shapes(SHAPE_SLIM, 0),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
            Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_SPECK),
            Tagma::new(4, Appendage::Limb).with_shapes(0, SHAPE_LEG),
            Tagma::bare(12).with_shapes(SHAPE_SLIM, 0),
        ]),
        0,
    )
}

/// The producer tier's authored bodies, in founding order.
pub const PRODUCERS: [fn() -> Recipe; 3] = [producer_mat, producer_shrub, producer_stalk];
/// The consumer tier's, in founding order. The browser leads because the
/// played critter takes the first of its tier.
pub const CONSUMERS: [fn() -> Recipe; 3] = [consumer_browser, consumer_pursuit, consumer_armoured];
/// The decomposer tier's, in founding order.
pub const DECOMPOSERS: [fn() -> Recipe; 2] = [decomposer_crust, decomposer_detritivore];

#[cfg(test)]
mod tests;

pub mod branching;
pub mod jointed;
pub mod spaced;

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a body plan is generated: segments, tagmata, and a lexicon.
//!
//! Bodies were being sculpted, which is the wrong shape of answer: a sculpt
//! produces one creature and no relatives. Real body plans come from an
//! **axial recipe** read head to tail, and almost the whole animal kingdom is
//! variations on it:
//!
//! 1. **Segments** repeated along one axis (metamerism).
//! 2. **Serial homology**: every segment carries the same appendage machinery.
//! 3. **Tagmata**: segments group into a few named stretches.
//! 4. **Regional identity**: a stretch decides what its segments' appendages
//!    become, or suppresses them.
//!
//! Symmetry, which [`BodyPlan`](crate::plan::BodyPlan) already owns, is the
//! fifth. Together they generate centipedes, insects, spiders, tetrapods, and
//! snakes as parameter sets rather than as special cases, which the tests in
//! this module demonstrate against the real catalogue.
//!
//! # The mutations are the real ones
//!
//! Every edit here is a change biology actually makes. Fusing segment pairs
//! turns a centipede into a millipede. Raising the count and suppressing two
//! limb positions turns a tetrapod into a snake. And **changing a tagma's
//! appendage is homeosis**: *antennapedia*, the textbook Hox mutant that grows
//! legs where antennae belong, is one field of one [`Tagma`] in this type.
//!
//! # It composes, it does not replace
//!
//! [`BodyPlan`](crate::plan::BodyPlan) stays the *placement policy*: which
//! facing a role prefers, whether it mirrors, how much tolerance growth has.
//! This is the *scaffold* above it. The scaffold says a thoracic segment bears
//! a limb; the policy says which way a limb points.
//!
//! # Acquisition is Hox-like
//!
//! A lineage cannot express an appendage it has never eaten. The [`lexicon`]
//! is the set of appendage kinds a line has acquired, and incorporating a
//! creature teaches its kinds. So kleptoplasty is not a bolt-on part: eating
//! something **teaches your line a word**, and the plan decides where to say
//! it. That is why a plan carries a lexicon and refuses assignments outside
//! it.
//!
//! [`lexicon`]: Recipe::lexicon

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::development::PALETTE_SHAPES;
use crate::organism::Kingdom;
use crate::plan::Role;
use crate::rng::Rng;

mod appendage_chain;
pub use appendage_chain::{AppendageStep, ChainFacing};

/// The mouth selector at which a mouth stops being bulk and becomes a jaw.
///
/// One shape bank up: selectors `0..JAW_SHAPE` name `Mass` shapes and
/// `JAW_SHAPE..` name `Limb` shapes. Sized from the palette so the two banks
/// cannot overlap when the palette widens.
pub const JAW_SHAPE: u8 = PALETTE_SHAPES as u8;

/// The plate selector at which a plate stops being a frond and becomes
/// covering.
///
/// The same trick as [`JAW_SHAPE`], one bank over: selectors `0..ARMOUR_SHAPE`
/// hang a plate **above** its segment, where light reaches it, and
/// `ARMOUR_SHAPE..` wear it as a mirrored pair on the segment's **flanks**,
/// where nothing does. A recipe carries selectors and nothing else, so this is
/// how an archetype says which of the two a stretch grows. (DC4)
pub const ARMOUR_SHAPE: u8 = PALETTE_SHAPES as u8;

/// What a stretch of segments grows.
///
/// Deliberately the same vocabulary the parts graph already speaks: an
/// appendage becomes a part with a [`Role`], so nothing downstream learns a
/// second word for the same thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Appendage {
    /// No appendage. The abdomen of an insect, the trunk of a snake.
    None,
    /// A walking, swimming, or grasping limb.
    Limb,
    /// A sensor: antenna, eye stalk, feeler.
    Feeler,
    /// Armour or shell.
    Plate,
    /// A feeding structure: mandible, proboscis, radula.
    Mouth,
    /// A wing or fin: a limb that works on a fluid.
    Vane,
}

impl Appendage {
    pub const ALL: [Appendage; 6] = [
        Appendage::None,
        Appendage::Limb,
        Appendage::Feeler,
        Appendage::Plate,
        Appendage::Mouth,
        Appendage::Vane,
    ];

    /// The part role this appendage becomes when it grows.
    ///
    /// **A mouth has two geometries and every other appendage has one.** A
    /// cropping mouth is bulk and a jaw is an actuator, and which one a stretch
    /// grows is what makes a grazer and a predator different bodies rather than
    /// different flags (DC1.5). The selector carries it: below [`JAW_SHAPE`] a
    /// mouth is drawn from the `Mass` bank, at or above it from the `Limb`
    /// bank, and [`Appendage::shape_index`] maps it back into that bank. Zero
    /// is still the default a recipe naming no shape develops.
    pub fn role(self, shape: u8) -> Option<Role> {
        match self {
            Appendage::None => None,
            Appendage::Limb | Appendage::Vane => Some(Role::Limb),
            Appendage::Feeler => Some(Role::Sensor),
            Appendage::Plate => Some(Role::Plate),
            Appendage::Mouth if shape >= JAW_SHAPE => Some(Role::Limb),
            Appendage::Mouth => Some(Role::Mass),
        }
    }

    /// The selector within its role's own shape bank.
    pub fn shape_index(self, shape: u8) -> u8 {
        match self {
            Appendage::Mouth => shape % JAW_SHAPE,
            Appendage::Plate => shape % ARMOUR_SHAPE,
            _ => shape,
        }
    }

    /// Whether this appendage, at this selector, is worn against the body
    /// rather than held out from it.
    ///
    /// Only a plate can be: everything else has one position. See
    /// [`ARMOUR_SHAPE`], and [`BodyDocument::canopy`] for what the position
    /// then means.
    ///
    /// [`BodyDocument::canopy`]: crate::body::BodyDocument::canopy
    pub fn covers(self, shape: u8) -> bool {
        self == Appendage::Plate && shape >= ARMOUR_SHAPE
    }

    /// Whether a lineage starts able to express this.
    ///
    /// Mass and the absence of an appendage are free; everything else is
    /// acquired by eating something that had it.
    pub fn is_innate(self) -> bool {
        matches!(self, Appendage::None | Appendage::Mouth)
    }
}

/// One stretch of the axis, and what it makes.
///
/// The two shape selectors are indices into the world's
/// [`PartPalette`](crate::development::PartPalette), which admits several
/// shapes per role. **Zero is every role's default**, so a recipe that names
/// no shape is built from exactly the templates recipes were built from when
/// a role admitted only one. A selector the world does not admit falls back to
/// that default rather than failing, which is what lets a recipe travel to a
/// world with a poorer vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tagma {
    /// How many segments this stretch holds.
    pub segments: u8,
    /// What each of its segments bears.
    pub appendage: Appendage,
    /// Appendages per segment. Two is a millipede's diplosegment; zero
    /// suppresses without changing the stretch's identity, which is how a
    /// snake keeps a trunk and loses its legs.
    pub per_segment: u8,
    /// Which admitted `Role::Mass` shape this stretch's segments are made of.
    pub segment_shape: u8,
    /// Which admitted shape this stretch's appendages are made of, indexed
    /// within the appendage's own role.
    pub appendage_shape: u8,
}

impl Tagma {
    pub fn new(segments: u8, appendage: Appendage) -> Self {
        Self {
            segments,
            appendage,
            per_segment: 1,
            segment_shape: 0,
            appendage_shape: 0,
        }
    }

    pub fn bare(segments: u8) -> Self {
        Self {
            segments,
            appendage: Appendage::None,
            per_segment: 0,
            segment_shape: 0,
            appendage_shape: 0,
        }
    }

    pub fn with_per_segment(mut self, per_segment: u8) -> Self {
        self.per_segment = per_segment;
        self
    }

    /// Picks this stretch's segment and appendage shapes out of the palette.
    pub fn with_shapes(mut self, segment_shape: u8, appendage_shape: u8) -> Self {
        self.segment_shape = segment_shape;
        self.appendage_shape = appendage_shape;
        self
    }

    /// How many appendages this stretch grows in total.
    pub fn appendage_count(&self) -> u32 {
        if self.appendage == Appendage::None {
            return 0;
        }
        self.segments as u32 * self.per_segment as u32
    }
}

/// A lineage's heritable axial recipe: the theme its members vary on.
///
/// Named `Recipe` rather than `Lineage` because [`species::Lineages`] already
/// owns lineage identity; this is what a lineage's bodies are made from.
///
/// [`species::Lineages`]: crate::species::Lineages
///
/// Epoch-bounded by ruling, because changing this is a regional-identity
/// change and bodies change between epochs rather than during them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    /// Stretches head to tail. At least one.
    pub tagmata: Vec<Tagma>,
    /// How far an individual's segment count may stray per tagma, either way.
    pub variance: u8,
    /// Appendage kinds this line has acquired and may assign.
    ///
    /// Ordered so iteration and serialization are deterministic.
    lexicon: BTreeSet<Appendage>,
    pub layout: Vec<Stretch>,
    /// Explicit appendage chains, indexed by tagma. Empty means the legacy
    /// layout-only recipe representation and retains its old wire shape.
    pub appendage_chains: Vec<Vec<AppendageStep>>,
}

/// Why an assignment was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unspeakable {
    /// The line has never eaten anything with this appendage.
    NotInLexicon(Appendage),
    /// No such stretch.
    NoSuchTagma(usize),
    /// Layout parent indices are encoded as `u8`.
    TooManyTagmata,
}

impl Recipe {
    /// The simplest body there is: one bare stretch.
    ///
    /// Every lineage starts here and grows a vocabulary by eating.
    pub fn founding(segments: u8) -> Self {
        let mut lexicon = BTreeSet::new();
        for appendage in Appendage::ALL {
            if appendage.is_innate() {
                lexicon.insert(appendage);
            }
        }
        Self {
            tagmata: vec![Tagma::bare(segments.max(1))],
            variance: 1,
            lexicon,
            layout: Vec::new(),
            appendage_chains: Vec::new(),
        }
    }

    /// The recipe a lineage has before worldgen gives it one.
    pub fn default_founding() -> Self {
        Self::founding(4)
    }

    /// Builds a recipe directly. Used by the catalogue and by tests; a played
    /// lineage reaches these shapes by acquisition and mutation.
    pub fn of(tagmata: Vec<Tagma>) -> Self {
        let mut lexicon: BTreeSet<Appendage> = Appendage::ALL
            .into_iter()
            .filter(|a| a.is_innate())
            .collect();
        for tagma in &tagmata {
            lexicon.insert(tagma.appendage);
        }
        Self {
            tagmata,
            variance: 1,
            lexicon,
            layout: Vec::new(),
            appendage_chains: Vec::new(),
        }
    }

    pub fn with_layout(mut self, layout: Vec<Stretch>) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_appendage_chains(mut self, appendage_chains: Vec<Vec<AppendageStep>>) -> Self {
        self.appendage_chains = appendage_chains;
        self
    }

    pub fn appendage_chain_for(&self, tagma: usize) -> Option<&[AppendageStep]> {
        self.appendage_chains.get(tagma).map(Vec::as_slice)
    }

    pub fn layout_for(&self, tagma: usize) -> Option<Stretch> {
        self.layout.get(tagma).copied()
    }

    pub fn lexicon(&self) -> impl Iterator<Item = Appendage> + '_ {
        self.lexicon.iter().copied()
    }

    pub fn can_express(&self, appendage: Appendage) -> bool {
        self.lexicon.contains(&appendage)
    }

    /// Learns an appendage kind by having eaten something that had it.
    ///
    /// **The acquisition half of kleptoplasty.** Returns whether this was new,
    /// which is what makes a meal a discovery rather than a calorie.
    pub fn acquire(&mut self, appendage: Appendage) -> bool {
        self.lexicon.insert(appendage)
    }

    /// Homeosis: change what a stretch grows.
    ///
    /// Refused when the line cannot say the word, which is the rule that makes
    /// a lexicon matter.
    pub fn assign(&mut self, tagma: usize, appendage: Appendage) -> Result<(), Unspeakable> {
        if !self.can_express(appendage) {
            return Err(Unspeakable::NotInLexicon(appendage));
        }
        let Some(target) = self.tagmata.get_mut(tagma) else {
            return Err(Unspeakable::NoSuchTagma(tagma));
        };
        let changed = target.appendage != appendage;
        target.appendage = appendage;
        if target.per_segment == 0 {
            target.per_segment = 1;
        }
        if changed {
            if let Some(chain) = self.appendage_chains.get_mut(tagma) {
                chain.clear();
            }
        }
        Ok(())
    }

    /// Splits one stretch into two, the mutation that makes a body regional.
    ///
    /// A one-tagma worm becoming a head-and-trunk creature is this, and it is
    /// how tagmatization actually arises.
    pub fn divide(&mut self, tagma: usize, at: u8) -> Result<usize, Unspeakable> {
        if !self.layout.is_empty() && self.tagmata.len() >= u8::MAX as usize {
            return Err(Unspeakable::TooManyTagmata);
        }
        let Some(target) = self.tagmata.get_mut(tagma) else {
            return Err(Unspeakable::NoSuchTagma(tagma));
        };
        let old_segments = target.segments;
        let front = at.clamp(1, old_segments.saturating_sub(1).max(1));
        let back = target.segments.saturating_sub(front).max(1);
        let tail = Tagma {
            segments: back,
            ..*target
        };
        target.segments = front;
        self.tagmata.insert(tagma + 1, tail);
        if !self.appendage_chains.is_empty() {
            let chain = self
                .appendage_chains
                .get(tagma)
                .cloned()
                .unwrap_or_default();
            self.appendage_chains.insert(tagma + 1, chain);
        }
        if !self.layout.is_empty() {
            crate::axis::recipe_wire::divide_layout(&mut self.layout, tagma, front, old_segments);
        }
        Ok(tagma + 1)
    }

    /// Total segments the recipe calls for, before individual variation.
    pub fn segments(&self) -> u32 {
        self.tagmata.iter().map(|t| t.segments as u32).sum()
    }

    /// Total appendages the recipe calls for.
    pub fn appendages(&self) -> u32 {
        self.tagmata.iter().map(Tagma::appendage_count).sum()
    }

    /// How elaborate the recipe is.
    ///
    /// **Repetition is cheap; vocabulary and regionalisation are expensive.**
    /// Serial homology means each extra identical segment adds almost no
    /// information, so a hundred-segment worm is long rather than elaborate,
    /// while a short creature that expresses five kinds of appendage across
    /// five stretches is genuinely intricate. Segments therefore carry a
    /// repetition discount, stretches carry more, and distinct appendage
    /// kinds carry most.
    ///
    /// This is the axis the complexity frontier reads, replacing raw part
    /// count: reaching a new lineage should mean reaching a richer *recipe*,
    /// not a longer one.
    pub fn complexity(&self) -> u32 {
        let kinds: BTreeSet<Appendage> = self
            .tagmata
            .iter()
            .map(|t| t.appendage)
            .filter(|a| *a != Appendage::None)
            .collect();
        // The lexicon counts too, at half weight: a line that *could* grow
        // five kinds has come further than one that could grow one, even
        // before it assigns them. This is also what keeps the frontier
        // connected to play, now that a bigger body no longer raises it:
        // eating something new teaches a word, and the ceiling lifts.
        let vocabulary = self.lexicon.iter().filter(|a| !a.is_innate()).count() as u32;
        self.segments() / 8
            + self.tagmata.len() as u32 * 4
            + kinds.len() as u32 * 8
            + vocabulary * 4
    }
}

/// One individual's realised body: the lineage's theme, varied.
///
/// Kin resemble each other because they share a recipe, not because they
/// share a shape. Vertebral count differs between snakes of one species and
/// humans vary in rib count; this is that.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Soma {
    /// Realised segment count per tagma, in the lineage's order.
    pub segments: Vec<u8>,
    /// Which segments lost their appendage, as (tagma, index within it).
    /// Development is imperfect, and an individual missing one limb is the
    /// cheapest evidence that individuals are not clones.
    pub absent: Vec<(u8, u8)>,
}

mod soma;

/// Seeds a lineage's recipe from a stream.
///
/// **Worldgen's job, not the catalogue's.** A seeded world's lines are not
/// centipedes and insects; they are their own creatures drawn from the same
/// rules, which is the difference between a generator and a bestiary. Kingdom
/// shapes the draw: producers grow long fronded stands and nothing that
/// contracts, consumers get a mouth and stretches and limbs, decomposers get
/// the limbs without the mouth.
///
/// **Plants are big** (2026-08-29 TD7). An unlimbed line draws more stretches
/// and longer ones, because determinate growth derives an adult mass from the
/// parts a recipe calls for, and TD6 measured the consequence of a producer
/// recipe realizing ~5 parts against a consumer's ~24: a grazer could outgrow
/// its own food's adult size by four times, an inverted pyramid written into
/// the body plans. Nothing new is invented to fix it — the same axial rules,
/// asked for a larger stand.
///
/// # The draw has to make the kingdom it was asked for (DC1.5)
///
/// A kingdom is read off feeding anatomy now, so this generator is the thing
/// that decides one: the tier no longer authors a `Kingdom` field that a body
/// then wears. A producer draw must carry a fixing part and no mouth, a
/// consumer draw a mouth of one geometry or the other, and a decomposer draw
/// neither — a body that absorbs across its bulk. **Transitional**: this is
/// worldgen's own lottery, kept until DC2's archetypes replace it, and it is
/// what keeps genesis founding a real pyramid in the meantime.
///
/// **The mouth follows the legs, and that is deliberate.** A line that draws
/// pursuit machinery draws the mouth to use it; one that draws none crops. It
/// is the natural authoring rule — an animal that chases has jaws and an animal
/// that does not has a crop — and it is also the *null change*: the pre-DC1.5
/// world read predator from "any part performs `Contract`", so a founding world
/// keeps exactly the feeding modes it always had while the *reason* moves from
/// the legs to the mouth. Drawing the mouth independently instead is a real
/// ecology change — it founds mobile grazers, which never existed — and the
/// instrument says what that costs; see the plan's DC1.5 findings.
pub fn seed(rng: &mut Rng, kingdom: Kingdom) -> Recipe {
    let rooted = kingdom == Kingdom::Producer;
    let stretches = 1 + rng.below(if rooted { 3 } else { 4 }) as usize;
    let mut tagmata = Vec::with_capacity(stretches + 1);

    // A producer needs its fixing stretch, so the draw is told which one it is
    // rather than asked to roll for it. Everything else about the stretch is
    // still drawn.
    let fixing = if rooted {
        rng.below(stretches as u64) as usize
    } else {
        usize::MAX
    };

    for index in 0..stretches {
        let segments = if rooted {
            4 + rng.below(8)
        } else {
            1 + rng.below(6)
        } as u8;
        let appendage = if rooted {
            // Fronds and bare stretches, and nothing that contracts: a stand
            // that grew an actuator would stop being sessile, which is the
            // rent asymmetry TD7 founded. The draw is taken either way so the
            // stream advances the same amount per stretch.
            let drawn = rng.below(3) == 0;
            if index == fixing || drawn {
                Appendage::Plate
            } else {
                Appendage::None
            }
        } else {
            // Most stretches are bare; the ones that are not carry limbs,
            // and rarely something else. Sparse assignment is what makes a
            // silhouette read rather than bristle. **No plates**: a plate
            // fixes now, and a body that both fixed and ate would be the
            // mixotroph the rulings register defers.
            match rng.below(6) {
                0 | 1 => Appendage::Limb,
                2 if index == 0 => Appendage::Feeler,
                _ => Appendage::None,
            }
        };
        let tagma = if appendage == Appendage::None {
            Tagma::bare(segments)
        } else {
            Tagma::new(segments, appendage).with_per_segment(1 + (rng.below(8) == 0) as u8)
        };
        tagmata.push(tagma);
    }

    // A head, always: something has to be the front. What it bears is what the
    // world will read this line as — a mouth for a consumer, nothing for the
    // two kingdoms that do not take a living meal through one.
    tagmata.insert(
        0,
        match kingdom {
            Kingdom::Consumer => {
                let pursues = tagmata
                    .iter()
                    .any(|t| matches!(t.appendage, Appendage::Limb | Appendage::Vane));
                Tagma::new(1, Appendage::Mouth).with_shapes(0, if pursues { JAW_SHAPE } else { 0 })
            },
            Kingdom::Producer | Kingdom::Decomposer => Tagma::bare(1),
        },
    );
    let mut recipe = Recipe::of(tagmata);
    recipe.variance = 1 + rng.below(2) as u8;
    recipe
}

pub mod archetype;
pub mod catalogue;
pub mod recipe_wire;
pub use recipe_wire::{Anchor, Stretch};
#[cfg(test)]
mod tests;

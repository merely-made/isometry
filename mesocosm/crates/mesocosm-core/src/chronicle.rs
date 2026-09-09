// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a creature is when it is not a body: a lineage, what it was made of,
//! and what happened to it.
//!
//! A body profile goes out; a chronicle comes back. Isometry does not return geometry, because the
//! wing's first law says morphology does not travel — what travels is choices
//! under scarcity. So a creature that goes to another game and lives a while
//! returns as a record, and this game regrows the body from it under its own
//! rules.
//!
//! # The protocol keystone, in three properties
//!
//! The founding record states it as **additive facts, opaque preservation,
//! deferred interpretation**. Each is a property of this type rather than a
//! convention around it:
//!
//! - **Additive.** [`append`](Chronicle::append) is the only way to change a
//!   chronicle. There is no edit and no delete, so no game can mutate another
//!   game's facts by accident, and set union is a legal merge.
//! - **Opaque preservation.** A [`Deed`] carries the appending game's own
//!   vessel name, its own verb, and a payload nobody else parses. This game
//!   keeps every deed it cannot read, byte for byte, and hands them back on
//!   the way out. Fact loss is the failure mode that makes a pipeline feel
//!   fake, and it happens by omission rather than by decision.
//! - **Deferred interpretation.** Re-entry is interpretation, not merging.
//!   Isometry writes "lost an arm at the ford" in Isometry's vocabulary; this
//!   game reads it and derives the *morphological* consequence itself, because
//!   only Mesocosm knows what an arm is here.
//!
//! # Why this is where Law C is proven
//!
//! Law C says the same seed format serves whether a creature was authored by
//! play or by RNG, and that the consuming game must not be able to tell them
//! apart structurally — only the player can, by pointing.
//!
//! That is a claim about *this type having no such field*, which is easy to
//! state and easy to violate later by adding a helpful `is_player_made` flag.
//! The generator in this module therefore produces chronicles with real
//! provenance, so a generated creature has a history of the same shape as a
//! played one, and `tests/proof_pair.rs` asserts the two are indistinguishable.

use serde::{Deserialize, Serialize};

use crate::axis::{Recipe, Soma};
use crate::body::{BodyDocument, Origin, PartId, Provenance, SpeciesId};
use crate::development::{DevelopmentError, PartPalette, develop_body};
use crate::rng::Rng;
use crate::wire::{WireError, frame, unframe};

/// The chronicle schema.
pub const CHRONICLE_SCHEMA: &str = "mesocosm.chronicle/v0";

/// Schema magic. See [`crate::wire`] for why this sits outside the payload.
pub const CHRONICLE_MAGIC: [u8; 8] = *b"MESOCHRN";

/// The only version this build accepts.
pub const CHRONICLE_VERSION: u16 = 0;

/// Where one part came from.
///
/// The wire form of [`Provenance`]. Flat on purpose: `None` for both fields
/// means the part was there at founding, and a foreign reader needs no enum
/// from this crate to tell that from a part that was taken off somebody.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct PartOrigin {
    /// The species this part was taken from. `None` at founding.
    pub from_species: Option<u32>,
    /// The part's identity in the body it was taken from. `None` at founding.
    pub from_part: Option<u32>,
    /// The epoch during which this part joined the body.
    pub epoch: u64,
}

impl PartOrigin {
    /// Whether this part was taken from another organism.
    pub fn is_incorporated(&self) -> bool {
        self.from_species.is_some()
    }
}

impl From<&Provenance> for PartOrigin {
    fn from(provenance: &Provenance) -> Self {
        match provenance.origin {
            Origin::Founding => Self {
                from_species: None,
                from_part: None,
                epoch: provenance.epoch,
            },
            Origin::Incorporated {
                from_species,
                from_part,
            } => Self {
                from_species: Some(from_species.0),
                from_part: Some(from_part.0),
                epoch: provenance.epoch,
            },
        }
    }
}

impl From<&PartOrigin> for Provenance {
    fn from(origin: &PartOrigin) -> Self {
        match (origin.from_species, origin.from_part) {
            (Some(species), Some(part)) => Provenance {
                origin: Origin::Incorporated {
                    from_species: SpeciesId(species),
                    from_part: PartId(part),
                },
                epoch: origin.epoch,
            },
            _ => Provenance {
                origin: Origin::Founding,
                epoch: origin.epoch,
            },
        }
    }
}

/// One thing that happened, recorded by whoever it happened to.
///
/// Every field except `at` is the appending game's own vocabulary. Readers
/// that do not recognise a `vessel` or a `verb` must keep the deed anyway, and
/// must never guess at `detail`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Deed {
    /// Which game appended this, named however that game names itself.
    pub vessel: String,
    /// What happened, in the appending game's words.
    pub verb: String,
    /// Causal order, not wall time. Cross-game clocks are unresolvable any
    /// other way: a colony decade and a tactics session have no shared second.
    pub at: u64,
    /// The appending game's payload. Opaque to every other game, preserved by
    /// all of them.
    pub detail: Vec<u8>,
}

impl Deed {
    /// A deed with no payload.
    pub fn new(vessel: impl Into<String>, verb: impl Into<String>, at: u64) -> Self {
        Self {
            vessel: vessel.into(),
            verb: verb.into(),
            at,
            detail: Vec::new(),
        }
    }

    /// A deed carrying a payload only its author understands.
    pub fn detailed(
        vessel: impl Into<String>,
        verb: impl Into<String>,
        at: u64,
        detail: Vec<u8>,
    ) -> Self {
        Self {
            vessel: vessel.into(),
            verb: verb.into(),
            at,
            detail,
        }
    }
}

/// A creature as a record: what lineage, what it was made of, what happened.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct Chronicle {
    pub species: u32,
    /// What the body was made of when it was last a body here.
    pub parts: Vec<PartOrigin>,
    /// Everything that happened, in append order.
    pub deeds: Vec<Deed>,
}

/// What this game makes of a foreign deed. Interpretation is deliberately
/// separate from the deed itself: the same record means different things to
/// different vessels, and none of them may rewrite it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consequence {
    /// A part is gone. The descendant is founded without it.
    LostPart { part: u32 },
    /// This game has no rule for this deed. It is kept, not dropped.
    Unread,
}

/// The verb this game reads. Anything else is [`Consequence::Unread`].
pub const LOST_PART: &str = "lost-part";

impl Chronicle {
    /// The record of a body as it stands.
    pub fn of(body: &BodyDocument) -> Self {
        Self {
            species: body.species.0,
            parts: body
                .parts
                .iter()
                .map(|part| PartOrigin::from(&part.provenance))
                .collect(),
            deeds: Vec::new(),
        }
    }

    /// Appends a fact. The only mutation there is.
    pub fn append(&mut self, deed: Deed) {
        self.deeds.push(deed);
    }

    /// How many parts were taken off other organisms.
    pub fn incorporated_parts(&self) -> usize {
        self.parts
            .iter()
            .filter(|part| part.is_incorporated())
            .count()
    }

    /// Deeds this game has a rule for, paired with what it makes of them.
    ///
    /// Note what this does *not* do: it does not remove anything. A reader
    /// interprets and the record stays whole, so the next game sees what this
    /// one saw.
    pub fn read(&self) -> impl Iterator<Item = (&Deed, Consequence)> {
        self.deeds.iter().map(|deed| (deed, interpret(deed)))
    }

    /// Deeds this game cannot interpret and is carrying for somebody else.
    pub fn unread(&self) -> impl Iterator<Item = &Deed> {
        self.deeds
            .iter()
            .filter(|deed| interpret(deed) == Consequence::Unread)
    }

    /// Founds the next body in this lineage, applying what this game can read.
    ///
    /// This is deferred interpretation in one call: another game recorded that
    /// something was lost, in its own words, and Mesocosm decides here what
    /// losing it means to a body. The chronicle is not consumed — the
    /// descendant carries the whole record forward, including the deeds this
    /// game could not read.
    pub fn found(
        &self,
        recipe: &Recipe,
        development_seed: u64,
        mass_mg: u64,
        palette: PartPalette,
    ) -> Result<BodyDocument, DevelopmentError> {
        let lost: Vec<u32> = self
            .read()
            .filter_map(|(_, consequence)| match consequence {
                Consequence::LostPart { part } => Some(part),
                Consequence::Unread => None,
            })
            .collect();

        let soma = Soma::develop(recipe, development_seed);
        let mut body = develop_body(SpeciesId(self.species), recipe, &soma, mass_mg, palette)?;

        // Geometry and topology come from the local developmental program.
        // Historical origins map by stable ordinal where the new body has a
        // site; origins without one remain in the chronicle, while additional
        // sites are founding tissue. Interpreted losses then tombstone the
        // locally grown subtree at that address. No foreign geometry travels.
        for (part, origin) in body.parts.iter_mut().zip(&self.parts) {
            part.provenance = Provenance::from(origin);
        }
        for part in lost {
            body.sever(PartId(part));
        }
        Ok(body)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, WireError> {
        frame(CHRONICLE_MAGIC, CHRONICLE_VERSION, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        let chronicle: Self = unframe(CHRONICLE_MAGIC, CHRONICLE_VERSION, bytes)?;
        if chronicle.parts.is_empty() {
            return Err(WireError::Inconsistent);
        }
        Ok(chronicle)
    }
}

/// What this game makes of one deed.
fn interpret(deed: &Deed) -> Consequence {
    if deed.verb != LOST_PART {
        return Consequence::Unread;
    }
    match <[u8; 4]>::try_from(deed.detail.as_slice()) {
        Ok(bytes) => Consequence::LostPart {
            part: u32::from_le_bytes(bytes),
        },
        // The verb is ours and the payload is not what we expect. Refusing to
        // guess is the point: a malformed detail is retained uninterpreted
        // rather than turned into a plausible part index.
        Err(_) => Consequence::Unread,
    }
}

/// Generates a creature nobody played.
///
/// **This is the load-bearing half of Law C.** An RNG critter that arrived as
/// a blank slate would be trivially distinguishable from a played one — the
/// played one would have provenance and the generated one would not — and the
/// no-homework guarantee would be decoration. So a generated creature is
/// generated *with a history*: it ate things too, off species that exist in
/// the world, in epochs that make sense.
///
/// **The range matters as much as the provenance.** A generator that always
/// made small creatures would leave *size* as an origin tell: a consumer could
/// guess "forty parts, so somebody played this" without reading a marker,
/// which defeats the law just as thoroughly as a marker would. The
/// distributions have to overlap rather than merely touch.
///
/// **This ceiling is coupled to the world's economics and must be kept in step
/// with them.** Upkeep scales with body mass, so there is a size past which a
/// creature cannot pay its own rent, and a generated creature larger than play
/// can sustain is not one the world could have produced. Recipe-developed
/// founders restored structural bodies in the low dozens before incorporation,
/// so the generator spans two through thirty-nine parts. Isometry's
/// `size_does_not_give_the_played_one_away` is the tripwire; when live anatomy
/// moves, this range must be measured again rather than treated as lore.
///
/// Deterministic from `seed`, so a generated lineage is reproducible.
pub fn generate(seed: u64, species: u32) -> Chronicle {
    let mut rng = Rng::from_seed(seed);
    let parts = 2 + rng.below(38) as usize;

    let mut chronicle = Chronicle {
        species,
        parts: Vec::with_capacity(parts),
        deeds: Vec::new(),
    };
    chronicle.parts.push(PartOrigin {
        from_species: None,
        from_part: None,
        epoch: 0,
    });

    let mut epoch = 0;
    for _ in 1..parts {
        epoch += 1 + rng.below(4);
        chronicle.parts.push(PartOrigin {
            from_species: Some(rng.below(64) as u32),
            from_part: Some(rng.below(4) as u32),
            epoch,
        });
    }
    chronicle
}

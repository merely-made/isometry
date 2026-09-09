// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Develops an axial recipe into the authoritative parts graph.
//!
//! [`Recipe`](crate::axis::Recipe) and [`Soma`](crate::axis::Soma) used to stop
//! at a renderer-only capsule body. That left the heritable program and the
//! organism's actual anatomy unrelated. This module is the join: segments and
//! appendages become ordinary [`Part`](crate::body::Part)s, so every later
//! projection reads the same body document.
//!
//! # Shape vocabulary
//!
//! A world admits [`PALETTE_SHAPES`] shapes per [`Role`], and a
//! [`Tagma`] names which of them its segments and its appendages are made
//! of. Before that a role admitted exactly one shape, so no body could hold
//! more than four — which is why bodies read as repeated blocks rather than as
//! creatures. Selector zero is still every role's default, so a recipe that
//! names no shape develops exactly the body it always did.
//!
//! The palette is world state and the recipe is the heritable program, which
//! is the boundary that keeps this one authority: a lineage carries *which*
//! shape, a world carries *what that shape is*.

use crate::axis::{Anchor, Appendage, AppendageStep, Recipe, Soma, Tagma};
use crate::body::{Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};
use crate::plan::{Facing, Role, classify};
use serde::{Deserialize, Serialize};

/// The shape and content address used when development expresses one role.
///
/// The core owns the role and exact integer shape, while an asset-bearing host
/// supplies the content address. This prevents development from inventing
/// renderer assets or assuming that one volume format is universal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartTemplate {
    pub volume: VolumeRef,
    pub half_extent: [i32; 3],
}

/// How many shapes one world admits for one role.
///
/// **Four.** The most shape-hungry body the default creatures plan sizes
/// (§2.4's carving B) spends three `Mass` shapes — trunk, head, snout — and one
/// each of `Limb`, `Plate` and `Sensor`, so four leaves a spare per role. It
/// also keeps the palette a small `Copy` value that a `World` snapshots
/// whole. Widening the vocabulary later is this one constant.
pub const PALETTE_SHAPES: usize = 4;

/// The shapes a world admits for one role.
///
/// The default is a plain field rather than slot zero of an array, so "every
/// role always has one shape" is a fact of the type instead of a rule
/// `validate` has to enforce and every reader has to trust.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleShapes {
    /// Selector 0: what a recipe that names no shape is built from.
    pub default: PartTemplate,
    /// Selectors 1..; `None` is a slot this world does not admit.
    pub extra: [Option<PartTemplate>; PALETTE_SHAPES - 1],
}

impl RoleShapes {
    /// A role that admits one shape, which is what every role admitted before
    /// the palette widened.
    pub fn only(default: PartTemplate) -> Self {
        Self {
            default,
            extra: [None; PALETTE_SHAPES - 1],
        }
    }

    /// Admits one more shape, in the next free slot. Authoring-time only: a
    /// palette asking for more shapes than a world holds is an authoring
    /// mistake, not a runtime condition.
    pub fn and(mut self, template: PartTemplate) -> Self {
        let slot = self
            .extra
            .iter()
            .position(Option::is_none)
            .expect("a role admits PALETTE_SHAPES shapes");
        self.extra[slot] = Some(template);
        self
    }

    /// The shape a selector names, or the default when this world does not
    /// admit that slot.
    pub fn at(self, shape: u8) -> PartTemplate {
        match usize::from(shape).checked_sub(1) {
            None => self.default,
            Some(index) => self
                .extra
                .get(index)
                .copied()
                .flatten()
                .unwrap_or(self.default),
        }
    }

    /// Every shape this world actually admits for the role.
    pub fn admitted(self) -> impl Iterator<Item = PartTemplate> {
        std::iter::once(self.default).chain(self.extra.into_iter().flatten())
    }
}

impl From<PartTemplate> for RoleShapes {
    fn from(template: PartTemplate) -> Self {
        Self::only(template)
    }
}

/// Templates available to one developmental realization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartPalette {
    pub mass: RoleShapes,
    pub limb: RoleShapes,
    pub plate: RoleShapes,
    pub sensor: RoleShapes,
}

impl PartPalette {
    /// The current world's baseline admitted part vocabulary.
    ///
    /// The references are fixture content addresses while the project has no
    /// pack loader. Keeping the palette as world state means another world can
    /// admit different materials without changing a lineage's recipe.
    ///
    /// One shape per role: this is the vocabulary bodies were built from
    /// before the palette could hold more, kept exactly so, so that widening
    /// the machinery changed no body.
    pub fn primitive() -> Self {
        Self {
            mass: RoleShapes::only(PartTemplate {
                volume: VolumeRef::from_tag(1),
                half_extent: [2, 2, 2],
            }),
            limb: RoleShapes::only(PartTemplate {
                volume: VolumeRef::from_tag(2),
                half_extent: [4, 1, 1],
            }),
            plate: RoleShapes::only(PartTemplate {
                volume: VolumeRef::from_tag(3),
                half_extent: [4, 4, 1],
            }),
            sensor: RoleShapes::only(PartTemplate {
                volume: VolumeRef::from_tag(4),
                half_extent: [1, 1, 1],
            }),
        }
    }

    pub fn shapes(self, role: Role) -> RoleShapes {
        match role {
            Role::Mass => self.mass,
            Role::Limb => self.limb,
            Role::Plate => self.plate,
            Role::Sensor => self.sensor,
        }
    }

    /// The role's default shape.
    pub fn template(self, role: Role) -> PartTemplate {
        self.shapes(role).default
    }

    /// The shape a recipe's selector names for this role.
    pub fn template_at(self, role: Role, shape: u8) -> PartTemplate {
        self.shapes(role).at(shape)
    }

    fn validate(self) -> Result<(), DevelopmentError> {
        for role in Role::ALL {
            for template in self.shapes(role).admitted() {
                let actual = classify(template.half_extent);
                if actual != role {
                    return Err(DevelopmentError::WrongRole {
                        expected: role,
                        actual,
                    });
                }
                if overpriced(role, template.half_extent) {
                    return Err(DevelopmentError::Overpriced {
                        role,
                        half_extent: template.half_extent,
                    });
                }
            }
        }
        Ok(())
    }
}

impl Default for PartPalette {
    fn default() -> Self {
        Self::primitive()
    }
}

/// The primitive `Limb` `[4,1,1]`: span 4 against a 64 mg ceiling, a build
/// price of 6.25. TD7's stated bound — "a body made of nothing but limbs reads
/// `4 * 100 / 64`, so no anatomy can price itself past ~7x" — *is* this shape,
/// written as if it were a property of the game.
const LIMB_PRICE_BOUND: (u64, u64) = (4, 64);
/// The primitive `Sensor` `[1,1,1]`: span 1 against a 21 mg ceiling, 4.76.
/// TD11's "no anatomy may see the enclosure, 46 voxels" is `8 * (1 + 4.76)`.
const SENSOR_PRICE_BOUND: (u64, u64) = (1, 21);

/// The `(span, ceiling_mg)` a role's shapes are held to.
///
/// `None` where the economy cannot read a shape at all: `Mass` has no span
/// term and `Plate` performs no process, so their detail is free.
fn price_bound(role: Role) -> Option<(u64, u64)> {
    match role {
        Role::Limb => Some(LIMB_PRICE_BOUND),
        Role::Sensor => Some(SENSOR_PRICE_BOUND),
        Role::Mass | Role::Plate => None,
    }
}

/// A part's contribution to a span: its longest half-extent, the same term
/// `Organism::actuator_span` and `Organism::sensor_span` sum.
fn span_voxels(half_extent: [i32; 3]) -> u64 {
    half_extent
        .iter()
        .map(|v| u64::from(v.unsigned_abs()))
        .max()
        .unwrap_or(0)
}

/// Whether this shape would price a body past the palette the whole TD series
/// was tuned against.
///
/// **Build price** is `100 * longest_half_extent / part_ceiling_mg`: what one
/// part of this shape adds to a body's build multiple per milligram of body it
/// hangs on. Ceiling is cubic in half-extent and span is linear, so the price
/// scales as `1 / cross-sectional area` — a limb thinned from a 3x3 section to
/// 1x1 raises TD7's ~7x ceiling to 61x in silence. Compared by
/// cross-multiplication so the guard stays integer-exact, and read off
/// `part_ceiling_mg` itself rather than a copy of its formula. (Plan §2.3)
fn overpriced(role: Role, half_extent: [i32; 3]) -> bool {
    let Some((bound_span, bound_ceiling)) = price_bound(role) else {
        return false;
    };
    let ceiling = crate::organism::ecology::part_ceiling_mg(half_extent);
    span_voxels(half_extent) * bound_ceiling > bound_span * ceiling
}

mod error;
pub use error::DevelopmentError;

/// Realizes one individual's anatomy from its lineage recipe.
///
/// Axial segments form the dependency spine. Appendages attach to the segment
/// that expressed them, so severing a segment cascades through everything it
/// bore. Every part has positive mass and the result conserves `mass_mg`
/// exactly. The initial proof uses founding provenance; learned-source
/// addresses remain the later filial-provenance gate.
pub fn develop_body(
    species: SpeciesId,
    recipe: &Recipe,
    soma: &Soma,
    mass_mg: u64,
    palette: PartPalette,
) -> Result<BodyDocument, DevelopmentError> {
    palette.validate()?;
    let count = minimum_body_mass_mg(recipe, soma)?;
    if mass_mg < u64::from(count) {
        return Err(DevelopmentError::InsufficientMass {
            mass_mg,
            parts: count,
        });
    }
    let each = mass_mg / u64::from(count);
    let root_mass = each + mass_mg % u64::from(count);
    // The root is the first stretch's first segment, so it is made of that
    // stretch's shape rather than the role's default.
    let root_template = palette.template_at(
        Role::Mass,
        recipe
            .tagmata
            .first()
            .map_or(0, |tagma| tagma.segment_shape),
    );
    let mut body = BodyDocument::new(
        species,
        root_template.volume,
        root_mass,
        root_template.half_extent,
    );

    let mut previous_segment = body.root;
    let mut stretches: Vec<Vec<crate::body::PartId>> = Vec::new();
    let mut first = true;
    for (tagma_index, tagma) in recipe.tagmata.iter().enumerate() {
        let realised = soma.segments[tagma_index];
        let segment_template = palette.template_at(Role::Mass, tagma.segment_shape);
        let stretch = recipe
            .layout_for(tagma_index)
            .unwrap_or(crate::axis::Stretch {
                parent: None,
                anchor: Anchor::Tip,
                facing: Facing::Back,
                variance: None,
            });
        let mut realised_segments = Vec::with_capacity(usize::from(realised));
        for segment_index in 0..realised {
            let segment = if first {
                first = false;
                body.root
            } else {
                let parent = if segment_index == 0 {
                    match stretch.parent {
                        None => previous_segment,
                        Some(parent) => anchor(&stretches[parent as usize], stretch.anchor),
                    }
                } else {
                    *realised_segments
                        .last()
                        .expect("a non-first segment follows its own stretch")
                };
                let previous_half = body
                    .part(parent)
                    .expect("development only records attached segments")
                    .half_extent;
                let offset =
                    segment_offset(previous_half, segment_template.half_extent, stretch.facing);
                body.attach(
                    segment_template.volume,
                    each,
                    segment_template.half_extent,
                    Attachment {
                        parent,
                        offset,
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )?
            };
            previous_segment = segment;
            realised_segments.push(segment);

            if is_absent(soma, tagma_index, segment_index) {
                continue;
            }
            attach_appendages(
                &mut body,
                segment,
                tagma,
                recipe
                    .appendage_chains
                    .get(tagma_index)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                each,
                palette,
            )?;
        }
        stretches.push(realised_segments);
    }

    debug_assert_eq!(body.len(), count as usize);
    debug_assert_eq!(body.total_mass_mg(), mass_mg);
    Ok(body)
}

/// The least biomass that can express this realization while keeping every
/// structural part positive-mass.
///
/// A birth below this floor waits for provisioning. It never clones anatomy
/// the parent did not pay for and never silently drops part of the recipe.
pub fn minimum_body_mass_mg(recipe: &Recipe, soma: &Soma) -> Result<u32, DevelopmentError> {
    validate_soma(recipe, soma)?;
    part_count(recipe, soma)
}

fn validate_soma(recipe: &Recipe, soma: &Soma) -> Result<(), DevelopmentError> {
    if recipe.tagmata.len() != soma.segments.len() {
        return Err(DevelopmentError::SomaLength {
            tagmata: recipe.tagmata.len(),
            realised: soma.segments.len(),
        });
    }
    if soma.segments.iter().all(|segments| *segments == 0) {
        return Err(DevelopmentError::EmptyAxis);
    }
    if !recipe.layout.is_empty() && recipe.layout.len() != recipe.tagmata.len() {
        return Err(DevelopmentError::LayoutLength {
            tagmata: recipe.tagmata.len(),
            layout: recipe.layout.len(),
        });
    }
    if !recipe.layout.is_empty() && soma.segments.first() == Some(&0) {
        return Err(DevelopmentError::LayoutEmptyRoot);
    }
    appendage_chain::validate(recipe)?;
    for (tagma, stretch) in recipe.layout.iter().enumerate() {
        if let Some(parent) = stretch.parent
            && usize::from(parent) >= tagma
        {
            return Err(DevelopmentError::InvalidLayoutParent {
                tagma,
                parent: usize::from(parent),
            });
        }
        if let Some(parent) = stretch.parent {
            let parent = usize::from(parent);
            if soma.segments[parent] == 0 {
                return Err(DevelopmentError::LayoutEmptyParent { tagma, parent });
            }
        }
    }
    for &(tagma, segment) in &soma.absent {
        let Some(realised) = soma.segments.get(tagma as usize) else {
            return Err(DevelopmentError::InvalidAbsence { tagma, segment });
        };
        if segment >= *realised {
            return Err(DevelopmentError::InvalidAbsence { tagma, segment });
        }
    }
    Ok(())
}

fn anchor(parts: &[crate::body::PartId], anchor: Anchor) -> crate::body::PartId {
    let index = match anchor {
        Anchor::Base => 0,
        Anchor::Middle => parts.len() / 2,
        Anchor::Tip => parts.len() - 1,
    };
    parts[index]
}

/// The face-to-face join for two segments along the stretch's declared axis.
/// This is deliberately only a placement change: it neither rotates a volume
/// nor changes its extent, role, price, or mass share.
fn segment_offset(parent: [i32; 3], child: [i32; 3], facing: Facing) -> [i32; 3] {
    let (axis, sign) = facing.axis();
    let mut offset = [0; 3];
    offset[axis] = sign * (parent[axis].abs() + child[axis].abs());
    offset
}

fn part_count(recipe: &Recipe, soma: &Soma) -> Result<u32, DevelopmentError> {
    let mut count = 0u32;
    for (tagma_index, tagma) in recipe.tagmata.iter().enumerate() {
        let realised = soma.segments[tagma_index];
        count = count
            .checked_add(u32::from(realised))
            .ok_or(DevelopmentError::TooManyParts)?;
        for segment in 0..realised {
            if is_absent(soma, tagma_index, segment) {
                continue;
            }
            let pieces = appendage_pieces(
                tagma,
                recipe
                    .appendage_chains
                    .get(tagma_index)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
            );
            count = count
                .checked_add(pieces)
                .ok_or(DevelopmentError::TooManyParts)?;
        }
    }
    if count == 0 {
        return Err(DevelopmentError::EmptyAxis);
    }
    Ok(count)
}

/// How many parts one stretch's appendage assignment makes per segment.
///
/// Paired for anything that grows on the flanks, single for anything borne on
/// the midline — which now includes a covering plate, since armour wraps a
/// body from both sides the way a limb pair does.
fn appendage_pieces(tagma: &Tagma, chain: &[AppendageStep]) -> u32 {
    let count = u32::from(tagma.per_segment)
        .checked_mul(u32::try_from(chain.len().max(1)).expect("chain length is bounded"))
        .expect("bounded chain count fits u32");
    match tagma.appendage {
        Appendage::None => 0,
        Appendage::Limb | Appendage::Feeler | Appendage::Vane => count * 2,
        Appendage::Plate if tagma.appendage.covers(tagma.appendage_shape) => count * 2,
        Appendage::Plate | Appendage::Mouth => count,
    }
}

fn is_absent(soma: &Soma, tagma: usize, segment: u8) -> bool {
    u8::try_from(tagma)
        .ok()
        .is_some_and(|tagma| soma.absent.contains(&(tagma, segment)))
}

fn attach_appendages(
    body: &mut BodyDocument,
    segment: crate::body::PartId,
    tagma: &Tagma,
    chain: &[AppendageStep],
    mass_mg: u64,
    palette: PartPalette,
) -> Result<(), DevelopmentError> {
    let appendage = tagma.appendage;
    let per_segment = tagma.per_segment;
    let Some(role) = appendage.role(tagma.appendage_shape) else {
        return Ok(());
    };
    if !chain.is_empty() {
        return appendage_chain::attach(body, segment, tagma, chain, mass_mg, palette);
    }
    // A mouth's selector picks its *role* as well as its shape, so it is mapped
    // back into that role's own bank before the palette sees it. (DC1.5)
    let template = palette.template_at(role, appendage.shape_index(tagma.appendage_shape));
    let segment_half = body
        .part(segment)
        .expect("development only addresses attached segments")
        .half_extent;

    // Flush against the segment's own face, on the axis this appendage is
    // borne on: limbs and coverings on the flanks as a mirrored pair, a plate
    // held above, a mouth borne below.
    let flush = |axis: usize, side: i32| {
        side * (segment_half[axis].abs() + template.half_extent[axis].abs())
    };
    let covers = appendage.covers(tagma.appendage_shape);
    for ordinal in 0..per_segment {
        let along = slot_offset(ordinal, per_segment, template.half_extent[2].abs());
        let sockets: &[[i32; 3]] = &match appendage {
            Appendage::None => continue,
            Appendage::Limb | Appendage::Feeler | Appendage::Vane => {
                vec![[flush(0, -1), 0, along], [flush(0, 1), 0, along]]
            },
            // **Where a plate hangs is what it means** (DC4). Held above the
            // segment it is a frond in the canopy; worn on the flanks it is
            // covering, and covering fixes nothing.
            Appendage::Plate if covers => {
                vec![[flush(0, -1), 0, along], [flush(0, 1), 0, along]]
            },
            Appendage::Plate => vec![[0, flush(1, 1), along]],
            Appendage::Mouth => vec![[0, flush(1, -1), along]],
        };
        for offset in sockets {
            body.attach(
                template.volume,
                mass_mg,
                template.half_extent,
                Attachment {
                    parent: segment,
                    offset: *offset,
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )?;
        }
    }
    Ok(())
}

fn slot_offset(ordinal: u8, count: u8, half: i32) -> i32 {
    (i32::from(ordinal) * 2 + 1 - i32::from(count)) * half.max(1)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "development/layout_tests.rs"]
mod layout_tests;

mod appendage_chain;

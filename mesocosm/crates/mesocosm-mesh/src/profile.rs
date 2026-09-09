// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The interchange artifact: a body, flattened, with its history attached.
//!
//! The product-free `wing-formats` library owns the primitive v0 layout.
//! This module projects Mesocosm's body and volumes into that layout, while
//! the tabletop independently adapts it into voxels and sprites. Neither
//! product needs the other's runtime to exchange a `mere.pack/v1` artifact.
//!
//! V0 crosses flattened appearance and provenance. The live body document
//! remains Mesocosm's authority; stable subject identity, parent topology and
//! body revisions belong to the separately planned v1 contract. Sharing the
//! wire layout does not share the games' capability or consequence rules.
//!
//! # Why the header is raw bytes
//!
//! The plan booked the cost of this seam honestly: drift between writer and
//! reader becomes a runtime failure rather than a build failure, so the
//! profile needs a version field and a refusal path from the first commit.
//!
//! A version field *inside* the serialized struct does not deliver that.
//! Postcard is not self-describing — it is a positional encoding with no field
//! tags — so when the payload layout changes, the decoder cannot reach the
//! version field to discover that it should have refused. It fails as a
//! malformed decode, or worse, succeeds and returns nonsense.
//!
//! So the schema tag and version sit in a **fixed-position header ahead of the
//! payload**, in a form that never changes: an 8-byte magic and a
//! little-endian `u16`. Those can be checked without decoding anything, which
//! is what makes [`ProfileError::UnknownVersion`] a real diagnosis instead of
//! a guess. The payload's shape is free to change underneath.
//!
//! # Why attribution is a second grid
//!
//! [`flatten`](crate::flatten) composes every part into one occupancy grid,
//! and in doing so it discards which part wrote each cell — the grid records
//! materials, not history. That is right for the mesher and wrong for this
//! seam, because the body's whole legibility rule is that **the world is
//! colour-coded by role and the player is colour-coded by history**. A sprite
//! baked from materials alone cannot show where a limb was taken from.
//!
//! So the profile carries a parallel [`attribution`](BodyProfile::attribution)
//! grid: same dimensions, each cell naming the part that wrote it. Provenance
//! is then recoverable per voxel, and a reader that does not care can ignore
//! it entirely.

use mesocosm_core::{BodyDocument, PartOrigin, wire};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

use crate::{MeshError, VolumeSource, flatten::flatten_attributed};

/// The schema this module reads and writes. Cited by name in a pack part so a
/// reader knows what it is holding before it opens it.
pub const PROFILE_SCHEMA: &str = wing_formats::BODY_SCHEMA;

/// Schema magic. See [`mesocosm_core::wire`] for why this sits outside the
/// payload rather than in a version field the decoder cannot reach.
pub const PROFILE_MAGIC: [u8; 8] = wing_formats::BODY_MAGIC;

/// The only version this build accepts.
pub const PROFILE_VERSION: u16 = wing_formats::BODY_VERSION;

/// Bytes before the payload: magic plus a little-endian `u16`.
pub const HEADER_LEN: usize = wing_formats::HEADER_LEN;

/// A body projected for crossing: the grid a baker can consume, and the
/// history that grid would otherwise lose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BodyProfile(pub wing_formats::BodyProfile);

impl Deref for BodyProfile {
    type Target = wing_formats::BodyProfile;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BodyProfile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Why a profile could not be read. The shared wire error, so every reader in
/// the wing refuses for the same reasons and handles one type.
pub type ProfileError = wire::WireError;

impl BodyProfile {
    /// Projects a body and the voxels its parts refer to into a crossable
    /// artifact.
    pub fn of(body: &BodyDocument, source: &impl VolumeSource) -> Result<Self, MeshError> {
        let (flattened, attribution) = flatten_attributed(body, source)?;
        let parts = body
            .parts
            .iter()
            .map(|part| PartOrigin::from(&part.provenance))
            .collect();

        Ok(Self(wing_formats::BodyProfile {
            species: body.species.0,
            size: flattened.volume.size,
            origin: flattened.origin,
            cells: flattened.volume.into_voxels(),
            attribution,
            parts,
        }))
    }

    /// Number of cells the grid claims, from its own dimensions.
    pub fn cell_count(&self) -> usize {
        self.size.iter().map(|d| *d as usize).product()
    }

    /// Which part wrote the cell at a body-space coordinate, as an index into
    /// [`parts`](Self::parts).
    pub fn part_at(&self, body_space: [i32; 3]) -> Option<usize> {
        let index = self.index(body_space)?;
        match self.attribution.get(index).copied() {
            Some(0) | None => None,
            Some(slot) => Some(slot as usize - 1),
        }
    }

    /// The history of whatever occupies a body-space coordinate.
    ///
    /// This is the reason attribution exists. A reader tinting a sprite by
    /// where its parts came from asks this per voxel.
    pub fn origin_at(&self, body_space: [i32; 3]) -> Option<PartOrigin> {
        self.parts.get(self.part_at(body_space)?).copied()
    }

    /// Reads the occupancy grid by body-space coordinate.
    pub fn material_at(&self, body_space: [i32; 3]) -> u8 {
        self.index(body_space)
            .and_then(|i| self.cells.get(i).copied())
            .unwrap_or(0)
    }

    /// Serializes behind the schema header.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProfileError> {
        wire::frame(PROFILE_MAGIC, PROFILE_VERSION, self)
    }

    /// Reads bytes written by [`to_bytes`](Self::to_bytes), refusing anything
    /// it cannot vouch for.
    ///
    /// Framing refuses on magic and version before touching the payload; this
    /// adds the checks only this schema can make. A well-formed decode can
    /// still be an incoherent document, and the three arrays are only useful
    /// together, so disagreement is a refusal rather than something every
    /// reader has to notice for itself.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProfileError> {
        let profile: Self = wire::unframe(PROFILE_MAGIC, PROFILE_VERSION, bytes)?;

        wing_formats::validate_body_profile(&profile.0)?;
        Ok(profile)
    }

    /// Flat cell index for a body-space coordinate, if it lands inside.
    fn index(&self, body_space: [i32; 3]) -> Option<usize> {
        let mut local = [0u32; 3];
        for axis in 0..3 {
            let value = body_space[axis] - self.origin[axis];
            if value < 0 || value as u32 >= self.size[axis] {
                return None;
            }
            local[axis] = value as u32;
        }
        Some((local[0] + local[1] * self.size[0] + local[2] * self.size[0] * self.size[1]) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Volume, VolumeMap};
    use mesocosm_core::{Attachment, Origin, PartId, Provenance, SpeciesId, VolumeRef, Yaw};

    /// A two-part body: a root, and one limb taken from another species.
    fn donated() -> (BodyDocument, VolumeMap) {
        let mut body = BodyDocument::new(SpeciesId(7), VolumeRef::from_tag(1), 1_000, [1, 1, 1]);
        body.attach(
            VolumeRef::from_tag(2),
            500,
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [2, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance {
                origin: Origin::Incorporated {
                    from_species: SpeciesId(42),
                    from_part: PartId(3),
                },
                epoch: 5,
            },
        )
        .expect("the limb attaches");

        let mut volumes = VolumeMap::default();
        volumes.insert(VolumeRef::from_tag(1), Volume::solid([3, 3, 3], 1));
        volumes.insert(VolumeRef::from_tag(2), Volume::solid([3, 3, 3], 2));
        (body, volumes)
    }

    fn profile() -> BodyProfile {
        let (body, volumes) = donated();
        BodyProfile::of(&body, &volumes).unwrap()
    }

    #[test]
    fn a_profile_round_trips() {
        let profile = profile();
        let bytes = profile.to_bytes().unwrap();
        assert_eq!(BodyProfile::from_bytes(&bytes).unwrap(), profile);
    }

    #[test]
    fn the_round_trip_keeps_part_provenance() {
        // Wave 1.4's done-condition, read strictly: the history has to survive
        // the crossing, not just the geometry.
        let read = BodyProfile::from_bytes(&profile().to_bytes().unwrap()).unwrap();

        assert_eq!(read.species, 7);
        assert_eq!(read.parts.len(), 2);
        assert!(
            !read.parts[0].is_incorporated(),
            "the root was there at founding"
        );
        assert_eq!(
            read.parts[1],
            PartOrigin {
                from_species: Some(42),
                from_part: Some(3),
                epoch: 5
            }
        );
    }

    #[test]
    fn the_profile_carries_no_core_types() {
        // The ruling this module exists to satisfy. Every field is a
        // primitive, a fixed-size array, or a Vec of those, so a reader can
        // mirror the struct without linking this game. If someone puts a core
        // type back on the wire, this is the test that should stop them: it
        // decodes the payload into a structurally identical local mirror that
        // knows nothing about mesocosm.
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct ForeignOrigin {
            from_species: Option<u32>,
            from_part: Option<u32>,
            epoch: u64,
        }
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct ForeignProfile {
            species: u32,
            size: [u32; 3],
            origin: [i32; 3],
            cells: Vec<u8>,
            attribution: Vec<u16>,
            parts: Vec<ForeignOrigin>,
        }

        let bytes = profile().to_bytes().unwrap();
        let foreign: ForeignProfile = postcard::from_bytes(&bytes[HEADER_LEN..]).unwrap();

        assert_eq!(foreign.species, 7);
        assert_eq!(foreign.parts.len(), 2);
        assert_eq!(
            foreign.parts[1],
            ForeignOrigin {
                from_species: Some(42),
                from_part: Some(3),
                epoch: 5
            }
        );
        assert_eq!(foreign.cells.len(), foreign.attribution.len());
    }

    #[test]
    fn provenance_is_recoverable_per_voxel() {
        // The reason attribution exists: a baker tinting by history asks this
        // question of a coordinate, not of a part list.
        let profile = profile();
        let mut founding = 0;
        let mut incorporated = 0;

        for z in 0..profile.size[2] as i32 {
            for y in 0..profile.size[1] as i32 {
                for x in 0..profile.size[0] as i32 {
                    let at = [
                        x + profile.origin[0],
                        y + profile.origin[1],
                        z + profile.origin[2],
                    ];
                    match profile.origin_at(at) {
                        Some(origin) if origin.is_incorporated() => incorporated += 1,
                        Some(_) => founding += 1,
                        None => {},
                    }
                }
            }
        }

        assert!(founding > 0, "the root's voxels are attributed to the root");
        assert!(
            incorporated > 0,
            "the donated limb's voxels carry its origin"
        );
    }

    #[test]
    fn attribution_agrees_with_occupancy_everywhere() {
        // The two grids are only useful together, so they must never disagree
        // about which cells are solid.
        let profile = profile();
        for (index, cell) in profile.cells.iter().enumerate() {
            assert_eq!(
                *cell != 0,
                profile.attribution[index] != 0,
                "cell {index} disagrees about being occupied"
            );
        }
    }

    #[test]
    fn empty_space_is_attributed_to_nobody() {
        let profile = profile();
        let far = [profile.origin[0] - 50, profile.origin[1], profile.origin[2]];
        assert_eq!(profile.part_at(far), None);
        assert_eq!(profile.origin_at(far), None);
        assert_eq!(profile.material_at(far), 0);
    }

    #[test]
    fn foreign_bytes_are_refused_as_not_a_profile() {
        assert_eq!(
            BodyProfile::from_bytes(b"NOTABODY and then some payload"),
            Err(ProfileError::WrongSchema {
                found: *b"NOTABODY",
                expected: PROFILE_MAGIC
            })
        );
    }

    #[test]
    fn a_short_read_is_refused_before_the_magic_check() {
        assert_eq!(
            BodyProfile::from_bytes(b"MESO"),
            Err(ProfileError::TooShort { got: 4 })
        );
    }

    #[test]
    fn a_future_version_is_diagnosed_rather_than_mis_decoded() {
        // The booked cost of coupling by data, paid: this is a profile, from a
        // build that does not agree with us, and the reader can say exactly
        // that.
        let mut bytes = profile().to_bytes().unwrap();
        bytes[8..10].copy_from_slice(&99u16.to_le_bytes());

        assert_eq!(
            BodyProfile::from_bytes(&bytes),
            Err(ProfileError::UnknownVersion {
                found: 99,
                expected: PROFILE_VERSION
            })
        );
    }

    #[test]
    fn a_future_version_refuses_even_when_the_payload_is_unreadable() {
        // The case a version field inside the payload cannot handle at all:
        // the layout changed, so the payload is undecodable by this build. The
        // header still answers, which is the whole argument for its position.
        let mut bytes = PROFILE_MAGIC.to_vec();
        bytes.extend_from_slice(&7u16.to_le_bytes());
        bytes.extend_from_slice(&[0xff; 32]);

        assert_eq!(
            BodyProfile::from_bytes(&bytes),
            Err(ProfileError::UnknownVersion {
                found: 7,
                expected: PROFILE_VERSION
            })
        );
    }

    #[test]
    fn a_truncated_payload_is_malformed_not_a_version_problem() {
        let bytes = profile().to_bytes().unwrap();
        assert_eq!(
            BodyProfile::from_bytes(&bytes[..HEADER_LEN + 3]),
            Err(ProfileError::Malformed)
        );
    }

    #[test]
    fn a_self_contradicting_profile_is_refused() {
        let mut profile = profile();
        let cells = profile.cell_count();
        profile.attribution.truncate(cells - 1);
        let _ = cells;
        assert_eq!(
            BodyProfile::from_bytes(&framed(&profile)),
            Err(ProfileError::Inconsistent)
        );
    }

    #[test]
    fn attribution_naming_a_part_that_does_not_exist_is_refused() {
        // Attribution indexes `parts`. A profile whose grid points past the
        // end of its own part list would panic a trusting reader, so it is
        // refused at the boundary instead.
        let mut profile = profile();
        profile.attribution[0] = 99;
        assert_eq!(
            BodyProfile::from_bytes(&framed(&profile)),
            Err(ProfileError::Inconsistent)
        );
    }

    #[test]
    fn the_grid_agrees_with_the_plain_flatten() {
        // Attribution is additive. Whatever the baker saw before this module
        // existed, it still sees.
        let (body, volumes) = donated();
        let plain = crate::flatten(&body, &volumes).unwrap();
        let profile = BodyProfile::of(&body, &volumes).unwrap();

        assert_eq!(profile.size, plain.volume.size);
        assert_eq!(profile.origin, plain.origin);
        assert_eq!(profile.cells, plain.volume.into_voxels());
    }

    /// Frames a hand-damaged profile so the reader sees a valid header and a
    /// bad payload. `to_bytes` cannot produce these; a foreign writer can.
    fn framed(profile: &BodyProfile) -> Vec<u8> {
        let mut bytes = PROFILE_MAGIC.to_vec();
        bytes.extend_from_slice(&PROFILE_VERSION.to_le_bytes());
        bytes.extend_from_slice(&postcard::to_allocvec(profile).unwrap());
        bytes
    }
}

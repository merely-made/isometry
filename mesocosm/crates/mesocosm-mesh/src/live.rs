// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Identity-preserving geometry for a live body.
//!
//! This is deliberately below a renderer and above a body document. It keeps
//! immutable part geometry reusable by [`VolumeRef`] while rebuilding the
//! small placement list for the body currently being read. World movement is
//! not part of this projection: a host applies an organism's world position
//! after it receives the body-space mesh.

use std::collections::BTreeMap;

use mesocosm_core::{BodyDocument, OrganismId, Provenance, VolumeRef, Yaw};

use crate::{BodyMesh, MeshError, PartMesh, Placement, VolumeSource, mesh_volume};

/// The bounded default for immutable, content-addressed part meshes.
///
/// The cache is an optimization only. A body with more distinct volumes still
/// projects completely; entries beyond the capacity are simply not retained.
pub const DEFAULT_MESH_CACHE_CAPACITY: usize = 256;

/// A deterministic dependency stamp for the body facts this projection reads.
///
/// It deliberately excludes allocation and expression state. VB1 makes no
/// visible expression claim, so those facts must not invalidate exact geometry
/// yet. It also excludes severed history: only surviving parts are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyDependencyRevision(pub u64);

/// One body ready for a renderer or hit-testing consumer.
///
/// `mesh.placements` carries the `PartId`, `VolumeRef`, rigid placement, and
/// provenance for each drawable part. `organism` keeps that part address in
/// its body owner; consumers must carry both rather than treating a `PartId`
/// as globally unique.
#[derive(Clone, Debug)]
pub struct LiveBodyProjection {
    pub organism: OrganismId,
    pub revision: BodyDependencyRevision,
    pub mesh: BodyMesh,
}

/// Reuses immutable volume geometry while producing complete live-body views.
#[derive(Debug)]
pub struct LiveBodyProjector {
    mesh_cache: BTreeMap<VolumeRef, CachedPartMesh>,
    mesh_capacity: usize,
}

#[derive(Debug)]
struct CachedPartMesh {
    mesh: PartMesh,
    volume: crate::Volume,
}

impl Default for LiveBodyProjector {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveBodyProjector {
    pub fn new() -> Self {
        Self::with_mesh_capacity(DEFAULT_MESH_CACHE_CAPACITY)
    }

    /// Creates a projector whose retained immutable geometry cannot exceed
    /// `mesh_capacity` distinct content addresses.
    pub fn with_mesh_capacity(mesh_capacity: usize) -> Self {
        Self {
            mesh_cache: BTreeMap::new(),
            mesh_capacity,
        }
    }

    /// Number of distinct volume meshes retained for later bodies.
    pub fn cached_mesh_count(&self) -> usize {
        self.mesh_cache.len()
    }

    /// Projects every surviving part of `body`.
    ///
    /// Resolution and placement are completed before the cache or output is
    /// changed. A missing volume or malformed attachment therefore returns one
    /// loud error rather than an incomplete, publishable body.
    pub fn project(
        &mut self,
        organism: OrganismId,
        body: &BodyDocument,
        source: &impl VolumeSource,
    ) -> Result<LiveBodyProjection, MeshError> {
        let mut placements = Vec::with_capacity(body.len());
        let mut volumes = BTreeMap::new();

        for part in body.living() {
            let Some(pivot_at) = body.world_pivot(part.id) else {
                return Err(MeshError::Unplaceable { part: part.id });
            };
            let Some(yaw) = body.world_yaw(part.id) else {
                return Err(MeshError::Unplaceable { part: part.id });
            };
            let Some(volume) = source.volume(part.volume) else {
                return Err(MeshError::MissingVolume {
                    part: part.id,
                    volume: part.volume,
                });
            };

            volumes.entry(part.volume).or_insert(volume);
            placements.push(Placement {
                part: part.id,
                volume: part.volume,
                pivot_at,
                pivot: part.pivot,
                yaw,
                provenance: Some(part.provenance.clone()),
            });
        }

        let revision = revision_for(&placements);
        let mut meshes = BTreeMap::new();
        let mut newly_meshed = Vec::new();
        for (reference, volume) in volumes {
            if let Some(cached) = self.mesh_cache.get(&reference) {
                if cached.volume != *volume {
                    return Err(MeshError::VolumeContentChanged { volume: reference });
                }
            }
            let mesh = match self.mesh_cache.get(&reference) {
                Some(cached) => cached.mesh.clone(),
                None => {
                    let mesh = mesh_volume(volume);
                    newly_meshed.push((reference, mesh.clone(), volume.clone()));
                    mesh
                },
            };
            meshes.insert(reference.0, mesh);
        }

        let mesh = BodyMesh { meshes, placements };
        if mesh.placements.iter().all(|placement| {
            mesh.mesh_for(placement.volume)
                .is_some_and(|part| part.quads.is_empty())
        }) {
            return Err(MeshError::EmptyBodyProjection { organism });
        }
        for (reference, part_mesh, volume) in newly_meshed {
            if self.mesh_cache.len() == self.mesh_capacity {
                break;
            }
            self.mesh_cache.insert(
                reference,
                CachedPartMesh {
                    mesh: part_mesh,
                    volume,
                },
            );
        }

        Ok(LiveBodyProjection {
            organism,
            revision,
            mesh,
        })
    }
}

fn revision_for(placements: &[Placement]) -> BodyDependencyRevision {
    let mut hasher = Fnv1a::new();
    hasher.write_u64(placements.len() as u64);
    for placement in placements {
        hasher.write_u32(placement.part.0);
        hasher.write(&placement.volume.0);
        hasher.write_i32s(placement.pivot_at);
        hasher.write_i32s(placement.pivot);
        hasher.write_u8(yaw_tag(placement.yaw));
        match placement.provenance.as_ref() {
            Some(provenance) => {
                hasher.write_u8(1);
                hash_provenance(&mut hasher, provenance);
            },
            None => hasher.write_u8(0),
        }
    }
    BodyDependencyRevision(hasher.finish())
}

fn hash_provenance(hasher: &mut Fnv1a, provenance: &Provenance) {
    match &provenance.origin {
        mesocosm_core::Origin::Founding => hasher.write_u8(0),
        mesocosm_core::Origin::Incorporated {
            from_species,
            from_part,
        } => {
            hasher.write_u8(1);
            hasher.write_u32(from_species.0);
            hasher.write_u32(from_part.0);
        },
    }
    hasher.write_u64(provenance.epoch);
}

fn yaw_tag(yaw: Yaw) -> u8 {
    match yaw {
        Yaw::Zero => 0,
        Yaw::Quarter => 1,
        Yaw::Half => 2,
        Yaw::ThreeQuarter => 3,
    }
}

struct Fnv1a(u64);

impl Fnv1a {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.write(&[value]);
    }

    fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    fn write_i32s(&mut self, values: [i32; 3]) {
        for value in values {
            self.write(&value.to_le_bytes());
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use mesocosm_core::{
        Attachment, BodyDocument, Kingdom, Organism, OrganismId, PartId, Provenance, SpeciesId,
        Stage, VolumeRef, Yaw,
    };

    use super::*;
    use crate::{Volume, VolumeMap};

    fn source() -> VolumeMap {
        let mut source = VolumeMap::new();
        source.insert(VolumeRef::from_tag(1), Volume::solid([3, 3, 3], 1));
        source.insert(VolumeRef::from_tag(2), Volume::solid([2, 1, 1], 2));
        source
    }

    fn body() -> BodyDocument {
        BodyDocument::new(SpeciesId(3), VolumeRef::from_tag(1), 100, [1, 1, 1])
    }

    #[test]
    fn retains_owner_part_volume_placement_and_provenance() {
        let mut body = body();
        let arm = body
            .attach(
                VolumeRef::from_tag(2),
                20,
                [1, 1, 1],
                Attachment {
                    parent: body.root,
                    offset: [5, 0, 0],
                    yaw: Yaw::Quarter,
                },
                Provenance::founding(),
            )
            .unwrap();
        let mut projector = LiveBodyProjector::new();

        let projected = projector.project(OrganismId(41), &body, &source()).unwrap();

        assert_eq!(projected.organism, OrganismId(41));
        let placement = projected
            .mesh
            .placements
            .iter()
            .find(|p| p.part == arm)
            .unwrap();
        assert_eq!(placement.volume, VolumeRef::from_tag(2));
        assert_eq!(placement.pivot_at, [5, 0, 0]);
        assert_eq!(placement.yaw, Yaw::Quarter);
        assert_eq!(placement.provenance, Some(Provenance::founding()));
    }

    #[test]
    fn attachment_changes_the_dependency_revision() {
        let mut body = body();
        let mut projector = LiveBodyProjector::new();
        let before = projector.project(OrganismId(1), &body, &source()).unwrap();
        body.attach(
            VolumeRef::from_tag(2),
            20,
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [4, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();

        let after = projector.project(OrganismId(1), &body, &source()).unwrap();
        assert_ne!(before.revision, after.revision);
        assert_eq!(after.mesh.placement_count(), 2);
    }

    #[test]
    fn severing_removes_geometry_and_changes_the_dependency_revision() {
        let mut body = body();
        let arm = body
            .attach(
                VolumeRef::from_tag(2),
                20,
                [1, 1, 1],
                Attachment {
                    parent: body.root,
                    offset: [4, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let mut projector = LiveBodyProjector::new();
        let before = projector.project(OrganismId(1), &body, &source()).unwrap();
        body.sever(arm);

        let after = projector.project(OrganismId(1), &body, &source()).unwrap();
        assert_ne!(before.revision, after.revision);
        assert_eq!(after.mesh.placement_count(), 1);
        assert!(after.mesh.placements.iter().all(|p| p.part != arm));
    }

    #[test]
    fn repeated_volumes_share_one_mesh_and_remain_cached_across_calls() {
        let mut body = body();
        for offset in [4, 7, 10] {
            body.attach(
                VolumeRef::from_tag(2),
                20,
                [1, 1, 1],
                Attachment {
                    parent: body.root,
                    offset: [offset, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        }
        let mut projector = LiveBodyProjector::new();
        let first = projector.project(OrganismId(1), &body, &source()).unwrap();
        let cached = projector.cached_mesh_count();
        let second = projector.project(OrganismId(1), &body, &source()).unwrap();

        assert_eq!(first.mesh.mesh_count(), 2);
        assert_eq!(first.mesh.placement_count(), 4);
        assert_eq!(cached, 2);
        assert_eq!(projector.cached_mesh_count(), cached);
        assert_eq!(first.revision, second.revision);
    }

    #[test]
    fn missing_content_fails_without_a_partial_projection_or_cache_change() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(99),
            20,
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [4, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
        let mut projector = LiveBodyProjector::new();

        let error = projector
            .project(OrganismId(1), &body, &source())
            .unwrap_err();
        assert_eq!(
            error,
            MeshError::MissingVolume {
                part: PartId(1),
                volume: VolumeRef::from_tag(99),
            }
        );
        assert_eq!(projector.cached_mesh_count(), 0);
    }

    #[test]
    fn changed_content_under_a_cached_reference_is_rejected_without_mutation() {
        let mut source = source();
        let mut projector = LiveBodyProjector::new();
        projector.project(OrganismId(1), &body(), &source).unwrap();
        let cached_before = projector.cached_mesh_count();
        source.insert(VolumeRef::from_tag(1), Volume::solid([9, 1, 1], 1));

        let error = projector
            .project(OrganismId(1), &body(), &source)
            .unwrap_err();

        assert_eq!(
            error,
            MeshError::VolumeContentChanged {
                volume: VolumeRef::from_tag(1)
            }
        );
        assert_eq!(projector.cached_mesh_count(), cached_before);
    }

    #[test]
    fn an_all_empty_volume_is_a_named_projection_failure() {
        let mut source = VolumeMap::new();
        source.insert(VolumeRef::from_tag(1), Volume::empty([2, 2, 2]));
        let mut projector = LiveBodyProjector::new();

        let error = projector
            .project(OrganismId(1), &body(), &source)
            .unwrap_err();

        assert_eq!(
            error,
            MeshError::EmptyBodyProjection {
                organism: OrganismId(1)
            }
        );
        assert_eq!(projector.cached_mesh_count(), 0);
    }

    #[test]
    fn a_carcass_keeps_its_intact_part_projection() {
        let mut carcass = Organism::founding(
            OrganismId(9),
            SpeciesId(3),
            Kingdom::Decomposer,
            VolumeRef::from_tag(1),
            [1, 1, 1],
            [12, 0, 0],
            100,
        );
        carcass.stage = Stage::Carrion;
        let mut projector = LiveBodyProjector::new();

        let projected = projector
            .project(carcass.id, carcass.body(), &source())
            .unwrap();
        assert_eq!(projected.organism, carcass.id);
        assert_eq!(
            projected.mesh.placement_count(),
            carcass.body().living().count()
        );
    }
}

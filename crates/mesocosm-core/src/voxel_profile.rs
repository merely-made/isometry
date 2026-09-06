// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's product-owned adapter from authoritative [`Ground`] bricks to
//! Nisus voxel mechanics (renamed from conatus-voxel 2026-08-28).
//!
//! `Ground` remains the serialized authority. This profile keeps disposable
//! Conatus chunks beside it so spatial consumers can share patch, dirty-region,
//! and occupancy lowering without placing a second voxel record in the world.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use nisus::{VoxelCellEdit, VoxelChunk, VoxelChunkError, VoxelEdit, VoxelPatch, VoxelRegion};

use crate::places::{AIR, BRICK, Ground};

pub type GroundChunkKey = [i16; 3];

const CHUNK_EXTENT: [u32; 3] = [BRICK as u32; 3];
const CELLS_PER_CHUNK: usize = (BRICK * BRICK * BRICK) as usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroundChunkChange {
    Inserted {
        key: GroundChunkKey,
        local_revision: u64,
        dirty: VoxelRegion,
        occupancy_edits: Vec<VoxelEdit>,
    },
    Patched {
        key: GroundChunkKey,
        patch: VoxelPatch<u8>,
        occupancy_edits: Vec<VoxelEdit>,
    },
    Removed {
        key: GroundChunkKey,
        previous_local_revision: u64,
        dirty: VoxelRegion,
        occupancy_edits: Vec<VoxelEdit>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroundVoxelUpdate {
    pub previous_source_revision: u64,
    pub source_revision: u64,
    pub chunks: Vec<GroundChunkChange>,
}

impl GroundVoxelUpdate {
    pub fn is_silent(&self) -> bool {
        self.chunks.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct GroundVoxelProfile {
    source_revision: u64,
    chunks: BTreeMap<GroundChunkKey, VoxelChunk<u8>>,
}

impl GroundVoxelProfile {
    pub fn from_ground(ground: &Ground) -> Result<Self, GroundVoxelProfileError> {
        let mut chunks = BTreeMap::new();
        for key in ground.keys() {
            let (brick, _) = ground
                .brick_materials(key)
                .expect("a key yielded by Ground has a brick");
            chunks.insert(
                key,
                VoxelChunk::from_cells(CHUNK_EXTENT, brick.raw().to_vec(), 0)
                    .map_err(|source| GroundVoxelProfileError::Chunk { key, source })?,
            );
        }
        Ok(Self {
            source_revision: ground.revision(),
            chunks,
        })
    }

    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub fn chunk(&self, key: GroundChunkKey) -> Option<&VoxelChunk<u8>> {
        self.chunks.get(&key)
    }

    /// Every resident chunk with its key, in key order, so a derived
    /// consumer (the tactile projection) can lower the whole profile
    /// without re-reading `Ground`.
    pub fn chunks(&self) -> impl ExactSizeIterator<Item = (GroundChunkKey, &VoxelChunk<u8>)> {
        self.chunks.iter().map(|(key, chunk)| (*key, chunk))
    }

    /// Synchronize one already-accepted `Ground` revision into the disposable
    /// Conatus view.
    ///
    /// `expected_source_revision` is the product authority gate. Conatus chunk
    /// revisions remain local mechanics and may advance at a different rate
    /// from the global `Ground` revision.
    pub fn sync(
        &mut self,
        expected_source_revision: u64,
        ground: &Ground,
    ) -> Result<GroundVoxelUpdate, GroundVoxelProfileError> {
        if expected_source_revision != self.source_revision {
            return Err(GroundVoxelProfileError::StaleSource {
                expected: expected_source_revision,
                actual: self.source_revision,
            });
        }
        if ground.revision() < self.source_revision {
            return Err(GroundVoxelProfileError::RegressedSource {
                current: self.source_revision,
                offered: ground.revision(),
            });
        }

        let previous_source_revision = self.source_revision;
        let mut next = self.chunks.clone();
        let mut changes = Vec::new();
        let offered_keys: BTreeSet<_> = ground.keys().collect();

        for key in &offered_keys {
            let (brick, _) = ground
                .brick_materials(*key)
                .expect("a key yielded by Ground has a brick");
            match next.get_mut(key) {
                Some(chunk) => {
                    let edits: Vec<_> = chunk
                        .iter()
                        .zip(brick.raw())
                        .filter_map(|((cell, current), offered)| {
                            (*current != *offered).then_some(VoxelCellEdit {
                                cell,
                                value: *offered,
                            })
                        })
                        .collect();
                    if edits.is_empty() {
                        continue;
                    }
                    let patch = chunk
                        .apply_edits(chunk.revision(), CELLS_PER_CHUNK, edits)
                        .map_err(|source| GroundVoxelProfileError::Chunk { key: *key, source })?;
                    let occupancy_edits = patch
                        .occupancy_edits(|material| *material != AIR)
                        .map_err(|source| GroundVoxelProfileError::Chunk { key: *key, source })?;
                    let drained = chunk.drain_dirty_regions();
                    debug_assert_eq!(drained, patch.dirty_regions);
                    changes.push(GroundChunkChange::Patched {
                        key: *key,
                        patch,
                        occupancy_edits,
                    });
                },
                None => {
                    let chunk = VoxelChunk::from_cells(CHUNK_EXTENT, brick.raw().to_vec(), 0)
                        .map_err(|source| GroundVoxelProfileError::Chunk { key: *key, source })?;
                    let occupancy_edits = chunk
                        .occupied_cells(|material| *material != AIR)
                        .into_iter()
                        .map(|cell| VoxelEdit { cell, filled: true })
                        .collect();
                    changes.push(GroundChunkChange::Inserted {
                        key: *key,
                        local_revision: chunk.revision(),
                        dirty: whole_chunk(),
                        occupancy_edits,
                    });
                    next.insert(*key, chunk);
                },
            }
        }

        let removed: Vec<_> = next
            .keys()
            .filter(|key| !offered_keys.contains(*key))
            .copied()
            .collect();
        for key in removed {
            let chunk = next
                .remove(&key)
                .expect("the removal key came from the chunk map");
            let occupancy_edits = chunk
                .occupied_cells(|material| *material != AIR)
                .into_iter()
                .map(|cell| VoxelEdit {
                    cell,
                    filled: false,
                })
                .collect();
            changes.push(GroundChunkChange::Removed {
                key,
                previous_local_revision: chunk.revision(),
                dirty: whole_chunk(),
                occupancy_edits,
            });
        }

        if ground.revision() == previous_source_revision && !changes.is_empty() {
            return Err(GroundVoxelProfileError::ChangedWithoutRevision {
                revision: ground.revision(),
            });
        }
        if ground.revision() > previous_source_revision && changes.is_empty() {
            return Err(GroundVoxelProfileError::RevisionWithoutChange {
                previous: previous_source_revision,
                offered: ground.revision(),
            });
        }

        self.source_revision = ground.revision();
        self.chunks = next;
        Ok(GroundVoxelUpdate {
            previous_source_revision,
            source_revision: ground.revision(),
            chunks: changes,
        })
    }
}

fn whole_chunk() -> VoxelRegion {
    VoxelRegion {
        origin: [0; 3],
        extent: CHUNK_EXTENT,
    }
}

#[derive(Debug)]
pub enum GroundVoxelProfileError {
    StaleSource {
        expected: u64,
        actual: u64,
    },
    RegressedSource {
        current: u64,
        offered: u64,
    },
    ChangedWithoutRevision {
        revision: u64,
    },
    RevisionWithoutChange {
        previous: u64,
        offered: u64,
    },
    Chunk {
        key: GroundChunkKey,
        source: VoxelChunkError,
    },
}

impl fmt::Display for GroundVoxelProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleSource { expected, actual } => {
                write!(
                    formatter,
                    "stale Ground revision {expected}; current is {actual}"
                )
            },
            Self::RegressedSource { current, offered } => {
                write!(
                    formatter,
                    "Ground revision regressed from {current} to {offered}"
                )
            },
            Self::ChangedWithoutRevision { revision } => {
                write!(
                    formatter,
                    "Ground bytes changed without advancing revision {revision}"
                )
            },
            Self::RevisionWithoutChange { previous, offered } => write!(
                formatter,
                "Ground revision advanced from {previous} to {offered} without a voxel change"
            ),
            Self::Chunk { key, source } => {
                write!(
                    formatter,
                    "Ground chunk {key:?} could not be projected: {source}"
                )
            },
        }
    }
}

impl Error for GroundVoxelProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Chunk { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Places, snapshot};

    fn ground() -> Ground {
        Ground::grow(&Places::grown(4_242, 4, 64), 64)
    }

    fn carved(ground: &mut Ground) -> [i32; 3] {
        let top = ground
            .surface(0, 0)
            .expect("the seed has ground at the origin");
        let at = [0, top, 0];
        assert!(ground.carve(at, 0) > 0);
        at
    }

    #[test]
    fn the_profile_is_disposable_and_snapshot_silent() {
        let ground = ground();
        let bytes = snapshot::encode(&ground).unwrap();
        let mut profile = GroundVoxelProfile::from_ground(&ground).unwrap();

        let update = profile.sync(ground.revision(), &ground).unwrap();

        assert!(update.is_silent());
        assert_eq!(snapshot::encode(&ground).unwrap(), bytes);
    }

    #[test]
    fn one_accepted_carve_becomes_one_revision_gated_projection_update() {
        let mut ground = ground();
        let mut profile = GroundVoxelProfile::from_ground(&ground).unwrap();
        let before = ground.revision();
        carved(&mut ground);

        let update = profile.sync(before, &ground).unwrap();

        assert_eq!(update.previous_source_revision, before);
        assert_eq!(update.source_revision, ground.revision());
        assert!(!update.is_silent());
        assert!(update.chunks.iter().any(|change| match change {
            GroundChunkChange::Patched {
                occupancy_edits, ..
            } => occupancy_edits.iter().any(|edit| !edit.filled),
            _ => false,
        }));
        assert!(
            profile
                .sync(ground.revision(), &ground)
                .unwrap()
                .is_silent()
        );
    }

    #[test]
    fn a_stale_source_revision_is_atomic() {
        let mut ground = ground();
        let mut profile = GroundVoxelProfile::from_ground(&ground).unwrap();
        let before = ground.revision();
        carved(&mut ground);
        profile.sync(before, &ground).unwrap();
        let current = profile.source_revision();

        assert!(matches!(
            profile.sync(before, &ground),
            Err(GroundVoxelProfileError::StaleSource { .. })
        ));
        assert_eq!(profile.source_revision(), current);
        assert!(profile.sync(current, &ground).unwrap().is_silent());
    }

    #[test]
    fn replay_produces_identical_ground_bytes_and_projection_changes() {
        let mut a = ground();
        let mut b = ground();
        let mut a_profile = GroundVoxelProfile::from_ground(&a).unwrap();
        let mut b_profile = GroundVoxelProfile::from_ground(&b).unwrap();
        let a_revision = a.revision();
        let b_revision = b.revision();

        assert_eq!(carved(&mut a), carved(&mut b));
        let a_update = a_profile.sync(a_revision, &a).unwrap();
        let b_update = b_profile.sync(b_revision, &b).unwrap();

        assert_eq!(snapshot::encode(&a).unwrap(), snapshot::encode(&b).unwrap());
        assert_eq!(a_update, b_update);
    }
}

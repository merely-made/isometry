//! Isometry's disposable GPU projection of accepted Conatus body positions.
//!
//! This module deliberately stays in the product profile. Quint owns the
//! resident allocation and sparse publication mechanics; Isometry owns the
//! capacity, source binding, coordinate meaning, and the decision to expose
//! this particular position plane to a selected tenant.

use std::{collections::BTreeMap, error::Error, fmt};

use conatus::BodyId;
use quint::resident::{
    ChunkBounds, ChunkStamp, DirtyRegion, PlaneClass, PlaneId, PlanePatch, RawKernelView,
    ReadEpoch, ResidentChunk, ResidentChunkError, ResidentClient,
};

use crate::{IsometrySpatialFrame, TokenSourceId};

const POSITION_WIDTH: usize = 4;

#[derive(Debug)]
struct ResidentBodyPlane;

/// One accepted publication into Isometry's product-local resident view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsometryResidentUpdate {
    pub revision: u64,
    pub read_epoch: ReadEpoch,
    pub changed_slots: Vec<u32>,
}

/// A fixed-capacity position plane keyed by Conatus body slots.
///
/// Rows are `[x, y, z, occupied]`. A body removal writes a zero row; a
/// replacement generation may reuse the row only after the profile has
/// processed that removal. Growth is intentionally explicit because replacing
/// the allocation requires every selected tenant to rebind.
#[derive(Debug)]
pub struct IsometryResidentBodies {
    capacity: usize,
    positions: PlaneId,
    chunk: ResidentChunk<ResidentBodyPlane>,
    bindings: BTreeMap<TokenSourceId, BodyId>,
}

impl IsometryResidentBodies {
    pub fn new(
        client: ResidentClient,
        capacity: usize,
        initial_read_epoch: ReadEpoch,
    ) -> Result<Self, IsometryResidentError> {
        let extent = u32::try_from(capacity)
            .ok()
            .filter(|extent| *extent > 0)
            .ok_or(IsometryResidentError::InvalidCapacity(capacity))?;
        let element_count = capacity
            .checked_mul(POSITION_WIDTH)
            .ok_or(IsometryResidentError::InvalidCapacity(capacity))?;
        let positions =
            PlaneId::new("isometry.body.positions").map_err(IsometryResidentError::Resident)?;
        let mut chunk = ResidentChunk::new(
            client,
            ResidentBodyPlane,
            ChunkBounds {
                origin: [0, 0, 0],
                extent: [extent, 1, 1],
            },
            0,
            initial_read_epoch,
            Vec::new(),
        );
        chunk
            .insert_plane(
                positions.clone(),
                PlaneClass::Derived,
                [capacity, POSITION_WIDTH, 1],
                &vec![0.0f32; element_count],
            )
            .map_err(IsometryResidentError::Resident)?;

        Ok(Self {
            capacity,
            positions,
            chunk,
            bindings: BTreeMap::new(),
        })
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub const fn stamp(&self) -> ChunkStamp {
        self.chunk.stamp()
    }

    pub fn body_for(&self, source: &TokenSourceId) -> Option<BodyId> {
        self.bindings.get(source).copied()
    }

    /// Export the exact allocation range for a product-selected GPU tenant.
    pub fn positions(&self) -> Result<RawKernelView, IsometryResidentError> {
        self.chunk
            .raw_kernel_view(&self.positions)
            .map_err(IsometryResidentError::Resident)
    }

    /// Publish one already-authorized product frame into the resident cache.
    ///
    /// A silent product frame performs no queue write and does not restamp the
    /// allocation. All source and capacity checks complete before Quint's own
    /// all-or-nothing patch validation begins.
    pub fn apply_frame(
        &mut self,
        queue: &wgpu::Queue,
        frame: &IsometrySpatialFrame,
        read_epoch: ReadEpoch,
    ) -> Result<Option<IsometryResidentUpdate>, IsometryResidentError> {
        if frame.is_silent() {
            return Ok(None);
        }

        let current = self.chunk.stamp();
        if frame.revision <= current.revision {
            return Err(IsometryResidentError::StaleFrame {
                current: current.revision,
                offered: frame.revision,
            });
        }
        if read_epoch.get() <= current.valid_read_epoch.get() {
            return Err(IsometryResidentError::NonAdvancingReadEpoch {
                current: current.valid_read_epoch,
                offered: read_epoch,
            });
        }

        let mut next_bindings = self.bindings.clone();
        let mut rows = BTreeMap::<u32, [f32; POSITION_WIDTH]>::new();

        for source in &frame.removed {
            let body = next_bindings
                .remove(source)
                .ok_or_else(|| IsometryResidentError::UnknownRemoval(source.clone()))?;
            rows.insert(body.slot(), [0.0; POSITION_WIDTH]);
        }

        for changed in &frame.changed {
            let body = changed.body.id;
            let slot = body.slot();
            if slot as usize >= self.capacity {
                return Err(IsometryResidentError::CapacityExceeded {
                    body,
                    capacity: self.capacity,
                });
            }
            if let Some(previous) = next_bindings.insert(changed.source.clone(), body) {
                if previous.slot() != slot {
                    rows.insert(previous.slot(), [0.0; POSITION_WIDTH]);
                }
            }
            if let Some((other, _)) = next_bindings
                .iter()
                .find(|(source, candidate)| *source != &changed.source && candidate.slot() == slot)
            {
                return Err(IsometryResidentError::SlotCollision {
                    slot,
                    first: other.clone(),
                    second: changed.source.clone(),
                });
            }
            let [x, y, z] = changed.body.transform.translation;
            rows.insert(slot, [x, y, z, 1.0]);
        }

        if rows.is_empty() {
            return Err(IsometryResidentError::EmptyPublication);
        }

        let staged: Vec<_> = rows.into_iter().collect();
        let patches: Vec<_> = staged
            .iter()
            .map(|(slot, values)| {
                PlanePatch::new(*slot as usize * POSITION_WIDTH, values.as_slice())
            })
            .collect();
        let dirty_regions: Vec<_> = staged
            .iter()
            .map(|(slot, _)| DirtyRegion {
                origin: [*slot, 0, 0],
                extent: [1, 1, 1],
            })
            .collect();
        let committed = ChunkStamp {
            revision: frame.revision,
            valid_read_epoch: read_epoch,
        };
        self.chunk
            .commit_plane_patches(
                queue,
                &self.positions,
                current,
                &patches,
                committed,
                dirty_regions,
            )
            .map_err(IsometryResidentError::Resident)?;
        self.bindings = next_bindings;

        Ok(Some(IsometryResidentUpdate {
            revision: committed.revision,
            read_epoch: committed.valid_read_epoch,
            changed_slots: staged.into_iter().map(|(slot, _)| slot).collect(),
        }))
    }
}

#[derive(Debug)]
pub enum IsometryResidentError {
    InvalidCapacity(usize),
    CapacityExceeded {
        body: BodyId,
        capacity: usize,
    },
    StaleFrame {
        current: u64,
        offered: u64,
    },
    NonAdvancingReadEpoch {
        current: ReadEpoch,
        offered: ReadEpoch,
    },
    UnknownRemoval(TokenSourceId),
    SlotCollision {
        slot: u32,
        first: TokenSourceId,
        second: TokenSourceId,
    },
    EmptyPublication,
    Resident(ResidentChunkError),
}

impl fmt::Display for IsometryResidentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCapacity(capacity) => {
                write!(formatter, "resident body capacity {capacity} is invalid")
            }
            Self::CapacityExceeded { body, capacity } => write!(
                formatter,
                "Conatus body {body:?} exceeds Isometry resident capacity {capacity}"
            ),
            Self::StaleFrame { current, offered } => write!(
                formatter,
                "Isometry resident revision is {current}, not older than offered frame {offered}"
            ),
            Self::NonAdvancingReadEpoch { current, offered } => write!(
                formatter,
                "Isometry resident read epoch {offered:?} does not advance {current:?}"
            ),
            Self::UnknownRemoval(source) => write!(
                formatter,
                "resident body view has no binding for removed token {:?} on map {}",
                source.token,
                source.map.as_str()
            ),
            Self::SlotCollision {
                slot,
                first,
                second,
            } => write!(
                formatter,
                "resident body slot {slot} is claimed by {:?} and {:?}",
                first.token, second.token
            ),
            Self::EmptyPublication => {
                formatter.write_str("Isometry frame contained no resident body row to publish")
            }
            Self::Resident(source) => {
                write!(formatter, "resident body projection failed: {source}")
            }
        }
    }
}

impl Error for IsometryResidentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resident(source) => Some(source),
            _ => None,
        }
    }
}

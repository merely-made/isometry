// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The real-Ground resident-view composition receipt.
//!
//! A Burn pass proposes a bounded carve from a material-derived field. Ground
//! accepts the integer consequence, Conatus patches and restamps the retained
//! atlas allocation, and BrickTracer observes that allocation without a CPU
//! voxel upload. The accepted delta then replays without Burn.

use std::mem::{size_of, size_of_val};

use burn::{
    backend::wgpu::{RuntimeOptions, WgpuDevice, graphics::AutoGraphicsApi, init_setup},
    tensor::Tensor,
};
use conatus::resident::{
    ChunkBounds, ChunkStamp, DirtyRegion, PlaneClass, PlaneElementType, PlaneId, RawKernelView,
    ReadEpoch, ResidentChunk, ResidentClient,
};
use mesocosm_core::places::{Ground, Places};
use mesocosm_lens::{
    BrickChange, BrickFrameInput, BrickMap, BrickProjectionRevision, BrickRevision, BrickTracer,
    Flight, Grade, LeasedAtlas,
};
use serde::{Deserialize, Serialize};

const WIDTH: u32 = 96;
const HEIGHT: u32 = 64;
const SOURCE_EPOCH: u64 = 10;
const COMMITTED_EPOCH: u64 = 11;
const FIELD_SHAPE: [usize; 3] = [3, 3, 3];
const FIELD_CENTER: usize = 13;

#[derive(Clone, Debug)]
struct CandidateGroundDelta {
    source_revision: u64,
    observed_read_epoch: u64,
    at: [i32; 3],
    radius: i32,
    proposed_occupancy: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AcceptedGroundDelta {
    source_revision: u64,
    committed_revision: u64,
    observed_read_epoch: u64,
    at: [i32; 3],
    radius: i32,
    removed_voxels: u32,
}

#[derive(Debug, Serialize)]
struct Receipt {
    gate: &'static str,
    adapter: String,
    backend: String,
    source_revision: u64,
    committed_revision: u64,
    burn_raw_same_allocation: bool,
    durability_readback_bytes: usize,
    proposed_occupancy: f32,
    accepted_removed_voxels: u32,
    retained_atlas_allocation: bool,
    resident_patch_bytes: usize,
    initial_resident_voxel_bytes: u64,
    committed_resident_voxel_bytes: u64,
    committed_read_epoch: u64,
    committed_cpu_projection_bytes: u64,
    expected_read_epoch: u64,
    epoch_mismatch_refusals: u32,
    epoch_fallback_matches_committed: bool,
    resident_bytes_in_use: u64,
    resident_number_allocs: u64,
    resident_commit_allocator_growth: u64,
    changed_pixels: usize,
    unchanged_atlas_bytes: u64,
    accepted_delta_bytes: usize,
    replay_matches: bool,
}

fn field_index([x, y, z]: [usize; 3]) -> usize {
    x * FIELD_SHAPE[1] * FIELD_SHAPE[2] + y * FIELD_SHAPE[2] + z
}

fn material_field(ground: &Ground, centre: [i32; 3]) -> Vec<f32> {
    let mut values = vec![0.0; FIELD_SHAPE.iter().product()];
    for x in 0..FIELD_SHAPE[0] {
        for y in 0..FIELD_SHAPE[1] {
            for z in 0..FIELD_SHAPE[2] {
                let at = [
                    centre[0] + x as i32 - 1,
                    centre[1] + y as i32 - 1,
                    centre[2] + z as i32 - 1,
                ];
                values[field_index([x, y, z])] = if ground.solid(at) { 1.0 } else { 0.0 };
            }
        }
    }
    values
}

fn shift_negative(tensor: Tensor<3>, dim: usize) -> Tensor<3> {
    let [x, y, z] = tensor.dims();
    match dim {
        0 => Tensor::cat(
            vec![
                tensor.clone().slice([0..1, 0..y, 0..z]),
                tensor.slice([0..x - 1, 0..y, 0..z]),
            ],
            0,
        ),
        1 => Tensor::cat(
            vec![
                tensor.clone().slice([0..x, 0..1, 0..z]),
                tensor.slice([0..x, 0..y - 1, 0..z]),
            ],
            1,
        ),
        2 => Tensor::cat(
            vec![
                tensor.clone().slice([0..x, 0..y, 0..1]),
                tensor.slice([0..x, 0..y, 0..z - 1]),
            ],
            2,
        ),
        _ => unreachable!("a 3D tensor has three axes"),
    }
}

fn shift_positive(tensor: Tensor<3>, dim: usize) -> Tensor<3> {
    let [x, y, z] = tensor.dims();
    match dim {
        0 => Tensor::cat(
            vec![
                tensor.clone().slice([1..x, 0..y, 0..z]),
                tensor.slice([x - 1..x, 0..y, 0..z]),
            ],
            0,
        ),
        1 => Tensor::cat(
            vec![
                tensor.clone().slice([0..x, 1..y, 0..z]),
                tensor.slice([0..x, y - 1..y, 0..z]),
            ],
            1,
        ),
        2 => Tensor::cat(
            vec![
                tensor.clone().slice([0..x, 0..y, 1..z]),
                tensor.slice([0..x, 0..y, z - 1..z]),
            ],
            2,
        ),
        _ => unreachable!("a 3D tensor has three axes"),
    }
}

fn diffuse(input: Tensor<3>) -> Tensor<3> {
    let neighbors = shift_negative(input.clone(), 0)
        + shift_positive(input.clone(), 0)
        + shift_negative(input.clone(), 1)
        + shift_positive(input.clone(), 1)
        + shift_negative(input.clone(), 2)
        + shift_positive(input.clone(), 2);
    input.clone() + (neighbors - input.mul_scalar(6.0)).mul_scalar(0.1)
}

fn propose(client: ResidentClient, ground: &Ground, at: [i32; 3]) -> (CandidateGroundDelta, bool) {
    let values = material_field(ground, at);
    assert_eq!(values[FIELD_CENTER], 1.0, "proposal target must be solid");
    let plane = PlaneId::new("candidate_occupancy").expect("valid plane name");
    let mut chunk = ResidentChunk::new(
        client,
        "ground:proposal",
        ChunkBounds {
            origin: at.map(|axis| i64::from(axis - 1)),
            extent: FIELD_SHAPE.map(|axis| axis as u32),
        },
        ground.revision(),
        ReadEpoch::new(SOURCE_EPOCH),
        vec![DirtyRegion {
            origin: [0; 3],
            extent: FIELD_SHAPE.map(|axis| axis as u32),
        }],
    );
    chunk
        .insert_plane(plane.clone(), PlaneClass::Temporary, FIELD_SHAPE, &values)
        .expect("resident candidate plane");
    let raw = chunk.raw_kernel_view(&plane).expect("raw candidate view");
    let burn = chunk.burn_f32_view(&plane).expect("Burn candidate view");
    let same_allocation = raw.allocation() == burn.allocation();
    let proposed_occupancy = diffuse(burn.into_tensor())
        .slice([1..2, 1..2, 1..2])
        .into_data()
        .to_vec::<f32>()
        .expect("one candidate scalar")[0];
    (
        CandidateGroundDelta {
            source_revision: raw.stamp().revision,
            observed_read_epoch: raw.stamp().valid_read_epoch.get(),
            at,
            radius: 0,
            proposed_occupancy,
        },
        same_allocation,
    )
}

fn accept(
    ground: &mut Ground,
    candidate: CandidateGroundDelta,
) -> Result<AcceptedGroundDelta, String> {
    if candidate.source_revision != ground.revision() {
        return Err("candidate was produced from a stale Ground revision".into());
    }
    if candidate.observed_read_epoch != SOURCE_EPOCH {
        return Err("candidate was produced outside the accepted read epoch".into());
    }
    if !candidate.proposed_occupancy.is_finite() || candidate.proposed_occupancy >= 1.0 {
        return Err("candidate does not propose a finite material reduction".into());
    }
    if !ground.solid(candidate.at) {
        return Err("candidate target is not an authoritative solid voxel".into());
    }
    let removed_voxels = ground.carve(candidate.at, candidate.radius);
    if removed_voxels == 0 {
        return Err("accepted carve changed no authoritative voxel".into());
    }
    Ok(AcceptedGroundDelta {
        source_revision: candidate.source_revision,
        committed_revision: ground.revision(),
        observed_read_epoch: candidate.observed_read_epoch,
        at: candidate.at,
        radius: candidate.radius,
        removed_voxels,
    })
}

fn resident_atlas(
    client: ResidentClient,
    identity: &'static str,
    values: &[u8],
    extent: [u32; 3],
    revision: u64,
    read_epoch: u64,
) -> (ResidentChunk<&'static str>, PlaneId) {
    let shape = [extent[2] as usize, extent[1] as usize, extent[0] as usize];
    let plane = PlaneId::new("atlas_zyx").expect("valid plane name");
    let mut chunk = ResidentChunk::new(
        client,
        identity,
        ChunkBounds {
            origin: [0; 3],
            extent,
        },
        revision,
        ReadEpoch::new(read_epoch),
        vec![DirtyRegion {
            origin: [0; 3],
            extent,
        }],
    );
    chunk
        .insert_plane(plane.clone(), PlaneClass::Exact, shape, values)
        .expect("resident exact atlas");
    (chunk, plane)
}

fn tracer_lease<'a>(
    view: &'a RawKernelView,
    projection_revision: BrickProjectionRevision,
    atlas_extent: [u32; 3],
    source_origin: [u32; 3],
    slot_origin: [u32; 3],
    extent: [u32; 3],
) -> LeasedAtlas<'a> {
    let lease = view.lease();
    assert_eq!(lease.element_type, PlaneElementType::U8);
    assert_eq!(
        lease.shape,
        [
            atlas_extent[2] as usize,
            atlas_extent[1] as usize,
            atlas_extent[0] as usize,
        ],
        "the texture adapter reverses z/y/x memory shape into x/y/z extent"
    );
    assert!(lease.fits(), "resident atlas shape overruns its allocation");
    let leased = LeasedAtlas {
        buffer: lease.buffer,
        offset: lease.offset,
        size: lease.byte_len(),
        source_origin,
        source_bytes_per_row: atlas_extent[0],
        source_rows_per_image: atlas_extent[1],
        slot_origin,
        extent,
        revision: BrickRevision(lease.stamp.revision),
        projection_revision,
        read_epoch: lease.stamp.valid_read_epoch.get(),
    };
    assert!(
        leased.copyable_into(atlas_extent),
        "resident atlas subregion is not copyable"
    );
    leased
}

fn camera(ground: &Ground) -> Flight {
    let top = ground.surface(4, 4).expect("fixture column");
    Flight {
        eye: [4.5, top as f32 + 14.0, 4.5],
        yaw: 0.0,
        pitch: -1.52,
        fov: 0.15,
        far: 48.0,
    }
}

fn main() {
    let mut ground = Ground::grow(&Places::grown(4_242, 4, 64), 64);
    let initial_ground = ground.clone();
    let mut map = BrickMap::from_ground(&ground).expect("Ground fits the brick atlas");
    let view = camera(&ground);
    let grade = Grade::clay();
    let target = [4, ground.surface(4, 4).expect("fixture column"), 4];

    let device_key = WgpuDevice::default();
    let setup = init_setup::<AutoGraphicsApi>(&device_key, RuntimeOptions::default());
    let adapter = setup.adapter.get_info();
    let client = ResidentClient::from_registered_device(device_key);
    let mut tracer =
        BrickTracer::with_device(setup.device.clone(), setup.queue.clone(), WIDTH, HEIGHT);

    let initial_extent = map.atlas_extent();
    let initial_atlas_values = map.atlas().to_vec();
    let (mut resident_atlas, atlas_plane) = resident_atlas(
        client.clone(),
        "ground:atlas",
        map.atlas(),
        initial_extent,
        ground.revision(),
        SOURCE_EPOCH,
    );
    let initial_atlas = resident_atlas
        .raw_kernel_view(&atlas_plane)
        .expect("initial raw atlas view");
    let initial = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &view, &grade)
                .with_leased_atlas(tracer_lease(
                    &initial_atlas,
                    map.projection_revision(),
                    initial_extent,
                    [0; 3],
                    [0; 3],
                    initial_extent,
                ))
                .with_expected_read_epoch(SOURCE_EPOCH),
        )
        .expect("initial resident Ground frame");
    assert_eq!(initial.diagnostics.epoch_lease_rejections, 0);
    assert_eq!(
        initial.diagnostics.leased_atlas_bytes,
        map.atlas().len() as u64
    );
    assert_eq!(
        initial.diagnostics.brick_upload_bytes,
        size_of_val(map.pointers()) as u64,
        "the tracer should upload pointers, not voxel materials"
    );

    let (candidate, same_allocation) = propose(client.clone(), &ground, target);
    let proposed_occupancy = candidate.proposed_occupancy;
    let accepted = accept(&mut ground, candidate).expect("Ground accepts bounded candidate");
    assert_eq!(accepted.committed_revision, accepted.source_revision + 1);
    let dirty = ground.drain_dirty();
    let slots = map
        .refresh(&ground, dirty)
        .expect("a carve preserves the brick map shape");
    assert_eq!(slots.len(), 1, "the radius-zero delta changes one brick");
    let slot = slots[0];
    let slot_origin = map.atlas_slot_origin(slot).expect("assigned atlas slot");
    let changed_indices = initial_atlas_values
        .iter()
        .zip(map.atlas())
        .enumerate()
        .filter_map(|(index, (before, after))| (before != after).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(
        changed_indices.len(),
        accepted.removed_voxels as usize,
        "the accepted radius-zero carve should alter one atlas texel"
    );
    let changed_index = changed_indices[0];
    let patch_start =
        changed_index / wgpu::COPY_BUFFER_ALIGNMENT as usize * wgpu::COPY_BUFFER_ALIGNMENT as usize;
    let patch_end = patch_start + wgpu::COPY_BUFFER_ALIGNMENT as usize;
    let atlas_xy = initial_extent[0] as usize * initial_extent[1] as usize;
    let dirty_origin = [
        (changed_index % initial_extent[0] as usize) as u32,
        ((changed_index / initial_extent[0] as usize) % initial_extent[1] as usize) as u32,
        (changed_index / atlas_xy) as u32,
    ];
    assert!((0..3).all(|axis| {
        dirty_origin[axis] >= slot_origin[axis] && dirty_origin[axis] < slot_origin[axis] + 8
    }));
    let committed_stamp = ChunkStamp {
        revision: ground.revision(),
        valid_read_epoch: ReadEpoch::new(COMMITTED_EPOCH),
    };
    // The commit must not touch the allocator: same slices, same bytes,
    // observed by CubeCL's own memory accounting rather than our word.
    let memory_before_commit = client
        .compute_client()
        .memory_usage()
        .expect("resident memory usage before commit");
    resident_atlas
        .commit_plane_patch(
            &setup.queue,
            &atlas_plane,
            initial_atlas.stamp(),
            patch_start,
            &map.atlas()[patch_start..patch_end],
            committed_stamp,
            vec![DirtyRegion {
                origin: dirty_origin,
                extent: [1; 3],
            }],
        )
        .expect("patch retained resident atlas");
    let memory_after_commit = client
        .compute_client()
        .memory_usage()
        .expect("resident memory usage after commit");
    assert_eq!(
        memory_before_commit.bytes_in_use, memory_after_commit.bytes_in_use,
        "the committed patch changed the allocator's bytes in use"
    );
    assert_eq!(
        memory_before_commit.number_allocs, memory_after_commit.number_allocs,
        "the committed patch changed the allocator's active allocations"
    );
    let committed_atlas = resident_atlas
        .raw_kernel_view(&atlas_plane)
        .expect("committed raw atlas view");
    let retained_atlas_allocation = initial_atlas.allocation() == committed_atlas.allocation();
    assert!(
        retained_atlas_allocation,
        "the Ground commit replaced its resident allocation"
    );
    assert_eq!(initial_atlas.stamp().revision, accepted.source_revision);
    assert_eq!(committed_atlas.stamp().revision, ground.revision());
    assert_eq!(
        committed_atlas.stamp().valid_read_epoch.get(),
        COMMITTED_EPOCH
    );
    let committed = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &view, &grade)
                .changed(BrickChange::Slots(&slots))
                .with_leased_atlas(tracer_lease(
                    &committed_atlas,
                    map.projection_revision(),
                    initial_extent,
                    slot_origin,
                    slot_origin,
                    [8; 3],
                ))
                .with_expected_read_epoch(COMMITTED_EPOCH),
        )
        .expect("committed resident Ground frame");
    assert_eq!(committed.diagnostics.epoch_lease_rejections, 0);
    assert_eq!(
        committed.diagnostics.leased_atlas_bytes,
        8 * 8 * 8,
        "the committed lease should copy only its retained brick"
    );
    assert_eq!(
        committed.diagnostics.observed_read_epoch,
        Some(COMMITTED_EPOCH)
    );
    assert_eq!(
        committed.diagnostics.brick_upload_bytes,
        size_of::<u32>() as u64,
        "the committed frame should upload one pointer and zero voxel bytes"
    );
    let changed_pixels = initial
        .pixels
        .chunks_exact(4)
        .zip(committed.pixels.chunks_exact(4))
        .filter(|(before, after)| before != after)
        .count();
    assert!(
        changed_pixels > 0,
        "the accepted resident delta is not visible"
    );

    // The stated epoch is validated identity: the same committed lease at
    // any other expected epoch is refused, and the CPU fallback draws the
    // same committed picture.
    let mut epoch_probe =
        BrickTracer::with_device(setup.device.clone(), setup.queue.clone(), WIDTH, HEIGHT);
    epoch_probe
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(accepted.source_revision),
            &view,
            &grade,
        ))
        .expect("epoch probe baseline");
    let refused = epoch_probe
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &view, &grade)
                .changed(BrickChange::Slots(&slots))
                .with_leased_atlas(tracer_lease(
                    &committed_atlas,
                    map.projection_revision(),
                    initial_extent,
                    slot_origin,
                    slot_origin,
                    [8; 3],
                ))
                .with_expected_read_epoch(COMMITTED_EPOCH + 1),
        )
        .expect("mismatched epoch frame");
    assert_eq!(refused.diagnostics.epoch_lease_rejections, 1);
    assert_eq!(refused.diagnostics.leased_atlas_bytes, 0);
    assert_eq!(
        refused.diagnostics.brick_upload_bytes,
        size_of::<u32>() as u64 + 512,
        "the refused lease falls back to one CPU slot"
    );
    assert_eq!(
        refused.pixels, committed.pixels,
        "the CPU fallback must draw the committed picture"
    );

    let steady = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &view,
            &grade,
        ))
        .expect("unchanged committed frame");
    assert_eq!(steady.pixels, committed.pixels);
    assert_eq!(steady.diagnostics.brick_upload_bytes, 0);
    assert_eq!(steady.diagnostics.leased_atlas_bytes, 0);

    let wire = serde_json::to_vec(&accepted).expect("serialize accepted delta");
    let replayed: AcceptedGroundDelta =
        serde_json::from_slice(&wire).expect("deserialize accepted delta");
    let mut replay_ground = initial_ground;
    let replay_removed = replay_ground.carve(replayed.at, replayed.radius);
    assert_eq!(replay_removed, replayed.removed_voxels);
    assert_eq!(replay_ground, ground);

    println!(
        "{}",
        serde_json::to_string_pretty(&Receipt {
            gate: "resident-ground-composition",
            adapter: adapter.name,
            backend: format!("{:?}", adapter.backend),
            source_revision: accepted.source_revision,
            committed_revision: accepted.committed_revision,
            burn_raw_same_allocation: same_allocation,
            durability_readback_bytes: size_of::<f32>(),
            proposed_occupancy,
            accepted_removed_voxels: accepted.removed_voxels,
            retained_atlas_allocation,
            resident_patch_bytes: patch_end - patch_start,
            initial_resident_voxel_bytes: initial.diagnostics.leased_atlas_bytes,
            committed_resident_voxel_bytes: committed.diagnostics.leased_atlas_bytes,
            committed_read_epoch: committed
                .diagnostics
                .observed_read_epoch
                .expect("committed lease epoch"),
            committed_cpu_projection_bytes: committed.diagnostics.brick_upload_bytes,
            expected_read_epoch: COMMITTED_EPOCH,
            epoch_mismatch_refusals: refused.diagnostics.epoch_lease_rejections,
            epoch_fallback_matches_committed: refused.pixels == committed.pixels,
            resident_bytes_in_use: memory_after_commit.bytes_in_use,
            resident_number_allocs: memory_after_commit.number_allocs,
            resident_commit_allocator_growth: memory_after_commit
                .bytes_in_use
                .saturating_sub(memory_before_commit.bytes_in_use),
            changed_pixels,
            unchanged_atlas_bytes: steady.diagnostics.leased_atlas_bytes,
            accepted_delta_bytes: wire.len(),
            replay_matches: true,
        })
        .expect("serialize receipt")
    );
}

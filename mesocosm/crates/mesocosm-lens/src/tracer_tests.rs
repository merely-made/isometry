// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::places::{Ground, Places};

use crate::{
    BrickChange, BrickFrameInput, BrickMap, BrickProjectionRevision, BrickRevision,
    BrickTraceError, BrickTracer, CritterPose, Flight, Grade, LeasedAtlas, critter::Capsule,
};

mod grade;
mod roster;

fn ground() -> Ground {
    let grown = Places::grown(4_242, 4, 64);
    Ground::grow(&grown, 64)
}

fn flight(ground: &Ground) -> Flight {
    let top = ground.surface(4, 4).expect("ground column");
    Flight {
        eye: [4.5, top as f32 + 14.0, 4.5],
        yaw: 0.0,
        pitch: -1.52,
        fov: 0.15,
        far: 48.0,
    }
}

#[test]
fn a_carved_ground_updates_only_its_brick_slots_in_the_live_tracer() {
    let mut ground = ground();
    let mut map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping brick tracer receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let before = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &camera,
            &grade,
        ))
        .expect("initial brick frame");
    assert!(before.diagnostics.brick_upload_bytes > 0);
    assert_eq!(before.diagnostics.trace_passes, 1);

    let top = ground.surface(4, 4).expect("ground column");
    assert!(ground.carve([4, top, 4], 2) > 0);
    let dirty = ground.drain_dirty();
    let slots = map
        .refresh(&ground, dirty)
        .expect("a carve preserves the brick map shape");
    assert!(!slots.is_empty());
    let after = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .changed(BrickChange::Slots(&slots)),
        )
        .expect("updated brick frame");

    assert_eq!(
        after.diagnostics.brick_upload_bytes,
        slots.len() as u64 * (8 * 8 * 8 + size_of::<u32>()) as u64,
        "a carve uploads just its changed brick slots"
    );
    assert_ne!(
        before.pixels, after.pixels,
        "the opened material is visible"
    );
}

#[test]
fn a_steady_brick_frame_has_no_upload_churn() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping brick tracer receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::retro(3);
    let input = BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade);
    let first = tracer.capture(input).expect("first brick frame");
    let steady = tracer.capture(input).expect("steady brick frame");

    assert_eq!(steady.diagnostics.brick_upload_bytes, 0);
    assert_eq!(steady.diagnostics.uniform_upload_bytes, 0);
    assert_eq!(steady.diagnostics.resource_creations, 0);
    assert_eq!(steady.pixels, first.pixels);
}

#[test]
fn an_equal_extent_projection_republishes_without_recreating_textures() {
    let ground = ground();
    let mut keys = ground.keys();
    let first_key = keys.next().expect("ground has a first brick");
    let second_key = keys
        .find(|key| *key != first_key)
        .expect("ground has a second brick");
    let first_map = BrickMap::from_ground_keys(&ground, BrickProjectionRevision(4), [first_key])
        .expect("first one-brick projection");
    let second_map = BrickMap::from_ground_keys(&ground, BrickProjectionRevision(5), [second_key])
        .expect("second one-brick projection");
    assert_eq!(first_map.pointer_extent(), second_map.pointer_extent());
    assert_eq!(first_map.atlas_extent(), second_map.atlas_extent());
    assert_ne!(first_map.origin(), second_map.origin());

    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping projection replacement receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let revision = BrickRevision(ground.revision());
    tracer
        .capture(BrickFrameInput::new(&first_map, revision, &camera, &grade))
        .expect("first projection");
    let replaced = tracer
        .capture(BrickFrameInput::new(&second_map, revision, &camera, &grade))
        .expect("equal-sized replacement projection");

    assert!(replaced.diagnostics.projection_replaced);
    assert!(!replaced.diagnostics.map_recreated);
    assert_eq!(replaced.diagnostics.resource_creations, 0);
    assert_eq!(replaced.diagnostics.bind_group_rebuilds, 0);
    assert_eq!(
        replaced.diagnostics.brick_upload_bytes,
        size_of_val(second_map.pointers()) as u64 + second_map.atlas().len() as u64
    );
    let mut control = BrickTracer::headless(64, 64).expect("control tracer");
    let control_frame = control
        .capture(BrickFrameInput::new(&second_map, revision, &camera, &grade))
        .expect("fresh second projection");
    assert_eq!(
        replaced.pixels, control_frame.pixels,
        "equal-sized replacement did not publish the selected page"
    );

    let steady = tracer
        .capture(BrickFrameInput::new(&second_map, revision, &camera, &grade))
        .expect("steady replacement projection");
    assert!(!steady.diagnostics.projection_replaced);
    assert_eq!(steady.diagnostics.brick_upload_bytes, 0);
}

#[test]
fn an_unknown_changed_slot_is_refused_before_the_cache_stamp_advances() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping invalid-slot receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let revision = BrickRevision(ground.revision());
    tracer
        .capture(BrickFrameInput::new(&map, revision, &camera, &grade))
        .expect("baseline frame");

    let next = BrickRevision(revision.0 + 1);
    let invalid = [u32::MAX];
    assert!(matches!(
        tracer.capture(
            BrickFrameInput::new(&map, next, &camera, &grade).changed(BrickChange::Slots(&invalid))
        ),
        Err(crate::BrickTraceError::UnknownBrickSlot(u32::MAX))
    ));

    let admitted = tracer
        .capture(BrickFrameInput::new(&map, next, &camera, &grade))
        .expect("full publication after refusal");
    assert!(
        admitted.diagnostics.brick_upload_bytes > 0,
        "the refused slot change advanced the resident cache stamp"
    );
}

#[test]
fn a_nearer_sdf_body_composes_in_front_of_ground() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping brick/body composition receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let terrain = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &camera,
            &grade,
        ))
        .expect("terrain frame");
    let top = ground.surface(4, 4).expect("ground column") as f32;
    let pose = CritterPose::from_capsules(
        vec![Capsule {
            a: [4.5, top + 7.0, 4.5],
            ra: 1.4,
            b: [4.5, top + 5.0, 4.5],
            rb: 1.1,
        }],
        [[4.5, top + 6.6, 4.5, 0.18], [4.5, top + 6.2, 4.5, 0.18]],
        [0.15, 0.86, 0.32],
    );
    let composed = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .with_pose(&pose),
        )
        .expect("composed frame");

    assert_eq!(composed.diagnostics.brick_upload_bytes, 0);
    assert!(composed.diagnostics.uniform_upload_bytes > 0);
    assert_ne!(
        terrain.pixels, composed.pixels,
        "body wins its nearer pixels"
    );
}

/// The resident-views seam, against the real tracer.
///
/// A producer holding voxels on the GPU fills the atlas without the CPU
/// seeing one, and the tracer's bindings do not change: it still samples
/// `texture_3d`, which is what keeps the downlevel path alive. The lease
/// carries the revision it was materialized at, so a stale one is
/// refused rather than presented as current.
#[test]
fn a_leased_atlas_fills_the_tracer_without_a_cpu_upload() {
    use wgpu::util::DeviceExt;

    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping leased atlas receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let source_revision = BrickRevision(ground.revision());
    let revision = BrickRevision(source_revision.0 + 1);

    tracer
        .capture(BrickFrameInput::new(&map, source_revision, &camera, &grade))
        .expect("baseline CPU frame");

    // A producer's resident voxels: one brick of solid material inside a
    // wider strided source allocation, on the tracer's own device.
    let edge = 8u32;
    let source_width = edge * 2;
    let mut voxels = vec![0u8; (source_width * edge * edge) as usize];
    for z in 0..edge {
        for y in 0..edge {
            for x in edge..source_width {
                voxels[((z * edge + y) * source_width + x) as usize] = 2;
            }
        }
    }
    let leased_buffer = tracer
        .device()
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("test lease"),
            contents: &voxels,
            usage: wgpu::BufferUsages::COPY_SRC,
        });
    let leased = LeasedAtlas {
        buffer: &leased_buffer,
        offset: 0,
        size: voxels.len() as u64,
        source_origin: [edge, 0, 0],
        source_bytes_per_row: source_width,
        source_rows_per_image: edge,
        slot_origin: map.atlas_slot_origin(1).expect("first atlas slot"),
        extent: [edge; 3],
        revision,
        projection_revision: map.projection_revision(),
        read_epoch: 73,
    };
    let changed_slots = [1];

    let leased_frame = tracer
        .capture(
            BrickFrameInput::new(&map, revision, &camera, &grade)
                .changed(BrickChange::Slots(&changed_slots))
                .with_leased_atlas(leased),
        )
        .expect("leased brick frame");
    assert_eq!(
        leased_frame.diagnostics.leased_atlas_bytes,
        u64::from(edge * edge * edge),
        "the lease did not reach the atlas"
    );
    assert_eq!(
        leased_frame.diagnostics.stale_lease_rejections, 0,
        "a current lease was refused"
    );
    assert_eq!(leased_frame.diagnostics.observed_read_epoch, Some(73));
    assert_eq!(leased_frame.diagnostics.incomplete_lease_rejections, 0);
    // The pointer volume still uploads (it identifies slots and carries
    // no material); the atlas does not.
    assert!(
        leased_frame.diagnostics.brick_upload_bytes < map.atlas().len() as u64,
        "the atlas was uploaded from the CPU despite the lease: {} bytes",
        leased_frame.diagnostics.brick_upload_bytes
    );

    // A lease stamped at another revision is refused, and the frame
    // falls back to the CPU upload rather than showing stale voxels.
    let mut fresh = BrickTracer::headless(64, 64).expect("second tracer");
    fresh
        .capture(BrickFrameInput::new(&map, source_revision, &camera, &grade))
        .expect("stale-case baseline");
    let stale = LeasedAtlas {
        revision: BrickRevision(revision.0 + 1),
        ..leased
    };
    let stale_frame = fresh
        .capture(
            BrickFrameInput::new(&map, revision, &camera, &grade)
                .changed(BrickChange::Slots(&changed_slots))
                .with_leased_atlas(stale),
        )
        .expect("stale lease frame");
    assert_eq!(stale_frame.diagnostics.stale_lease_rejections, 1);
    assert_eq!(stale_frame.diagnostics.leased_atlas_bytes, 0);
    assert_eq!(stale_frame.diagnostics.observed_read_epoch, None);
    assert_eq!(stale_frame.diagnostics.incomplete_lease_rejections, 0);
    assert!(
        stale_frame.diagnostics.brick_upload_bytes >= u64::from(edge * edge * edge + 4),
        "a refused lease must fall back to the CPU upload"
    );

    // Equal-sized pages can reuse every physical coordinate while assigning
    // slot one to another world brick. A lease for the previous projection is
    // therefore stale even when its Ground revision and range still match.
    let mut projection_keys = ground.keys();
    let previous_key = projection_keys.next().expect("one ground brick");
    let replacement_key = projection_keys
        .find(|key| *key != previous_key)
        .expect("another ground brick");
    let previous_map =
        BrickMap::from_ground_keys(&ground, BrickProjectionRevision(0), [previous_key])
            .expect("previous projected map");
    let projected_map =
        BrickMap::from_ground_keys(&ground, BrickProjectionRevision(1), [replacement_key])
            .expect("replacement projected map");
    assert_eq!(
        previous_map.pointer_extent(),
        projected_map.pointer_extent()
    );
    assert_eq!(previous_map.atlas_extent(), projected_map.atlas_extent());
    let mut projected = BrickTracer::headless(64, 64).expect("projected tracer");
    projected
        .capture(BrickFrameInput::new(
            &previous_map,
            source_revision,
            &camera,
            &grade,
        ))
        .expect("projection baseline");
    let wrong_projection = LeasedAtlas {
        revision: source_revision,
        projection_revision: BrickProjectionRevision(0),
        slot_origin: projected_map
            .atlas_slot_origin(1)
            .expect("projected first slot"),
        ..leased
    };
    let projection_frame = projected
        .capture(
            BrickFrameInput::new(&projected_map, source_revision, &camera, &grade)
                .with_leased_atlas(wrong_projection),
        )
        .expect("projection-mismatch lease frame");
    assert_eq!(projection_frame.diagnostics.projection_lease_rejections, 1);
    assert_eq!(projection_frame.diagnostics.leased_atlas_bytes, 0);
    assert!(projection_frame.diagnostics.projection_replaced);
    assert!(!projection_frame.diagnostics.map_recreated);
    assert_eq!(projection_frame.diagnostics.resource_creations, 0);
    assert_eq!(projection_frame.diagnostics.bind_group_rebuilds, 0);
    assert_eq!(
        projection_frame.diagnostics.brick_upload_bytes,
        size_of_val(projected_map.pointers()) as u64 + projected_map.atlas().len() as u64,
        "a projection-mismatch lease must fall back to a full CPU publication"
    );
    let mut projection_control =
        BrickTracer::headless(64, 64).expect("projection CPU control tracer");
    let projection_control_frame = projection_control
        .capture(BrickFrameInput::new(
            &projected_map,
            source_revision,
            &camera,
            &grade,
        ))
        .expect("fresh projection CPU control");
    assert_eq!(
        projection_frame.pixels, projection_control_frame.pixels,
        "projection lease refusal did not render the CPU fallback"
    );

    // A lease whose extent overruns its range is refused too. The
    // producer pools planes into one buffer, so copying this would paint
    // a neighbouring allocation into the world rather than fault.
    let mut third = BrickTracer::headless(64, 64).expect("third tracer");
    third
        .capture(BrickFrameInput::new(&map, source_revision, &camera, &grade))
        .expect("misfit-case baseline");
    let misfit = LeasedAtlas {
        extent: [edge * 2, edge, edge],
        ..leased
    };
    assert!(!misfit.fits());
    let misfit_frame = third
        .capture(
            BrickFrameInput::new(&map, revision, &camera, &grade)
                .changed(BrickChange::Slots(&changed_slots))
                .with_leased_atlas(misfit),
        )
        .expect("misfit lease frame");
    assert_eq!(misfit_frame.diagnostics.misfit_lease_rejections, 1);
    assert_eq!(misfit_frame.diagnostics.leased_atlas_bytes, 0);
    assert_eq!(misfit_frame.diagnostics.observed_read_epoch, None);
    assert_eq!(misfit_frame.diagnostics.incomplete_lease_rejections, 0);
    assert!(
        misfit_frame.diagnostics.brick_upload_bytes >= u64::from(edge * edge * edge + 4),
        "a misfit lease must fall back to the CPU upload: uploaded {} of atlas {} (stale case uploaded {})",
        misfit_frame.diagnostics.brick_upload_bytes,
        map.atlas().len(),
        stale_frame.diagnostics.brick_upload_bytes
    );

    // A valid partial lease cannot stand in for a declared full refresh.
    let mut fourth = BrickTracer::headless(64, 64).expect("fourth tracer");
    let incomplete_frame = fourth
        .capture(BrickFrameInput::new(&map, revision, &camera, &grade).with_leased_atlas(leased))
        .expect("incomplete lease frame");
    assert_eq!(incomplete_frame.diagnostics.incomplete_lease_rejections, 1);
    assert_eq!(incomplete_frame.diagnostics.leased_atlas_bytes, 0);
    assert_eq!(incomplete_frame.diagnostics.observed_read_epoch, None);
    assert!(
        incomplete_frame.diagnostics.brick_upload_bytes >= map.atlas().len() as u64,
        "an incomplete full-frame lease suppressed the CPU fallback"
    );
}

/// The depth join, proven without a raster engine: a caller-owned depth
/// texture pre-cleared to a chosen plane stands in for a raster tenant's
/// stored depth, and the traced pass must lose exactly where that plane is
/// nearer than the traced surface's own clip depth.
#[test]
fn the_depth_join_settles_pixels_between_tracer_and_raster_depth() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("no adapter; skipping depth join receipt");
        return;
    };
    let Ok((device, queue)) = pollster::block_on(adapter.request_device(&Default::default()))
    else {
        eprintln!("no device; skipping depth join receipt");
        return;
    };
    let (width, height) = (96u32, 64u32);
    let mut tracer = BrickTracer::with_device(device.clone(), queue.clone(), width, height);
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let camera = flight(&ground);
    let grade = Grade::clay();

    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth join colour"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::FRAME_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&Default::default());
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth join depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth.create_view(&Default::default());

    // The stand-in raster: sentinel red everywhere, depth at a chosen value.
    let clear = |depth_value: f32| {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("stand-in raster clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &colour_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(depth_value),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        drop(pass);
        queue.submit([encoder.finish()]);
    };
    let sentinel_pixels = |label: &str| -> usize {
        let padded = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("depth join readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            colour.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect(label);
        let data = slice.get_mapped_range().expect("map range");
        let mut sentinels = 0;
        for row in 0..height {
            let start = (row * padded) as usize;
            for pixel in data[start..start + (width * 4) as usize].chunks_exact(4) {
                if pixel[..3] == [255, 0, 0] {
                    sentinels += 1;
                }
            }
        }
        sentinels
    };
    // Depth as a world-z ramp: clip.z = (z - 4.6) / 1.2, clip.w = 1. The
    // flight camera's rays leave eye z 4.5 and descend roughly 14 voxels
    // with forward z slopes of 0.009 to 0.093, so hits land near z 4.6 at
    // the frame's bottom and z 5.8 at its top. Against a raster plane at
    // 0.5 the trace then wins below z 5.2 and loses above it, and the
    // gentle ramp keeps both classes present under relief variation.
    // Columns are WGSL mat4x4 columns.
    let z_plane = [
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0 / 1.2, 0.0],
        [0.0, 0.0, -4.6 / 1.2, 1.0],
    ];
    let input = |clip: [[f32; 4]; 4]| {
        BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
            .with_clip_from_world(clip)
    };
    let trace = |tracer: &mut BrickTracer, clip: [[f32; 4]; 4]| {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        tracer
            .encode_with_depth(&mut encoder, &colour_view, &depth_view, input(clip))
            .expect("depth join frame");
        queue.submit([encoder.finish()]);
    };
    let total = (width * height) as usize;

    // A frame input without the matrix must refuse rather than write
    // identity depth.
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let refused = tracer.encode_with_depth(
        &mut encoder,
        &colour_view,
        &depth_view,
        BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade),
    );
    assert_eq!(refused, Err(BrickTraceError::MissingClipFromWorld));
    drop(encoder);

    // Constant traced depth on either side of the raster plane first: the
    // whole frame must change hands, both ways.
    let constant_depth = |depth: f32| {
        [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, depth, 1.0],
        ]
    };
    clear(0.5);
    trace(&mut tracer, constant_depth(0.0));
    assert_eq!(sentinel_pixels("nearer trace"), 0);

    clear(0.5);
    trace(&mut tracer, constant_depth(1.0));
    assert_eq!(sentinel_pixels("farther trace"), total);

    // Raster mid-plane: the trace wins exactly where its surface sits
    // before world z 4.55, and only there.
    clear(0.5);
    trace(&mut tracer, z_plane);
    let split = sentinel_pixels("mid raster");
    assert!(
        split > 0 && split < total,
        "the mid-plane raster must occlude some traced pixels and lose others, kept {split} of {total}"
    );
}

/// V1b's tracer half: a projection advance that declares its loaded slots
/// re-uploads the pointer volume and exactly those slots' atlas bytes over
/// retained textures, and the picture matches a map rebuilt from scratch.
#[test]
fn a_retargeted_projection_uploads_pointers_and_only_loaded_slots() {
    let ground = ground();
    let keys: Vec<[i16; 3]> = ground.keys().collect();
    assert!(keys.len() > 8, "the fixture world has a real brick set");
    let mut low = keys[0];
    let mut high = keys[0];
    for key in &keys {
        for axis in 0..3 {
            low[axis] = low[axis].min(key[axis]);
            high[axis] = high[axis].max(key[axis]);
        }
    }
    let pointer_extent =
        [0, 1, 2].map(|axis| (i32::from(high[axis]) - i32::from(low[axis]) + 1) as u32);
    let rows = (keys.len() as u32 + 1).div_ceil(256);

    // Selection A drops the last key; selection B drops the first. The
    // retarget between them retains everything but one brick each way.
    let a: Vec<_> = keys[..keys.len() - 1].to_vec();
    let b: Vec<_> = keys[1..].to_vec();

    let mut map = BrickMap::with_capacity(BrickProjectionRevision(0), rows, pointer_extent)
        .expect("capacity map");
    map.retarget(&ground, BrickProjectionRevision(1), a.iter().copied())
        .expect("first selection");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping retarget receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let revision = BrickRevision(ground.revision());
    tracer
        .capture(BrickFrameInput::new(&map, revision, &camera, &grade))
        .expect("first full frame");

    let delta = map
        .retarget(&ground, BrickProjectionRevision(2), b.iter().copied())
        .expect("travelled selection");
    assert_eq!(delta.loaded_slots.len(), 1);
    assert_eq!(delta.evicted, 1);
    let travelled = tracer
        .capture(
            BrickFrameInput::new(&map, revision, &camera, &grade)
                .changed(BrickChange::Slots(&delta.loaded_slots)),
        )
        .expect("retargeted frame");

    assert_eq!(travelled.diagnostics.resource_creations, 0);
    assert!(!travelled.diagnostics.map_recreated);
    assert!(travelled.diagnostics.projection_replaced);
    let pointer_bytes = std::mem::size_of_val(map.pointers()) as u64;
    assert_eq!(
        travelled.diagnostics.brick_upload_bytes,
        pointer_bytes + delta.loaded_slots.len() as u64 * 512,
        "a retarget publishes the pointer volume plus only its loaded slots"
    );

    // The picture must not know the cache's slot history: a fresh tracer
    // over a from-scratch map of the same selection draws the same frame.
    let rebuilt = BrickMap::from_ground_keys(&ground, BrickProjectionRevision(2), b)
        .expect("rebuilt selection");
    let mut fresh = BrickTracer::headless(96, 64).expect("fresh tracer");
    let reference = fresh
        .capture(BrickFrameInput::new(&rebuilt, revision, &camera, &grade))
        .expect("rebuilt frame");
    assert_eq!(travelled.pixels, reference.pixels);
}

/// A stated expected read epoch turns the lease's epoch into validated
/// identity: any other epoch is refused and the CPU path fills the frame.
#[test]
fn a_lease_at_the_wrong_epoch_is_refused() {
    use wgpu::util::DeviceExt;

    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping epoch receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let source_revision = BrickRevision(ground.revision());
    let revision = BrickRevision(source_revision.0 + 1);
    tracer
        .capture(BrickFrameInput::new(&map, source_revision, &camera, &grade))
        .expect("baseline CPU frame");

    let edge = 8u32;
    let voxels = vec![2u8; (edge * edge * edge) as usize];
    let lease_buffer = |device: &wgpu::Device| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("epoch lease"),
            contents: &voxels,
            usage: wgpu::BufferUsages::COPY_SRC,
        })
    };
    fn lease<'a>(
        buffer: &'a wgpu::Buffer,
        map: &BrickMap,
        revision: BrickRevision,
        size: u64,
        read_epoch: u64,
    ) -> LeasedAtlas<'a> {
        LeasedAtlas {
            buffer,
            offset: 0,
            size,
            source_origin: [0, 0, 0],
            source_bytes_per_row: 8,
            source_rows_per_image: 8,
            slot_origin: map.atlas_slot_origin(1).expect("first atlas slot"),
            extent: [8; 3],
            revision,
            projection_revision: map.projection_revision(),
            read_epoch,
        }
    }
    let changed_slots = [1];

    let refused_buffer = lease_buffer(tracer.device());
    let refused = tracer
        .capture(
            BrickFrameInput::new(&map, revision, &camera, &grade)
                .changed(BrickChange::Slots(&changed_slots))
                .with_leased_atlas(lease(
                    &refused_buffer,
                    &map,
                    revision,
                    voxels.len() as u64,
                    72,
                ))
                .with_expected_read_epoch(73),
        )
        .expect("mismatched epoch frame");
    assert_eq!(refused.diagnostics.epoch_lease_rejections, 1);
    assert_eq!(refused.diagnostics.leased_atlas_bytes, 0);
    assert!(
        refused.diagnostics.brick_upload_bytes > 0,
        "the CPU path must fill a frame whose lease was refused"
    );

    // The matching case runs on its own tracer: the refused frame above
    // already advanced the first tracer's revision through the CPU path.
    let mut fresh = BrickTracer::headless(64, 64).expect("second tracer");
    fresh
        .capture(BrickFrameInput::new(&map, source_revision, &camera, &grade))
        .expect("matching-case baseline");
    let accepted_buffer = lease_buffer(fresh.device());
    let accepted = fresh
        .capture(
            BrickFrameInput::new(&map, revision, &camera, &grade)
                .changed(BrickChange::Slots(&changed_slots))
                .with_leased_atlas(lease(
                    &accepted_buffer,
                    &map,
                    revision,
                    voxels.len() as u64,
                    73,
                ))
                .with_expected_read_epoch(73),
        )
        .expect("matching epoch frame");
    assert_eq!(accepted.diagnostics.epoch_lease_rejections, 0);
    assert_eq!(
        accepted.diagnostics.leased_atlas_bytes,
        u64::from(edge * edge * edge)
    );
    assert_eq!(accepted.diagnostics.observed_read_epoch, Some(73));
}

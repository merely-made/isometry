// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P3 of mere's spatial compute plan: the wing projection.
//!
//! The second consumer of a resident spatial buffer, and the one that
//! decides whether the lease is domain-neutral. Ambience motes live in
//! a padded-3D position buffer that explicit-regime kernels advance;
//! renderling draws them as instanced geometry under a real 3D camera,
//! reading this frame's positions with no readback between the compute
//! that wrote them and the draw that consumed them.
//!
//! What P2 could not test, this does: z is meaningful (the camera looks
//! down at an angle, so depth is visible truth rather than a reserved
//! lane), and the consumer has a *different memory model*. renderling
//! addresses geometry through a craballoc slab rather than through
//! buffers a caller binds, so the lease is read by an adapter kernel
//! that writes into renderling's own storage. That mismatch, and its
//! cost, is the finding this gate exists to produce.

use std::time::Instant;

use ambience_lease::compose::Composer;
use ambience_lease::lease::Ambience;
use ambience_lease::tenant::{TRANSFORM_WORDS, Tenant};
use netrender::{Scene, TenantNeeds, boot_shared};

const SIZE: [u32; 2] = [1280, 720];
const MOTES: u32 = 20_000;
const EXTENT: f32 = 120.0;
const FRAMES: u32 = 300;

fn main() {
    // One device for the ambience kernels, renderling, and netrender.
    // The mesh tenant's usual asks, declared rather than greedy: this
    // is not a JIT runtime, so it states what it needs.
    // The control run skips the re-attach, to prove the guard is load
    // bearing rather than decorative.
    let reattach = !std::env::args().any(|arg| arg == "--no-reattach");

    let handles = boot_shared(
        wgpu::Backends::all(),
        None,
        &TenantNeeds {
            optional_features: wgpu::Features::INDIRECT_FIRST_INSTANCE
                | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
                | wgpu::Features::VERTEX_WRITABLE_STORAGE
                | wgpu::Features::CLEAR_TEXTURE,
            label: Some("ambience-lease probe"),
            ..Default::default()
        },
    )
    .expect("one device for kernels, renderling, and netrender");
    println!("adapter: {}", handles.adapter.get_info().name);
    println!("motes: {MOTES}, extent {EXTENT}");

    let mut ambience = Ambience::new(&handles.device, &handles.queue, MOTES, EXTENT);
    let lease = ambience.lease();
    // The consumer frames itself from the lease rather than from the
    // probe's constants: the camera's distance is the lease's extent,
    // and the adapter's stride assumption is checked against the
    // lease's declared stride instead of assumed.
    assert_eq!(
        lease.stride_bytes, 16,
        "the adapter kernel indexes vec4f; a different stride needs a different kernel"
    );
    let tenant = Tenant::new(&handles, SIZE, lease.count, lease.extent);

    // The lease's consumer-side truth: where renderling put the
    // transforms, and which buffer they live in. Both are read *from
    // renderling*, never assumed.
    let (slab, base) = tenant.slab_and_transform_base();
    println!("renderling slab: transforms begin at word {base}, stride {TRANSFORM_WORDS}");
    ambience.attach_slab(&slab);

    // Warm up: compile pipelines, let the cloud start moving.
    for _ in 0..30 {
        ambience.step(1.0 / 60.0, base, TRANSFORM_WORDS);
        tenant.draw();
    }

    // Composed before the loop so the run can be sampled mid-flight:
    // the control below compares a frame taken just before the forced
    // growth against the last frame.
    let composer = Composer::new(handles.clone(), SIZE);
    let chrome = Scene::new(SIZE[0], SIZE[1]);
    let mut before_growth: Option<Vec<u8>> = None;

    let start = Instant::now();
    let mut worst = 0.0f64;
    let mut regrows = 0u32;
    // Held so the forced growth is not immediately freed again.
    let mut ballast: Option<renderling::geometry::Vertices> = None;
    for frame_index in 0..FRAMES {
        let frame = Instant::now();

        // The epoch check, every frame, before anything is published:
        // if the consumer's allocator recreated its buffer, the
        // producer's binding is stale and must be rebuilt.
        let (slab, grew) = tenant.commit();
        if grew {
            regrows += 1;
            if reattach {
                ambience.attach_slab(&slab);
            }
        }

        ambience.step(1.0 / 60.0, base, TRANSFORM_WORDS);
        tenant.draw();
        handles
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("frame poll");
        let ms = frame.elapsed().as_secs_f64() * 1e3;
        worst = worst.max(ms);

        // Halfway through, make the consumer's allocator grow. Without
        // the check above this is where the cloud would quietly stop.
        if frame_index == FRAMES / 2 {
            let master = composer.compose(&chrome, &tenant.view);
            before_growth = Some(composer.capture(&master).pixels);
            ballast = Some(tenant.grow(400_000));
        }
    }
    let avg = start.elapsed().as_secs_f64() * 1e3 / FRAMES as f64;
    println!("wing projection: {FRAMES} frames, avg {avg:.2} ms (worst {worst:.2})");
    println!(
        "slab regrows detected: {regrows}, re-attached: {}",
        if reattach { "yes" } else { "NO (control run)" }
    );
    assert!(
        regrows > 0,
        "the forced growth never recreated the buffer: the epoch receipt proved nothing"
    );
    assert!(ballast.is_some(), "ballast was not allocated");

    // The positive control. With the epoch honoured, the motes kept
    // moving across the growth; without it, the producer publishes into
    // an orphaned buffer while renderling reads the new one, and the
    // cloud freezes at its last published positions. The same run
    // measures it either way, so an unchanged picture is evidence
    // rather than an assumption.
    let after = {
        let master = composer.compose(&chrome, &tenant.view);
        composer.capture(&master).pixels
    };
    let before = before_growth.expect("a frame was sampled before growth");
    let changed = before
        .chunks_exact(4)
        .zip(after.chunks_exact(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count() as f32
        / (SIZE[0] * SIZE[1]) as f32;
    println!("pixels changed across the growth: {:.2}%", changed * 100.0);
    if reattach {
        assert!(
            changed > 0.02,
            "the cloud stopped moving across the growth even with the epoch honoured"
        );
    } else {
        assert!(
            changed < 0.001,
            "the control did not freeze: {changed} of the frame still changed, so the              guard is not what keeps the cloud alive"
        );
        println!("control: the cloud froze, which is what the epoch check prevents");
        return;
    }

    // The receipt that renderling drew *these* motes: read the resident
    // positions once (a diagnostic, outside the frame discipline), and
    // check the picture's lit pixels sit where the cloud projects.
    let positions = ambience.read_positions_once();
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in &positions {
        for axis in 0..3 {
            lo[axis] = lo[axis].min(p[axis]);
            hi[axis] = hi[axis].max(p[axis]);
        }
    }
    println!(
        "cloud bounds: x [{:.0}, {:.0}]  y [{:.0}, {:.0}]  z [{:.0}, {:.0}]",
        lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]
    );
    // z must carry real extent, not sit in a plane: this is the axis a
    // 2D canvas never exercised, and the reason padded 3D was chosen
    // before any consumer needed it.
    let depth_span = hi[2] - lo[2];
    assert!(
        depth_span > EXTENT / 2.0,
        "the cloud is flat: depth span {depth_span:.1} is not carrying anything"
    );

    let master = composer.compose(&chrome, &tenant.view);
    let capture = composer.capture(&master);
    let path = std::path::Path::new("../../../testing/paredros/p3_ambience_lease.png");
    capture.write_png(path).expect("write receipt png");
    println!(
        "receipt: {} distinct colours, {:.1}% lit -> {}",
        capture.distinct,
        capture.lit * 100.0,
        path.display()
    );
    // Shaded 3D motes, so the distinct-colour guard is the right one
    // here (unlike mere's flat 2D graph, which needed coverage).
    assert!(
        capture.distinct > 64,
        "only {} colours: renderling drew nothing lit",
        capture.distinct
    );
    assert!(capture.lit > 0.01, "the cloud is not in frame");
}

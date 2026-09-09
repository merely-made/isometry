// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The pose roster: many bodies in the slab, and the cap that bounds them.

use crate::{
    BrickFrameInput, BrickMap, BrickRevision, BrickTracer, CritterPose, Grade, MAX_ROSTER,
    TraceCamera, critter::Capsule,
};

use super::{flight, ground};

/// TD3's whole claim: the section shows every body in the slab, not one.
///
/// A side-on orthographic slab like the terrarium's, two bodies apart along
/// x, and each one has to win its own pixels. The single-pose path is left
/// untouched beside them: an empty roster must trace exactly what it always
/// did.
#[test]
fn a_roster_draws_every_body_in_the_slab_and_an_empty_one_changes_nothing() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping brick roster receipt");
        return;
    };
    let top = ground.surface(4, 4).expect("ground column") as f32;
    let camera = TraceCamera::orthographic_slab(
        [4.5, top + 4.0, 4.5],
        [0.0, 0.0, -1.0],
        [0.0, 1.0, 0.0],
        8.0,
        1.5,
        16.0,
    )
    .expect("slab camera");
    let grade = Grade::clay();
    let revision = BrickRevision(ground.revision());
    let frame = || BrickFrameInput::for_camera(&map, revision, camera, &grade);
    let body = |x: f32, tint: [f32; 3]| {
        CritterPose::from_capsules(
            vec![Capsule {
                a: [x, top + 5.0, 4.5],
                ra: 1.4,
                b: [x, top + 3.0, 4.5],
                rb: 1.1,
            }],
            [[x, top + 4.6, 5.4, 0.18], [x, top + 4.2, 5.4, 0.18]],
            tint,
        )
    };
    let differs = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(l, r)| l != r).count();

    let terrain = tracer.capture(frame()).expect("terrain frame");
    assert_eq!(terrain.diagnostics.roster_members, 0);

    // An empty roster is the single-pose path, byte for byte.
    let empty = tracer
        .capture(frame().with_roster(&[]))
        .expect("empty roster frame");
    assert_eq!(empty.pixels, terrain.pixels);
    assert_eq!(empty.diagnostics.roster_members, 0);

    let one = [body(0.5, [0.15, 0.86, 0.32])];
    let single = tracer
        .capture(frame().with_roster(&one))
        .expect("one-member frame");
    assert_eq!(single.diagnostics.roster_members, 1);
    let single_pixels = differs(&terrain.pixels, &single.pixels);
    assert!(single_pixels > 0, "one roster body wins pixels");

    let two = [body(0.5, [0.15, 0.86, 0.32]), body(8.5, [0.86, 0.24, 0.20])];
    let pair = tracer
        .capture(frame().with_roster(&two))
        .expect("two-member frame");
    assert_eq!(pair.diagnostics.roster_members, 2);
    assert_eq!(pair.diagnostics.roster_dropped, 0);
    assert!(
        differs(&terrain.pixels, &pair.pixels) > single_pixels,
        "the second body is drawn beside the first, not instead of it"
    );

    // Both tints are on screen at once, which is what "every organism in
    // view" means: the roster is not one body wearing two colours.
    let dominant = |pixels: &[u8], channel: usize| {
        pixels
            .chunks(4)
            .filter(|texel| {
                (0..3).all(|other| {
                    other == channel || i32::from(texel[channel]) > i32::from(texel[other]) + 24
                })
            })
            .count()
    };
    let greener = |pixels: &[u8]| dominant(pixels, 1);
    let redder = |pixels: &[u8]| dominant(pixels, 0);
    assert!(greener(&pair.pixels) > greener(&terrain.pixels));
    assert!(redder(&pair.pixels) > redder(&terrain.pixels));
}

/// The documented cap binds, and says so, rather than overrunning the uniform.
#[test]
fn a_roster_past_the_cap_is_truncated_and_counted() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping brick roster cap receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let top = ground.surface(4, 4).expect("ground column") as f32;
    let crowd: Vec<CritterPose> = (0..MAX_ROSTER + 6)
        .map(|index| {
            let x = index as f32 * 0.5;
            CritterPose::from_capsules(
                vec![Capsule {
                    a: [x, top + 5.0, 4.5],
                    ra: 0.6,
                    b: [x, top + 4.0, 4.5],
                    rb: 0.6,
                }],
                [[x, top + 4.8, 4.5, 0.1], [x, top + 4.6, 4.5, 0.1]],
                [0.4, 0.6, 0.5],
            )
        })
        .collect();
    let crowded = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .with_roster(&crowd),
        )
        .expect("crowded frame");
    assert_eq!(crowded.diagnostics.roster_members, MAX_ROSTER as u32);
    assert_eq!(crowded.diagnostics.roster_dropped, 6);
    assert_eq!(crowded.diagnostics.roster_capsules_dropped, 0);
}

/// DC-R1: a member over its capsule budget spends the budget on its
/// silhouette. Document order is the axial chain, so the old rule kept the
/// head end; the fat parts here are last on purpose.
#[test]
fn a_member_over_its_capsule_budget_keeps_its_widest_capsules() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping roster truncation receipt");
        return;
    };
    let top = ground.surface(4, 4).expect("ground column") as f32;
    // Twenty capsules: the first ten thin, the last ten fat.
    let capsules: Vec<Capsule> = (0..20)
        .map(|index| {
            let y = top + 3.0 + index as f32 * 0.2;
            let radius = if index < 10 { 0.1 } else { 0.6 };
            Capsule {
                a: [4.5, y, 4.5],
                ra: radius,
                b: [4.5, y + 0.1, 4.5],
                rb: radius,
            }
        })
        .collect();
    let member = CritterPose::from_capsules(capsules, [[0.0; 4]; 2], [0.4, 0.6, 0.5]);
    let roster = [member];
    let frame = tracer
        .capture(
            BrickFrameInput::new(
                &map,
                BrickRevision(ground.revision()),
                &flight(&ground),
                &Grade::clay(),
            )
            .with_roster(&roster),
        )
        .expect("truncated frame");
    assert_eq!(frame.diagnostics.roster_members, 1);
    assert_eq!(
        frame.diagnostics.roster_capsules_dropped,
        20 - crate::MAX_ROSTER_CAPSULES as u32
    );
    // Which capsules survive is asserted against the uniform itself, in
    // `tracer::params`, where the layout is in scope.
}

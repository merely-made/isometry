// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::maps;

fn flight() -> Flight {
    Flight {
        eye: [32.0, 80.0, 32.0],
        yaw: 0.5,
        pitch: -0.2,
        fov: 0.9,
        far: 300.0,
    }
}

#[test]
fn a_steady_frame_has_no_resource_or_upload_churn() {
    let Some(mut lens) = Lens::headless(96, 64) else {
        eprintln!("no adapter; skipping retained lens receipt");
        return;
    };
    let maps = maps::synthesize(17, 64);
    let flight = flight();
    let grade = Grade::retro(maps.palette.len() as u32);
    let input = FrameInput::new(&maps, MapRevision(1), &flight, &grade);

    let first = lens.capture(input).expect("first frame");
    assert!(first.diagnostics.map_upload_bytes > 0);
    assert!(first.diagnostics.resource_creations > 0);
    assert!(first.pixels.iter().any(|byte| *byte != 0));

    let second = lens.capture(input).expect("steady frame");
    assert_eq!(second.diagnostics.map_upload_bytes, 0);
    assert_eq!(second.diagnostics.uniform_upload_bytes, 0);
    assert_eq!(second.diagnostics.resource_creations, 0);
    assert_eq!(second.diagnostics.bind_group_rebuilds, 0);
    assert_eq!(second.pixels, first.pixels);
    assert_eq!(second.diagnostics.march_passes, 1);
    assert_eq!(second.diagnostics.grade_passes, 1);
    assert_eq!(second.diagnostics.readback_bytes, (96 * 64 * 4) as u64);
}

#[test]
fn camera_and_dirty_region_updates_stay_narrow() {
    let Some(mut lens) = Lens::headless(96, 64) else {
        eprintln!("no adapter; skipping retained lens receipt");
        return;
    };
    let mut maps = maps::synthesize(23, 64);
    let grade = Grade::clay();
    let first_flight = flight();
    lens.capture(FrameInput::new(
        &maps,
        MapRevision(1),
        &first_flight,
        &grade,
    ))
    .expect("first frame");

    let mut moved = first_flight;
    moved.eye[0] += 1.0;
    let camera = lens
        .capture(FrameInput::new(&maps, MapRevision(1), &moved, &grade))
        .expect("camera frame");
    assert_eq!(camera.diagnostics.map_upload_bytes, 0);
    assert_eq!(
        camera.diagnostics.uniform_upload_bytes,
        size_of::<MarchParams>() as u64
    );
    assert_eq!(camera.diagnostics.resource_creations, 0);

    let pose = CritterPose {
        bounds_radius: 1.0,
        tint: [0.2, 0.7, 0.4],
        ..Default::default()
    };
    let posed = lens
        .capture(FrameInput::new(&maps, MapRevision(1), &moved, &grade).with_pose(&pose))
        .expect("pose frame");
    assert_eq!(posed.diagnostics.map_upload_bytes, 0);
    assert_eq!(
        posed.diagnostics.uniform_upload_bytes,
        size_of::<CritterParams>() as u64
    );
    assert_eq!(posed.diagnostics.resource_creations, 0);

    let dirty = DirtyRect {
        x: 3,
        y: 4,
        width: 2,
        height: 2,
    };
    for y in dirty.y..dirty.y + dirty.height {
        for x in dirty.x..dirty.x + dirty.width {
            let index = (y * maps.side + x) as usize;
            maps.height[index] = maps.height[index].wrapping_add(1);
            maps.color[index * 4] = maps.color[index * 4].wrapping_add(1);
        }
    }
    let changed = lens
        .capture(
            FrameInput::new(&maps, MapRevision(2), &moved, &grade)
                .changed(MapChange::Region(dirty))
                .with_pose(&pose),
        )
        .expect("dirty frame");
    assert_eq!(changed.diagnostics.map_upload_bytes, (2 * 2 * 5) as u64);
    assert_eq!(changed.diagnostics.uniform_upload_bytes, 0);
    assert_eq!(changed.diagnostics.resource_creations, 0);
    assert_eq!(changed.diagnostics.bind_group_rebuilds, 0);
}

#[test]
fn resize_rebuilds_only_size_dependent_resources() {
    let Some(mut lens) = Lens::headless(64, 64) else {
        eprintln!("no adapter; skipping retained lens receipt");
        return;
    };
    let maps = maps::synthesize(31, 64);
    let flight = flight();
    let grade = Grade::clay();
    lens.capture(FrameInput::new(&maps, MapRevision(1), &flight, &grade))
        .expect("first frame");

    lens.resize(80, 48);
    let resized = lens
        .capture(FrameInput::new(&maps, MapRevision(1), &flight, &grade))
        .expect("resized frame");
    assert_eq!(resized.diagnostics.map_upload_bytes, 0);
    assert_eq!(resized.diagnostics.uniform_upload_bytes, 0);
    assert!(resized.diagnostics.target_recreated);
    assert_eq!(resized.diagnostics.bind_group_rebuilds, 1);
    assert_eq!((resized.width, resized.height), (80, 48));
}

#[test]
fn oversized_bodies_are_refused_instead_of_truncated() {
    let Some(mut lens) = Lens::headless(64, 64) else {
        eprintln!("no adapter; skipping capsule admission receipt");
        return;
    };
    let maps = maps::synthesize(37, 64);
    let flight = flight();
    let grade = Grade::clay();
    let capsule = crate::critter::Capsule {
        a: [0.0; 3],
        ra: 1.0,
        b: [1.0, 0.0, 0.0],
        rb: 1.0,
    };
    let pose = CritterPose {
        capsules: vec![capsule; MAX_CAPSULES + 1],
        ..Default::default()
    };

    assert_eq!(
        lens.capture(FrameInput::new(&maps, MapRevision(1), &flight, &grade).with_pose(&pose))
            .unwrap_err(),
        LensError::TooManyCapsules {
            actual: MAX_CAPSULES + 1,
            maximum: MAX_CAPSULES,
        }
    );
}

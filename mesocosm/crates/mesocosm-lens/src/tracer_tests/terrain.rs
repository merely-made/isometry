// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::places::{Ground, Places};

use crate::{
    BrickFrameInput, BrickMap, BrickRevision, BrickTracer, Flight, Grade, TerrainAppearance,
};

fn ground() -> Ground {
    let grown = Places::grown(4_242, 4, 64);
    Ground::grow(&grown, 64)
}

#[test]
fn habitat_appearance_uses_material_and_plane_classified_background_colours() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping habitat appearance receipt");
        return;
    };
    let camera = Flight {
        eye: [4.5, 20.0, 4.5],
        yaw: 0.0,
        pitch: -0.4,
        fov: 0.15,
        far: 48.0,
    };
    let grade = Grade {
        fog_start: 1.0,
        ..Grade::clay()
    };
    let appearance = TerrainAppearance {
        soil: [1.0, 0.0, 0.0],
        rock: [0.9, 0.1, 0.0],
        unknown: [0.7, 0.0, 0.1],
        sky: [0.0, 1.0, 0.0],
        underground: [0.0, 0.0, 1.0],
        section_centre: [4.5, 8.0, 4.5],
        clearing_y: 8.0,
    };
    let frame = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .with_terrain_appearance(appearance),
        )
        .expect("habitat frame");
    assert!(
        frame
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] > 200 && pixel[1] < pixel[0] / 3 && pixel[2] < pixel[0] / 3)
    );

    let sky_camera = Flight {
        pitch: 0.4,
        ..camera
    };
    let sky = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &sky_camera, &grade)
                .with_terrain_appearance(appearance),
        )
        .expect("habitat sky frame");
    assert!(
        sky.pixels
            .chunks_exact(4)
            .any(|pixel| pixel == [0, 255, 0, 255])
    );

    let underground = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &sky_camera, &grade)
                .with_terrain_appearance(TerrainAppearance {
                    clearing_y: 100.0,
                    ..appearance
                }),
        )
        .expect("habitat underground frame");
    assert!(
        underground
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel == [0, 0, 255, 255])
    );
}

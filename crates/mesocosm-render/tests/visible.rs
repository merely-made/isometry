// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The visual half of wave 1.2, as assertions.
//!
//! Wave 1.2's done-condition says the new part must be *visible and physically
//! legible*. Visibility is testable: render headless, count what is drawn, and
//! compare frames before and after an attachment. Legibility, meaning whether
//! it looks good, stays a judgment for a human at a screen.
//!
//! These tests need a GPU adapter. On a machine without one they report and
//! skip rather than pass silently, because a visual test that passes when
//! nothing rendered is worse than no test.

use mesocosm_core::{Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};
use mesocosm_mesh::{Volume, VolumeMap, mesh_body};
use mesocosm_render::{Camera, RenderError, Renderer};

const SIZE: u32 = 256;

fn source() -> VolumeMap {
    let mut map = VolumeMap::new();
    map.insert(VolumeRef::from_tag(1), Volume::solid([4, 4, 4], 1));
    map.insert(VolumeRef::from_tag(2), Volume::solid([3, 2, 2], 9));
    map
}

fn body() -> BodyDocument {
    BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2])
}

/// Builds a renderer, or returns `None` with a printed reason on a machine
/// with no adapter.
fn renderer() -> Option<Renderer> {
    match Renderer::headless(SIZE, SIZE) {
        Ok(r) => Some(r),
        Err(RenderError::NoAdapter) => {
            eprintln!("skipping: no GPU adapter available on this machine");
            None
        },
        Err(other) => panic!("renderer failed for a reason other than adapter: {other:?}"),
    }
}

#[test]
fn a_body_actually_draws() {
    let Some(renderer) = renderer() else { return };
    let source = source();
    let mesh = mesh_body(&body(), &source).unwrap();
    let (min, max) = mesh.bounds().unwrap();
    let frame = renderer
        .render(&mesh, &Camera::framing(min, max, 1.0))
        .unwrap();

    assert!(!frame.is_blank(), "a solid body must put pixels on screen");
    let covered = frame.covered();
    assert!(
        covered > (SIZE * SIZE) / 50,
        "the body should occupy a real share of the frame, got {covered} px"
    );
}

#[test]
fn an_empty_scene_is_blank() {
    let Some(renderer) = renderer() else { return };
    let mesh = mesocosm_mesh::BodyMesh::default();
    let frame = renderer.render(&mesh, &Camera::default()).unwrap();
    assert!(
        frame.is_blank(),
        "nothing to draw must leave the frame clear"
    );
}

#[test]
fn an_attached_part_is_visible_in_the_frame() {
    let Some(renderer) = renderer() else { return };
    let source = source();

    // Frame both states with the same camera, so the comparison is about the
    // body rather than about zoom.
    let mut grown = body();
    grown
        .attach(
            VolumeRef::from_tag(2),
            400,
            [1, 1, 1],
            Attachment {
                parent: grown.root,
                offset: [10, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();

    let grown_mesh = mesh_body(&grown, &source).unwrap();
    let (min, max) = grown_mesh.bounds().unwrap();
    let camera = Camera::framing(min, max, 1.0);

    let bare_mesh = mesh_body(&body(), &source).unwrap();
    let before = renderer.render(&bare_mesh, &camera).unwrap();
    let after = renderer.render(&grown_mesh, &camera).unwrap();

    assert!(
        after.covered() > before.covered(),
        "the attached part must add drawn pixels: {} -> {}",
        before.covered(),
        after.covered()
    );

    let (bx0, _, bx1, _) = before.covered_bounds().expect("the bare body drew");
    let (ax0, _, ax1, _) = after.covered_bounds().expect("the grown body drew");
    assert!(
        (ax1 - ax0) > (bx1 - bx0),
        "the drawn silhouette must widen when a limb is added"
    );
}

#[test]
fn faces_are_shaded_so_the_form_reads() {
    let Some(renderer) = renderer() else { return };
    let source = source();
    let mesh = mesh_body(&body(), &source).unwrap();
    let (min, max) = mesh.bounds().unwrap();
    let frame = renderer
        .render(&mesh, &Camera::framing(min, max, 1.0))
        .unwrap();

    // A single-material cube must still show more than one brightness, or it
    // reads as a flat silhouette rather than a solid.
    let mut shades = std::collections::BTreeSet::new();
    for y in 0..frame.height {
        for x in 0..frame.width {
            let p = frame.pixel(x, y);
            let luma = (p[0] as u32 * 3 + p[1] as u32 * 6 + p[2] as u32) / 10;
            if luma > 40 {
                shades.insert(luma / 16);
            }
        }
    }
    assert!(
        shades.len() >= 2,
        "a lit solid needs distinguishable faces, found {} band(s)",
        shades.len()
    );
}

#[test]
fn rendering_the_same_body_twice_agrees() {
    let Some(renderer) = renderer() else { return };
    let source = source();
    let mesh = mesh_body(&body(), &source).unwrap();
    let camera = Camera::default();

    let a = renderer.render(&mesh, &camera).unwrap();
    let b = renderer.render(&mesh, &camera).unwrap();

    // Same device and driver, so coverage must match exactly. Byte equality is
    // deliberately not asserted: it would not hold across GPUs, and gameplay
    // identity lives in the core's state hash rather than in a raster.
    assert_eq!(a.covered(), b.covered());
    assert_eq!(a.covered_bounds(), b.covered_bounds());
}

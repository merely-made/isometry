// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::PartId;
use mesocosm_mesh::VolumeMap;
use mesocosm_render::{RenderError, Renderer};

use super::bodies::BodyLayer;
use super::*;

fn layer() -> Option<(Renderer, BodyLayer)> {
    match Renderer::headless(16, 16) {
        Ok(renderer) => {
            let bodies = BodyLayer::new(renderer.device(), 16, 16);
            Some((renderer, bodies))
        },
        Err(RenderError::NoAdapter) => None,
        Err(error) => panic!("headless renderer failed: {error:?}"),
    }
}

fn prepare(layer: &mut BodyLayer, world: &World, volumes: &mesocosm_mesh::VolumeMap) {
    layer.prepare(
        world,
        volumes,
        SlabWindow::new(CameraMode::Side, [0.0, 20.0, 0.0], 256.0, 1.0),
    );
}

#[test]
fn preview_depth_and_bounds_are_temporary() {
    let Some((renderer, _)) = layer() else {
        return;
    };
    let world = World::new(7, 3);
    let mut section = Section::new(
        renderer.device().clone(),
        renderer.queue().clone(),
        16,
        16,
        wgpu::TextureFormat::Rgba8Unorm,
        world.ground(),
        Framing::new(28.0, CameraMode::TerrariumEast),
    )
    .unwrap();
    for habitat in [None, Some(framed_habitat(&world))] {
        if let Some(habitat) = habitat {
            section.configure_terrarium(&habitat, 12.0, Cutaway::Always, [0, 0, 0]);
        }
        let centre = [0.0, 20.0, 0.0];
        let original = section.view(centre);
        section.set_body_preview(true, 80.0);
        let preview = section.view(centre);
        assert_eq!(preview.depth, 80.0);
        assert_eq!(preview.clip().bounds, None);
        assert!(preview.window().half[2] >= 40.0);
        section.set_body_preview(false, 80.0);
        let restored = section.view(centre);
        assert_eq!(restored.depth, original.depth);
        assert_eq!(restored.bounds, original.bounds);
        assert_eq!(restored.matrix(), original.matrix());
    }
}

#[test]
fn selection_carries_owner_and_expires_after_severing() {
    let Some((_renderer, mut layer)) = layer() else {
        eprintln!("no adapter; skipping inspection identity receipt");
        return;
    };
    let mut world = World::new(41, 40);
    let volumes = crate::fixture::volumes_for(&world);
    prepare(&mut layer, &world, &volumes);
    let subject = world.controlled_id().expect("played organism");
    let root = layer
        .select_part(subject, None, false)
        .expect("drawn root part");
    let selected = layer
        .select_part(subject, Some(root), false)
        .expect("drawn non-root part");
    assert_ne!(selected.part, PartId(0));
    assert!(layer.validate_selection(selected, &world, &volumes));

    let other = world
        .organisms
        .iter()
        .find(|organism| organism.id != subject)
        .expect("another organism")
        .id;
    let other_selection = layer
        .select_part(other, Some(root), false)
        .expect("other drawn body");
    assert_eq!(other_selection.part, root.part);
    assert_ne!(other_selection.organism, root.organism);

    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == subject)
        .expect("selected organism")
        .position[0] += 5;
    assert!(layer.validate_selection(selected, &world, &volumes));

    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == subject)
        .expect("selected organism")
        .phenotype
        .sever(selected.part);
    assert!(!layer.validate_selection(selected, &world, &volumes));
}

#[test]
fn failed_projection_never_offers_a_part() {
    let Some((_renderer, mut layer)) = layer() else {
        eprintln!("no adapter; skipping inspection fallback receipt");
        return;
    };
    let world = World::new(42, 40);
    prepare(&mut layer, &world, &VolumeMap::new());
    let subject = world.controlled_id().expect("played organism");
    assert_eq!(layer.select_part(subject, None, false), None);
}

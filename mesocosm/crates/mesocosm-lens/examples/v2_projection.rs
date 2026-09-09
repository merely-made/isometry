// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V2 receipt: one played body revision enters Lens, mesh, and Isometry data.
//!
//! Run with an output directory. The emitted `.body` is accepted directly by
//! Isometry's `critter_sprite --body` path.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    native::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // The projection itself is portable and covered by library tests. This
    // receipt writes files and intentionally runs only on a native host.
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{collections::BTreeSet, fs, path::PathBuf};

    use mesocosm_core::{
        BodyDocument, Intent, OrganismId, Outcome, PartId, PartOrigin, Placement, VolumeRef, World,
        snapshot, world::organism_extent,
    };
    use mesocosm_lens::{
        BodyLensProjection, BodyPlacement, Flight, FrameInput, Grade, Lens, MapRevision, maps,
    };
    use mesocosm_mesh::{BodyProfile, Volume, VolumeMap, mesh_body};
    use serde::Serialize;

    const SIDE: u32 = 256;
    const WIDTH: u32 = 960;
    const HEIGHT: u32 = 540;

    #[derive(Serialize)]
    struct PartReceipt {
        id: u32,
        volume_tag: u8,
        incorporated: bool,
        lens_capsule: u16,
        mesh_placement: bool,
        isometry_attributed_voxels: usize,
    }

    #[derive(Serialize)]
    struct Receipt {
        gate: &'static str,
        body_revision: String,
        body_bytes: usize,
        body_decodes_without_projections: bool,
        parts: Vec<PartReceipt>,
        incorporated_parts: Vec<u32>,
        lens_capsules: usize,
        lens_invalidated_parts: Vec<u32>,
        mesh_placements: usize,
        mesh_distinct_volumes: usize,
        mesh_invalidated_parts: Vec<u32>,
        isometry_profile_bytes: usize,
        isometry_profile_digest: String,
        isometry_cells: usize,
        isometry_changed_cells: usize,
        lens_frame_size: [u32; 2],
    }

    pub fn run() {
        let dir = std::env::args()
            .nth(1)
            .map(PathBuf::from)
            .unwrap_or_else(|| ".".into());
        fs::create_dir_all(&dir).expect("output directory");

        let volumes = volumes();
        let mut world = World::new(0x00A7_7AC4, 90);
        grow_to(&mut world, 3);
        let before = world.body().expect("played critter survives").clone();
        grow_to(&mut world, before.len() + 1);
        let body = world.body().expect("played critter survives").clone();
        let incorporated: Vec<_> = (before.len()..body.len())
            .map(|id| PartId(id as u32))
            .collect();
        assert!(
            !incorporated.is_empty(),
            "the incorporation adds at least one provenance-bearing part"
        );

        let maps = maps::synthesize(0x00A7_7AC4, SIDE);
        let ground = sample_ground(&maps, 128.0, 128.0);
        let placement = BodyPlacement {
            ground: [128.0, ground, 128.0],
            scale: 0.72,
            tint: [0.34, 0.72, 0.45],
        };
        let before_lens = BodyLensProjection::project(&before, placement).expect("before lens");
        let projected = BodyLensProjection::project(&body, placement).expect("after lens");
        let lens_invalidated = projected.changed_parts(&before_lens);
        assert_eq!(lens_invalidated, incorporated);

        let before_mesh = mesh_body(&before, &volumes).expect("before mesh");
        let mesh = mesh_body(&body, &volumes).expect("after mesh");
        let mesh_invalidated = changed_mesh_parts(&before_mesh, &mesh);
        assert_eq!(mesh_invalidated, incorporated);

        let before_profile = BodyProfile::of(&before, &volumes).expect("before profile");
        let profile = BodyProfile::of(&body, &volumes).expect("after profile");
        let changed_cells = changed_profile_cells(&before_profile, &profile, &incorporated);
        assert!(
            changed_cells > 0,
            "the incorporated part reaches the bake projection"
        );
        let profile_bytes = profile.to_bytes().expect("profile bytes");
        fs::write(dir.join("v2_played.body"), &profile_bytes).expect("write body profile");

        let body_bytes = snapshot::encode(&body).expect("body bytes");
        let decoded: BodyDocument = snapshot::decode(&body_bytes).expect("body decodes alone");
        assert_eq!(decoded, body);

        let mut lens = Lens::headless(WIDTH, HEIGHT).expect("GPU adapter");
        let flight = frame_body(&projected, &maps);
        println!(
            "pose centre {:?}, radius {:.2}, ground {:.2}, eye {:?}, pitch {:.3}",
            projected.pose.bounds_centre,
            projected.pose.bounds_radius,
            ground,
            flight.eye,
            flight.pitch,
        );
        let grade = Grade::clay();
        let capture = lens
            .capture(
                FrameInput::new(&maps, MapRevision(1), &flight, &grade).with_pose(&projected.pose),
            )
            .expect("lens capture");
        write_png(
            dir.join("v2_lens.png"),
            &capture.pixels,
            capture.width,
            capture.height,
        );

        let parts = projected
            .parts
            .iter()
            .map(|lens| {
                let placement = mesh
                    .placements
                    .iter()
                    .find(|placement| placement.part == lens.part)
                    .expect("mesh retains every lens part");
                assert_eq!(placement.provenance.as_ref(), Some(&lens.provenance));
                assert_eq!(
                    profile.parts[lens.part.0 as usize],
                    PartOrigin::from(&lens.provenance),
                    "Isometry profile and lens name the same history",
                );
                PartReceipt {
                    id: lens.part.0,
                    volume_tag: lens.volume.0[0],
                    incorporated: matches!(
                        lens.provenance.origin,
                        mesocosm_core::Origin::Incorporated { .. }
                    ),
                    lens_capsule: lens.capsule,
                    mesh_placement: true,
                    isometry_attributed_voxels: profile
                        .attribution
                        .iter()
                        .filter(|slot| **slot == lens.part.0 as u16 + 1)
                        .count(),
                }
            })
            .collect();

        let receipt = Receipt {
            gate: "V2",
            body_revision: format!("fnv1a64:{:016x}", projected.revision.0),
            body_bytes: body_bytes.len(),
            body_decodes_without_projections: true,
            parts,
            incorporated_parts: incorporated.iter().map(|part| part.0).collect(),
            lens_capsules: projected.pose.capsules.len(),
            lens_invalidated_parts: lens_invalidated.iter().map(|part| part.0).collect(),
            mesh_placements: mesh.placement_count(),
            mesh_distinct_volumes: mesh.mesh_count(),
            mesh_invalidated_parts: mesh_invalidated.iter().map(|part| part.0).collect(),
            isometry_profile_bytes: profile_bytes.len(),
            isometry_profile_digest: format!(
                "fnv1a64:{:016x}",
                snapshot::hash_bytes(&profile_bytes)
            ),
            isometry_cells: profile.cell_count(),
            isometry_changed_cells: changed_cells,
            lens_frame_size: [capture.width, capture.height],
        };
        fs::write(
            dir.join("v2_receipt.json"),
            serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
        )
        .expect("write receipt");
        println!(
            "V2 body {}: {} parts, changed {:?}; wrote {}",
            receipt.body_revision,
            receipt.parts.len(),
            receipt.incorporated_parts,
            dir.display(),
        );
    }

    fn grow_to(world: &mut World, parts: usize) {
        for _ in 0..1_200 {
            if world.body().is_some_and(|body| body.len() >= parts) {
                return;
            }
            let Some(here) = world.position() else { break };
            let Some((prey, at)) = nearest_prey(world, here) else {
                break;
            };
            let outcome = if world.in_reach(at) {
                world.apply(Intent::Metabolize {
                    organism: prey,
                    placement: Placement::Planned,
                })
            } else {
                world.apply(Intent::Move {
                    delta: [0, 1, 2].map(|axis| (at[axis] - here[axis]).signum()),
                })
            };
            if matches!(outcome, Outcome::Rejected(_)) && !world.is_embodied() {
                break;
            }
        }
        assert_eq!(world.body().map(BodyDocument::len), Some(parts));
    }

    fn nearest_prey(world: &World, here: [i32; 3]) -> Option<(OrganismId, [i32; 3])> {
        world
            .organisms
            .iter()
            .filter(|organism| Some(organism.id) != world.controlled_id() && organism.is_alive())
            .map(|organism| (organism.id, organism.position))
            .min_by_key(|(_, at)| {
                (0..3)
                    .map(|axis| (at[axis] - here[axis]).abs())
                    .max()
                    .unwrap_or(0)
            })
    }

    fn volumes() -> VolumeMap {
        let mut volumes = VolumeMap::new();
        volumes.insert(VolumeRef::from_tag(1), Volume::solid([4, 4, 4], 1));
        for tag in 16..24u8 {
            let half = organism_extent(tag);
            let size = half.map(|axis| (axis * 2).max(1) as u32);
            volumes.insert(VolumeRef::from_tag(tag), Volume::solid(size, tag));
        }
        volumes
    }

    fn changed_mesh_parts(
        before: &mesocosm_mesh::BodyMesh,
        after: &mesocosm_mesh::BodyMesh,
    ) -> Vec<PartId> {
        let before = before
            .placements
            .iter()
            .map(|placement| (placement.part, placement))
            .collect::<std::collections::BTreeMap<_, _>>();
        let after = after
            .placements
            .iter()
            .map(|placement| (placement.part, placement))
            .collect::<std::collections::BTreeMap<_, _>>();
        before
            .keys()
            .chain(after.keys())
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|part| before.get(part) != after.get(part))
            .collect()
    }

    fn changed_profile_cells(before: &BodyProfile, after: &BodyProfile, added: &[PartId]) -> usize {
        let min = [0, 1, 2].map(|axis| before.origin[axis].min(after.origin[axis]));
        let max = [0, 1, 2].map(|axis| {
            (before.origin[axis] + before.size[axis] as i32)
                .max(after.origin[axis] + after.size[axis] as i32)
        });
        let added: BTreeSet<_> = added.iter().map(|part| part.0 as usize).collect();
        let mut changed = 0;
        for z in min[2]..max[2] {
            for y in min[1]..max[1] {
                for x in min[0]..max[0] {
                    let at = [x, y, z];
                    if before.material_at(at) != after.material_at(at) {
                        assert!(
                            after.part_at(at).is_some_and(|part| added.contains(&part)),
                            "only the new part may change the flattened body at {at:?}",
                        );
                        changed += 1;
                    }
                }
            }
        }
        changed
    }

    fn sample_ground(maps: &mesocosm_lens::maps::BiomeMaps, x: f32, z: f32) -> f32 {
        let index = (z as u32 % maps.side) * maps.side + (x as u32 % maps.side);
        maps.height[index as usize] as f32
    }

    fn frame_body(projected: &BodyLensProjection, maps: &mesocosm_lens::maps::BiomeMaps) -> Flight {
        let centre = projected.pose.bounds_centre;
        let radius = projected.pose.bounds_radius.max(4.0);
        let mut eye = [
            centre[0] + radius * 6.0,
            centre[1] + radius * 2.0,
            centre[2],
        ];
        eye[1] = eye[1].max(sample_ground(maps, eye[0], eye[2]) + 5.0);
        let flat = ((centre[0] - eye[0]).powi(2) + (centre[2] - eye[2]).powi(2)).sqrt();
        Flight {
            eye,
            yaw: f32::atan2(centre[0] - eye[0], centre[2] - eye[2]),
            pitch: f32::atan2(centre[1] - eye[1], flat),
            fov: 0.9,
            far: 500.0,
        }
    }

    fn write_png(path: PathBuf, pixels: &[u8], width: u32, height: u32) {
        let file = fs::File::create(path).expect("create PNG");
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .expect("PNG header")
            .write_image_data(pixels)
            .expect("PNG pixels");
    }
}

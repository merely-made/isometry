// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use mesocosm_core::{PartId, VolumeRef, process::Process};
use mesocosm_mesh::{Volume, place_point};

fn gpu() -> Option<crate::Renderer> {
    match crate::Renderer::headless(16, 16) {
        Ok(renderer) => Some(renderer),
        Err(crate::RenderError::NoAdapter) => None,
        Err(error) => panic!("headless renderer failed: {error:?}"),
    }
}

fn attachments(device: &wgpu::Device) -> (wgpu::Texture, wgpu::Texture) {
    let size = wgpu::Extent3d {
        width: 16,
        height: 16,
        depth_or_array_layers: 1,
    };
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("live body test colour"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("live body test depth"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    (colour, depth)
}

fn clear(
    encoder: &mut wgpu::CommandEncoder,
    colour: &wgpu::TextureView,
    depth: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("live body test clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: colour,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

#[test]
fn one_greedy_cube_keeps_six_quads_of_triangles() {
    let mesh = mesocosm_mesh::mesh_volume(&Volume::solid([4, 3, 2], 7));
    assert_eq!(part_vertices(&mesh).len(), 36);
}

#[test]
fn quarter_turn_uses_the_mesh_placement_convention() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 1, 1], 1));
    let body = LiveBody::new(&mesh, [10.0, 0.0, 0.0]);
    let transformed = model_matrix(body, Yaw::Quarter, [0; 3], [2, 0, 0])
        .transform_point3(glam::Vec3::X)
        .round();
    assert_eq!(transformed.to_array(), [12.0, 0.0, -1.0]);
    assert_eq!(
        place_point([1, 0, 0], Yaw::Quarter, [0; 3], [2, 0, 0]),
        [2, 0, -1]
    );
}

#[test]
fn parent_yaw_turns_nonzero_pivots_and_attachment_offsets_together() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 1, 1], 1));
    let mut body = LiveBody::new(&mesh, [10.0, 20.0, 30.0]);
    assert_eq!(body.yaw_radians, 0.0);
    body.scale = 2.0;
    // The part's quarter turn maps point - pivot [2, 2, 4] to [4, 2, -2].
    // Its attachment then places that point at [8, 7, 4] in body space.
    for (yaw, expected) in [
        (core::f32::consts::FRAC_PI_2, [18.0, 34.0, 14.0]),
        (
            core::f32::consts::FRAC_PI_4,
            [
                10.0 + 12.0 * core::f32::consts::SQRT_2,
                34.0,
                30.0 - 4.0 * core::f32::consts::SQRT_2,
            ],
        ),
    ] {
        body.yaw_radians = yaw;
        let point = model_matrix(body, Yaw::Quarter, [1, 2, 3], [4, 5, 6])
            .transform_point3(glam::Vec3::new(3.0, 4.0, 7.0));
        assert!(
            (point - glam::Vec3::from_array(expected))
                .abs()
                .max_element()
                < 1e-5
        );
    }
}

#[test]
fn an_unchanged_frame_reuses_all_gpu_uploads() {
    let Some(host) = gpu() else { return };
    let (colour, depth) = attachments(host.device());
    let colour_view = colour.create_view(&Default::default());
    let depth_view = depth.create_view(&Default::default());
    let volume = Volume::solid([1, 1, 1], 1);
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &volume);
    let body = [LiveBody::new(&mesh, [0.0; 3])];
    let mut live = LiveBodyRenderer::new(host.device(), wgpu::TextureFormat::Rgba8Unorm, 4);
    let clip = Mat4::IDENTITY.to_cols_array_2d();

    let mut first = host.device().create_command_encoder(&Default::default());
    clear(&mut first, &colour_view, &depth_view);
    let uploaded = live
        .draw(
            host.device(),
            host.queue(),
            &mut first,
            &colour_view,
            &depth_view,
            clip,
            None,
            &body,
        )
        .unwrap();
    host.queue().submit(Some(first.finish()));
    assert!(uploaded.mesh_upload_bytes > 0);
    assert!(uploaded.instance_upload_bytes > 0);
    assert!(uploaded.frame_upload_bytes > 0);

    let mut second = host.device().create_command_encoder(&Default::default());
    let reused = live
        .draw(
            host.device(),
            host.queue(),
            &mut second,
            &colour_view,
            &depth_view,
            clip,
            None,
            &body,
        )
        .unwrap();
    host.queue().submit(Some(second.finish()));
    assert_eq!(reused.mesh_upload_bytes, 0);
    assert_eq!(reused.instance_upload_bytes, 0);
    assert_eq!(reused.frame_upload_bytes, 0);
    assert_eq!(reused.draw_parts, 1);
    eprintln!("VB1 static receipt: {reused:?}");

    let moved_body = [LiveBody::new(&mesh, [0.25, 0.0, 0.0])];
    let mut movement = host.device().create_command_encoder(&Default::default());
    let moved = live
        .draw(
            host.device(),
            host.queue(),
            &mut movement,
            &colour_view,
            &depth_view,
            clip,
            None,
            &moved_body,
        )
        .unwrap();
    host.queue().submit(Some(movement.finish()));
    assert_eq!(
        moved.mesh_upload_bytes, 0,
        "movement reuses immutable volume geometry"
    );
    assert!(
        moved.instance_upload_bytes > 0,
        "movement updates only instance data"
    );
    assert_eq!(moved.draw_parts, 1);
    eprintln!("VB1 movement receipt: {moved:?}");

    let selected_body = [LiveBody {
        selected_part: Some(PartId(0)),
        ..moved_body[0]
    }];
    let mut selection = host.device().create_command_encoder(&Default::default());
    let selected = live
        .draw(
            host.device(),
            host.queue(),
            &mut selection,
            &colour_view,
            &depth_view,
            clip,
            None,
            &selected_body,
        )
        .unwrap();
    host.queue().submit(Some(selection.finish()));
    assert_eq!(
        selected.mesh_upload_bytes, 0,
        "selection reuses static geometry"
    );
    assert!(
        selected.instance_upload_bytes > 0,
        "selection changes instance tint"
    );

    let mut unchanged_selection = host.device().create_command_encoder(&Default::default());
    let unchanged_selected = live
        .draw(
            host.device(),
            host.queue(),
            &mut unchanged_selection,
            &colour_view,
            &depth_view,
            clip,
            None,
            &selected_body,
        )
        .unwrap();
    host.queue().submit(Some(unchanged_selection.finish()));
    assert_eq!(unchanged_selected.mesh_upload_bytes, 0);
    assert_eq!(unchanged_selected.instance_upload_bytes, 0);

    let mut attached_mesh = mesh.clone();
    let mut attached = mesh.placements[0].clone();
    attached.part = PartId(1);
    attached.pivot_at = [2, 0, 0];
    attached_mesh.placements.push(attached);
    let attached_body = [LiveBody::new(&attached_mesh, [0.25, 0.0, 0.0])];
    let mut attachment = host.device().create_command_encoder(&Default::default());
    let grown = live
        .draw(
            host.device(),
            host.queue(),
            &mut attachment,
            &colour_view,
            &depth_view,
            clip,
            None,
            &attached_body,
        )
        .unwrap();
    host.queue().submit(Some(attachment.finish()));
    assert_eq!(
        grown.mesh_upload_bytes, 0,
        "a shared volume needs no immutable upload"
    );
    assert!(
        grown.instance_upload_bytes > 0,
        "attachment changes the instance batch"
    );
    assert_eq!(grown.draw_parts, 2, "attachment adds one rigid placement");
    eprintln!("VB1 attachment receipt: {grown:?}");

    let mut severing = host.device().create_command_encoder(&Default::default());
    let severed = live
        .draw(
            host.device(),
            host.queue(),
            &mut severing,
            &colour_view,
            &depth_view,
            clip,
            None,
            &moved_body,
        )
        .unwrap();
    host.queue().submit(Some(severing.finish()));
    assert_eq!(
        severed.mesh_upload_bytes, 0,
        "severing does not rebuild shared geometry"
    );
    assert!(
        severed.instance_upload_bytes > 0,
        "severing changes the instance batch"
    );
    assert_eq!(
        severed.draw_parts, 1,
        "severing removes the added placement"
    );
    eprintln!("VB1 sever receipt: {severed:?}");

    for changed in [true, false] {
        let yaw = core::f32::consts::FRAC_PI_4;
        let rotated_body = [LiveBody {
            yaw_radians: yaw,
            ..moved_body[0]
        }];
        let mut rotation = host.device().create_command_encoder(&Default::default());
        let rotated = live
            .draw(
                host.device(),
                host.queue(),
                &mut rotation,
                &colour_view,
                &depth_view,
                clip,
                None,
                &rotated_body,
            )
            .unwrap();
        host.queue().submit(Some(rotation.finish()));
        assert_eq!(
            rotated.mesh_builds, 0,
            "parent pose reuses immutable geometry"
        );
        assert_eq!(rotated.mesh_upload_bytes, 0);
        assert_eq!(rotated.frame_upload_bytes, 0);
        assert_eq!(rotated.instance_upload_bytes > 0, changed);
        assert_eq!(rotated.instances, 1);
        assert_eq!(rotated.draw_parts, 1);
        eprintln!("parent yaw receipt: {rotated:?}");
    }

    for yaw in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let invalid = [LiveBody {
            yaw_radians: yaw,
            ..moved_body[0]
        }];
        let mut encoder = host.device().create_command_encoder(&Default::default());
        assert_eq!(
            live.draw(
                host.device(),
                host.queue(),
                &mut encoder,
                &colour_view,
                &depth_view,
                clip,
                None,
                &invalid,
            ),
            Err(LiveBodyError::InvalidBody)
        );
    }
}

#[test]
fn inspection_tint_prioritizes_the_addressed_part() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 1, 1], 1));
    let focused = LiveBody {
        focused: true,
        ..LiveBody::new(&mesh, [0.0; 3])
    };
    assert_eq!(
        part_appearance(focused, PartId(9)).tint,
        [1.08, 1.08, 1.08, 0.0]
    );
    let selected = LiveBody {
        selected_part: Some(PartId(9)),
        ..focused
    };
    assert_eq!(
        part_appearance(selected, PartId(9)).tint,
        [1.08, 1.08, 1.08, 1.0],
        "selection is an accent in the shader, leaving tissue marks readable"
    );
}

#[test]
fn expression_materials_aggregate_by_addressed_part_and_process() {
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 1, 1], 1));
    let materials = [
        PartMaterial {
            part: PartId(3),
            process: Process::Fix,
            fraction: 0.25,
        },
        PartMaterial {
            part: PartId(3),
            process: Process::Secrete,
            fraction: 0.25,
        },
        PartMaterial {
            part: PartId(3),
            process: Process::Fix,
            fraction: 0.25,
        },
        PartMaterial {
            part: PartId(4),
            process: Process::Sense,
            fraction: 1.0,
        },
    ];
    let body = LiveBody {
        materials: &materials,
        ..LiveBody::new(&mesh, [0.0; 3])
    };
    let appearance = part_appearance(body, PartId(3));
    assert_eq!(appearance.expression, [0.0, 0.0, 0.0, 0.5]);
    assert_eq!(appearance.expression_tail[0], 0.25);
    assert_eq!(part_appearance(body, PartId(4)).expression[2], 1.0);
    assert!(valid_materials(&materials));
}

#[test]
fn expression_materials_refuse_impossible_capacity_claims() {
    let over_capacity = [
        PartMaterial {
            part: PartId(3),
            process: Process::Fix,
            fraction: 0.75,
        },
        PartMaterial {
            part: PartId(3),
            process: Process::Secrete,
            fraction: 0.5,
        },
    ];
    assert!(!valid_materials(&over_capacity));
    assert!(!valid_materials(&[PartMaterial {
        part: PartId(3),
        process: Process::Fix,
        fraction: f32::NAN,
    }]));
}

#[test]
fn changed_expression_reuses_the_immutable_volume_upload() {
    let Some(host) = gpu() else { return };
    let (colour, depth) = attachments(host.device());
    let colour_view = colour.create_view(&Default::default());
    let depth_view = depth.create_view(&Default::default());
    let mesh = BodyMesh::single(VolumeRef::from_tag(41), &Volume::solid([2, 2, 2], 1));
    let fix = [PartMaterial {
        part: PartId(0),
        process: Process::Fix,
        fraction: 1.0,
    }];
    let secrete = [PartMaterial {
        part: PartId(0),
        process: Process::Secrete,
        fraction: 1.0,
    }];
    let mut live = LiveBodyRenderer::new(host.device(), wgpu::TextureFormat::Rgba8Unorm, 4);
    let clip = Mat4::IDENTITY.to_cols_array_2d();

    let mut first = host.device().create_command_encoder(&Default::default());
    clear(&mut first, &colour_view, &depth_view);
    let first_stats = live
        .draw(
            host.device(),
            host.queue(),
            &mut first,
            &colour_view,
            &depth_view,
            clip,
            None,
            &[
                LiveBody {
                    materials: &fix,
                    ..LiveBody::new(&mesh, [0.0; 3])
                },
                LiveBody {
                    materials: &secrete,
                    ..LiveBody::new(&mesh, [4.0, 0.0, 0.0])
                },
            ],
        )
        .unwrap();
    host.queue().submit(Some(first.finish()));
    assert!(first_stats.mesh_upload_bytes > 0);
    assert_eq!(
        first_stats.instances, 2,
        "one VolumeRef can have distinct process material instances"
    );

    let mut second = host.device().create_command_encoder(&Default::default());
    let changed = live
        .draw(
            host.device(),
            host.queue(),
            &mut second,
            &colour_view,
            &depth_view,
            clip,
            None,
            &[
                LiveBody {
                    materials: &secrete,
                    ..LiveBody::new(&mesh, [0.0; 3])
                },
                LiveBody {
                    materials: &fix,
                    ..LiveBody::new(&mesh, [4.0, 0.0, 0.0])
                },
            ],
        )
        .unwrap();
    host.queue().submit(Some(second.finish()));
    assert_eq!(
        changed.mesh_upload_bytes, 0,
        "expression must not rebuild VolumeRef geometry"
    );
    assert!(
        changed.instance_upload_bytes > 0,
        "expression changes only per-instance data"
    );
}

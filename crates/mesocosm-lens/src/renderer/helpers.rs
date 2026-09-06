// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! GPU construction, uploads, and pass encoding.

use super::*;

pub(super) fn pipeline(
    device: &wgpu::Device,
    label: &str,
    source: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn make_target(
    device: &wgpu::Device,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

pub(super) fn create_maps(
    device: &wgpu::Device,
    maps: &BiomeMaps,
    diagnostics: &mut FrameDiagnostics,
) -> MapResources {
    let texture = |label, width, height, format| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    };
    let height = texture(
        "lens height",
        maps.side,
        maps.side,
        wgpu::TextureFormat::R8Unorm,
    );
    let height_view = height.create_view(&Default::default());
    let color = texture("lens color", maps.side, maps.side, FRAME_FORMAT);
    let color_view = color.create_view(&Default::default());
    let palette = texture("lens palette", 256, 1, FRAME_FORMAT);
    let palette_view = palette.create_view(&Default::default());
    diagnostics.resource_creations += 3;
    MapResources {
        side: maps.side,
        revision: MapRevision(u64::MAX),
        height,
        height_view,
        color,
        color_view,
        palette,
        palette_view,
        palette_bytes: Vec::new(),
    }
}

pub(super) fn validate_maps(maps: &BiomeMaps) -> Result<(), LensError> {
    if maps.side == 0 {
        return Err(LensError::EmptyMap);
    }
    let texels = (maps.side as usize).checked_mul(maps.side as usize).ok_or(
        LensError::InvalidHeightLength {
            expected: usize::MAX,
            actual: maps.height.len(),
        },
    )?;
    if maps.height.len() != texels {
        return Err(LensError::InvalidHeightLength {
            expected: texels,
            actual: maps.height.len(),
        });
    }
    let color = texels.checked_mul(4).ok_or(LensError::InvalidColorLength {
        expected: usize::MAX,
        actual: maps.color.len(),
    })?;
    if maps.color.len() != color {
        return Err(LensError::InvalidColorLength {
            expected: color,
            actual: maps.color.len(),
        });
    }
    if maps.palette.len() > 256 {
        return Err(LensError::PaletteTooLarge(maps.palette.len()));
    }
    Ok(())
}

pub(super) fn upload_maps(
    queue: &wgpu::Queue,
    resident: &mut MapResources,
    maps: &BiomeMaps,
    change: MapChange,
    diagnostics: &mut FrameDiagnostics,
) -> Result<(), LensError> {
    match change {
        MapChange::Full => {
            write_texture(
                queue,
                &resident.height,
                [0, 0],
                [maps.side, maps.side],
                1,
                &maps.height,
            );
            write_texture(
                queue,
                &resident.color,
                [0, 0],
                [maps.side, maps.side],
                4,
                &maps.color,
            );
            diagnostics.map_upload_bytes += (maps.height.len() + maps.color.len()) as u64;
        },
        MapChange::Region(rect) => {
            validate_rect(rect, maps.side)?;
            let height = region_bytes(&maps.height, maps.side, 1, rect);
            let color = region_bytes(&maps.color, maps.side, 4, rect);
            write_texture(
                queue,
                &resident.height,
                [rect.x, rect.y],
                [rect.width, rect.height],
                1,
                &height,
            );
            write_texture(
                queue,
                &resident.color,
                [rect.x, rect.y],
                [rect.width, rect.height],
                4,
                &color,
            );
            diagnostics.map_upload_bytes += (height.len() + color.len()) as u64;
        },
    }
    let palette = palette_bytes(maps);
    if resident.palette_bytes != palette {
        write_texture(queue, &resident.palette, [0, 0], [256, 1], 4, &palette);
        diagnostics.map_upload_bytes += palette.len() as u64;
        resident.palette_bytes = palette;
    }
    Ok(())
}

pub(super) fn write_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: [u32; 2],
    extent: [u32; 2],
    bytes_per_texel: u32,
    data: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: origin[0],
                y: origin[1],
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(extent[0] * bytes_per_texel),
            rows_per_image: Some(extent[1]),
        },
        wgpu::Extent3d {
            width: extent[0],
            height: extent[1],
            depth_or_array_layers: 1,
        },
    );
}

pub(super) fn palette_bytes(maps: &BiomeMaps) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(256 * 4);
    for entry in &maps.palette {
        bytes.extend(entry.iter().map(|channel| (channel * 255.0) as u8));
        bytes.push(255);
    }
    bytes.resize(256 * 4, 0);
    bytes
}

pub(super) fn validate_rect(rect: DirtyRect, side: u32) -> Result<(), LensError> {
    let valid = rect.width > 0
        && rect.height > 0
        && rect
            .x
            .checked_add(rect.width)
            .is_some_and(|right| right <= side)
        && rect
            .y
            .checked_add(rect.height)
            .is_some_and(|bottom| bottom <= side);
    if valid {
        Ok(())
    } else {
        Err(LensError::DirtyRegionOutsideMap(rect))
    }
}

pub(super) fn region_bytes(
    source: &[u8],
    side: u32,
    bytes_per_texel: usize,
    rect: DirtyRect,
) -> Vec<u8> {
    let row_bytes = rect.width as usize * bytes_per_texel;
    let mut out = Vec::with_capacity(row_bytes * rect.height as usize);
    for y in rect.y..rect.y + rect.height {
        let start = ((y * side + rect.x) as usize) * bytes_per_texel;
        out.extend_from_slice(&source[start..start + row_bytes]);
    }
    out
}

pub(super) fn write_changed<T: Pod + PartialEq + Copy>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    previous: &mut Option<T>,
    value: T,
    diagnostics: &mut FrameDiagnostics,
) {
    if previous.as_ref() == Some(&value) {
        return;
    }
    queue.write_buffer(buffer, 0, bytemuck::bytes_of(&value));
    diagnostics.uniform_upload_bytes += size_of::<T>() as u64;
    *previous = Some(value);
}

pub(super) fn critter_params(pose: Option<&CritterPose>) -> CritterParams {
    let mut pairs = [[0.0f32; 4]; MAX_CAPSULES * 2];
    let (mut bounds, mut tint_count, mut eyes) = ([0.0f32; 4], [0.0f32; 4], [[0.0f32; 4]; 2]);
    if let Some(pose) = pose {
        for (index, capsule) in pose.capsules.iter().take(MAX_CAPSULES).enumerate() {
            pairs[2 * index] = [capsule.a[0], capsule.a[1], capsule.a[2], capsule.ra];
            pairs[2 * index + 1] = [capsule.b[0], capsule.b[1], capsule.b[2], capsule.rb];
        }
        bounds = [
            pose.bounds_centre[0],
            pose.bounds_centre[1],
            pose.bounds_centre[2],
            pose.bounds_radius,
        ];
        tint_count = [
            pose.tint[0],
            pose.tint[1],
            pose.tint[2],
            pose.capsules.len().min(MAX_CAPSULES) as f32,
        ];
        eyes = pose.eyes;
    }
    CritterParams {
        bounds,
        tint_count,
        eyes,
        pairs,
    }
}

pub(super) fn encode_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
    target: &wgpu::TextureView,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}

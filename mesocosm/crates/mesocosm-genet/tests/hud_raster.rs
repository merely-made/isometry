// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The HUD raster path, probed headlessly.
//!
//! The first windowed run showed the minimap on an opaque white ground where
//! transparency was expected. This pins down which stage owns the pixels:
//! paint a known square through the exact leaf → translate → render_vello
//! path and read the texture back.

use netrender::{ColorLoad, create_netrender_instance};
use sprigging::{ColorF, PaintCx, Path, Size};

fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    side: u32,
) -> Vec<u8> {
    let padded = (side * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (padded * side) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(side),
            },
        },
        wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = slice.get_mapped_range().expect("map range");
    let mut out = Vec::with_capacity((side * side * 4) as usize);
    for row in 0..side {
        let start = (row * padded) as usize;
        out.extend_from_slice(&data[start..start + (side * 4) as usize]);
    }
    out
}

#[test]
fn the_composited_ground_is_the_background_colour() {
    // The backdrop chain crosses three textures: an sRGB scene render, a
    // linear vello raster, and the target. This walks a bare backdrop (no
    // cells) through Composite and checks the ground arrives unchanged.
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("no adapter; skipping");
        return;
    };
    let (device, queue) =
        pollster::block_on(adapter.request_device(&Default::default())).expect("a headless device");

    const SIDE: u32 = 32;
    let shot = mesocosm_render::Renderer::with_device(device.clone(), queue.clone(), SIDE, SIDE);
    let make = |format: wgpu::TextureFormat, extra: wgpu::TextureUsages| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: SIDE,
                height: SIDE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | extra,
            view_formats: &[],
        })
    };
    let backdrop = make(shot.format(), wgpu::TextureUsages::empty());
    let backdrop_view = backdrop.create_view(&Default::default());
    let target = make(shot.format(), wgpu::TextureUsages::empty());
    let target_view = target.create_view(&Default::default());

    // An empty scene: pure background.
    let camera = mesocosm_render::Camera::default();
    let mut encoder = device.create_command_encoder(&Default::default());
    shot.draw_scene(&mut encoder, &backdrop_view, &[], &camera);
    queue.submit(Some(encoder.finish()));

    let direct = read_texture(&device, &queue, &backdrop, SIDE);
    println!("backdrop texel: {:?}", &direct[0..4]);

    let composite = mesocosm_render::composite::Composite::new(&device, shot.format());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        // Clear the target to a sentinel so pass-through is visible.
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    composite.draw(
        &device,
        &queue,
        &mut encoder,
        &target_view,
        &backdrop_view,
        (0.0, 0.0, SIDE as f32, SIDE as f32),
        (SIDE, SIDE),
    );
    queue.submit(Some(encoder.finish()));

    let composited = read_texture(&device, &queue, &target, SIDE);
    println!("composited texel: {:?}", &composited[0..4]);
    let drift = (0..3)
        .map(|c| (direct[c] as i32 - composited[c] as i32).abs())
        .max()
        .unwrap();
    assert!(
        drift <= 2,
        "the ground changed crossing the composite: {:?} became {:?}",
        &direct[0..4],
        &composited[0..4]
    );
}

#[test]
fn the_minimap_texture_is_transparent_where_nothing_was_painted() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("no adapter; skipping");
        return;
    };
    let (device, queue) =
        pollster::block_on(adapter.request_device(&Default::default())).expect("a headless device");

    const SIDE: u32 = 64;
    let net = create_netrender_instance(
        netrender::WgpuHandles {
            instance,
            adapter,
            device: device.clone(),
            queue: queue.clone(),
        },
        netrender::NetrenderOptions {
            tile_cache_size: Some(SIDE),
            enable_vello: true,
            ..Default::default()
        },
    )
    .expect("netrender accepts the default device");

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("probe"),
        size: wgpu::Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());

    // A red square in the middle third; everything else untouched. Built
    // through PaintCx, the same door the leaf uses.
    let third = SIDE as f32 / 3.0;
    let mut cmds = Vec::new();
    let mut cx = PaintCx::new(
        &mut cmds,
        Size {
            width: SIDE as f32,
            height: SIDE as f32,
        },
    );
    cx.fill_path(
        Path::new()
            .move_to(third, third)
            .line_to(third * 2.0, third)
            .line_to(third * 2.0, third * 2.0)
            .line_to(third, third * 2.0)
            .close()
            .build(),
        ColorF::new(1.0, 0.0, 0.0, 1.0),
    );
    let translated = paint_list_render::translate_paint_cmd_stream(
        paint_list_api::DeviceIntSize::new(SIDE as i32, SIDE as i32),
        &cmds,
        &[],
        &[],
    );
    net.render_vello(
        &translated.scene,
        &view,
        ColorLoad::Clear(wgpu::Color::TRANSPARENT),
    );

    // The windowed host renders repeatedly through the tile differ; a
    // second render with a slightly changed scene is where the white
    // ground appeared. Nudge one vertex so a tile is genuinely dirty.
    let mut cmds2 = Vec::new();
    let mut cx2 = PaintCx::new(
        &mut cmds2,
        Size {
            width: SIDE as f32,
            height: SIDE as f32,
        },
    );
    cx2.fill_path(
        Path::new()
            .move_to(third, third)
            .line_to(third * 2.0 + 1.0, third)
            .line_to(third * 2.0, third * 2.0)
            .line_to(third, third * 2.0)
            .close()
            .build(),
        ColorF::new(1.0, 0.0, 0.0, 1.0),
    );
    let translated2 = paint_list_render::translate_paint_cmd_stream(
        paint_list_api::DeviceIntSize::new(SIDE as i32, SIDE as i32),
        &cmds2,
        &[],
        &[],
    );
    net.render_vello(
        &translated2.scene,
        &view,
        ColorLoad::Clear(wgpu::Color::TRANSPARENT),
    );

    let pixels = read_texture(&device, &queue, &texture, SIDE);
    let at = |x: u32, y: u32| {
        let i = ((y * SIDE + x) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    };

    // Third render: the real minimap content, the exact commands the
    // windowed host rasterizes. If the white ground reproduces, it is in
    // this content; if not, it is environmental.
    {
        let mut world = mesocosm_core::World::new(4_242, 40);
        for _ in 0..50 {
            world.apply(mesocosm_core::Intent::Idle);
        }
        let mut leaf = mesocosm_views::minimap_leaf(&world);
        let mut cmds = Vec::new();
        let mut cx = PaintCx::new(
            &mut cmds,
            Size {
                width: SIDE as f32,
                height: SIDE as f32,
            },
        );
        use sprigging::Leaf as _;
        leaf.paint(&mut cx);
        let translated = paint_list_render::translate_paint_cmd_stream(
            paint_list_api::DeviceIntSize::new(SIDE as i32, SIDE as i32),
            &cmds,
            &[],
            &[],
        );
        net.render_vello(
            &translated.scene,
            &view,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
        );
        let pixels = read_texture(&device, &queue, &texture, SIDE);
        // The regression this probe exists for: paint_list_render used to
        // copy unpremultiplied colors into premultiplied scene fields, and
        // the consumer's divide-by-alpha turned every 35%-alpha cell white.
        // A correctly premultiplied fill texel has each channel <= alpha.
        let translucent = pixels
            .chunks_exact(4)
            .filter(|t| t[3] > 0 && t[3] < 255)
            .count();
        assert!(translucent > 0, "the 0.35 fills rendered opaque");
        let overbright = pixels
            .chunks_exact(4)
            .filter(|t| t[3] > 0 && t[3] < 200)
            .filter(|t| t[0] > t[3] + 4 && t[1] > t[3] + 4 && t[2] > t[3] + 4)
            .count();
        // AA edges where a bright dot composites over a fill legitimately
        // exceed the bound on a handful of texels; the broken state failed it
        // on essentially all of them (every fill texel read white).
        assert!(
            overbright < translucent / 4,
            "{overbright} of {translucent} translucent texels brighter than              their alpha on every channel: the premultiply round trip is              broken again"
        );
    }

    let corner = at(2, 2);
    let center = at(SIDE / 2, SIDE / 2);
    assert_eq!(center[3], 255, "the square is opaque, got {center:?}");
    assert!(
        center[0] > 200 && center[1] < 50,
        "the square is red, got {center:?}"
    );
    assert_eq!(
        corner[3], 0,
        "unpainted texels are transparent, got {corner:?}; an opaque ground here is \
         what painted the windowed minimap white"
    );
}

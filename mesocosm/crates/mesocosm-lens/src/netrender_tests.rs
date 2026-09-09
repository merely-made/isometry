// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G2's tenant receipt: the DDA target enters netrender's frame on the same
//! device. The browser example reuses this exact external-texture seam.

use mesocosm_core::places::{Ground, Places};
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, PresentedFrame, Scene,
    WgpuHandles, create_netrender_instance,
};

use crate::{BrickFrameInput, BrickMap, BrickRevision, BrickTracer, Flight, Grade};

const SIDE: u32 = 96;

struct Master {
    texture: Option<wgpu::Texture>,
}

impl Compositor for Master {
    fn declare_surface(&mut self, _key: netrender::SurfaceKey, _world_bounds: [f32; 4]) {}

    fn destroy_surface(&mut self, _key: netrender::SurfaceKey) {}

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.texture = Some(frame.master.clone());
    }
}

fn ground() -> Ground {
    Ground::grow(&Places::grown(4_242, 4, 64), 64)
}

#[test]
fn ground_trace_enters_the_same_netrender_frame() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("no adapter; skipping netrender composition receipt");
        return;
    };
    let Ok((device, queue)) = pollster::block_on(adapter.request_device(&Default::default()))
    else {
        eprintln!("adapter declined a device; skipping netrender composition receipt");
        return;
    };
    let net = create_netrender_instance(
        WgpuHandles {
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
    .expect("netrender accepts the tracer device");
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("G2 trace external texture"),
        size: wgpu::Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let top = ground.surface(4, 4).expect("ground column");
    let flight = Flight {
        eye: [4.5, top as f32 + 14.0, 4.5],
        yaw: 0.0,
        pitch: -1.52,
        fov: 0.15,
        far: 48.0,
    };
    let mut tracer = BrickTracer::with_format(
        device.clone(),
        queue.clone(),
        SIDE,
        SIDE,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("G2 trace encode"),
    });
    let trace = tracer
        .encode(
            &mut encoder,
            &view,
            BrickFrameInput::new(
                &map,
                BrickRevision(ground.revision()),
                &flight,
                &Grade::retro(3),
            ),
        )
        .expect("trace encodes into the external texture");
    queue.submit([encoder.finish()]);

    let mut chrome = Scene::new(SIDE, SIDE);
    chrome.push_rect(0.0, 0.0, SIDE as f32, 5.0, [0.2, 0.9, 0.5, 1.0]);
    let external = [ExternalTextureComposite::new(
        &view,
        ExternalTexturePlacement::new([0.0, 0.0, SIDE as f32, SIDE as f32]),
    )
    .with_scene_op_boundary(0)];
    let mut master = Master { texture: None };
    net.render_with_compositor_and_external_textures(
        &chrome,
        wgpu::TextureFormat::Rgba8Unorm,
        &mut master,
        netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
        &external,
    );
    let master = master
        .texture
        .expect("netrender presented one master frame");
    let pixels = net.wgpu_device.read_rgba8_texture(&master, SIDE, SIDE);
    let timings = net.last_frame_timings().expect("netrender records spans");

    assert_eq!(trace.trace_passes, 1);
    assert!(trace.brick_upload_bytes > 0);
    assert!(!timings.spans.is_empty());
    assert!(
        pixels.chunks_exact(4).any(|pixel| pixel[1] > pixel[0] + 40),
        "the external trace and chrome reached the composed master"
    );
}

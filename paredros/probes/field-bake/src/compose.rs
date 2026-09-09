// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Offscreen composition, headed presentation, and capture for the receipt.

use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Renderer, Scene, SurfaceKey, WgpuHandles, create_netrender_instance,
};

pub const MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub struct Composer {
    net: Renderer,
    size: [u32; 2],
}

impl Composer {
    pub fn new(handles: WgpuHandles, size: [u32; 2]) -> Self {
        let net = create_netrender_instance(
            handles,
            NetrenderOptions {
                tile_cache_size: Some(64),
                enable_vello: true,
                ..Default::default()
            },
        )
        .expect("netrender init");
        Self { net, size }
    }

    pub fn compose(&self, chrome: &Scene, tenant: &wgpu::TextureView) -> wgpu::Texture {
        let external = [ExternalTextureComposite::new(
            tenant,
            ExternalTexturePlacement::new([0.0, 0.0, self.size[0] as f32, self.size[1] as f32]),
        )
        .with_scene_op_boundary(0)];
        let mut grab = MasterGrab { master: None };
        self.net.render_with_compositor_and_external_textures(
            chrome,
            MASTER_FORMAT,
            &mut grab,
            netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &external,
        );
        grab.master.expect("netrender presented no master")
    }

    pub fn present(
        &self,
        master: &wgpu::Texture,
        target: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        target_size: [u32; 2],
    ) {
        let view = master.create_view(&Default::default());
        self.net.compose_external_texture(
            &view,
            target,
            format,
            target_size[0],
            target_size[1],
            ExternalTexturePlacement::new([0.0, 0.0, target_size[0] as f32, target_size[1] as f32]),
        );
    }

    pub fn capture(&self, master: &wgpu::Texture) -> Capture {
        Capture {
            pixels: self
                .net
                .wgpu_device
                .read_rgba8_texture(master, self.size[0], self.size[1]),
            size: self.size,
        }
    }
}

struct MasterGrab {
    master: Option<wgpu::Texture>,
}

impl Compositor for MasterGrab {
    fn declare_surface(&mut self, _key: SurfaceKey, _bounds: [f32; 4]) {}

    fn destroy_surface(&mut self, _key: SurfaceKey) {}

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.master = Some(frame.master.clone());
    }
}

pub struct Capture {
    pub pixels: Vec<u8>,
    size: [u32; 2],
}

impl Capture {
    pub fn write_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut encoder = png::Encoder::new(file, self.size[0], self.size[1]);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut writer| writer.write_image_data(&self.pixels))
            .map_err(std::io::Error::other)
    }
}

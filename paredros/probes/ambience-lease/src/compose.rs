// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The netrender half: an offscreen master with the tenant composited
//! at scene-op boundary zero, a present path for the windowed run, and
//! a capture that measures whether anything actually rendered.

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

    /// One composed master: chrome over the tenant's frame.
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

    /// Blit the composed master onto a surface texture, filling it. The
    /// window may be any size; the master is always [`Self::size`].
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
        let pixels = self
            .net
            .wgpu_device
            .read_rgba8_texture(master, self.size[0], self.size[1]);
        let mut seen = std::collections::HashSet::new();
        let background = [pixels[0], pixels[1], pixels[2]];
        let mut lit = 0usize;
        for pixel in pixels.chunks_exact(4) {
            seen.insert([pixel[0], pixel[1], pixel[2]]);
            if pixel[..3] != background {
                lit += 1;
            }
        }
        Capture {
            pixels,
            size: self.size,
            distinct: seen.len(),
            lit: lit as f32 / (self.size[0] * self.size[1]) as f32,
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
    pub size: [u32; 2],
    pub distinct: usize,
    pub lit: f32,
}

impl Capture {
    pub fn write_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
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

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The chrome device: one physical wgpu device and the blend passes for the
//! frame master and host surface, shared by every chrome surface and the frame
//! graph.
//!
//! Both lanes end the same way — a vello scene rasterized into a transparent
//! texture on the game's own device, then blended over the frame. The minimap
//! (painted lane) and the vitals panel (cambium lane) differ only in what
//! builds the scene, so the device, the texture pair, and the composite live
//! here once rather than twice.
//!
//! `Renderer::render_vello` takes `&self`, so the chrome surfaces share one
//! rasterizer and use separate targets. RG3 temporarily adds a second
//! Netrender instance over the exact same wgpu handles for the current graph
//! API because Genet and Mere still expose an older paint-list source identity.
//! Pin alignment removes that compatibility instance; it is not a second GPU
//! authority.

use mesocosm_render::composite::Composite;
use netrender::{
    ColorLoad, Renderer as NetRenderer, Scene, WgpuHandles, create_netrender_instance,
};
use netrender_graph::{
    Compositor, ExternalTexturePlacement, OpaqueTenantInput, OpaqueTenantMetadata,
    OpaqueTenantReceipt, PresentedFrame, SurfaceKey,
};

pub(crate) struct FrameMaster {
    pub texture: wgpu::Texture,
    pub receipt: OpaqueTenantReceipt,
}

const FRAME_MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[derive(Default)]
struct MasterCapture {
    texture: Option<wgpu::Texture>,
}

impl Compositor for MasterCapture {
    fn declare_surface(&mut self, _key: SurfaceKey, _world_bounds: [f32; 4]) {}

    fn destroy_surface(&mut self, _key: SurfaceKey) {}

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        assert!(
            frame.layers.is_empty(),
            "Mesocosm's frame scene has no OS compositor layers"
        );
        self.texture = Some(frame.master.clone());
    }
}

pub struct Chrome {
    net: NetRenderer,
    /// Temporary RG3 facade over the same physical handles. The retained UI
    /// stack still names the older paint-list source identity through `net`;
    /// this current facade owns only the frame graph until those pins align.
    frame_net: netrender_graph::Renderer,
    /// Chrome lands in the graph's unorm master before presentation.
    master_composite: Composite,
    /// Only the completed master crosses this surface-format pass.
    surface_composite: Composite,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// A vello target and the sRGB-tagged twin it is sampled through.
///
/// Vello writes display-encoded values into a plain `Unorm` texture (its target
/// must be, because it writes through a storage binding and sRGB formats
/// cannot be). Sampling those bytes raw and writing to an sRGB target encodes
/// twice and brightens everything, so each raster is byte-copied into this
/// copy-compatible twin, whose decode-on-sample cancels the target's encode.
pub struct Raster {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sample_texture: wgpu::Texture,
    sample_view: wgpu::TextureView,
    size: (u32, u32),
}

impl Raster {
    pub fn new(device: &wgpu::Device, label: &str, width: u32, height: u32) -> Self {
        let extent = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let sample_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&Default::default()),
            sample_view: sample_texture.create_view(&Default::default()),
            texture,
            sample_texture,
            size: (extent.width, extent.height),
        }
    }

    /// What a composite samples.
    pub fn sample_view(&self) -> &wgpu::TextureView {
        &self.sample_view
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

impl Chrome {
    /// Builds the chrome device. `None` if netrender declines it, in which case
    /// the game runs chromeless rather than not at all.
    pub fn new(handles: WgpuHandles, format: wgpu::TextureFormat, tiles: u32) -> Option<Self> {
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let graph_handles = netrender_graph::WgpuHandles {
            instance: handles.instance.clone(),
            adapter: handles.adapter.clone(),
            device: device.clone(),
            queue: queue.clone(),
        };
        let net = create_netrender_instance(
            handles,
            netrender::NetrenderOptions {
                tile_cache_size: Some(tiles),
                enable_vello: true,
                ..Default::default()
            },
        )
        .ok()?;
        let frame_net = netrender_graph::create_netrender_instance(
            graph_handles,
            netrender_graph::NetrenderOptions {
                tile_cache_size: Some(tiles),
                enable_vello: true,
                ..Default::default()
            },
        )
        .ok()?;
        Some(Self {
            master_composite: Composite::new(&device, FRAME_MASTER_FORMAT),
            surface_composite: Composite::new(&device, format),
            net,
            frame_net,
            device,
            queue,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Imports the completed section texture as one opaque tenant and returns
    /// Netrender's initialized master. Section internals stay behind the named
    /// producer boundary; only its display texture crosses it.
    pub(crate) fn frame_master(
        &self,
        section: &wgpu::Texture,
        frame: (u32, u32),
        fallback_count: u64,
    ) -> FrameMaster {
        let scene = netrender_graph::Scene::new(frame.0, frame.1);
        let metadata = OpaqueTenantMetadata::new(
            "mesocosm-section",
            "mesocosm_lens::BrickTracer + mesocosm_genet::Section::render",
            fallback_count,
            0,
            ExternalTexturePlacement::new([0.0, 0.0, frame.0 as f32, frame.1 as f32]),
        )
        .with_reported_physical_submission_count(1);
        let tenant = OpaqueTenantInput::new(section, metadata);
        let mut capture = MasterCapture::default();
        let receipt = self.frame_net.render_with_opaque_tenant(
            &scene,
            FRAME_MASTER_FORMAT,
            &mut capture,
            netrender_graph::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &tenant,
        );
        FrameMaster {
            texture: capture
                .texture
                .expect("Netrender returns one master for an opaque tenant frame"),
            receipt,
        }
    }

    /// Rasterizes `scene` into `raster` and refreshes its sRGB twin.
    pub fn raster(&self, raster: &Raster, scene: &Scene) {
        self.net.render_vello(
            scene,
            &raster.view,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("srgb twin"),
            });
        encoder.copy_texture_to_texture(
            raster.texture.as_image_copy(),
            raster.sample_texture.as_image_copy(),
            wgpu::Extent3d {
                width: raster.size.0,
                height: raster.size.1,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
    }

    /// Blends chrome `content` into the Netrender master at `dest`.
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        content: &wgpu::TextureView,
        dest: (f32, f32, f32, f32),
        frame: (u32, u32),
    ) {
        self.master_composite.draw(
            &self.device,
            &self.queue,
            encoder,
            target,
            content,
            dest,
            frame,
        );
    }

    /// Blits the completed frame master to the host surface.
    pub fn draw_surface(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        content: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        self.surface_composite.draw(
            &self.device,
            &self.queue,
            encoder,
            target,
            content,
            (0.0, 0.0, frame.0 as f32, frame.1 as f32),
            frame,
        );
    }

    /// The same blend into a capture frame, which uses the offscreen format
    /// rather than the surface's.
    pub fn draw_as(
        &self,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        content: &wgpu::TextureView,
        dest: (f32, f32, f32, f32),
        frame: (u32, u32),
    ) {
        Composite::new(&self.device, format).draw(
            &self.device,
            &self.queue,
            encoder,
            target,
            content,
            dest,
            frame,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const WIDTH: u32 = 32;
    const HEIGHT: u32 = 24;
    const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    fn source(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RG3 Mesocosm tenant fixture"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut bytes = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let rgba = if x < WIDTH / 2 {
                    [24, 72 + y as u8, 208, 255]
                } else {
                    [224, 176, 24 + y as u8, 255]
                };
                bytes.extend_from_slice(&rgba);
            }
        }
        queue.write_texture(
            texture.as_image_copy(),
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(HEIGHT),
            },
            texture.size(),
        );
        texture
    }

    fn legacy_presented(chrome: &Chrome, source: &wgpu::Texture) -> wgpu::Texture {
        let target = chrome.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RG3 Mesocosm legacy presented frame"),
            size: source.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SURFACE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&Default::default());
        let source_view = source.create_view(&Default::default());
        let mut encoder = chrome.device.create_command_encoder(&Default::default());
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RG3 baseline clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        // A separate compositor models the old Section-owned full-frame draw.
        // Its rectangle cannot be rewritten by the chrome draw below.
        let legacy = Composite::new(&chrome.device, SURFACE_FORMAT);
        legacy.draw(
            &chrome.device,
            &chrome.queue,
            &mut encoder,
            &target_view,
            &source_view,
            (0.0, 0.0, WIDTH as f32, HEIGHT as f32),
            (WIDTH, HEIGHT),
        );
        legacy.draw(
            &chrome.device,
            &chrome.queue,
            &mut encoder,
            &target_view,
            &source_view,
            (WIDTH as f32 - 8.0, HEIGHT as f32 - 6.0, 8.0, 6.0),
            (WIDTH, HEIGHT),
        );
        chrome.queue.submit([encoder.finish()]);
        target
    }

    fn legacy_master(chrome: &Chrome, source: &wgpu::Texture) -> wgpu::Texture {
        let target = chrome.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RG3 Mesocosm legacy linear master"),
            size: source.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FRAME_MASTER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let target_view = target.create_view(&Default::default());
        let source_view = source.create_view(&Default::default());
        let mut encoder = chrome.device.create_command_encoder(&Default::default());
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RG3 linear master clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        Composite::new(&chrome.device, FRAME_MASTER_FORMAT).draw(
            &chrome.device,
            &chrome.queue,
            &mut encoder,
            &target_view,
            &source_view,
            (0.0, 0.0, WIDTH as f32, HEIGHT as f32),
            (WIDTH, HEIGHT),
        );
        chrome.queue.submit([encoder.finish()]);
        target
    }

    fn presented(
        chrome: &Chrome,
        master: &wgpu::Texture,
        overlay: &wgpu::Texture,
    ) -> wgpu::Texture {
        let master_view = master.create_view(&Default::default());
        let overlay_view = overlay.create_view(&Default::default());
        let mut overlay_encoder = chrome.device.create_command_encoder(&Default::default());
        chrome.draw(
            &mut overlay_encoder,
            &master_view,
            &overlay_view,
            (WIDTH as f32 - 8.0, HEIGHT as f32 - 6.0, 8.0, 6.0),
            (WIDTH, HEIGHT),
        );
        chrome.queue.submit([overlay_encoder.finish()]);

        let target = chrome.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RG3 Mesocosm presented frame"),
            size: master.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SURFACE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&Default::default());
        let mut encoder = chrome.device.create_command_encoder(&Default::default());
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RG3 presented clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
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
        }
        chrome.draw_surface(&mut encoder, &target_view, &master_view, (WIDTH, HEIGHT));
        chrome.queue.submit([encoder.finish()]);
        target
    }

    #[test]
    #[ignore = "physical RG3 Mesocosm opaque-tenant receipt"]
    fn opaque_section_graph_byte_matches_direct_composite() {
        let handles = netrender::boot().expect("shared wgpu device");
        let adapter = handles.adapter.get_info();
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let chrome = Chrome::new(handles, SURFACE_FORMAT, 32)
            .expect("both Netrender facades on shared handles");
        let source = source(&device, &queue);
        let direct_master = legacy_master(&chrome, &source);
        let framed = chrome.frame_master(&source, (WIDTH, HEIGHT), 3);
        let direct_master_bytes =
            chrome
                .frame_net
                .wgpu_device
                .read_rgba8_texture(&direct_master, WIDTH, HEIGHT);
        let graph_master_bytes =
            chrome
                .frame_net
                .wgpu_device
                .read_rgba8_texture(&framed.texture, WIDTH, HEIGHT);
        assert_eq!(graph_master_bytes, direct_master_bytes);
        let legacy_presented = legacy_presented(&chrome, &source);
        let graph_presented = presented(&chrome, &framed.texture, &source);
        let legacy_presented_bytes =
            chrome
                .frame_net
                .wgpu_device
                .read_rgba8_texture(&legacy_presented, WIDTH, HEIGHT);
        let graph_presented_bytes =
            chrome
                .frame_net
                .wgpu_device
                .read_rgba8_texture(&graph_presented, WIDTH, HEIGHT);
        let presentation_max_channel_delta = graph_presented_bytes
            .iter()
            .zip(&legacy_presented_bytes)
            .map(|(graph, legacy)| graph.abs_diff(*legacy))
            .max()
            .unwrap_or_default();
        assert!(
            presentation_max_channel_delta <= 3,
            "presentation conversion drifted by {presentation_max_channel_delta} channel levels"
        );
        assert!(
            HashSet::<[u8; 4]>::from_iter(
                graph_presented_bytes
                    .chunks_exact(4)
                    .map(|pixel| pixel.try_into().expect("RGBA pixel"))
            )
            .len()
                > 8,
            "fixture must retain visible variation"
        );

        let receipt = &framed.receipt;
        assert_eq!(receipt.tenant_name, "mesocosm-section");
        assert_eq!(receipt.fallback_count, 3);
        assert_eq!(receipt.scene_op_boundary, 0);
        assert_eq!(receipt.caller_reported_physical_submission_count, Some(1));
        assert_eq!(receipt.logical_opaque_producer_boundaries, 1);
        assert_eq!(receipt.graph_encoder_batches, 1);
        assert_eq!(receipt.graph_submission_boundaries, 1);
        for needle in [
            "mesocosm-section",
            "opaque tenant composite",
            "netrender initialized master",
            "rasterizer=Classic execution_boundary=opaque_submission",
        ] {
            assert!(
                receipt.logical_plan_dump.contains(needle),
                "missing {needle}"
            );
        }

        let path = crate::played::default_out_dir().join("rg3b_opaque_tenant.json");
        std::fs::create_dir_all(path.parent().expect("receipt parent"))
            .expect("create receipt directory");
        let json = serde_json::json!({
            "adapter": adapter.name,
            "backend": format!("{:?}", adapter.backend),
            "byte_matches_direct_composite": true,
            "presented_surface_max_channel_delta": presentation_max_channel_delta,
            "master_format": format!("{FRAME_MASTER_FORMAT:?}"),
            "presentation_format": format!("{SURFACE_FORMAT:?}"),
            "width": WIDTH,
            "height": HEIGHT,
            "tenant_name": receipt.tenant_name,
            "producer_path": receipt.producer_path,
            "fallback_count": receipt.fallback_count,
            "scene_op_boundary": receipt.scene_op_boundary,
            "caller_reported_physical_submission_count": receipt.caller_reported_physical_submission_count,
            "logical_opaque_producer_boundaries": receipt.logical_opaque_producer_boundaries,
            "graph_encoder_batches": receipt.graph_encoder_batches,
            "graph_submission_boundaries": receipt.graph_submission_boundaries,
            "logical_plan_dump": receipt.logical_plan_dump,
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&json).unwrap())
            .expect("write RG3 Mesocosm receipt");
        println!("RG3 Mesocosm receipt: {}", path.display());
    }
}

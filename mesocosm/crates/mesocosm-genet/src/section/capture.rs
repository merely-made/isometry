// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Section {
    /// Reads the most recently traced frame back as RGBA8, with `overlay`
    /// given the chance to composite chrome over it first.
    ///
    /// The last frame rather than a fresh trace: the capture is meant to be
    /// the picture the player was looking at, and re-tracing would show a
    /// world one frame further on than the receipt's own hash.
    pub fn capture(
        &self,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat),
    ) -> Option<(u32, u32, Vec<u8>)> {
        self.capture_from(&self.display, overlay)
    }

    /// Reads a completed frame master back as RGBA8, with `overlay` given the
    /// chance to composite chrome over it first. RG3 uses this route so its
    /// evidence crosses the same imported-tenant master as the live window.
    pub fn capture_from(
        &self,
        master: &wgpu::Texture,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat),
    ) -> Option<(u32, u32, Vec<u8>)> {
        let (shot, view) = target(
            &self.device,
            self.width,
            self.height,
            CAPTURE_FORMAT,
            "section capture",
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("section capture"),
            });
        Composite::new(&self.device, CAPTURE_FORMAT).draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            &master.create_view(&Default::default()),
            (0.0, 0.0, self.width as f32, self.height as f32),
            (self.width, self.height),
        );
        overlay(&mut encoder, &view, CAPTURE_FORMAT);
        self.read_back(encoder, &shot)
            .map(|pixels| (self.width, self.height, pixels))
    }

    fn read_back(
        &self,
        mut encoder: wgpu::CommandEncoder,
        colour: &wgpu::Texture,
    ) -> Option<Vec<u8>> {
        // Copy rows must be aligned, so the staging buffer is padded and the
        // padding is stripped after mapping.
        let unpadded = self.width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("section readback"),
            size: (padded * self.height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: colour,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let mapped = slice.get_mapped_range().ok()?;
        let mut pixels = Vec::with_capacity((unpadded * self.height) as usize);
        for row in 0..self.height {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        drop(mapped);
        staging.unmap();
        Some(pixels)
    }
}

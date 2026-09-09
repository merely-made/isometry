// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The minimap: a sprigging leaf rasterized by netrender on the game's own
//! device.
//!
//! Route A of the staged host ruling (2026-08-02): the leaf paints `PaintCmd`s,
//! `paint_list_render` lowers them, and [`crate::chrome`] rasterizes and
//! blends. **Painted, not textless** — the guard was lane discipline, not a
//! text ban (views founding plan §6, amended 2026-08-29). This lane draws
//! marks in the scene; words go through cambium, which
//! [`crate::vitals`] is the first consumer of. What the guard still forbids is
//! teaching this lane lettering as a shortcut.

use mesocosm_core::World;
use mesocosm_render::{Camera, Renderer, SceneItem};
use mesocosm_views::MinimapLeaf;
use sprigging::{Leaf, PaintCx, Size};

use crate::chrome::{Chrome, Raster};

/// The minimap's square side, in pixels. Also the rasterized texture's size,
/// so the leaf paints at the resolution it is shown.
pub const SIDE: u32 = 160;

/// Distance from the frame's corner.
pub const MARGIN: f32 = 12.0;

/// Steps between backdrop re-renders. The ruling asks for dynamically
/// generated, not per-frame: the enclosure drifts on ecology time, and a
/// cadence keeps the world's self-portrait current without paying a second
/// scene render every frame.
///
/// **An ambient background**: the non-interactive backdrop subtype, mere's
/// Game of Life tier. Backdrop names where a layer sits, not whether it
/// acts (ruled 2026-08-03); this one does not act. Nothing in it has a
/// hull, exerts a field, or is navigable, because the world it portrays
/// has no such thing yet. When the world grows props and fields, their
/// projection enters as an interactive backdrop through the scene lane,
/// not by enriching this one.
/// Retimed with the tempo (TD2): ten world steps was a redraw every ~6 frames
/// at 60 ticks a second, and at the canonical 10 it would have become one a
/// second. Two steps restores both the wall cadence and the frame cost.
const BACKDROP_CADENCE: u64 = 2;

pub struct Hud {
    leaf: MinimapLeaf,
    raster: Raster,
    /// The backdrop's own small renderer, sized to the minimap.
    shot: Renderer,
    backdrop_view: wgpu::TextureView,
    _backdrop: wgpu::Texture,
    /// The step count the backdrop was last rendered at.
    rendered_at: Option<u64>,
}

/// Straight down at the whole enclosure, aligned with the minimap's mapping:
/// world +x reads right and +z reads down, so the cells sit over the terrain
/// they govern. Pitch stops a degree short of vertical because a look-at with
/// view parallel to up is singular.
fn overhead() -> Camera {
    Camera {
        target: [0.0, 0.0, 0.0],
        extent: mesocosm_core::world::ENCLOSURE as f32 + 2.0,
        yaw: std::f32::consts::FRAC_PI_2,
        pitch: 1.553_343_f32,
        aspect: 1.0,
    }
}

/// Where the minimap sits in a frame of this size.
pub fn placement(frame: (u32, u32)) -> (f32, f32, f32, f32) {
    let x = frame.0 as f32 - SIDE as f32 - MARGIN;
    (x.max(0.0), MARGIN, SIDE as f32, SIDE as f32)
}

impl Hud {
    pub fn new(chrome: &Chrome, world: &World) -> Self {
        let device = chrome.device();
        let shot = Renderer::with_device(device.clone(), chrome.queue().clone(), SIDE, SIDE);
        let backdrop = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("minimap backdrop"),
            size: wgpu::Extent3d {
                width: SIDE,
                height: SIDE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: shot.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        Self {
            leaf: mesocosm_views::minimap_leaf(world),
            raster: Raster::new(device, "minimap", SIDE, SIDE),
            backdrop_view: backdrop.create_view(&Default::default()),
            _backdrop: backdrop,
            shot,
            rendered_at: None,
        }
    }

    /// Refresh the cached image after replacing the starting world.
    pub fn invalidate(&mut self) {
        self.rendered_at = None;
    }

    /// Re-renders the enclosure's self-portrait if the cadence has elapsed.
    ///
    /// The same scene items the frame draws, seen from straight above: the
    /// backdrop is the world's own image, generated, never an asset.
    pub fn render_backdrop(&mut self, items: &[SceneItem], steps: u64) {
        if self
            .rendered_at
            .is_some_and(|then| steps.saturating_sub(then) < BACKDROP_CADENCE)
        {
            return;
        }
        self.rendered_at = Some(steps);

        let mut encoder =
            self.shot
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("backdrop"),
                });
        self.shot
            .draw_scene(&mut encoder, &self.backdrop_view, items, &overhead());
        self.shot.queue().submit(Some(encoder.finish()));
    }

    /// Reprojects the world and rasterizes the minimap if anything changed.
    ///
    /// The leaf dedups identical projections, so an idle world costs a scene
    /// build and no raster.
    pub fn refresh(&mut self, chrome: &Chrome, world: &World) {
        self.leaf.refresh_from(mesocosm_views::minimap_leaf(world));

        if !self.leaf.paint_dirty() {
            return;
        }
        let mut cmds = Vec::new();
        let mut cx = PaintCx::new(
            &mut cmds,
            Size {
                width: SIDE as f32,
                height: SIDE as f32,
            },
        );
        self.leaf.paint(&mut cx);

        let translated = paint_list_render::translate_paint_cmd_stream(
            paint_list_api::DeviceIntSize::new(SIDE as i32, SIDE as i32),
            &cmds,
            &[],
            &[],
        );
        chrome.raster(&self.raster, &translated.scene);
    }

    /// Blends the minimap into the frame's top-right corner: terrain under
    /// territory, the cells whose translucency exists for exactly this on top.
    pub fn composite(
        &self,
        chrome: &Chrome,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        let dest = placement(frame);
        chrome.draw(encoder, target, &self.backdrop_view, dest, frame);
        chrome.draw(encoder, target, self.raster.sample_view(), dest, frame);
    }

    /// The same, into a capture frame's offscreen format.
    pub fn capture_composite(
        &self,
        chrome: &Chrome,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        let dest = placement(frame);
        chrome.draw_as(format, encoder, target, &self.backdrop_view, dest, frame);
        chrome.draw_as(
            format,
            encoder,
            target,
            self.raster.sample_view(),
            dest,
            frame,
        );
    }
}

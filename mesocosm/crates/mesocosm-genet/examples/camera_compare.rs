// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! DC4's three-camera contact sheet: one tick, three cameras, one picture.
//!
//! **A composition, not a render.** The three panes are the captures the
//! three `camera_*.scenario` runs already wrote — the same golden trace, the
//! same tick, the same state hash, and nothing between them but which way the
//! section looked. Re-tracing them here would put three different ticks on
//! one sheet, which is exactly the comparison Mark's 2026-09-02 ruling asks
//! this slice *not* to make.
//!
//! The captions are rasterized through the real cambium/netrender chrome over
//! a headless device, the way `p3_receipt` rasterizes its panel: no text
//! rendering lives in this repo, so the words are Livery's and the sentences
//! are [`mesocosm_views::caption`]'s. The panes are blitted with the same
//! `blit` this crate's other sheet examples use.
//!
//! ```text
//! # after the three scenario runs have written their captures:
//! cargo run -p mesocosm-genet --release --example camera_compare
//! ```
//!
//! Writes `Code/testing/mesocosm/camera_compare.png`, full resolution and
//! undownscaled — legibility is the question being judged, so a sheet that
//! shrank the evidence would answer it by construction.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cambium::GenetAppRunner;
use genet_livery::{
    Device, InteractionStates, StyleSet, TextSystem,
    emit_paint_list_with_text_system_scrolled_with_images, layout_with_text_system, resolve_styles,
};
use genet_scripted_dom::ScriptedDom;
use mesocosm_genet::chrome::{Chrome, Raster};
use mesocosm_genet::section::CameraMode;
use mesocosm_genet::{hud, played};
use mesocosm_views::{Caption, CaptionChild, caption_css, caption_root};
use paint_list_api::{DeviceIntSize, PaintList as _};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// The caption band under each pane. Tall enough for the two lines the sheet
/// gives every arm.
const BAND: u32 = 62;

/// A hairline between the panes, in the caption band's own ground, so three
/// screenshots do not read as one very wide one.
const GUTTER: u32 = 4;

fn main() {
    let dir = played::default_out_dir();
    let panes: Vec<(CameraMode, Image)> = CameraMode::ALL
        .into_iter()
        .map(|mode| {
            let path = dir.join(format!("camera_{}.png", mode.name()));
            let image = read_png(&path).unwrap_or_else(|why| {
                panic!(
                    "{}: {why}\nrun the three camera_*.scenario replays first",
                    path.display()
                )
            });
            (mode, image)
        })
        .collect();

    let (width, height) = panes[0].1.size;
    for (mode, image) in &panes {
        assert_eq!(
            image.size,
            (width, height),
            "the {} pane is a different size; the three captures must be one run's framing",
            mode.name()
        );
    }

    let chrome = pollster::block_on(headless_chrome());
    let mut strip = CaptionStrip::new(&chrome, width, BAND);

    let sheet_width = width * 3 + GUTTER * 2;
    let sheet_height = height + BAND;
    let mut sheet = vec![0u8; (sheet_width * sheet_height * 4) as usize];
    for pixel in sheet.chunks_exact_mut(4) {
        // The caption band's own ground, carried across the gutters, so the
        // sheet reads as one thing rather than three photographs on a mat.
        pixel.copy_from_slice(&[0x11, 0x16, 0x1a, 0xff]);
    }

    for (column, (mode, image)) in panes.iter().enumerate() {
        let x = column as u32 * (width + GUTTER);
        blit(
            &mut sheet,
            sheet_width,
            &image.pixels,
            (width, height),
            (x, 0),
        );
        let caption = strip.raster(&chrome, &Caption::new(mode.name(), note_for(*mode)));
        blit(
            &mut sheet,
            sheet_width,
            &caption,
            (width, BAND),
            (x, height),
        );
    }

    let path = dir.join("camera_compare.png");
    played::write_png(&path, sheet_width, sheet_height, &sheet).expect("write png");
    println!(
        "wrote {} — {sheet_width}x{sheet_height}, three {width}x{height} panes of one tick",
        path.display()
    );
}

/// What each arm did to the frame, in the line under its name. Descriptive
/// and not a verdict: the ruling is Mark's and this sheet is the instrument.
fn note_for(mode: CameraMode) -> &'static str {
    match mode {
        CameraMode::Side => "the shipped section, looking down -z: bodies chain into the camera",
        CameraMode::Across => "turned a quarter, looking down -x: bodies chain across the view",
        CameraMode::Oblique => "tilted 20 degrees both ways: bodies chain along a short diagonal",
    }
}

/// One decoded capture.
struct Image {
    size: (u32, u32),
    pixels: Vec<u8>,
}

/// Reads a capture back as RGBA8. The captures are written by
/// [`played::write_png`], so this is that encoder's own inverse.
fn read_png(path: &std::path::Path) -> Result<Image, String> {
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().map_err(|error| error.to_string())?;
    let mut pixels = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut pixels)
        .map_err(|error| error.to_string())?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "expected 8-bit RGBA, found {:?} at {:?}",
            info.color_type, info.bit_depth
        ));
    }
    pixels.truncate(info.buffer_size());
    Ok(Image {
        size: (info.width, info.height),
        pixels,
    })
}

/// Copies one RGBA image into the sheet at `(x, y)`.
fn blit(sheet: &mut [u8], sheet_width: u32, src: &[u8], size: (u32, u32), at: (u32, u32)) {
    let (width, height) = size;
    for row in 0..height {
        let from = (row * width * 4) as usize;
        let to = (((at.1 + row) * sheet_width + at.0) * 4) as usize;
        sheet[to..to + (width * 4) as usize]
            .copy_from_slice(&src[from..from + (width * 4) as usize]);
    }
}

/// The caption band, laid out and rasterized by the real cambium path.
///
/// The same five steps every chrome lane in this crate takes — style, lay
/// out, paint, lower, rasterize — over a headless device instead of the
/// game's. One runner reused across the three columns, because building a
/// font context per caption is what makes a panel expensive.
struct CaptionStrip {
    runner: GenetAppRunner<Caption, fn(&Caption) -> CaptionChild, CaptionChild>,
    raster: Raster,
    text: TextSystem,
    style_set: StyleSet,
    device: Device,
    generation: u64,
    size: (u32, u32),
}

impl CaptionStrip {
    fn new(chrome: &Chrome, width: u32, height: u32) -> Self {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        Self {
            runner: GenetAppRunner::new(
                dom,
                caption_root as fn(&Caption) -> CaptionChild,
                Caption::default(),
            ),
            raster: Raster::new(chrome.device(), "caption", width, height),
            text: TextSystem::new(),
            style_set: StyleSet::cambium(&[caption_css()]),
            device: Device::screen(width as f32, height as f32),
            generation: 0,
            size: (width, height),
        }
    }

    /// One caption, rasterized and read back as RGBA over the band's ground.
    fn raster(&mut self, chrome: &Chrome, caption: &Caption) -> Vec<u8> {
        self.runner.update(|state| *state = caption.clone());
        self.generation += 1;
        let (width, height) = self.size;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let styles = resolve_styles(
            &*dom,
            &self.style_set,
            &self.device,
            &InteractionStates::default(),
        );
        let (styles, fragments) = layout_with_text_system(
            &*dom,
            &styles,
            width as f32,
            height as f32,
            genet_livery::ViewportSizes::uniform(width as f32, height as f32),
            &mut self.text,
            &HashMap::new(),
        )
        .expect("the caption lays out");
        let list = emit_paint_list_with_text_system_scrolled_with_images(
            &*dom,
            &styles,
            &fragments,
            DeviceIntSize::new(width as i32, height as i32),
            self.generation,
            &mut self.text,
            &HashMap::new(),
            &HashMap::new(),
        );
        let translated = paint_list_render::translate_paint_cmd_stream(
            list.viewport(),
            list.commands(),
            list.fonts(),
            list.images(),
        );
        chrome.raster(&self.raster, &translated.scene);
        read_back(chrome, &self.raster, width, height)
    }
}

/// The rasterized band, off the sRGB twin the composite would have sampled.
fn read_back(chrome: &Chrome, raster: &Raster, width: u32, height: u32) -> Vec<u8> {
    let device = chrome.device();
    let queue = chrome.queue();
    let (target, view) = {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("caption band"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        (texture, view)
    };
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("caption band"),
    });
    {
        // The band's own ground under the words, so a caption that is mostly
        // transparent still lands on the sheet's colour and not on nothing.
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("caption ground"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.067,
                        g: 0.086,
                        b: 0.102,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    chrome.draw_as(
        FORMAT,
        &mut encoder,
        &view,
        raster.sample_view(),
        (0.0, 0.0, width as f32, height as f32),
        (width, height),
    );

    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("caption readback"),
        size: (padded * height) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll");
    let mapped = slice.get_mapped_range().expect("mapped range");
    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    for row in 0..height {
        let start = (row * padded) as usize;
        pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    pixels
}

async fn headless_chrome() -> Chrome {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
            compatible_surface: None,
        })
        .await
        .expect("an adapter");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("camera compare"),
            ..Default::default()
        })
        .await
        .expect("a device");
    Chrome::new(
        netrender::WgpuHandles {
            instance,
            adapter,
            device,
            queue,
        },
        FORMAT,
        hud::SIDE,
    )
    .expect("the chrome device")
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The checkpoint lane: the third chrome surface, and the only one that is
//! usually not there.
//!
//! Same arrangement as [`crate::vitals`] — [`mesocosm_views::succession_root`]
//! diffed into a `ScriptedDom`, Livery styling and laying it out, the paint
//! list lowered and rasterized, [`crate::chrome`] blending it over the frame.
//! It differs in two ways only: it sits in the middle of the frame rather than
//! a corner, because the world behind it has stopped; and it is built from the
//! driver's question rather than from the world, because a world can say what
//! is and never that it is waiting.
//!
//! No text rendering lives here. Words are Livery's, and the sentences are
//! `mesocosm-views`'.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cambium::GenetAppRunner;
use genet_livery::{
    Device, InteractionStates, LiveryLayout, StylePlane, StyleSet, TextSystem, ViewportSizes,
    emit_paint_list_with_text_system_scrolled_with_images, layout_with_text_system, resolve_styles,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use mesocosm_runtime::{Checkpoint, Occasion};
use mesocosm_views::{Succession, SuccessionChild};
use paint_list_api::{DeviceIntSize, PaintList as _};

use crate::chrome::{Chrome, Raster};

/// The panel's box, in pixels, and the raster's size with it.
///
/// Sized to the taller of the two questions — a birth, which carries four facts
/// and both answers — with a little slack. A box with an inch of empty bottom
/// reads as a panel waiting for something that never arrives.
const WIDTH: u32 = 468;
const HEIGHT: u32 = 208;

type Runner = GenetAppRunner<Succession, fn(&Succession) -> SuccessionChild, SuccessionChild>;

/// Reads the driver's question into the words a player sees.
///
/// The one conversion in the lane, and it is deliberately dumb: every sentence
/// belongs to `mesocosm-views`, and this supplies numbers.
pub fn succession_of(checkpoint: &Checkpoint) -> Succession {
    match checkpoint.occasion {
        Occasion::Birth(birth) => Succession::birth(
            birth.parent.0,
            birth.offspring.0,
            birth.lineage.0,
            birth.substance_mg,
            birth.reserve_mg,
            checkpoint.heir().is_some(),
        ),
        Occasion::Loss(loss) => Succession::loss(
            loss.organism.0,
            loss.lineage.0,
            checkpoint.heirs.len(),
            checkpoint.heir().map(|heir| heir.0),
        ),
        // The lineage checkpoint (PE3a). Same lane, same two keys — Enter is
        // still `default_answer`, which is `Resume` here as everywhere — and no
        // new panel: the review is PE3b's.
        Occasion::Epoch(boundary) => Succession::epoch(
            boundary.epoch,
            boundary.lineage.0,
            boundary.turned,
            boundary.committed,
        ),
    }
}

pub struct SuccessionChrome {
    runner: Runner,
    raster: Raster,
    text: TextSystem,
    style_set: StyleSet,
    device: Device,
    generation: u64,
    /// The question the raster currently holds. A frame whose checkpoint is
    /// unchanged — which, since the world is stopped, is nearly all of them —
    /// pays a comparison and no raster.
    shown: Option<Succession>,
}

impl SuccessionChrome {
    pub fn new(chrome: &Chrome) -> Self {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = Runner::new(
            dom,
            mesocosm_views::succession_root as fn(&Succession) -> SuccessionChild,
            Succession::loss(0, 0, 0, None),
        );
        Self {
            runner,
            raster: Raster::new(chrome.device(), "checkpoint", WIDTH, HEIGHT),
            text: TextSystem::new(),
            style_set: StyleSet::cambium(&[mesocosm_views::succession_css()]),
            device: Device::screen(WIDTH as f32, HEIGHT as f32),
            generation: 0,
            shown: None,
        }
    }

    /// Takes the frame's question and rasterizes it if it changed. `None`
    /// clears the surface, which is what "play resumes" looks like.
    pub fn refresh(&mut self, chrome: &Chrome, checkpoint: Option<&Checkpoint>) {
        let reading = checkpoint.map(succession_of);
        if self.shown == reading {
            return;
        }
        if let Some(reading) = &reading {
            self.runner.update(|state| *state = reading.clone());
            self.shown = reading.clone().into();
            self.raster_panel(chrome);
        } else {
            self.shown = None;
        }
    }

    /// Whether there is anything to draw.
    pub fn standing(&self) -> bool {
        self.shown.is_some()
    }

    /// This lane's retained DOM, its box, and its sheet. See
    /// [`crate::vitals::VitalsChrome::probe`]. (DT4)
    pub fn probe(&self, frame: (u32, u32)) -> (cambium::DomHandle, [f32; 4], &'static str) {
        let (x, y, w, h) = Self::placement(frame);
        (
            self.runner.dom(),
            [x, y, w, h],
            mesocosm_views::succession_css(),
        )
    }

    fn raster_panel(&mut self, chrome: &Chrome) {
        self.generation = self.generation.saturating_add(1);
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let width = WIDTH as f32;
        let height = HEIGHT as f32;

        let styles = resolve_styles(
            &*dom,
            &self.style_set,
            &self.device,
            &InteractionStates::default(),
        );
        let Ok((styles, fragments)) = layout_with_text_system(
            &*dom,
            &styles,
            width,
            height,
            ViewportSizes::uniform(width, height),
            &mut self.text,
            &HashMap::new(),
        ) else {
            eprintln!("checkpoint: the panel did not lay out");
            return;
        };
        let list = self.emit(&dom, &styles, &fragments);
        let translated = paint_list_render::translate_paint_cmd_stream(
            list.viewport(),
            list.commands(),
            list.fonts(),
            list.images(),
        );
        chrome.raster(&self.raster, &translated.scene);
    }

    fn emit(
        &mut self,
        dom: &ScriptedDom,
        styles: &StylePlane<NodeId>,
        fragments: &LiveryLayout<NodeId>,
    ) -> genet_livery::LiveryPaintList {
        emit_paint_list_with_text_system_scrolled_with_images(
            dom,
            styles,
            fragments,
            DeviceIntSize::new(WIDTH as i32, HEIGHT as i32),
            self.generation,
            &mut self.text,
            &HashMap::new(),
            &HashMap::new(),
        )
    }

    /// Middle of the frame, a little above centre. The world behind it has
    /// stopped, so this is the thing being looked at rather than a corner
    /// reading glanced at.
    fn placement(frame: (u32, u32)) -> (f32, f32, f32, f32) {
        let x = (frame.0 as f32 - WIDTH as f32) / 2.0;
        let y = (frame.1 as f32 - HEIGHT as f32) / 2.0 - 40.0;
        (x.max(0.0), y.max(0.0), WIDTH as f32, HEIGHT as f32)
    }

    pub fn composite(
        &self,
        chrome: &Chrome,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        if !self.standing() {
            return;
        }
        chrome.draw(
            encoder,
            target,
            self.raster.sample_view(),
            Self::placement(frame),
            frame,
        );
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
        if !self.standing() {
            return;
        }
        chrome.draw_as(
            format,
            encoder,
            target,
            self.raster.sample_view(),
            Self::placement(frame),
            frame,
        );
    }
}

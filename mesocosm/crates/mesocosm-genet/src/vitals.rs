// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The cambium lane, hosted: the vitals panel over the traced section.
//!
//! Genet does **not** own the window here. The runner diffs
//! [`mesocosm_views::vitals_root`] into a `ScriptedDom`, Livery resolves the
//! sheet and lays the tree out, its paint list lowers through
//! `paint_list_render`, and [`crate::chrome`] rasterizes and blends it over the
//! frame exactly as it does the minimap. The full host inversion — genet
//! owning the window with the game view embedded as an external texture — is a
//! scope decision that stays Mark's, and is not needed for a panel.
//!
//! No text rendering lives here or anywhere in this repo. Words are Livery's.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cambium::GenetAppRunner;
use genet_livery::{
    Device, InteractionStates, LiveryLayout, StylePlane, StyleSet, TextSystem, ViewportSizes,
    emit_paint_list_with_text_system_scrolled_with_images, layout_with_text_system, resolve_styles,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use mesocosm_core::World;
use mesocosm_views::{Vitals, VitalsChild};
use paint_list_api::{DeviceIntSize, PaintList as _};

use crate::chrome::{Chrome, Raster};

/// The panel's box, in pixels. It is also the raster's size, so the tree is
/// laid out at the resolution it is shown at.
const WIDTH: u32 = 300;
/// Wider and taller since PE0, and wider **because** taller: the panel now
/// carries a replacement reading and, when the support path has run short, a
/// warning that states its evidence. Both wrap, so the box has to be sized for
/// the longest of them — and a hundred extra pixels of width buys back more
/// height than it costs, which keeps the ordinary panel from being mostly
/// empty box waiting for a warning that is usually not there.
/// Taller again since PE2: a body that has come to something carries three
/// more rows — what, by what route, and what it grants — plus the last
/// evidence, which wraps. The box is sized for the panel's fullest state
/// (a gland, a discovery and a live warning at once) rather than its usual
/// one, because a row that ran off the bottom would be worse than an empty
/// margin.
const HEIGHT: u32 = 320;

/// Distance from the frame's corner. Matches the minimap's, so the two chrome
/// surfaces sit on one margin.
const MARGIN: f32 = 12.0;

/// How many steps a notice stays up.
///
/// Presentation-timed, in the world's own ticks: at the host's canonical ten
/// ticks a second this is about two and a half seconds, and it stays put
/// across a replay because it counts what the world counted rather than wall
/// time. Nothing about it reaches the world, the trace, or the hash.
///
/// Retimed with the tempo (TD2: 150 steps at 60 t/s was the same 2.5s). A
/// constant counted in ticks is only a duration relative to a tick rate, so it
/// moves whenever the rate does.
const NOTICE_STEPS: u64 = 25;

type Runner = GenetAppRunner<Vitals, fn(&Vitals) -> VitalsChild, VitalsChild>;

pub struct VitalsChrome {
    runner: Runner,
    raster: Raster,
    /// Livery's retained text shaping. Reused across frames because building
    /// a font context per frame is what makes a panel expensive.
    text: TextSystem,
    style_set: StyleSet,
    device: Device,
    /// Advances with every relayout; the paint list carries it so a consumer
    /// can tell one frame's list from the next.
    generation: u64,
    /// The reading the raster currently holds. A frame whose vitals are
    /// unchanged pays a read and no raster.
    shown: Option<Vitals>,
    /// The notice on screen — a refusal, or what the body did with a meal —
    /// and the step it stops being shown at.
    notice: Option<(&'static str, u64)>,
    /// The most energy the played critter has held this session — the bar's
    /// denominator. The world has no capacity, so there is no other honest
    /// one; a new body resets it.
    high_water: u64,
}

impl VitalsChrome {
    pub fn new(chrome: &Chrome) -> Self {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = Runner::new(
            dom,
            mesocosm_views::vitals_root as fn(&Vitals) -> VitalsChild,
            Vitals::default(),
        );
        Self {
            runner,
            raster: Raster::new(chrome.device(), "vitals", WIDTH, HEIGHT),
            text: TextSystem::new(),
            style_set: StyleSet::cambium(&[mesocosm_views::vitals_css()]),
            device: Device::screen(WIDTH as f32, HEIGHT as f32),
            generation: 0,
            shown: None,
            notice: None,
            high_water: 0,
        }
    }

    /// Takes the frame's reading and rasterizes it if it changed.
    ///
    /// `outcomes` are the results of the steps this frame ran; a rejection or
    /// a landed meal among them starts (or restarts) the notice's window.
    pub fn refresh(
        &mut self,
        chrome: &Chrome,
        world: &World,
        outcomes: &[mesocosm_core::Outcome],
        steps: u64,
        trend: &mesocosm_core::Trend,
    ) {
        match world.energy_mg() {
            Some(energy) => self.high_water = self.high_water.max(energy),
            // Control lost: the bar's scale went with the body, and a new one
            // will set its own.
            None => self.high_water = 0,
        }
        if let Some(words) = mesocosm_views::notice_in(outcomes) {
            self.notice = Some((words, steps + NOTICE_STEPS));
        }
        if self.notice.is_some_and(|(_, until)| steps >= until) {
            self.notice = None;
        }

        let reading = mesocosm_views::vitals_of(
            world,
            self.high_water,
            self.notice.map(|(words, _)| words),
            Some(trend),
        );
        if self.shown.as_ref() == Some(&reading) {
            return;
        }
        self.runner.update(|vitals| *vitals = reading.clone());
        self.shown = Some(reading);
        self.raster_panel(chrome);
    }

    /// Style, lay out, paint, lower, rasterize. The whole cambium frame.
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
            // A panel that will not lay out is not worth taking the game down
            // for; the frame simply keeps the last raster.
            eprintln!("vitals: the panel did not lay out");
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

    /// Bottom left, clear of the minimap in the opposite corner.
    fn placement(frame: (u32, u32)) -> (f32, f32, f32, f32) {
        let y = frame.1 as f32 - HEIGHT as f32 - MARGIN;
        (MARGIN, y.max(0.0), WIDTH as f32, HEIGHT as f32)
    }

    /// This lane's retained DOM, where it sits in the window, and the sheet it
    /// lays out under — what a `genet_probe::ProbeSurface` is made of. (DT4)
    ///
    /// The driver searches and asserts against exactly the tree this lane draws
    /// from, so an `assert text` is a claim about what is on screen rather than
    /// about a second rendering of it. See [`crate::drive`].
    pub fn probe(&self, frame: (u32, u32)) -> (cambium::DomHandle, [f32; 4], &'static str) {
        let (x, y, w, h) = Self::placement(frame);
        (
            self.runner.dom(),
            [x, y, w, h],
            mesocosm_views::vitals_css(),
        )
    }

    pub fn composite(
        &self,
        chrome: &Chrome,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
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

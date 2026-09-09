// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The trait board's lane: the fourth chrome surface. (PE3b)
//!
//! Same arrangement as [`crate::vitals`] and [`crate::succession`] —
//! [`mesocosm_views::board_root`] diffed into a `ScriptedDom`, Livery styling
//! and laying it out, the paint list lowered and rasterized,
//! [`crate::chrome`] blending it over the frame.
//!
//! # Why a fourth lane rather than the checkpoint widened
//!
//! The checkpoint panel is **four facts, two answers, out**, and a
//! `mesocosm-views` test asserts in so many words that it never mentions a
//! program, a trait, a budget, an epoch, a revision or a founder. Widening it
//! to carry a candidate table would delete that claim and make the individual
//! checkpoint into the editor PE1's stop rule forbids it from becoming. The two
//! surfaces also change on different clocks — a checkpoint is fixed once shown,
//! a board re-reads after every commit — and want different boxes. So: two
//! lanes, and at a lineage checkpoint the board is the one that draws.
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
use mesocosm_runtime::Review;
use mesocosm_views::{Board, BoardChild};
use paint_list_api::{DeviceIntSize, PaintList as _};

use crate::chrome::{Chrome, Raster};

/// The panel's box, in pixels, and the raster's size with it.
///
/// Sized for the boundary this game actually reaches: the four headline facts,
/// a handful of noted readings, and a table of the status quo plus what one
/// played line has come to. Wider than the checkpoint because the rows carry
/// three figures each and wrapping a row of numbers makes a table unreadable.
const WIDTH: u32 = 620;
const HEIGHT: u32 = 420;

type Runner = GenetAppRunner<Board, fn(&Board) -> BoardChild, BoardChild>;

/// Reads the driver's review into the words a player sees.
///
/// The one conversion in the lane, and deliberately dumb: every sentence
/// belongs to `mesocosm-views`, and this supplies numbers and the cursor.
pub fn board_of(review: &Review, selected: usize) -> Board {
    let mut board = Board::of(
        review.epoch,
        review.lineage.0,
        review.budget_mg,
        review.current.map(|revision| revision.0),
        &review.trend,
    );
    board.readings = mesocosm_views::evidence_words(&review.readings, review.lineage.0);
    board.rows = review
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let sources: Vec<String> = row
                .sources
                .iter()
                .map(|proposed| match &proposed.refused {
                    Some(why) => format!("{}: {why}", proposed.source.name()),
                    None => proposed.source.name().to_owned(),
                })
                .collect();
            mesocosm_views::row_words(&row.offer, &sources, index == selected)
        })
        .collect();
    // The commit line appears only when the selected row is one the world would
    // actually admit — `Offer::takeable`, the same question `Review::commit`
    // answers, so the panel and the keyboard cannot disagree about what R does.
    board.commit = review
        .rows
        .get(selected)
        .and_then(|row| mesocosm_views::commit_words(&row.offer));
    board
}

pub struct BoardChrome {
    runner: Runner,
    raster: Raster,
    text: TextSystem,
    style_set: StyleSet,
    device: Device,
    generation: u64,
    /// The board the raster currently holds. A frame whose review and cursor
    /// are unchanged — which, since the world is stopped, is nearly all of
    /// them — pays a comparison and no raster.
    shown: Option<Board>,
}

impl BoardChrome {
    pub fn new(chrome: &Chrome) -> Self {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = Runner::new(
            dom,
            mesocosm_views::board_root as fn(&Board) -> BoardChild,
            Board::default(),
        );
        Self {
            runner,
            raster: Raster::new(chrome.device(), "board", WIDTH, HEIGHT),
            text: TextSystem::new(),
            style_set: StyleSet::cambium(&[mesocosm_views::board_css()]),
            device: Device::screen(WIDTH as f32, HEIGHT as f32),
            generation: 0,
            shown: None,
        }
    }

    /// Takes the frame's review and rasterizes it if it changed. `None` clears
    /// the surface, which is what "play resumes" looks like.
    pub fn refresh(&mut self, chrome: &Chrome, review: Option<&Review>, selected: usize) {
        let board = review.map(|review| board_of(review, selected));
        if self.shown == board {
            return;
        }
        if let Some(board) = &board {
            self.runner.update(|state| *state = board.clone());
            self.shown = board.clone().into();
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
        (self.runner.dom(), [x, y, w, h], mesocosm_views::board_css())
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
            eprintln!("board: the panel did not lay out");
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

    /// Middle of the frame. The world behind it has stopped and this is the
    /// thing being read, so it takes the centre rather than a corner.
    fn placement(frame: (u32, u32)) -> (f32, f32, f32, f32) {
        let x = (frame.0 as f32 - WIDTH as f32) / 2.0;
        let y = (frame.1 as f32 - HEIGHT as f32) / 2.0;
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

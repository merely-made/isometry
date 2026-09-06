// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The contextual graft menu chrome, rendered on the shared device.

use crate::chrome::{Chrome, Raster};
use cambium::GenetAppRunner;
use genet_livery::{
    Device, InteractionStates, StyleSet, TextSystem, ViewportSizes,
    emit_paint_list_with_text_system_scrolled_with_images, layout_with_text_system, resolve_styles,
};
use genet_scripted_dom::ScriptedDom;
use mesocosm_views::{GRAFT_HEIGHT, GRAFT_WIDTH, GraftChild, GraftMenu};
use paint_list_api::{DeviceIntSize, PaintList as _};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type Runner = GenetAppRunner<GraftMenu, fn(&GraftMenu) -> GraftChild, GraftChild>;

pub struct GraftChrome {
    runner: Runner,
    raster: Raster,
    text: TextSystem,
    style_set: StyleSet,
    device: Device,
    generation: u64,
    shown: Option<GraftMenu>,
}

impl GraftChrome {
    pub fn new(chrome: &Chrome) -> Self {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = Runner::new(
            dom,
            mesocosm_views::grafting_root as fn(&GraftMenu) -> GraftChild,
            GraftMenu::default(),
        );
        Self {
            runner,
            raster: Raster::new(chrome.device(), "grafting", GRAFT_WIDTH, GRAFT_HEIGHT),
            text: TextSystem::new(),
            style_set: StyleSet::cambium(&[mesocosm_views::grafting_css()]),
            device: Device::screen(GRAFT_WIDTH as f32, GRAFT_HEIGHT as f32),
            generation: 0,
            shown: None,
        }
    }

    pub fn update(&mut self, chrome: &Chrome, menu: Option<&GraftMenu>) {
        if self.shown.as_ref() == menu {
            return;
        }
        self.shown = menu.cloned();
        if let Some(menu) = menu {
            self.runner.update(|state| *state = menu.clone());
            self.raster_panel(chrome);
        }
    }

    pub fn standing(&self) -> bool {
        self.shown.is_some()
    }

    pub fn probe(&self, frame: (u32, u32)) -> (cambium::DomHandle, [f32; 4], &'static str) {
        (
            self.runner.dom(),
            Self::placement(frame),
            mesocosm_views::grafting_css(),
        )
    }

    fn raster_panel(&mut self, chrome: &Chrome) {
        self.generation = self.generation.saturating_add(1);
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let styles = resolve_styles(
            &*dom,
            &self.style_set,
            &self.device,
            &InteractionStates::default(),
        );
        let Ok((styles, fragments)) = layout_with_text_system(
            &*dom,
            &styles,
            GRAFT_WIDTH as f32,
            GRAFT_HEIGHT as f32,
            ViewportSizes::uniform(GRAFT_WIDTH as f32, GRAFT_HEIGHT as f32),
            &mut self.text,
            &HashMap::new(),
        ) else {
            return;
        };
        let list = emit_paint_list_with_text_system_scrolled_with_images(
            &*dom,
            &styles,
            &fragments,
            DeviceIntSize::new(GRAFT_WIDTH as i32, GRAFT_HEIGHT as i32),
            self.generation,
            &mut self.text,
            &HashMap::new(),
            &HashMap::new(),
        );
        let scene = paint_list_render::translate_paint_cmd_stream(
            list.viewport(),
            list.commands(),
            list.fonts(),
            list.images(),
        );
        chrome.raster(&self.raster, &scene.scene);
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
            Self::placement(frame).into(),
            frame,
        );
    }

    /// Keeps the menu on the right edge so the preview body remains visible
    /// to its left. The raster is fixed-size; clamping is the honest narrow
    /// window behavior until the host supplies a resized surface.
    fn placement(frame: (u32, u32)) -> [f32; 4] {
        let x = frame.0 as f32 - GRAFT_WIDTH as f32 - 12.0;
        let y = (frame.1 as f32 - GRAFT_HEIGHT as f32) / 2.0;
        [
            x.max(0.0),
            y.max(0.0),
            GRAFT_WIDTH as f32,
            GRAFT_HEIGHT as f32,
        ]
    }
}

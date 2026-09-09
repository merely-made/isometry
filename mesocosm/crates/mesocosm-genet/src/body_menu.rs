// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The shared body-menu chrome for grafting and expression.

use crate::chrome::{Chrome, Raster};
use cambium::GenetAppRunner;
use genet_livery::{
    Device, InteractionStates, StyleSet, TextSystem, ViewportSizes,
    emit_paint_list_with_text_system_scrolled_with_images, layout_with_text_system, resolve_styles,
};
use genet_scripted_dom::ScriptedDom;
use mesocosm_views::{BODY_MENU_HEIGHT, BODY_MENU_WIDTH, BodyMenu, BodyMenuChild};
use paint_list_api::{DeviceIntSize, PaintList as _};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type Runner = GenetAppRunner<BodyMenu, fn(&BodyMenu) -> BodyMenuChild, BodyMenuChild>;

pub struct BodyMenuChrome {
    runner: Runner,
    raster: Raster,
    text: TextSystem,
    style_set: StyleSet,
    device: Device,
    generation: u64,
    shown: Option<BodyMenu>,
}

impl BodyMenuChrome {
    pub fn new(chrome: &Chrome) -> Self {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = Runner::new(
            dom,
            mesocosm_views::body_menu_root as fn(&BodyMenu) -> BodyMenuChild,
            BodyMenu::default(),
        );
        Self {
            runner,
            raster: Raster::new(
                chrome.device(),
                "body-menu",
                BODY_MENU_WIDTH,
                BODY_MENU_HEIGHT,
            ),
            text: TextSystem::new(),
            style_set: StyleSet::cambium(&[mesocosm_views::body_menu_css()]),
            device: Device::screen(BODY_MENU_WIDTH as f32, BODY_MENU_HEIGHT as f32),
            generation: 0,
            shown: None,
        }
    }

    pub fn update(&mut self, chrome: &Chrome, menu: Option<&BodyMenu>) {
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
            mesocosm_views::body_menu_css(),
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
            BODY_MENU_WIDTH as f32,
            BODY_MENU_HEIGHT as f32,
            ViewportSizes::uniform(BODY_MENU_WIDTH as f32, BODY_MENU_HEIGHT as f32),
            &mut self.text,
            &HashMap::new(),
        ) else {
            return;
        };
        let list = emit_paint_list_with_text_system_scrolled_with_images(
            &*dom,
            &styles,
            &fragments,
            DeviceIntSize::new(BODY_MENU_WIDTH as i32, BODY_MENU_HEIGHT as i32),
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
        let x = frame.0 as f32 - BODY_MENU_WIDTH as f32 - 12.0;
        let y = (frame.1 as f32 - BODY_MENU_HEIGHT as f32) / 2.0;
        [
            x.max(0.0),
            y.max(0.0),
            BODY_MENU_WIDTH as f32,
            BODY_MENU_HEIGHT as f32,
        ]
    }
}

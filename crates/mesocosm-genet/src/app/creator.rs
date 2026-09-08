// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Disposable local starting-life selection. Only confirmation replaces the
//! untouched startup runtime; generated previews never step a played world.

use super::{Host, grafting};
use mesocosm_core::{
    Kingdom, PartPalette, World, state_hash,
    world::generation::{Prepared, Request, Selection},
};
use mesocosm_views::BodyMenu;
use winit::keyboard::{Key, NamedKey};

mod reading;
mod worker;

pub(super) struct Creator {
    original_camera: crate::section::CameraMode,
    pub request: Request,
    pub selected: usize,
    pub prepared: Option<Prepared>,
    preview: Option<Box<World>>,
    empty: World,
    pub reading: BodyMenu,
    pub pending: bool,
    serial: u64,
    worker: worker::Worker,
    pub notice: String,
}

impl Creator {
    pub fn new(
        request: Request,
        palette: PartPalette,
        original: &World,
        original_camera: crate::section::CameraMode,
    ) -> Self {
        let mut empty = original.clone();
        empty.organisms.clear();
        let mut result = Self {
            original_camera,
            request,
            selected: 0,
            prepared: None,
            preview: None,
            empty,
            reading: BodyMenu::default(),
            pending: false,
            serial: 0,
            worker: worker::Worker::new(palette),
            notice: String::new(),
        };
        result.regenerate();
        result
    }

    pub fn world(&self) -> &World {
        self.preview.as_deref().unwrap_or(&self.empty)
    }

    fn regenerate(&mut self) {
        self.serial += 1;
        self.prepared = None;
        self.preview = None;
        self.selected = 0;
        self.notice.clear();
        self.pending = self.worker.send(self.serial, self.request.clone());
        if !self.pending {
            self.notice = "Generation worker stopped. Cancel and reopen the creator.".into();
        }
        self.refresh();
    }

    pub fn poll(&mut self) {
        loop {
            let (serial, result) = match self.worker.results.try_recv() {
                Ok(message) => message,
                Err(std::sync::mpsc::TryRecvError::Disconnected) if self.pending => {
                    self.pending = false;
                    self.notice = "Generation stopped. Cancel and reopen the creator.".into();
                    self.refresh();
                    break;
                },
                Err(_) => break,
            };
            if serial != self.serial {
                continue;
            }
            self.pending = false;
            match result {
                Ok(prepared) => {
                    self.prepared = Some(prepared);
                    self.select(0);
                },
                Err(error) => self.notice = format!("Request refused: {error:?}"),
            }
            self.refresh();
        }
    }

    fn select(&mut self, index: usize) {
        self.selected = index;
        self.preview = self
            .prepared
            .as_ref()
            .and_then(|p| p.enter(index).ok())
            .map(Box::new);
        self.refresh();
    }

    fn count(&self) -> usize {
        self.prepared
            .as_ref()
            .map_or(0, |p| p.draft().candidates.len())
    }

    fn move_selection(&mut self, backwards: bool) {
        let count = self.count();
        if count > 0 {
            self.select((self.selected + if backwards { count - 1 } else { 1 }) % count);
        }
    }
}

impl Host {
    pub(super) fn poll_creator(&mut self) {
        if let Some(creator) = &mut self.creator {
            creator.poll();
        }
    }

    pub(super) fn try_creator_key(&mut self, key: &Key) -> bool {
        let Some(creator) = &mut self.creator else {
            return false;
        };
        let letter = match key {
            Key::Character(c) => c.to_lowercase(),
            _ => String::new(),
        };
        match key {
            Key::Named(NamedKey::Escape) => {
                self.config.camera = creator.original_camera;
                if let Some(gpu) = &mut self.gpu {
                    gpu.section.set_mode(self.config.camera);
                }
                self.creator = None;
                self.last = None;
                self.events.push("creator-cancelled".into());
                return true;
            },
            Key::Named(NamedKey::Enter) => {
                self.confirm_creator();
                return true;
            },
            Key::Named(NamedKey::ArrowUp | NamedKey::ArrowLeft) => {
                creator.move_selection(true);
                return true;
            },
            Key::Named(NamedKey::ArrowDown | NamedKey::ArrowRight) => {
                creator.move_selection(false);
                return true;
            },
            _ => {},
        }
        match letter.as_str() {
            "j" => {
                creator.move_selection(true);
                return true;
            },
            "l" => {
                creator.move_selection(false);
                return true;
            },
            "z" | "v" => {
                self.try_camera_key(key);
                return true;
            },
            "r" => creator.request.variation = creator.request.variation.wrapping_add(1),
            "n" => creator.request.seed = creator.request.seed.wrapping_add(1),
            "p" => creator.request.place = (creator.request.place % 9 + 1) % 9,
            "c" => {
                creator.request.criteria.role = match creator.request.criteria.role {
                    None => Some(Kingdom::Producer),
                    Some(Kingdom::Producer) => Some(Kingdom::Consumer),
                    Some(Kingdom::Consumer) => Some(Kingdom::Decomposer),
                    Some(Kingdom::Decomposer) => None,
                }
            },
            "m" => {
                creator.request.criteria.movement_organs =
                    match creator.request.criteria.movement_organs {
                        None => Some(true),
                        Some(true) => Some(false),
                        Some(false) => None,
                    }
            },
            "+" | "=" => {
                creator.request.criteria.mass_mg = creator
                    .request
                    .criteria
                    .mass_mg
                    .saturating_add(100)
                    .min(10_000)
            },
            "-" => {
                creator.request.criteria.mass_mg =
                    creator.request.criteria.mass_mg.saturating_sub(100).max(64)
            },
            _ => return true,
        }
        creator.regenerate();
        true
    }

    fn confirm_creator(&mut self) {
        let Some(creator) = &self.creator else {
            return;
        };
        if creator.pending || creator.preview.is_none() {
            return;
        }
        let selection = Selection {
            request: creator.request.clone(),
            candidate: creator.selected,
        };
        let expected = state_hash(creator.world());
        let original_camera = creator.original_camera;
        let palette = self
            .content
            .as_ref()
            .expect("creator admits generated content")
            .palette;
        let entered = mesocosm_runtime::Runtime::generated_start(
            &selection,
            palette,
            self.config.ticks_per_second,
        );
        let runtime = match entered {
            Ok(runtime) if runtime.state_hash() == expected => runtime,
            other => {
                let creator = self.creator.as_mut().unwrap();
                creator.notice = match other {
                    Err(why) => format!("Start refused: {why:?}"),
                    Ok(_) => "The preview changed. Vary bodies to review it again.".into(),
                };
                creator.refresh();
                return;
            },
        };
        // A fresh world can have revision zero just like the previous world.
        // Rebind the section on the same device rather than reuse ground and
        // body caches whose revisions belong to the parked startup world.
        if let Some(gpu) = &mut self.gpu {
            match crate::section::Section::new(
                gpu.device.clone(),
                gpu.queue.clone(),
                gpu.config.width,
                gpu.config.height,
                gpu.config.format,
                runtime.world().ground(),
                crate::section::Framing::new(self.config.slab_half_height, original_camera),
            ) {
                Ok(section) => gpu.section = section,
                Err(why) => {
                    let creator = self.creator.as_mut().unwrap();
                    creator.notice = format!("Could not display the new world: {why}");
                    creator.refresh();
                    return;
                },
            }
        }
        self.runtime = super::content::with_authored(runtime);
        self.config.camera = original_camera;
        if let Some(lanes) = self.gpu.as_mut().and_then(|gpu| gpu.chrome.as_mut()) {
            lanes.hud.invalidate();
        }
        self.config.seed = selection.request.seed;
        self.config.organisms = selection.request.organisms;
        self.config.start = Some(selection);
        self.creator = None;
        self.grafting = grafting::Grafting::default();
        self.last = None;
        self.pan = Default::default();
        self.follow = None;
        self.events.push("creator-entered".into());
    }
}

#[cfg(test)]
mod tests;

// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use paredros_world::SubjectSheet;

pub struct LifeSheet {
    pub label: &'static str,
    pub summary: &'static str,
    pub sheet: SubjectSheet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Part,
    Action,
}

pub struct SheetView {
    pub life: usize,
    pub part: usize,
    pub action: usize,
    pub focus: Focus,
    pub scroll: f32,
    pub detail_scroll: usize,
}

impl Default for SheetView {
    fn default() -> Self {
        Self {
            life: 0,
            part: 0,
            action: 0,
            focus: Focus::Part,
            scroll: 0.,
            detail_scroll: 0,
        }
    }
}

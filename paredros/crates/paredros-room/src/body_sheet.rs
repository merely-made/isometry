// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Read-only native presentation for the authored subject-sheet scenario.
//!
//! This owns view-local selection and scrolling only. The `SubjectSheet`
//! projection remains caller supplied, and none of the controls are game verbs.

use netrender::Scene;
mod text;
use text::Text;
mod details;
#[cfg(test)]
mod tests;
mod view;
pub use view::{Focus, LifeSheet, SheetView};

pub const LOGICAL_SIZE: [u32; 2] = [1280, 720];
const ROW_HEIGHT: f32 = 30.;
const PART_X: f32 = 26.;
const ACTION_X: f32 = 370.;
const DETAIL_X: f32 = 720.;
const LIST_TOP: f32 = 208.;
const LIST_BOTTOM: f32 = 649.;

impl SheetView {
    pub fn select_life(&mut self, lives: &[LifeSheet], life: usize) {
        self.life = life.min(lives.len().saturating_sub(1));
        self.part = 0;
        self.action = 0;
        self.focus = Focus::Part;
        self.scroll = 0.;
        self.detail_scroll = 0;
        self.clamp(lives);
    }

    pub fn key(&mut self, lives: &[LifeSheet], key: SheetKey) {
        if lives.is_empty() {
            return;
        }
        match key {
            SheetKey::PreviousLife => self.select_life(lives, self.life.saturating_sub(1)),
            SheetKey::NextLife => self.select_life(lives, (self.life + 1).min(lives.len() - 1)),
            SheetKey::SwitchFocus => {
                self.focus = match self.focus {
                    Focus::Part => Focus::Action,
                    Focus::Action => Focus::Part,
                };
                self.detail_scroll = 0;
            },
            SheetKey::Up => match self.focus {
                Focus::Part => self.part = self.part.saturating_sub(1),
                Focus::Action => self.action = self.action.saturating_sub(1),
            },
            SheetKey::Down => match self.focus {
                Focus::Part => self.part += 1,
                Focus::Action => self.action += 1,
            },
        }
        if matches!(key, SheetKey::Up | SheetKey::Down) {
            self.detail_scroll = 0;
        }
        self.clamp(lives);
        self.reveal_selection();
    }

    pub fn click(&mut self, lives: &[LifeSheet], point: [f32; 2]) {
        if lives.is_empty() {
            return;
        }
        if (24. ..330.).contains(&point[0]) && (80. ..122.).contains(&point[1]) {
            self.select_life(lives, ((point[0] - 24.) / 102.).floor() as usize);
            return;
        }
        if !(LIST_TOP..LIST_BOTTOM).contains(&point[1]) {
            return;
        }
        let row = ((point[1] - LIST_TOP + self.scroll) / ROW_HEIGHT).floor();
        if row < 0. {
            return;
        }
        if (PART_X..ACTION_X - 12.).contains(&point[0])
            && (row as usize) < lives[self.life].sheet.parts.len()
        {
            self.focus = Focus::Part;
            self.part = row as usize;
            self.detail_scroll = 0;
        } else if (ACTION_X..DETAIL_X - 12.).contains(&point[0])
            && (row as usize) < lives[self.life].sheet.actions.len()
        {
            self.focus = Focus::Action;
            self.action = row as usize;
            self.detail_scroll = 0;
        }
        self.clamp(lives);
    }

    pub fn scroll_by(&mut self, lives: &[LifeSheet], delta: f32) {
        self.scroll = (self.scroll + delta).max(0.);
        self.clamp(lives);
    }

    pub fn scroll_detail_by(&mut self, lives: &[LifeSheet], delta: i32) {
        let Some(life) = lives.get(self.life) else {
            return;
        };
        let maximum = details::detail_lines(life, self).len().saturating_sub(18);
        self.detail_scroll = self
            .detail_scroll
            .saturating_add_signed(delta as isize)
            .min(maximum);
    }

    fn clamp(&mut self, lives: &[LifeSheet]) {
        let Some(life) = lives.get(self.life) else {
            return;
        };
        self.part = self.part.min(life.sheet.parts.len().saturating_sub(1));
        self.action = self.action.min(life.sheet.actions.len().saturating_sub(1));
        let rows = life.sheet.parts.len().max(life.sheet.actions.len()) as f32;
        self.scroll = self
            .scroll
            .min((rows * ROW_HEIGHT - (LIST_BOTTOM - LIST_TOP)).max(0.));
    }

    fn reveal_selection(&mut self) {
        let selected = match self.focus {
            Focus::Part => self.part,
            Focus::Action => self.action,
        } as f32
            * ROW_HEIGHT;
        let viewport = LIST_BOTTOM - LIST_TOP - ROW_HEIGHT;
        if selected < self.scroll {
            self.scroll = selected;
        } else if selected > self.scroll + viewport {
            self.scroll = selected - viewport;
        }
    }
}

#[derive(Clone, Copy)]
pub enum SheetKey {
    PreviousLife,
    NextLife,
    SwitchFocus,
    Up,
    Down,
}

pub struct Hud {
    text: Text,
}

impl Hud {
    pub fn new() -> Result<Self, String> {
        Ok(Self { text: Text::new()? })
    }

    pub fn scene(&mut self, lives: &[LifeSheet], view: &SheetView) -> Scene {
        let mut scene = Scene::new(LOGICAL_SIZE[0], LOGICAL_SIZE[1]);
        scene.push_rect(0., 0., 1280., 720., [0.035, 0.05, 0.065, 1.]);
        let Some(life) = lives.get(view.life) else {
            return scene;
        };
        self.panel(&mut scene, 14., 12., 1266., 154.);
        self.panel(&mut scene, 14., 162., 350., 678.);
        self.panel(&mut scene, 358., 162., 700., 678.);
        self.panel(&mut scene, 708., 162., 1266., 678.);
        let ink = [0.91, 0.95, 0.96, 1.];
        self.text.label(
            &mut scene,
            "PAREDROS / BODY + ACTIONS / authored scenario inspection",
            [27., 22.],
            23.,
            ink,
            1200.,
        );
        self.text.label(
            &mut scene,
            "Read-only comparison. It does not select a player body or change any fact.",
            [27., 51.],
            17.,
            [0.72, 0.80, 0.84, 1.],
            1200.,
        );
        for (index, item) in lives.iter().enumerate() {
            let x = 24. + index as f32 * 102.;
            let selected = index == view.life;
            scene.push_rect(
                x,
                83.,
                x + 94.,
                120.,
                if selected {
                    [0.18, 0.42, 0.48, 1.]
                } else {
                    [0.10, 0.16, 0.20, 1.]
                },
            );
            self.text
                .label(&mut scene, item.label, [x + 8., 91.], 15., ink, 86.);
        }
        self.text.label(
            &mut scene,
            &format!(
                "{}  |  subject {}  |  body revision {}",
                life.label, life.sheet.subject.0, life.sheet.revision.0
            ),
            [342., 89.],
            18.,
            ink,
            900.,
        );
        self.text.label(
            &mut scene,
            life.summary,
            [342., 116.],
            16.,
            [0.95, 0.78, 0.42, 1.],
            880.,
        );
        self.text
            .label(&mut scene, "PARTS", [PART_X, 173.], 18., ink, 300.);
        self.text.label(
            &mut scene,
            "ACTION BINDINGS",
            [ACTION_X, 173.],
            18.,
            ink,
            300.,
        );
        self.text
            .label(&mut scene, "INSPECTION", [DETAIL_X, 173.], 18., ink, 500.);
        for (index, part) in life.sheet.parts.iter().enumerate() {
            let y = LIST_TOP + index as f32 * ROW_HEIGHT - view.scroll;
            if !(LIST_TOP..LIST_BOTTOM - ROW_HEIGHT).contains(&y) {
                continue;
            }
            let selected = view.focus == Focus::Part && index == view.part;
            let sourced = view.focus == Focus::Action
                && life.sheet.actions.get(view.action).is_some_and(|action| {
                    action
                        .sources
                        .iter()
                        .any(|source| *source == paredros_world::SourceQuery::Part(part.id))
                });
            if selected || sourced {
                scene.push_rect(
                    PART_X - 5.,
                    y - 3.,
                    ACTION_X - 17.,
                    y + 23.,
                    if selected {
                        [0.16, 0.38, 0.42, 1.]
                    } else {
                        [0.12, 0.24, 0.28, 1.]
                    },
                );
            }
            let state = if part.severed { "SEVERED" } else { "living" };
            self.text.label(
                &mut scene,
                &format!("{}  {}", part.name, state),
                [PART_X, y],
                16.,
                if part.severed {
                    [1., 0.56, 0.48, 1.]
                } else {
                    ink
                },
                300.,
            );
        }
        for (index, action) in life.sheet.actions.iter().enumerate() {
            let y = LIST_TOP + index as f32 * ROW_HEIGHT - view.scroll;
            if !(LIST_TOP..LIST_BOTTOM - ROW_HEIGHT).contains(&y) {
                continue;
            }
            let selected = view.focus == Focus::Action && index == view.action;
            if selected {
                scene.push_rect(
                    ACTION_X - 5.,
                    y - 3.,
                    DETAIL_X - 17.,
                    y + 23.,
                    [0.16, 0.38, 0.42, 1.],
                );
            }
            let state = if action.available { "ready" } else { "blocked" };
            let related = view.focus == Focus::Part
                && life.sheet.parts.get(view.part).is_some_and(|part| {
                    action
                        .sources
                        .iter()
                        .any(|source| *source == paredros_world::SourceQuery::Part(part.id))
                });
            if related && !selected {
                scene.push_rect(
                    ACTION_X - 5.,
                    y - 3.,
                    DETAIL_X - 17.,
                    y + 23.,
                    [0.12, 0.24, 0.28, 1.],
                );
            }
            self.text.label(
                &mut scene,
                &format!(
                    "{}  {}{}",
                    binding_name(action.kind),
                    state,
                    if related { "  uses part" } else { "" }
                ),
                [ACTION_X, y],
                16.,
                if action.available {
                    ink
                } else {
                    [1., 0.64, 0.43, 1.]
                },
                320.,
            );
        }
        self.details(&mut scene, life, view, ink);
        self.text.label(&mut scene, "1/2/3 compare lives  |  Tab change list  |  arrows select  |  wheel list/detail scroll  |  PgUp/PgDn inspect  |  Esc quit", [27., 690.], 16., ink, 1200.);
        scene
    }

    fn panel(&self, scene: &mut Scene, x: f32, y: f32, xx: f32, yy: f32) {
        scene.push_rect(x, y, xx, yy, [0.025, 0.04, 0.055, 0.96]);
    }

    fn details(&mut self, scene: &mut Scene, life: &LifeSheet, view: &SheetView, ink: [f32; 4]) {
        let lines = details::detail_lines(life, view);
        let end = (view.detail_scroll + 18).min(lines.len());
        self.text.label(
            scene,
            &format!(
                "lines {}-{} of {}  PgUp/PgDn",
                view.detail_scroll + 1,
                end,
                lines.len()
            ),
            [DETAIL_X, 657.],
            13.,
            [0.7, 0.78, 0.8, 1.],
            500.,
        );
        for (index, line) in lines.iter().skip(view.detail_scroll).take(18).enumerate() {
            self.text.label(
                scene,
                line,
                [DETAIL_X, 205. + index as f32 * 25.],
                16.,
                ink,
                500.,
            );
        }
    }
}

fn binding_name(kind: paredros_world::BindingKind) -> &'static str {
    match kind {
        paredros_world::BindingKind::GripLine => "grip + line",
        paredros_world::BindingKind::Adhesion => "adhesion",
        paredros_world::BindingKind::HarnessLine => "harness + line",
    }
}

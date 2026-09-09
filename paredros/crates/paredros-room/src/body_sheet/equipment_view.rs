// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use netrender::Scene;
use paredros_world::{Item, ItemLocation, SubjectSheet};

use super::{Hud, LOGICAL_SIZE, LifeSheet, SheetView, Text, schematic};

const LEFT: f32 = 22.;
const TOP: f32 = 202.;
const RIGHT: f32 = 342.;
const BOTTOM: f32 = 650.;
const ITEM_LEFT: f32 = 370.;
const ITEM_RIGHT: f32 = 700.;
const DETAIL_LEFT: f32 = 720.;
const ROW: f32 = 30.;
const LIST_TOP: f32 = 208.;
const ATTACH_BUTTON: [f32; 4] = [720., 500., 900., 538.];
const DETACH_BUTTON: [f32; 4] = [910., 500., 1090., 538.];
const SAVE_BUTTON: [f32; 4] = [720., 552., 900., 590.];
const LOAD_BUTTON: [f32; 4] = [910., 552., 1090., 590.];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquipmentView {
    pub part: usize,
    pub item: usize,
    pub parts_view: bool,
}

impl Default for EquipmentView {
    fn default() -> Self {
        Self {
            part: 0,
            item: 0,
            parts_view: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipmentCommand {
    Attach,
    Detach,
    Save,
    Load,
}

impl EquipmentView {
    pub fn click(
        &mut self,
        sheet: &SubjectSheet,
        items: &[Item],
        point: [f32; 2],
    ) -> Option<EquipmentCommand> {
        if inside(ATTACH_BUTTON, point) {
            return Some(EquipmentCommand::Attach);
        }
        if inside(DETACH_BUTTON, point) {
            return Some(EquipmentCommand::Detach);
        }
        if inside(SAVE_BUTTON, point) {
            return Some(EquipmentCommand::Save);
        }
        if inside(LOAD_BUTTON, point) {
            return Some(EquipmentCommand::Load);
        }
        if self.parts_view && (LEFT..RIGHT).contains(&point[0]) && (TOP..BOTTOM).contains(&point[1])
        {
            if let Some(index) = schematic::hit(&sheet.parts, point) {
                if sheet.parts.get(index).is_some() {
                    self.part = index;
                }
            }
            return None;
        }
        if !(LIST_TOP..BOTTOM - ROW).contains(&point[1]) {
            return None;
        }
        if (LEFT..RIGHT).contains(&point[0]) {
            let index = ((point[1] - LIST_TOP) / ROW).floor() as usize;
            if sheet.parts.get(index).is_some() {
                self.part = index;
            }
        } else if (ITEM_LEFT..ITEM_RIGHT).contains(&point[0]) {
            let index = ((point[1] - LIST_TOP) / ROW).floor() as usize;
            if items.get(index).is_some() {
                self.item = index;
            }
        }
        None
    }

    pub fn step_part(&mut self, sheet: &SubjectSheet, delta: i32) {
        if sheet.parts.is_empty() {
            self.part = 0;
            return;
        }
        let index = self.part.min(sheet.parts.len() - 1);
        let next = index.saturating_add_signed(delta as isize);
        self.part = sheet.parts.get(next).map_or(index, |_| next);
    }

    pub fn step_item(&mut self, items: &[Item], delta: i32) {
        let rows = items;
        if rows.is_empty() {
            self.item = 0;
            return;
        }
        let index = self.item.min(rows.len() - 1);
        let next = index.saturating_add_signed(delta as isize);
        self.item = rows.get(next).map_or(index, |_| next);
    }
}

fn inside(rect: [f32; 4], point: [f32; 2]) -> bool {
    (rect[0]..rect[2]).contains(&point[0]) && (rect[1]..rect[3]).contains(&point[1])
}

impl Hud {
    pub fn equipment_scene(
        &mut self,
        sheet: &SubjectSheet,
        items: &[Item],
        view: &EquipmentView,
        status: &str,
    ) -> Scene {
        let mut scene = Scene::new(LOGICAL_SIZE[0], LOGICAL_SIZE[1]);
        scene.push_rect(0., 0., 1280., 720., [0.035, 0.05, 0.065, 1.]);
        self.panel(&mut scene, 14., 12., 1266., 154.);
        self.panel(&mut scene, 14., 162., 350., 678.);
        self.panel(&mut scene, 358., 162., 708., 678.);
        self.panel(&mut scene, 702., 162., 1266., 678.);
        let ink = [0.91, 0.95, 0.96, 1.];
        self.text.label(
            &mut scene,
            "PAREDROS / LIVE EQUIPMENT / Keeper",
            [27., 22.],
            23.,
            ink,
            1200.,
        );
        self.text.label(&mut scene, "Generated-world session with authored anatomy. C switches authored read-only comparison.", [27., 51.], 17., [0.72, 0.80, 0.84, 1.], 1200.);
        self.text.label(
            &mut scene,
            "Up/Down parts  |  Left/Right items  |  L diagram/list  |  C compare  |  Esc exit",
            [760., 51.],
            15.,
            [0.72, 0.80, 0.84, 1.],
            490.,
        );
        self.text.label(
            &mut scene,
            if view.parts_view {
                "BODY / DIAGRAM"
            } else {
                "BODY / LIST"
            },
            [LEFT, 178.],
            17.,
            ink,
            320.,
        );
        self.text.label(
            &mut scene,
            "DRESSING ITEMS",
            [ITEM_LEFT, 178.],
            17.,
            ink,
            320.,
        );
        self.text.label(
            &mut scene,
            "SELECTION + BINDING",
            [DETAIL_LEFT, 178.],
            17.,
            ink,
            500.,
        );

        let selected_index =
            (!sheet.parts.is_empty()).then(|| view.part.min(sheet.parts.len() - 1));
        let life = LifeSheet {
            label: "Keeper",
            summary: "Authored anatomy",
            sheet: sheet.clone(),
        };
        let temporary = SheetView {
            part: selected_index.unwrap_or(0),
            parts_view: true,
            ..SheetView::default()
        };
        if view.parts_view {
            schematic::draw(&mut scene, &life, &temporary, &mut self.text, ink);
        } else {
            for (index, part) in sheet.parts.iter().enumerate() {
                let y = LIST_TOP + index as f32 * ROW;
                if y >= BOTTOM - ROW {
                    break;
                }
                if index == view.part {
                    scene.push_rect(
                        LEFT - 5.,
                        y - 3.,
                        RIGHT - 5.,
                        y + 23.,
                        [0.16, 0.38, 0.42, 1.],
                    );
                }
                self.text.label(
                    &mut scene,
                    &format!("#{}  {}", part.id.0, part.name),
                    [LEFT, y],
                    16.,
                    ink,
                    300.,
                );
            }
        }
        for (index, item) in items.iter().enumerate() {
            let y = LIST_TOP + index as f32 * ROW;
            if y >= BOTTOM - ROW {
                break;
            }
            if index == view.item {
                scene.push_rect(
                    ITEM_LEFT - 5.,
                    y - 3.,
                    ITEM_RIGHT - 5.,
                    y + 23.,
                    [0.16, 0.38, 0.42, 1.],
                );
            }
            let location = match item.location {
                ItemLocation::Attached { part, .. } => format!("attached #{}", part.0),
                ItemLocation::Carried(_) => "carried".into(),
                _ => "not carried".into(),
            };
            self.text.label(
                &mut scene,
                &format!("item {}  {}", item.id.0, location),
                [ITEM_LEFT, y],
                16.,
                ink,
                300.,
            );
        }
        let part = sheet.parts.get(view.part);
        let item = items.get(view.item);
        self.text.label(
            &mut scene,
            &format!(
                "Part: {}",
                part.map_or("none".into(), |part| format!(
                    "#{} {}",
                    part.id.0, part.name
                ))
            ),
            [DETAIL_LEFT, 215.],
            17.,
            ink,
            500.,
        );
        let item_detail = item.map_or_else(
            || "none".into(),
            |item| match item.location {
                ItemLocation::Attached { part, .. } => {
                    format!(
                        "#{} {:?}, attached to part #{}",
                        item.id.0, item.kind, part.0
                    )
                },
                _ => format!("#{} {:?}", item.id.0, item.kind),
            },
        );
        self.text.label(
            &mut scene,
            &format!("Item: {item_detail}"),
            [DETAIL_LEFT, 250.],
            17.,
            ink,
            500.,
        );
        let attachment_hint = if sheet.global_blockers.is_empty() {
            "Attach a dressing to the selected living part."
        } else {
            "New attachments unavailable; see body state below."
        };
        self.text.label(
            &mut scene,
            attachment_hint,
            [DETAIL_LEFT, 300.],
            15.,
            [0.72, 0.80, 0.84, 1.],
            500.,
        );
        for (index, line) in blocker_lines(&sheet.global_blockers).iter().enumerate() {
            self.text.label(
                &mut scene,
                line,
                [DETAIL_LEFT, 326. + index as f32 * 18.],
                14.,
                [0.95, 0.78, 0.42, 1.],
                500.,
            );
        }
        button(&mut scene, &mut self.text, ATTACH_BUTTON, "ATTACH [E]", ink);
        button(&mut scene, &mut self.text, DETACH_BUTTON, "DETACH [D]", ink);
        button(&mut scene, &mut self.text, SAVE_BUTTON, "SAVE [F5]", ink);
        button(&mut scene, &mut self.text, LOAD_BUTTON, "LOAD [F9]", ink);
        self.text.label(
            &mut scene,
            "Load restores the latest saved equipment session; changes since save are lost.",
            [DETAIL_LEFT, 610.],
            14.,
            [0.72, 0.80, 0.84, 1.],
            500.,
        );
        self.text.label(
            &mut scene,
            status,
            [27., 94.],
            16.,
            [0.95, 0.78, 0.42, 1.],
            1200.,
        );
        scene
    }
}

/// Keep the small, state-owned blocker set readable without giving the view
/// authority to decide which condition matters.
fn blocker_lines(blockers: &[String]) -> Vec<String> {
    const WIDTH: usize = 58;
    let mut lines = Vec::new();
    for blocker in blockers {
        let mut line = String::new();
        for word in blocker.split_whitespace() {
            if !line.is_empty() && line.len() + word.len() + 1 > WIDTH {
                lines.push(line);
                line = String::new();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}

fn button(scene: &mut Scene, text: &mut Text, rect: [f32; 4], label: &str, ink: [f32; 4]) {
    scene.push_rect(rect[0], rect[1], rect[2], rect[3], [0.10, 0.25, 0.30, 1.]);
    text.label(
        scene,
        label,
        [rect[0] + 10., rect[1] + 11.],
        15.,
        ink,
        rect[2] - rect[0] - 20.,
    );
}

#[cfg(test)]
mod persistence_tests {
    use super::*;

    fn sheet() -> SubjectSheet {
        SubjectSheet {
            subject: paredros_identity::SubjectId(1),
            revision: paredros_identity::BodyRevisionId(0),
            learned: Vec::new(),
            parts: Vec::new(),
            actions: Vec::new(),
            global_blockers: Vec::new(),
            resources: Vec::new(),
        }
    }

    #[test]
    fn save_and_load_buttons_have_bounded_hits() {
        let mut view = EquipmentView::default();
        let empty: [Item; 0] = [];
        assert_eq!(
            view.click(&sheet(), &empty, [800., 570.]),
            Some(EquipmentCommand::Save)
        );
        assert_eq!(
            view.click(&sheet(), &empty, [1000., 570.]),
            Some(EquipmentCommand::Load)
        );
        assert_eq!(view.click(&sheet(), &empty, [800., 600.]), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body_sheet::EquipmentSession;

    #[test]
    fn clicks_and_steps_select_rows_not_item_ids() {
        let session = EquipmentSession::new().unwrap();
        let mut items = session.items();
        items[0].id = paredros_world::ItemId(42);
        items[1].id = paredros_world::ItemId(99);
        let sheet = session.sheet();
        let mut view = EquipmentView::default();
        assert!(view.parts_view);
        view.click(&sheet, &items, [400., LIST_TOP + ROW + 2.]);
        assert_eq!(view.item, 1);
        assert_eq!(items[view.item].id.0, 99);
        view.step_item(&items, -1);
        assert_eq!(view.item, 0);
        view.parts_view = false;
        view.click(&sheet, &items, [40., LIST_TOP + ROW * 2. + 2.]);
        assert_eq!(view.part, 2);
        let before = view.clone();
        view.click(&sheet, &items, [400., 700.]);
        view.click(&sheet, &items, [40., 700.]);
        assert_eq!(view, before);
    }

    #[test]
    fn button_clicks_dispatch_through_the_same_session_and_replay() {
        let mut session = EquipmentSession::new().unwrap();
        let mut view = EquipmentView::default();
        view.part = 1;
        let sheet = session.sheet();
        let items = session.items();
        assert_eq!(
            view.click(&sheet, &items, [730., 510.]),
            Some(EquipmentCommand::Attach)
        );
        session.attach(items[view.item].id, sheet.parts[view.part].id);
        assert_eq!(session.game().attachments(session.subject()).len(), 1);
        assert_eq!(
            view.click(&sheet, &items, [920., 510.]),
            Some(EquipmentCommand::Detach)
        );
        session.detach(items[view.item].id);
        assert!(session.game().attachments(session.subject()).is_empty());
        assert_eq!(view.click(&sheet, &items, [900., 539.]), None);
        assert_eq!(
            paredros_world::GameState::restore(&session.game().save().unwrap()).unwrap(),
            *session.game()
        );
    }

    #[test]
    fn empty_selection_and_large_steps_are_safe() {
        let session = EquipmentSession::new().unwrap();
        let mut sheet = session.sheet();
        sheet.parts.clear();
        let mut view = EquipmentView {
            part: usize::MAX,
            item: usize::MAX,
            parts_view: false,
        };
        view.step_part(&sheet, i32::MAX);
        view.step_item(&[], i32::MIN);
        assert_eq!((view.part, view.item), (0, 0));
        view.click(&sheet, &[], [50., 210.]);
        view.click(&sheet, &[], [400., 210.]);
        assert_eq!((view.part, view.item), (0, 0));
    }

    #[test]
    fn blocker_lines_keep_each_body_state_visible() {
        let lines = blocker_lines(&[
            "Detailed anatomy revision 0 is stale; current body revision is 1. Reconcile before attaching equipment.".to_owned(),
            "Keeper is dead; this sheet is read-only.".to_owned(),
        ]);
        assert!(lines.iter().any(|line| line.contains("stale")));
        assert!(lines.iter().any(|line| line.contains("dead")));
        assert!(lines.iter().all(|line| line.len() <= 58));
    }
}

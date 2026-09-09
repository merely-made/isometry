// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use paredros_world::{SubjectSheet, SubjectSheetInput, fixtures::three_lives as fixture};

fn lives() -> Vec<LifeSheet> {
    fixture::three_lives()
        .iter()
        .map(|life| LifeSheet {
            label: life.name,
            summary: life.ordinary_task,
            sheet: SubjectSheet::from_input(SubjectSheetInput {
                subject: life.subject,
                revision: life.revision,
                current_revision: life.revision,
                body: &life.body,
                knowledge: &fixture::knowledge(life),
                inputs: &fixture::inputs(life),
                part_names: fixture::PART_NAMES,
                selected_part: None,
            }),
        })
        .collect()
}

#[test]
fn selection_is_view_local_and_life_change_resets_it() {
    let lives = lives();
    let before = lives
        .iter()
        .map(|life| life.sheet.clone())
        .collect::<Vec<_>>();
    let mut view = SheetView::default();
    view.select_life(&lives, 2);
    view.click(&lives, [30., LIST_TOP + ROW_HEIGHT + 1.]);
    assert!(lives[2].sheet.parts[view.part].severed);
    view.key(&lives, SheetKey::SwitchFocus);
    for _ in 0..20 {
        view.key(&lives, SheetKey::Down);
    }
    assert_eq!(view.action, lives[2].sheet.actions.len() - 1);
    view.select_life(&lives, 0);
    assert_eq!((view.part, view.action, view.detail_scroll), (0, 0, 0));
    assert_eq!(view.focus, Focus::Part);
    assert_eq!(
        before,
        lives
            .iter()
            .map(|life| life.sheet.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn scrolling_reaches_tail_without_changing_selection() {
    let mut lives = lives();
    let part = lives[0].sheet.parts[0].clone();
    lives[0].sheet.parts.extend(std::iter::repeat_n(part, 30));
    lives[0]
        .sheet
        .global_blockers
        .extend((0..40).map(|i| format!("test explanation {i}")));
    let mut view = SheetView::default();
    view.scroll_by(&lives, 300.);
    assert_eq!(view.scroll, 300.);
    assert_eq!(view.part, 0);
    view.scroll_detail_by(&lives, i32::MAX);
    assert_eq!(
        view.detail_scroll,
        details::detail_lines(&lives[0], &view).len() - 18
    );
    view.scroll_detail_by(&lives, i32::MIN);
    assert_eq!(view.detail_scroll, 0);
    view.key(&lives, SheetKey::Down);
    assert_eq!(view.scroll, ROW_HEIGHT);
    view.select_life(&lives, 1);
    assert_eq!(view.scroll, 0.);
}

#[test]
fn clicks_outside_rows_do_not_select_or_clear_details() {
    let lives = lives();
    let mut view = SheetView::default();
    view.detail_scroll = 1;
    for point in [[30., 170.], [30., 650.], [380., 600.], [30., 600.]] {
        view.click(&lives, point);
        assert_eq!(view.part, 0);
        assert_eq!(view.focus, Focus::Part);
        assert_eq!(view.detail_scroll, 1);
    }
}

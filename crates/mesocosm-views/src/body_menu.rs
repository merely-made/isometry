// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared presentation for offered grafts and discovered body developments.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};

pub const MAX_BODY_MENU_ROWS: usize = 6;
pub const BODY_MENU_WIDTH: u32 = 620;
pub const BODY_MENU_HEIGHT: u32 = 500;

pub type BodyMenuChild = Box<dyn AnyView<BodyMenu, (), GenetCtx, GenetElement>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BodyMenuRow {
    pub source: String,
    pub offer: String,
    pub mass: String,
    pub cost: String,
    pub attachment: String,
    pub refusal: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BodyMenu {
    pub headline: String,
    pub facts_title: String,
    pub choice_label: String,
    pub facts: Vec<(String, String)>,
    pub rows: Vec<BodyMenuRow>,
    pub selected: usize,
    pub detail: String,
    pub choice: String,
    pub status: String,
    pub keys: String,
}

impl BodyMenu {
    pub fn new(
        facts: Vec<(String, String)>,
        mut rows: Vec<BodyMenuRow>,
        selected: usize,
        detail: impl Into<String>,
        choice: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        rows.truncate(MAX_BODY_MENU_ROWS);
        Self {
            headline: "Graft tissue".into(),
            facts_title: "the graft".into(),
            choice_label: "keep/regrow".into(),
            facts,
            rows,
            selected: selected.min(MAX_BODY_MENU_ROWS.saturating_sub(1)),
            detail: detail.into(),
            choice: choice.into(),
            status: status.into(),
            keys: "[H] open  [↑/↓, J/L] select  [Tab] keep/regrow  [Enter] confirm  [Esc] cancel"
                .into(),
        }
    }
}

pub fn body_menu_root(state: &BodyMenu) -> BodyMenuChild {
    let mut children: Vec<BodyMenuChild> = vec![Box::new(
        el::<_, BodyMenu, ()>("div", text(state.headline.clone()))
            .attr("class", "body-menu-headline"),
    )];
    let facts = state
        .facts
        .iter()
        .map(|(key, value)| DetailRow::new(key.clone(), value.clone()))
        .collect();
    children.push(Box::new(detail_panel::<BodyMenu, ()>(&[
        DetailSection::new(state.facts_title.clone(), facts),
    ])));
    for (index, row) in state.rows.iter().enumerate() {
        children.push(graft_row(row, index == state.selected));
    }
    children.push(Box::new(
        el::<_, BodyMenu, ()>("div", text(state.detail.clone())).attr("class", "body-menu-detail"),
    ));
    children.push(Box::new(
        el::<_, BodyMenu, ()>(
            "div",
            text(format!("{}: {}", state.choice_label, state.choice)),
        )
        .attr("class", "body-menu-choice"),
    ));
    children.push(Box::new(
        el::<_, BodyMenu, ()>("div", text(state.status.clone())).attr("class", "body-menu-status"),
    ));
    children.push(Box::new(
        el::<_, BodyMenu, ()>("div", text(state.keys.clone())).attr("class", "body-menu-keys"),
    ));
    Box::new(el::<_, BodyMenu, ()>("div", children).attr("class", "body-menu"))
}

fn graft_row(row: &BodyMenuRow, selected: bool) -> BodyMenuChild {
    let class = if selected {
        "body-menu-row body-menu-row-on"
    } else {
        "body-menu-row"
    };
    let name = if selected {
        format!("{}  /  {}", row.source, row.offer)
    } else {
        format!("{}  /  {}  /  {}", row.source, row.offer, row.mass)
    };
    let mut lines: Vec<BodyMenuChild> = vec![Box::new(
        el::<_, BodyMenu, ()>("div", text(name)).attr("class", "body-menu-row-name"),
    )];
    if selected {
        lines.push(Box::new(
            el::<_, BodyMenu, ()>(
                "div",
                text(format!(
                    "{}  ·  {}  ·  {}",
                    row.mass, row.cost, row.attachment
                )),
            )
            .attr("class", "body-menu-row-facts"),
        ));
        lines.push(Box::new(
            el::<_, BodyMenu, ()>("div", text(row_status(row)))
                .attr("class", "body-menu-row-refusal"),
        ));
    }
    Box::new(el::<_, BodyMenu, ()>("div", lines).attr("class", class))
}

fn row_status(row: &BodyMenuRow) -> &str {
    match row.refusal.as_deref() {
        Some(refusal) => refusal,
        None if !row.cost.is_empty() => "ready",
        None => "Select to preview",
    }
}

pub fn body_menu_css() -> &'static str {
    r#"
.body-menu { width: 588px; padding: 14px 16px; background-color: #0b0f14f7; color: #dfe6dd; font-family: sans-serif; font-size: 13px; }
.body-menu-headline { color: #e6d9a8; font-size: 18px; margin-bottom: 8px; }
.body-menu .detail-key { margin-right: 8px; color: #8fa08c; }
.body-menu-row { margin-top: 5px; padding: 5px 7px; border: 1px solid #26313a; }
.body-menu-row-on { border-color: #d5b86a; background-color: #20251d; }
.body-menu-row-name { color: #eaf2e6; font-weight: bold; }
.body-menu-row-facts, .body-menu-row-refusal, .body-menu-detail, .body-menu-choice, .body-menu-status, .body-menu-keys { margin-top: 3px; }
.body-menu-row-refusal, .body-menu-status { color: #e2a06a; }
.body-menu-detail, .body-menu-choice, .body-menu-keys { color: #8fa08c; }
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graft_menu_bounds_rows_and_selection() {
        let rows = (0..9)
            .map(|i| BodyMenuRow {
                source: i.to_string(),
                ..Default::default()
            })
            .collect();
        let menu = BodyMenu::new(vec![], rows, 99, "detail", "keep", "status");
        assert_eq!(menu.rows.len(), MAX_BODY_MENU_ROWS);
        assert_eq!(menu.selected, MAX_BODY_MENU_ROWS - 1);
    }

    #[test]
    fn unselected_unchecked_rows_never_claim_ready() {
        let row = BodyMenuRow {
            source: "carcass".into(),
            offer: "branch".into(),
            mass: "40 mg".into(),
            ..Default::default()
        };
        assert_eq!(row_status(&row), "Select to preview");
        assert_eq!(
            row_status(&BodyMenuRow {
                cost: "12 mg".into(),
                ..row
            }),
            "ready"
        );
    }
}

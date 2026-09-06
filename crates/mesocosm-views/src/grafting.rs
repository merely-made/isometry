// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The contextual graft menu: a bounded reading of offered tissue.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};

pub const MAX_GRAFT_ROWS: usize = 6;
pub const GRAFT_WIDTH: u32 = 620;
pub const GRAFT_HEIGHT: u32 = 500;

pub type GraftChild = Box<dyn AnyView<GraftMenu, (), GenetCtx, GenetElement>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraftRow {
    pub donor: String,
    pub tissue: String,
    pub mass: String,
    pub cost: String,
    pub attachment: String,
    pub refusal: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraftMenu {
    pub headline: String,
    pub facts: Vec<(String, String)>,
    pub rows: Vec<GraftRow>,
    pub selected: usize,
    pub detail: String,
    pub keep_regrow: String,
    pub status: String,
    pub keys: String,
}

impl GraftMenu {
    pub fn new(
        facts: Vec<(String, String)>,
        mut rows: Vec<GraftRow>,
        selected: usize,
        detail: impl Into<String>,
        keep_regrow: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        rows.truncate(MAX_GRAFT_ROWS);
        Self {
            headline: "Graft tissue".into(),
            facts,
            rows,
            selected: selected.min(MAX_GRAFT_ROWS.saturating_sub(1)),
            detail: detail.into(),
            keep_regrow: keep_regrow.into(),
            status: status.into(),
            keys: "[H] open  [↑/↓, J/L] select  [Tab] keep/regrow  [Enter] confirm  [Esc] cancel"
                .into(),
        }
    }
}

pub fn grafting_root(state: &GraftMenu) -> GraftChild {
    let mut children: Vec<GraftChild> = vec![Box::new(
        el::<_, GraftMenu, ()>("div", text(state.headline.clone())).attr("class", "graft-headline"),
    )];
    let facts = state
        .facts
        .iter()
        .map(|(key, value)| DetailRow::new(key.clone(), value.clone()))
        .collect();
    children.push(Box::new(detail_panel::<GraftMenu, ()>(&[
        DetailSection::new("the graft", facts),
    ])));
    for (index, row) in state.rows.iter().enumerate() {
        children.push(graft_row(row, index == state.selected));
    }
    children.push(Box::new(
        el::<_, GraftMenu, ()>("div", text(state.detail.clone())).attr("class", "graft-detail"),
    ));
    children.push(Box::new(
        el::<_, GraftMenu, ()>("div", text(format!("keep/regrow: {}", state.keep_regrow)))
            .attr("class", "graft-choice"),
    ));
    children.push(Box::new(
        el::<_, GraftMenu, ()>("div", text(state.status.clone())).attr("class", "graft-status"),
    ));
    children.push(Box::new(
        el::<_, GraftMenu, ()>("div", text(state.keys.clone())).attr("class", "graft-keys"),
    ));
    Box::new(el::<_, GraftMenu, ()>("div", children).attr("class", "grafting"))
}

fn graft_row(row: &GraftRow, selected: bool) -> GraftChild {
    let class = if selected {
        "graft-row graft-row-on"
    } else {
        "graft-row"
    };
    let name = if selected {
        format!("{}  /  {}", row.donor, row.tissue)
    } else {
        format!("{}  /  {}  /  {}", row.donor, row.tissue, row.mass)
    };
    let mut lines: Vec<GraftChild> = vec![Box::new(
        el::<_, GraftMenu, ()>("div", text(name)).attr("class", "graft-row-name"),
    )];
    if selected {
        lines.push(Box::new(
            el::<_, GraftMenu, ()>(
                "div",
                text(format!(
                    "{}  ·  {}  ·  {}",
                    row.mass, row.cost, row.attachment
                )),
            )
            .attr("class", "graft-row-facts"),
        ));
        lines.push(Box::new(
            el::<_, GraftMenu, ()>("div", text(row_status(row))).attr("class", "graft-row-refusal"),
        ));
    }
    Box::new(el::<_, GraftMenu, ()>("div", lines).attr("class", class))
}

fn row_status(row: &GraftRow) -> &str {
    match row.refusal.as_deref() {
        Some(refusal) => refusal,
        None if !row.cost.is_empty() => "ready",
        None => "Select to preview",
    }
}

pub fn grafting_css() -> &'static str {
    r#"
.grafting { width: 588px; padding: 14px 16px; background-color: #0b0f14f7; color: #dfe6dd; font-family: sans-serif; font-size: 13px; }
.graft-headline { color: #e6d9a8; font-size: 18px; margin-bottom: 8px; }
.grafting .detail-key { margin-right: 8px; color: #8fa08c; }
.graft-row { margin-top: 5px; padding: 5px 7px; border: 1px solid #26313a; }
.graft-row-on { border-color: #d5b86a; background-color: #20251d; }
.graft-row-name { color: #eaf2e6; font-weight: bold; }
.graft-row-facts, .graft-row-refusal, .graft-detail, .graft-choice, .graft-status, .graft-keys { margin-top: 3px; }
.graft-row-refusal, .graft-status { color: #e2a06a; }
.graft-detail, .graft-choice, .graft-keys { color: #8fa08c; }
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graft_menu_bounds_rows_and_selection() {
        let rows = (0..9)
            .map(|i| GraftRow {
                donor: i.to_string(),
                ..Default::default()
            })
            .collect();
        let menu = GraftMenu::new(vec![], rows, 99, "detail", "keep", "status");
        assert_eq!(menu.rows.len(), MAX_GRAFT_ROWS);
        assert_eq!(menu.selected, MAX_GRAFT_ROWS - 1);
    }

    #[test]
    fn unselected_unchecked_rows_never_claim_ready() {
        let row = GraftRow {
            donor: "carcass".into(),
            tissue: "branch".into(),
            mass: "40 mg".into(),
            ..Default::default()
        };
        assert_eq!(row_status(&row), "Select to preview");
        assert_eq!(
            row_status(&GraftRow {
                cost: "12 mg".into(),
                ..row
            }),
            "ready"
        );
    }
}

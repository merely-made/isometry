// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The trait board: the played line's own turn, on screen. (PE3b)
//!
//! The fourth cambium surface, and the second that appears because the world is
//! **waiting**. Where `succession` asks one question about one critter, this
//! asks the lineage's: the epoch is reckoned, every other line has already
//! weighed what it could, and yours has not.
//!
//! **It is a table, not an editor.** Nothing here arranges tissue, names a
//! cell, or prices anything: every number on it was read off the world by
//! `World::offers` and the driver's review, and the only things a player can do
//! are move the selection, commit the selected candidate, and leave. The
//! candidate's own proposals are *shown* — the game's, and a pack's where one
//! applies — because two proposal sources over one validator is a fact worth
//! being able to see, not because a player picks between them.
//!
//! Host-agnostic like the panels beside it: this crate says what the surface is
//! and what its words are, the host says where the raster lands, and the driver
//! decides when there is anything to show.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};
use mesocosm_core::{Feat, Offer, Reading, Scale, Trend, Untakeable};

pub type BoardChild = Box<dyn AnyView<Board, (), GenetCtx, GenetElement>>;

/// One candidate's row, already in words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardRow {
    /// What it is: the status quo, or the condition the line came to.
    pub name: String,
    /// Where a proposal for it comes from, in table order.
    pub source: String,
    /// What growing it earned, with the window it was measured over.
    pub net: String,
    /// What the next descendant would pay for it.
    pub price: String,
    /// The founder preview it would grow.
    pub preview: String,
    /// Why it cannot be taken. `None` when it can.
    pub reason: Option<String>,
    /// Whether the cursor is on it.
    pub selected: bool,
}

/// What the board says. A **reading**, taken from the driver's review: nothing
/// here is stored in the world, enters the trace, or reaches the state hash.
/// The answers do — and they are ordinary intents the host's keyboard sends.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Board {
    pub headline: String,
    /// The boundary's pointable facts: which epoch, whose line, what a founder
    /// will have to spend, and what the enclosure is doing.
    pub facts: Vec<(String, String)>,
    /// The reckoning: what this epoch came to, and which of it took the record.
    /// Empty when nothing happened worth noting, which is a real epoch.
    pub readings: Vec<String>,
    pub rows: Vec<BoardRow>,
    /// What committing would do, when the selected row can be committed.
    pub commit: Option<String>,
    /// What moving the selection does.
    pub next: String,
    /// What carrying on does.
    pub stay: String,
}

impl Board {
    /// The boundary's headline facts.
    ///
    /// `current` is the revision the line is already born under, so a player who
    /// has just committed can see that the table is now describing the new one.
    pub fn of(
        epoch: u64,
        lineage: u32,
        budget_mg: u64,
        current: Option<u32>,
        trend: &Trend,
    ) -> Self {
        Self {
            headline: "the epoch is over".into(),
            facts: vec![
                ("epoch".into(), format!("{epoch} ended")),
                (
                    "your line".into(),
                    match current {
                        Some(revision) => format!("line {lineage}, born under revision {revision}"),
                        None => format!("line {lineage}, born as it always was"),
                    },
                ),
                // The budget, stated as what it actually is. A revision is
                // priced flat, so what a price is weighed against is the
                // founder's own material and not a pool somewhere.
                (
                    "a founder holds".into(),
                    format!("{budget_mg} mg to develop with"),
                ),
                (
                    "the enclosure".into(),
                    crate::vitals::replacement_words(trend),
                ),
            ],
            readings: Vec::new(),
            rows: Vec::new(),
            commit: None,
            next: "another candidate".into(),
            stay: "back to the terrarium".into(),
        }
    }

    /// Whether anything on this table could actually be committed.
    pub fn has_a_choice(&self) -> bool {
        self.rows
            .iter()
            .any(|row| row.reason.is_none() && !row.name.starts_with("the status"))
    }
}

/// One reading of the reckoning, in words.
///
/// **What was done, how far it reached, how much of it, and whether the world
/// had seen the like.** The last clause is the whole of significance as the
/// epoch boundary plan rules it: abnormality against this world's own record,
/// never a difficulty table.
pub fn reading_words(reading: &Reading) -> String {
    format!(
        "{} of line {}, {} — {}{}",
        feat_word(reading.feat),
        reading.species.0,
        scale_word(reading.scale),
        reading.value,
        if reading.took {
            ", the most this world has seen"
        } else {
            ""
        }
    )
}

/// The reckoning, narrowed to what this review is evidence for.
///
/// **Your line's readings whole, and one line for everyone else's.** A young
/// enclosure reckons twenty-odd marks across six lines, which is a scrolling
/// log rather than evidence — and a board whose table and answers are pushed
/// off the bottom by it is worse than one that says less. The question this
/// screen asks is what *your* line should do next, so what your line did stays
/// whole; what the rest of the enclosure took is still stated, because
/// significance is abnormality against a record everyone writes into.
pub fn evidence_words(readings: &[Reading], lineage: u32) -> Vec<String> {
    let mut words: Vec<String> = readings
        .iter()
        .filter(|reading| reading.species.0 == lineage)
        .map(reading_words)
        .collect();
    let others: Vec<&Reading> = readings
        .iter()
        .filter(|reading| reading.species.0 != lineage && reading.took)
        .collect();
    if !others.is_empty() {
        let mut lines: Vec<u32> = others.iter().map(|reading| reading.species.0).collect();
        lines.sort_unstable();
        lines.dedup();
        words.push(format!(
            "{} mark{} taken by {} other line{}",
            others.len(),
            if others.len() == 1 { "" } else { "s" },
            lines.len(),
            if lines.len() == 1 { "" } else { "s" },
        ));
    }
    words
}

fn feat_word(feat: Feat) -> &'static str {
    match feat {
        Feat::Growth => "growth",
        Feat::Predation => "hunting",
        Feat::Symbiosis => "giving",
        Feat::Endurance => "living long",
        Feat::Spread => "reaching",
        Feat::Construction => "building",
    }
}

fn scale_word(scale: Scale) -> &'static str {
    match scale {
        Scale::Local => "in one place",
        Scale::Regional => "across a region",
        Scale::Worldwide => "across the enclosure",
    }
}

/// One candidate's row.
///
/// `sources` are the names of the proposals that would express it, in the order
/// the review found them; the host supplies them because which sources exist is
/// the driver's question, not this crate's.
pub fn row_words(offer: &Offer, sources: &[String], selected: bool) -> BoardRow {
    let name = match offer.candidate {
        None => "the status quo".to_string(),
        Some(condition) => crate::vitals::condition_word(condition),
    };
    BoardRow {
        name,
        source: match sources {
            [] => "nothing to build".to_string(),
            named => named.join(", "),
        },
        // Signed, because a candidate that earns less than standing still is a
        // real outcome and the sign is the whole reading.
        net: format!(
            "{}{} mg over {} ticks",
            if offer.score.net_mg() < 0 { "-" } else { "+" },
            offer.score.net_mg().unsigned_abs(),
            offer.score.ticks
        ),
        price: match offer.price_mg {
            0 => "nothing to develop".to_string(),
            mg => format!("{mg} mg at the next birth"),
        },
        preview: format!("founder {:016x}", offer.preview),
        reason: offer.why_not.as_ref().map(Untakeable::words),
        selected,
    }
}

/// What committing this row would do, in words.
///
/// `None` when it cannot be committed — the status quo, or a candidate carrying
/// a reason — so the panel's commit line and the keyboard's commit key are
/// answering one question rather than two that could drift apart.
pub fn commit_words(offer: &Offer) -> Option<String> {
    let condition = offer.takeable().then_some(offer.candidate).flatten()?;
    Some(format!(
        "take {} into the line",
        crate::vitals::condition_word(condition)
    ))
}

/// The board: the reckoning, the table, and the three answers.
pub fn board_root(state: &Board) -> BoardChild {
    let mut children: Vec<BoardChild> = vec![Box::new(
        el::<_, Board, ()>("div", text(state.headline.clone())).attr("class", "board-headline"),
    )];

    let facts = state
        .facts
        .iter()
        .map(|(key, value)| DetailRow::new(key.clone(), value.clone()))
        .collect();
    children.push(Box::new(detail_panel::<Board, ()>(&[DetailSection::new(
        "the boundary",
        facts,
    )])));

    // The evidence, and only when there is some. An epoch in which nothing was
    // worth noting says nothing rather than printing a heading over an
    // absence.
    if !state.readings.is_empty() {
        let readings = state
            .readings
            .iter()
            .map(|words| DetailRow::new("noted", words.clone()))
            .collect();
        children.push(Box::new(detail_panel::<Board, ()>(&[DetailSection::new(
            "what the epoch came to",
            readings,
        )])));
    }

    for row in &state.rows {
        children.push(row_view(row));
    }

    // Three answers, one line each. Two of them are keys that send an intent;
    // the third only moves a cursor, which is why it is stated last.
    if let Some(commit) = &state.commit {
        children.push(Box::new(
            el::<_, Board, ()>("div", text(format!("[R]  {commit}"))).attr("class", "board-answer"),
        ));
    }
    children.push(Box::new(
        el::<_, Board, ()>("div", text(format!("[Tab]  {}", state.next)))
            .attr("class", "board-answer"),
    ));
    children.push(Box::new(
        el::<_, Board, ()>("div", text(format!("[Enter]  {}", state.stay)))
            .attr("class", "board-answer"),
    ));

    Box::new(el::<_, Board, ()>("div", children).attr("class", "board"))
}

/// One row, as three lines: what it is, what it came to, and what stops it.
fn row_view(row: &BoardRow) -> BoardChild {
    let class = if row.selected {
        "board-row board-row-on"
    } else {
        "board-row"
    };
    let mut lines: Vec<BoardChild> = vec![
        Box::new(
            el::<_, Board, ()>("div", text(format!("{}  ({})", row.name, row.source)))
                .attr("class", "board-row-name"),
        ),
        Box::new(
            el::<_, Board, ()>(
                "div",
                text(format!("{} / {} / {}", row.net, row.price, row.preview)),
            )
            .attr("class", "board-row-figures"),
        ),
    ];
    // Only when it is true. A row that always carried a reason line would be
    // saying "nothing is wrong" in a place a player learns to stop reading.
    if let Some(reason) = &row.reason {
        lines.push(Box::new(
            el::<_, Board, ()>("div", text(reason.clone())).attr("class", "board-row-reason"),
        ));
    }
    Box::new(el::<_, Board, ()>("div", lines).attr("class", class))
}

/// The sheet the board is styled by. Heavier than the vitals panel and darker
/// than the checkpoint's, because this is the screen the world stopped for.
pub fn board_css() -> &'static str {
    r#"
.board {
    width: 588px;
    padding: 14px 16px;
    background-color: #0b0f14f7;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 13px;
}
.board-headline {
    color: #e6d9a8;
    font-size: 18px;
    font-weight: bold;
    margin-bottom: 8px;
}
.detail-section-title {
    color: #8fa08c;
    font-size: 12px;
    margin-top: 6px;
    margin-bottom: 3px;
}
.detail-row { margin-bottom: 2px; }
.detail-key { color: #8fa08c; }
.detail-value { color: #eaf2e6; margin-left: 8px; }
.board-row {
    margin-top: 6px;
    padding: 4px 6px;
    background-color: #141a20;
}
.board-row-on {
    background-color: #1d2a22;
}
.board-row-name { color: #eaf2e6; font-weight: bold; }
.board-row-figures { color: #a8b7a4; font-size: 12px; }
.board-row-reason { color: #d8a06a; font-size: 12px; }
.board-answer {
    margin-top: 6px;
    color: #9fd08a;
    font-size: 13px;
}
"#
}

#[cfg(test)]
mod tests;

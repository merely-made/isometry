// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The individual checkpoint's surface: the game stopped, and this is what it
//! is asking.
//!
//! The second cambium consumer, and the first one that arrives because the
//! world is **waiting**. Vitals report while play runs; this appears only at a
//! birth involving the critter under your hand, or at the moment that critter
//! stops being one, and it goes away again the tick either question is answered.
//!
//! **It is not the trait board.** Nothing here revises a developmental program,
//! spends a lineage budget, previews a founder, or lists a brood to shop
//! through. Reproduction is the checkpoint at the scale of one critter and the
//! epoch is the one at the scale of a lineage, and PE1's stop rule is that
//! neither may quietly become the other. So: four facts, two keys, out.
//!
//! Host-agnostic like the panel beside it. This crate says what the surface
//! *is* and what its words are; the host says where the raster lands, and the
//! driver decides when there is anything to show.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};

pub type SuccessionChild = Box<dyn AnyView<Succession, (), GenetCtx, GenetElement>>;

/// What the checkpoint says, already in words.
///
/// A **reading**, taken from the driver's question: nothing here is stored in
/// the world, enters the trace, or reaches the state hash. The answer does —
/// but the answer is an ordinary intent, and it is the host's keyboard that
/// sends it, not this.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Succession {
    /// What happened, in four words or so.
    pub headline: String,
    /// The pointable facts: who, what, and what it cost.
    pub facts: Vec<(String, String)>,
    /// What carrying on unchanged does here.
    pub stay: String,
    /// What taking the offered body does, when there is one to take.
    pub take: Option<String>,
}

impl Succession {
    /// A birth the played critter is the parent of.
    ///
    /// Parent, offspring, cost and descent, each a line. The cost is stated in
    /// both accounts because they are not the same loss: body is what the
    /// parent will not get back, and reserve is what it will have to earn again.
    pub fn birth(
        parent: u32,
        offspring: u32,
        lineage: u32,
        substance_mg: u64,
        reserve_mg: u64,
        offerable: bool,
    ) -> Self {
        Self {
            headline: "a birth".into(),
            facts: vec![
                ("parent".into(), format!("critter {parent}")),
                ("offspring".into(), format!("critter {offspring}")),
                (
                    "cost".into(),
                    format!(
                        "{} mg — {substance_mg} of body, {reserve_mg} of reserve",
                        substance_mg + reserve_mg
                    ),
                ),
                (
                    "descent".into(),
                    format!("child of critter {parent}, line {lineage}"),
                ),
            ],
            stay: "stay in the parent".into(),
            // The offspring, not the brood. Every other child this line has
            // ever had is out there living its own life and is reached the
            // ordinary way, in the world.
            take: offerable.then(|| format!("become critter {offspring}")),
        }
    }

    /// The epoch ended, and every other line has already taken its turn.
    ///
    /// **The lineage checkpoint's words, in the panel that already exists**
    /// (PE3a). Not the trait board: there is no candidate list here, no
    /// preview, and nothing to spend. What it says is that the round happened,
    /// how much of the enclosure moved in it, and that yours is the line that
    /// has not answered yet — which is PE3b's review.
    pub fn epoch(epoch: u64, lineage: u32, turned: usize, committed: usize) -> Self {
        Self {
            headline: "the epoch is over".into(),
            facts: vec![
                ("epoch".into(), format!("{epoch} ended")),
                ("your line".into(), format!("line {lineage}, yet to answer")),
                (
                    "the others".into(),
                    match turned {
                        0 => "no line had anything to weigh".to_string(),
                        1 => "one line took a turn".to_string(),
                        many => format!("{many} lines took turns"),
                    },
                ),
                (
                    "changed".into(),
                    match committed {
                        0 => "none of them changed".to_string(),
                        1 => "one committed a change".to_string(),
                        many => format!("{many} committed changes"),
                    },
                ),
            ],
            stay: "back to the terrarium".into(),
            // No body is offered at a lineage checkpoint. It is not that kind
            // of question.
            take: None,
        }
    }

    /// The played critter stopped being one.
    ///
    /// `heirs` is how many living descendants this world would let anyone
    /// inhabit, and `heir` is the eldest of them — the one a single key takes.
    /// The count is stated so the player can tell "no line survives" from "a
    /// line survives and you are taking the eldest of it".
    pub fn loss(organism: u32, lineage: u32, heirs: usize, heir: Option<u32>) -> Self {
        Self {
            headline: "the body is gone".into(),
            facts: vec![
                ("was".into(), format!("critter {organism}, line {lineage}")),
                (
                    "descendants".into(),
                    match heirs {
                        0 => "none living".to_string(),
                        1 => "one living".to_string(),
                        many => format!("{many} living"),
                    },
                ),
            ],
            stay: match heirs {
                0 => "look on".into(),
                _ => "let the line go".into(),
            },
            take: heir.map(|heir| format!("continue as critter {heir}, your eldest")),
        }
    }
}

/// The checkpoint panel: what happened, what it cost, and the two answers.
pub fn succession_root(state: &Succession) -> SuccessionChild {
    let rows = state
        .facts
        .iter()
        .map(|(key, value)| DetailRow::new(key.clone(), value.clone()))
        .collect();

    let mut children: Vec<SuccessionChild> = vec![
        Box::new(
            el::<_, Succession, ()>("div", text(state.headline.clone()))
                .attr("class", "checkpoint-headline"),
        ),
        Box::new(detail_panel::<Succession, ()>(&[DetailSection::new(
            "checkpoint",
            rows,
        )])),
    ];

    // Two answers, one line each, and no third. A checkpoint with a growing
    // list of things to do is an editor.
    if let Some(take) = &state.take {
        children.push(Box::new(
            el::<_, Succession, ()>("div", text(format!("[T]  {take}")))
                .attr("class", "checkpoint-answer"),
        ));
    }
    children.push(Box::new(
        el::<_, Succession, ()>("div", text(format!("[Enter]  {}", state.stay)))
            .attr("class", "checkpoint-answer"),
    ));

    Box::new(el::<_, Succession, ()>("div", children).attr("class", "checkpoint"))
}

/// The sheet the checkpoint is styled by. Heavier than the vitals panel on
/// purpose: this one is the world holding still, so it reads as a thing that
/// stopped rather than another corner reading.
pub fn succession_css() -> &'static str {
    r#"
.checkpoint {
    width: 436px;
    padding: 14px 16px;
    background-color: #0d1116f2;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 14px;
}
.checkpoint-headline {
    color: #e6d9a8;
    font-size: 18px;
    font-weight: bold;
    margin-bottom: 8px;
}
.detail-section-title {
    color: #8fa08c;
    font-size: 12px;
    margin-bottom: 4px;
}
.detail-row { margin-bottom: 3px; }
.detail-key { color: #8fa08c; }
.detail-value { color: #eaf2e6; margin-left: 8px; }
.checkpoint-answer {
    margin-top: 8px;
    color: #9fd08a;
    font-size: 14px;
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parent, offspring, cost and descent, all four pointable off the panel
    /// itself — which is the done-condition, in the place a player reads it.
    #[test]
    fn a_birth_states_who_what_and_what_it_cost() {
        let birth = Succession::birth(0, 1173, 1, 505, 505, true);
        let facts: Vec<&str> = birth.facts.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(facts, ["parent", "offspring", "cost", "descent"]);
        let cost = &birth.facts[2].1;
        assert!(cost.contains("1010 mg"), "the whole debit: {cost}");
        assert!(cost.contains("505 of body"), "and both accounts: {cost}");
        assert!(cost.contains("505 of reserve"), "and both accounts: {cost}");
        assert_eq!(birth.take.as_deref(), Some("become critter 1173"));
    }

    /// The lineage checkpoint says the round happened and offers no body.
    /// It is still not a review: PE3b owns candidates, prices and previews.
    #[test]
    fn the_lineage_checkpoint_reports_the_round_and_offers_nothing_to_take() {
        let boundary = Succession::epoch(3, 1, 4, 2);
        let facts: Vec<&str> = boundary.facts.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(facts, ["epoch", "your line", "the others", "changed"]);
        assert_eq!(boundary.take, None, "no body is offered at a lineage's own");
        assert_eq!(boundary.stay, "back to the terrarium");

        let quiet = Succession::epoch(1, 1, 0, 0);
        assert_eq!(quiet.facts[2].1, "no line had anything to weigh");
        assert_eq!(quiet.facts[3].1, "none of them changed");
    }

    /// Two answers and no third. A checkpoint that grew a menu would be the
    /// epoch's job, and PE1's stop rule is that it must not become one.
    #[test]
    fn the_checkpoint_offers_exactly_two_answers_and_no_program() {
        for state in [
            Succession::birth(0, 9, 1, 100, 40, true),
            Succession::loss(0, 1, 3, Some(9)),
            Succession::loss(0, 1, 0, None),
        ] {
            let answers = usize::from(state.take.is_some()) + 1;
            assert!(answers <= 2, "one to stay, at most one to take");
            let words = format!("{state:?}").to_lowercase();
            for editorial in ["program", "trait", "budget", "epoch", "revise", "founder"] {
                assert!(
                    !words.contains(editorial),
                    "the individual checkpoint says nothing about {editorial}: {words}"
                );
            }
        }
    }

    /// A line with nobody left in it says so, rather than offering a body that
    /// is not there.
    #[test]
    fn a_loss_with_no_descendant_offers_nothing_to_take() {
        let empty = Succession::loss(4, 1, 0, None);
        assert_eq!(empty.take, None);
        assert_eq!(empty.facts[1].1, "none living");
        assert_eq!(empty.stay, "look on");

        let carried = Succession::loss(4, 1, 2, Some(11));
        assert_eq!(carried.facts[1].1, "2 living");
        assert!(
            carried.take.is_some_and(|words| words.contains("11")),
            "and it names the one a key would take"
        );
    }
}

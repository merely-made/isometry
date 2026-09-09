// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the board says, in the exact words a player reads. (PE3b)

use super::*;
use mesocosm_core::{ConditionId, Role, Score, SpeciesId, Unexpressed};

fn scored(income_mg: u64, rent_mg: u64) -> Score {
    Score {
        ticks: 480,
        income_mg,
        rent_mg,
        outflow_mg: 0,
        born: 2,
    }
}

fn offer(candidate: Option<u64>, price_mg: u64, why_not: Option<Untakeable>) -> Offer {
    Offer {
        candidate: candidate.map(ConditionId),
        score: scored(9_000, 4_000),
        price_mg,
        preview: 0x0f2a_1122_3344_5566,
        program: 7,
        why_not,
    }
}

fn trend() -> Trend {
    Trend {
        replacement_ticks: 240,
        matured: 4,
        died: 6,
        stand_ticks: 60,
        stand_change_mg: -20,
        grazed_mg: 90,
        shortfall_ticks: 0,
    }
}

fn board() -> Board {
    let mut board = Board::of(3, 1, 1_200, Some(0), &trend());
    board.rows = vec![
        row_words(&offer(None, 0, None), &[], true),
        row_words(
            &offer(Some(1), 230, None),
            &["discovered".to_string(), "authored".to_string()],
            false,
        ),
    ];
    board
}

/// The four facts a boundary is owed, and the budget among them — stated as
/// what a founder will actually hold rather than as a pool.
#[test]
fn the_board_states_the_epoch_the_line_and_the_budget() {
    let board = board();
    let keys: Vec<&str> = board.facts.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(
        keys,
        ["epoch", "your line", "a founder holds", "the enclosure"]
    );
    assert_eq!(board.facts[0].1, "3 ended");
    assert!(
        board.facts[1].1.contains("revision 0"),
        "the revision it is currently born under: {}",
        board.facts[1].1
    );
    assert_eq!(board.facts[2].1, "1200 mg to develop with");

    let founding = Board::of(1, 4, 0, None, &trend());
    assert_eq!(founding.facts[1].1, "line 4, born as it always was");
}

/// The status quo is a row, so *nothing beat what I have* is something a
/// player reads rather than infers.
#[test]
fn the_status_quo_is_a_row_and_cannot_be_committed() {
    let board = board();
    assert_eq!(board.rows[0].name, "the status quo");
    assert_eq!(board.rows[0].price, "nothing to develop");
    assert_eq!(board.rows[0].source, "nothing to build");
    assert!(board.rows[0].reason.is_none());
    assert!(
        board.has_a_choice(),
        "and the candidate beside it is the choice"
    );
}

/// A candidate that cannot be taken keeps its place and says why. PE2's
/// residue, in the place a player acts on it.
#[test]
fn an_untakeable_candidate_carries_its_reason() {
    let nowhere = row_words(
        &offer(
            Some(2),
            0,
            Some(Untakeable::Unexpressed(Unexpressed::NoSite {
                role: Role::Plate,
            })),
        ),
        &["discovered".to_string()],
        false,
    );
    assert_eq!(
        nowhere.reason.as_deref(),
        Some("nowhere on this body is a plate")
    );

    let poor = row_words(
        &offer(
            Some(3),
            900,
            Some(Untakeable::Unexpressed(Unexpressed::Unaffordable {
                needed_mg: 900,
                held_mg: 120,
            })),
        ),
        &["discovered".to_string()],
        false,
    );
    assert_eq!(
        poor.price, "900 mg at the next birth",
        "an unaffordable price is still a price, and it is the number to earn"
    );
    assert!(poor.reason.is_some_and(|why| why.contains("900")));

    let missing = row_words(&offer(Some(4), 0, Some(Untakeable::Nothing)), &[], false);
    assert_eq!(
        missing.reason.as_deref(),
        Some("this world does not hold what it needs")
    );
}

/// Two proposal sources over one validator, marked by name — and one source
/// when no pack expression applies.
#[test]
fn a_row_names_every_source_that_would_build_it() {
    let board = board();
    assert_eq!(board.rows[1].source, "discovered, authored");
    let alone = row_words(
        &offer(Some(1), 20, None),
        &["discovered".to_string()],
        false,
    );
    assert_eq!(alone.source, "discovered");
}

/// The figures, with the window they were measured over and the sign of the
/// net. A score without its window is not a reading.
#[test]
fn a_row_states_its_net_its_window_its_price_and_its_preview() {
    let row = row_words(
        &offer(Some(1), 230, None),
        &["discovered".to_string()],
        false,
    );
    assert_eq!(row.net, "+5000 mg over 480 ticks");
    assert_eq!(row.price, "230 mg at the next birth");
    assert_eq!(row.preview, "founder 0f2a112233445566");

    let losing = Offer {
        score: scored(1_000, 4_000),
        ..offer(Some(1), 5, None)
    };
    assert_eq!(
        row_words(&losing, &["discovered".to_string()], false).net,
        "-3000 mg over 480 ticks",
        "earning less than standing still is a real outcome"
    );
}

/// What the epoch came to, and whether the world had seen the like — which is
/// significance as the boundary plan rules it and not a difficulty table.
#[test]
fn a_reading_says_what_was_done_how_far_and_whether_it_was_a_first() {
    let took = Reading {
        species: SpeciesId(2),
        feat: Feat::Predation,
        scale: Scale::Regional,
        value: 4_100,
        took: true,
    };
    assert_eq!(
        reading_words(&took),
        "hunting of line 2, across a region — 4100, the most this world has seen"
    );
    let ordinary = Reading {
        took: false,
        ..took
    };
    assert!(!reading_words(&ordinary).contains("the most"));
}

/// The evidence stays your line's, and everyone else's is one line — because a
/// young enclosure reckons twenty-odd marks and a board whose answers are
/// pushed off the bottom by them says less, not more.
#[test]
fn the_evidence_keeps_your_lines_readings_whole_and_summarizes_the_rest() {
    let reading = |line: u32, feat: Feat, took: bool| Reading {
        species: SpeciesId(line),
        feat,
        scale: Scale::Local,
        value: 10,
        took,
    };
    let readings = [
        reading(1, Feat::Growth, true),
        reading(1, Feat::Spread, false),
        reading(2, Feat::Growth, true),
        reading(3, Feat::Growth, true),
        reading(3, Feat::Predation, true),
        reading(4, Feat::Growth, false),
    ];

    let words = evidence_words(&readings, 1);
    assert_eq!(words.len(), 3, "two of mine and one summary: {words:?}");
    assert!(words[0].contains("of line 1"));
    assert!(words[1].contains("of line 1"));
    assert_eq!(
        words[2], "3 marks taken by 2 other lines",
        "a line that took nothing is not a mark"
    );

    // A line nobody else beat anything against says only its own.
    assert_eq!(evidence_words(&readings[..2], 1).len(), 2);
    // And an epoch in which nothing was worth noting says nothing at all.
    assert!(evidence_words(&[], 1).is_empty());
}

/// Three answers and no fourth, and the commit line appears only when there is
/// something to commit. A board that always offered one would be asking a
/// player to spend for the sake of spending.
#[test]
fn the_board_offers_the_three_answers_and_no_menu() {
    let mut board = board();
    assert_eq!(board.commit, None, "nothing selected but the status quo");
    board.commit = Some("take endured hunger".into());
    assert_eq!(board.next, "another candidate");
    assert_eq!(board.stay, "back to the terrarium");

    let empty = Board::of(1, 1, 0, None, &trend());
    assert!(
        !empty.has_a_choice(),
        "a boundary with nothing to weigh says so"
    );
}

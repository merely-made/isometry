// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a program *is*, at the scale of the type. What a birth and a founder
//! preview make of one is a world claim and lives in
//! `tests/embodied/lineage.rs`.

use super::*;
use crate::discovery::ConditionId;
use crate::process::Process;

fn cites(n: u64) -> Citation {
    Citation {
        condition: ConditionId(n),
        discovery: n * 31,
    }
}

fn site(cells: u32) -> DeclaredSite {
    DeclaredSite {
        role: Role::Plate,
        process: Registry::native().of_native(Process::Secrete).reference(),
        cells,
    }
}

#[test]
fn a_founding_program_is_the_absence_of_one() {
    // The whole reason nothing a world serializes moves until a line commits:
    // geometry seeding *is* the founding revision, so there is no record of it
    // to keep.
    let program = Program::default();
    assert!(program.is_empty());
    assert_eq!(program.current(), None);
    assert_eq!(program.digest(), 0);
}

#[test]
fn a_second_commit_appends_and_names_its_parent() {
    // Epoch-boundary plan §2: every committed adaptation creates an immutable
    // child revision, and nothing edits the parent.
    let mut program = Program::default();
    let first = program.commit(cites(1), vec![site(5)], 100);
    let before = program.get(first).cloned().expect("committed");

    let second = program.commit(cites(2), vec![site(3)], 200);
    assert_ne!(first, second);
    assert_eq!(program.len(), 2);
    assert_eq!(
        program.get(first),
        Some(&before),
        "the parent is byte-identical after a child was committed"
    );
    assert_eq!(
        program.get(first).unwrap().parent,
        None,
        "the first descends from the founding revision, which is stored nowhere"
    );
    assert_eq!(program.get(second).unwrap().parent, Some(first));
    assert_eq!(program.current().map(|r| r.id), Some(second));
}

#[test]
fn a_revision_digest_is_over_what_it_rules() {
    // Two lines that agree about a name and disagree about the sites cannot
    // agree about a revision, which is `ConditionId`'s discipline one scale up.
    let mut wide = Program::default();
    let mut narrow = Program::default();
    wide.commit(cites(1), vec![site(5)], 10);
    narrow.commit(cites(1), vec![site(1)], 10);
    assert_ne!(wide.digest(), narrow.digest());

    // And the tick it was committed on is not rule-bearing: the same program
    // committed later is the same program.
    let mut later = Program::default();
    later.commit(cites(1), vec![site(5)], 9_999);
    assert_eq!(wide.digest(), later.digest());
}

#[test]
fn a_declared_site_is_a_candidates_three_rule_bearing_fields() {
    // No second vocabulary: committing reads what the discovery already
    // granted, and cannot state a cell address or a price.
    let candidate = crate::discovery::Candidate {
        process: Registry::native().of_native(Process::Secrete).reference(),
        site: Role::Plate,
        cells: 5,
        word: Some(crate::axis::Appendage::Plate),
    };
    let declared = DeclaredSite::of(&candidate);
    assert_eq!(declared.role, candidate.site);
    assert_eq!(declared.process, candidate.process);
    assert_eq!(declared.cells, candidate.cells);
}

#[test]
fn the_ground_charges_what_a_line_grows() {
    // `Organism::charged_mg`'s rule, asked at development time. PD4's own two
    // fixtures, in the numbers they were recorded with: a five-cell gland on
    // tissue worth 23 mg a cell holds 115 mg.
    let rich = Conditions {
        ground_mg: 400,
        material_mg: 1_500,
    };
    let lean = Conditions {
        ground_mg: 20,
        material_mg: 1_500,
    };
    assert_eq!(rich.affords(5, 23), 5, "the ground can charge the ask");
    assert_eq!(lean.affords(5, 23), 1, "and here it cannot, so a token one");
}

#[test]
fn a_program_round_trips() {
    let mut program = Program::default();
    program.commit(cites(4), vec![site(2)], 7);
    let bytes = crate::snapshot::encode(&program).unwrap();
    assert_eq!(crate::snapshot::decode::<Program>(&bytes).unwrap(), program);
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Split out of `history.rs` at the 600-line ceiling when PD5 gave a birth
//! under a lineage revision its own records. What stayed next door is the
//! event vocabulary and the causal log; this is what pins them.

use super::*;

const A: OrganismId = OrganismId(1);
const B: OrganismId = OrganismId(2);
const SPECIES: SpeciesId = SpeciesId(7);

/// An event with its envelope. The causal claims below are about what is
/// inside it; the tick is what a windowed reading later counts by.
fn at(tick: u64, event: Event) -> RecordedEvent {
    RecordedEvent::new(tick, None, event)
}

fn born(organism: OrganismId, parent: Option<OrganismId>) -> Event {
    Event::Born {
        organism,
        species: SPECIES,
        parent,
    }
}

fn meal() -> Event {
    Event::Fed {
        eater: A,
        from: B,
        mass_mg: 40,
        kind: MealKind::Predation,
    }
}

#[test]
fn a_first_event_has_no_cause() {
    let mut history = History::new();
    let seq = history.record(at(1, born(A, None)));

    assert!(
        history.antecedents(seq).is_empty(),
        "nothing in this log led to it"
    );
    assert_eq!(history.latest(A), Some(seq));
}

#[test]
fn a_record_keeps_the_tick_it_happened_on() {
    // The envelope's whole job: a past with no tick cannot answer how many
    // died in the last two hundred ticks.
    let mut history = History::new();
    let seq = history.record(at(931, born(A, None)));
    assert_eq!(history.get(seq).map(|record| record.tick), Some(931));
    assert_eq!(history.event(seq), Some(&born(A, None)));
}

#[test]
fn a_creatures_events_form_its_line() {
    let mut history = History::new();
    let opening = history.record(at(1, born(A, None)));
    let matured = history.record(at(2, Event::Matured { organism: A }));
    let died = history.record(at(
        3,
        Event::Died {
            organism: A,
            species: SPECIES,
        },
    ));

    assert_eq!(history.antecedents(matured), vec![opening]);
    assert_eq!(
        history.antecedents(died),
        vec![matured, opening],
        "nearest first"
    );
    assert_eq!(history.line_of(A), vec![opening, matured, died]);
}

#[test]
fn independent_creatures_have_independent_lines() {
    // The property a flat log destroys: two creatures that never met are
    // concurrent, not ordered.
    let mut history = History::new();
    let a = history.record(at(1, born(A, None)));
    let b = history.record(at(1, born(B, None)));

    assert!(history.concurrent(a, b), "neither led to the other");
    assert!(history.consequences(a).is_empty());
}

#[test]
fn eating_joins_two_lines() {
    // The join that makes this a graph. Until they met, these creatures
    // had nothing to do with each other; afterwards, one's past is part of
    // the other's.
    let mut history = History::new();
    let a_born = history.record(at(1, born(A, None)));
    let b_born = history.record(at(1, born(B, None)));
    assert!(history.concurrent(a_born, b_born));

    let fed = history.record(at(9, meal()));

    assert_eq!(
        history.antecedents(fed),
        vec![a_born, b_born],
        "both lines are cited"
    );
    assert_eq!(history.consequences(a_born), vec![fed]);
    assert_eq!(
        history.consequences(b_born),
        vec![fed],
        "the eaten one led here too"
    );
}

#[test]
fn a_birth_descends_from_its_parent() {
    let mut history = History::new();
    let parent = history.record(at(1, born(A, None)));
    let child = history.record(at(40, born(B, Some(A))));

    assert_eq!(history.antecedents(child), vec![parent]);
    assert_eq!(history.consequences(parent), vec![child]);
    assert_eq!(
        history.latest(A),
        Some(child),
        "the parent's line continues through it"
    );
}

#[test]
fn consequences_reach_through_a_chain() {
    // The retroactive definition of significance: what a thing led to,
    // however far downstream.
    let mut history = History::new();
    history.record(at(1, born(A, None)));
    let b_born = history.record(at(1, born(B, None)));
    let fed = history.record(at(9, meal()));
    let grew = history.record(at(
        10,
        Event::Grew {
            organism: A,
            part: PartId(1),
        },
    ));

    assert_eq!(
        history.consequences(b_born),
        vec![fed, grew],
        "being eaten led, eventually, to somebody else's new limb"
    );
}

#[test]
fn severing_continues_the_line_it_happened_to() {
    let mut history = History::new();
    history.record(at(1, born(A, None)));
    let grew = history.record(at(
        2,
        Event::Grew {
            organism: A,
            part: PartId(1),
        },
    ));
    let lost = history.record(at(
        3,
        Event::Severed {
            organism: A,
            part: PartId(1),
        },
    ));

    assert!(
        history.antecedents(lost).contains(&grew),
        "you can only lose what you grew"
    );
}

#[test]
fn a_history_round_trips() {
    let mut history = History::new();
    history.record(at(1, born(A, None)));
    history.record(at(2, meal()));

    let bytes = crate::snapshot::encode(&history).unwrap();
    assert_eq!(crate::snapshot::decode::<History>(&bytes).unwrap(), history);
}

#[test]
fn every_variant_names_its_subjects() {
    // The links are built from `subjects`, so a variant that forgets one
    // silently forks that creature's history. Cheap to assert, expensive
    // to discover later.
    let all = [
        born(A, Some(B)),
        Event::Matured { organism: A },
        meal(),
        Event::Grew {
            organism: A,
            part: PartId(0),
        },
        Event::Burned {
            organism: A,
            energy_mg: 1,
        },
        Event::Severed {
            organism: A,
            part: PartId(0),
        },
        Event::Died {
            organism: A,
            species: SPECIES,
        },
        Event::Returned { organism: A },
        Event::Inhabited { organism: A },
    ];
    for event in all {
        assert!(!event.subjects().is_empty(), "{event:?} names nobody");
    }
}

/// The ending reading: the record's own event, and the tick off its envelope.
/// (DT2)
#[test]
fn an_ending_is_the_records_own_event_with_its_tick() {
    let mut history = History::new();
    history.record(at(1, born(A, None)));
    history.record(at(1, born(B, None)));

    // Still alive: this past holds no ending for either of them.
    assert_eq!(history.ending(A), None);
    assert_eq!(history.ending(B), None);

    history.record(at(
        812,
        Event::Died {
            organism: A,
            species: SPECIES,
        },
    ));
    assert_eq!(
        history.ending(A),
        Some(Ending {
            organism: A,
            tick: 812,
            how: Passing::Died,
        })
    );
    assert_eq!(history.ending(B), None, "somebody else's death is not B's");

    // Eaten to nothing leaves no corpse, and the record says so with the other
    // event.
    history.record(at(900, Event::Returned { organism: B }));
    assert_eq!(
        history.ending(B),
        Some(Ending {
            organism: B,
            tick: 900,
            how: Passing::Returned,
        })
    );

    // A corpse that finishes decaying returns after it died, and the later of
    // the two is the one that is true now.
    history.record(at(1_100, Event::Returned { organism: A }));
    assert_eq!(
        history.ending(A),
        Some(Ending {
            organism: A,
            tick: 1_100,
            how: Passing::Returned,
        })
    );
}

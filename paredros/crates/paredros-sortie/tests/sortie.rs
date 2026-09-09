// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S3's receipt: one sortie and return, the joint proof.
//!
//! The done-conditions, each as an assertion: the whole run replays to the
//! same hash; a tag-in occurs mid-action under the pre-agreed pact; an
//! injury persists as a body-revision fact; a post-sortie answer is
//! explained by a deed from the sortie; and the canary holds, watched on a
//! companion who refused and cannot be moved by anything.

use paredros_identity::{BodyRevisionId, ControlIntent, Tick};
use paredros_social::scene::{AUD, BRAM, ODRIS, SELA};
use paredros_social::{DeedKind, Premise, Verdict};
use paredros_sortie::scene;
use paredros_sortie::sortie::SortieEvent;

fn tick_of(event: &SortieEvent) -> Tick {
    match event {
        SortieEvent::Departed { at }
        | SortieEvent::Fell { at, .. }
        | SortieEvent::Wounded { at, .. }
        | SortieEvent::Downed { at, .. }
        | SortieEvent::PactInvoked { at, .. }
        | SortieEvent::TaggedIn { at, .. }
        | SortieEvent::Tended { at, .. }
        | SortieEvent::TaggedOut { at }
        | SortieEvent::Dug { at, .. }
        | SortieEvent::Took { at, .. }
        | SortieEvent::SharedOut { at, .. }
        | SortieEvent::Returned { at } => *at,
    }
}

#[test]
fn the_sortie_returns_and_replays_to_the_same_hash() {
    let (_, first) = scene::played_through();
    let (_, second) = scene::played_through();

    assert!(first.done(), "the sortie never came home");
    assert!(first.carried.is_some(), "nothing came back");
    assert_eq!(first.hash(), second.hash());
    assert_eq!(first.events, second.events);
    assert_eq!(first.control.log(), second.control.log());
    println!(
        "sortie hash {:#018x}, {} events, {} ticks, {} hewn",
        first.hash(),
        first.events.len(),
        first.tick().0 - scene::DEPART.0,
        first.hewn,
    );
}

#[test]
fn a_tag_in_occurs_mid_action_under_the_pact() {
    let (_, sortie) = scene::played_through();

    let invoked = sortie
        .events
        .iter()
        .find(|event| matches!(event, SortieEvent::PactInvoked { .. }))
        .expect("the pact never fired");
    let tagged_in = sortie
        .events
        .iter()
        .find_map(|event| match event {
            SortieEvent::TaggedIn { at, to } => Some((*at, *to)),
            _ => None,
        })
        .expect("nobody tagged in");
    let tagged_out = sortie
        .events
        .iter()
        .find_map(|event| match event {
            SortieEvent::TaggedOut { at } => Some(*at),
            _ => None,
        })
        .expect("nobody tagged back out");
    let returned = sortie
        .events
        .iter()
        .find_map(|event| match event {
            SortieEvent::Returned { at } => Some(*at),
            _ => None,
        })
        .expect("the sortie never returned");

    // Mid-action: after departure, before the return, and the rescue was a
    // walk rather than a reach from where she stood.
    assert!(tick_of(invoked).0 > scene::DEPART.0);
    assert!(
        tagged_out.0 > tagged_in.0.0,
        "the rescue took no time at all"
    );
    assert!(tagged_out.0 < returned.0);
    assert_eq!(tagged_in.1, SELA);

    // The control log carries the same story, and control came home.
    assert!(
        sortie
            .control
            .log()
            .iter()
            .any(|intent| matches!(intent, ControlIntent::TagIn { to, .. } if *to == SELA))
    );
    assert_eq!(sortie.control.played(), AUD);
    assert!(!sortie.control.tagged_in());

    // The tending exercised the pact's standing agreement and stood by the
    // fallen: both deeds are in the log, done by Sela to Aud.
    let deeds = sortie.society.log();
    assert!(deeds.deeds().iter().any(|deed| deed.doer == SELA
        && deed.toward == Some(AUD)
        && matches!(deed.kind, DeedKind::PerformedUnderAgreement(_))));
    assert!(deeds.deeds().iter().any(|deed| deed.doer == SELA
        && deed.toward == Some(AUD)
        && deed.kind == DeedKind::StoodBy));
}

#[test]
fn an_injury_persists_as_a_body_revision_fact() {
    let (_, sortie) = scene::played_through();

    let wound = sortie
        .wounds
        .iter()
        .find(|wound| wound.subject == AUD)
        .expect("the played body was never wounded");
    assert!(wound.fell > paredros_sortie::sortie::SAFE_FALL);
    assert_eq!(wound.revision, BodyRevisionId(1));

    // The facets wear the wounded revision after the sortie: the injury is
    // a fact about the body worn, not a status effect that expired on the
    // walk home.
    assert_eq!(sortie.facets.body_of(AUD), Some(wound.revision));

    // And the wound law is uniform: Bram took the same scarp and carries
    // his own revision, undowned.
    assert!(
        sortie
            .wounds
            .iter()
            .any(|wound| wound.subject == BRAM && wound.revision == BodyRevisionId(1))
    );
    assert!(
        !sortie
            .events
            .iter()
            .any(|event| matches!(event, SortieEvent::Downed { who, .. } if *who == BRAM)),
        "a companion downed; only the home body downs"
    );
}

#[test]
fn a_sortie_deed_explains_a_later_answer() {
    // The ask S2 saw counteroffered: danger 4, one past the 3 Sela would
    // carry. Before the sortie, the same answer again.
    let mut before = scene::settled_society();
    let asked = scene::ask_again(&mut before, scene::DEPART);
    assert!(matches!(asked.verdict, Verdict::Counteroffer(_)));

    // After the sortie, the same ask is accepted, and the premises cite
    // the share on the walk home: one deed from the sortie is exactly the
    // margin between "not for anyone" and "for you".
    let (_, sortie) = scene::played_through();
    let mut society = sortie.society;
    let shared_at = sortie
        .events
        .iter()
        .find_map(|event| match event {
            SortieEvent::SharedOut { at, with } if *with == SELA => Some(*at),
            _ => None,
        })
        .expect("the salvage was never shared with Sela");

    let answer = scene::ask_again(&mut society, Tick(sortie.events.len() as u64 + 200));
    assert_eq!(answer.verdict, Verdict::Accept);
    let cited = answer.cited_deeds();
    let sortie_deed = cited
        .iter()
        .find(|id| {
            let deed = society.log().get(**id).unwrap();
            deed.at == shared_at && deed.kind == DeedKind::Shared && deed.doer == AUD
        })
        .expect("the answer does not cite the sortie's share");
    let deed = society.log().get(*sortie_deed).unwrap();
    assert_eq!(deed.toward, Some(SELA));
}

#[test]
fn a_refusal_stands_and_nobody_is_ordered() {
    // The canary. Aud abandons Bram on the eve; Bram refuses the
    // expedition; the sortie goes without a scout and nothing in the crate
    // can put him on the march.
    let (answers, sortie) = scene::grudged();

    let bram = answers
        .iter()
        .find(|answer| answer.by == BRAM)
        .expect("Bram was asked");
    assert_eq!(bram.verdict, Verdict::Refuse);
    assert!(
        bram.premises
            .iter()
            .any(|premise| matches!(premise, Premise::TrustAsked { met: false, .. }))
    );

    // He stood at home the whole run: every trail row holds him at the
    // muster point.
    assert!(sortie.done(), "the sortie still comes home without a scout");
    let start = sortie.trail()[0].clone();
    for row in sortie.trail() {
        assert_eq!(row[1], start[1], "the refused companion moved");
    }

    // The one who never marched is never wounded and never cited.
    assert!(sortie.wounds.iter().all(|wound| wound.subject != BRAM));

    // And Odris, who holds no part at all, stands likewise.
    for row in sortie.trail() {
        assert_eq!(row[2], start[2], "a partless companion moved");
    }
    let _ = ODRIS;
}

#[test]
fn the_negotiation_is_real_and_the_agreement_closes() {
    let (answers, sortie) = scene::played_through();

    // Bram's part exists because he said yes, with premises.
    let bram = answers
        .iter()
        .find(|answer| answer.by == BRAM)
        .expect("Bram was asked");
    assert_eq!(bram.verdict, Verdict::Accept);
    assert!(!bram.premises.is_empty());

    // The expedition agreement was formed at departure and ended for
    // WorkDone at the return: a whole bounded arrangement inside one run.
    let outing = sortie
        .society
        .agreements()
        .find(|agreement| agreement.holder == BRAM && agreement.work == scene::OUTING)
        .expect("the expedition agreement exists");
    assert!(!outing.standing());
    let ended = sortie
        .society
        .log()
        .deeds()
        .iter()
        .any(|deed| deed.kind == DeedKind::AgreementEnded(outing.id));
    assert!(ended);
}

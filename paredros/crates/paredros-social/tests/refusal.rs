// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S1's receipt: three companions, one offer, three answers, and one standing
//! agreement lived through.
//!
//! The machine-checkable half of the gate's done-condition is here. Every
//! assertion about *why* someone answered is an assertion about the premises
//! they stated, not about the rule that produced them, so a rule that stopped
//! citing its evidence would fail even while giving the same answers.

use mesocosm_core::snapshot::{decode, encode, hash_bytes};
use paredros_identity::Tick;
use paredros_social::agreement::{AgreementChange, EndReason};
use paredros_social::scene::{self, AUD, BRAM, ODRIS, SELA, THE_THREE, THE_WORK};
use paredros_social::{
    Craft, DeedKind, Premise, Response, Ruling, RulingKind, SocialError, Society, Terms, Verdict,
    Work,
};

fn margin_of(response: &Response) -> i16 {
    response
        .premises
        .iter()
        .find_map(|premise| match premise {
            Premise::Confidence { margin, .. } => Some(*margin),
            _ => None,
        })
        .expect("every answer reports its own read of the work")
}

fn deed_kinds(society: &Society, response: &Response) -> Vec<DeedKind> {
    response
        .cited_deeds()
        .iter()
        .map(|id| society.log().get(*id).expect("a cited deed exists").kind)
        .collect()
}

#[test]
fn the_same_offer_gets_three_different_answers() {
    let mut society = scene::society();
    let answers = scene::answers(&mut society).unwrap();

    assert_eq!(answers.len(), 3);
    assert_eq!(answers[0].by, BRAM);
    assert_eq!(answers[0].verdict, Verdict::Accept);
    assert_eq!(answers[1].by, ODRIS);
    assert_eq!(answers[1].verdict, Verdict::Refuse);
    assert_eq!(answers[2].by, SELA);
    assert_eq!(answers[2].verdict, Verdict::Counteroffer(Terms::new(3, 3)));
}

#[test]
fn each_answer_names_the_deeds_that_differ() {
    let mut society = scene::society();
    let answers = scene::answers(&mut society).unwrap();

    // Nobody's evidence is anybody else's, and every cited deed is one Aud
    // actually did to the one citing it.
    for (index, answer) in answers.iter().enumerate() {
        let cited = answer.cited_deeds();
        assert!(
            !cited.is_empty(),
            "{} cited nothing",
            society.name_of(answer.by)
        );
        for id in &cited {
            let deed = society.log().get(*id).unwrap();
            assert_eq!(deed.doer, AUD);
            assert_eq!(deed.toward, Some(THE_THREE[index]));
        }
        for other in answers.iter().skip(index + 1) {
            for id in &cited {
                assert!(
                    !other.cited_deeds().contains(id),
                    "two answers rest on the same deed"
                );
            }
        }
    }

    let bram = deed_kinds(&society, &answers[0]);
    let odris = deed_kinds(&society, &answers[1]);
    let sela = deed_kinds(&society, &answers[2]);
    assert_eq!(bram, vec![DeedKind::StoodBy, DeedKind::StoodBy]);
    assert_eq!(odris, vec![DeedKind::Shared, DeedKind::Abandoned]);
    assert_eq!(
        sela,
        vec![DeedKind::Shared, DeedKind::Shared, DeedKind::StoodBy]
    );
}

#[test]
fn each_answer_names_the_check_that_decided_it() {
    let mut society = scene::society();
    let answers = scene::answers(&mut society).unwrap();

    let trust_met = |response: &Response| {
        response.premises.iter().find_map(|premise| match premise {
            Premise::TrustAsked { met, .. } => Some(*met),
            _ => None,
        })
    };
    let danger_borne = |response: &Response| {
        response.premises.iter().find_map(|premise| match premise {
            Premise::DangerWeighed { borne, .. } => Some(*borne),
            _ => None,
        })
    };

    // Bram passes both gates.
    assert_eq!(trust_met(&answers[0]), Some(true));
    assert_eq!(danger_borne(&answers[0]), Some(true));

    // Odris stops at trust, and the danger was never weighed, so it is never
    // claimed: premises are what decided it, not a full form.
    assert_eq!(trust_met(&answers[1]), Some(false));
    assert_eq!(danger_borne(&answers[1]), None);

    // Sela trusts Aud and will not carry that much for her anyway.
    assert_eq!(trust_met(&answers[2]), Some(true));
    assert_eq!(danger_borne(&answers[2]), Some(false));
}

#[test]
fn the_one_who_refuses_is_the_most_capable_of_the_three() {
    // Willingness is not competence. If the model ever collapses the two, this
    // is the assertion that notices.
    let mut society = scene::society();
    let answers = scene::answers(&mut society).unwrap();
    let margins: Vec<i16> = answers.iter().map(margin_of).collect();

    assert_eq!(answers[1].verdict, Verdict::Refuse);
    assert_eq!(margins, vec![1, 2, 0]);
    assert!(margins[1] > margins[0] && margins[1] > margins[2]);
}

#[test]
fn a_refusal_is_an_outcome_and_costs_the_refuser_nothing() {
    let mut society = scene::society();
    let answers = scene::answers(&mut society).unwrap();

    let refusal = society.log().get(answers[1].recorded).unwrap();
    assert_eq!(refusal.kind, DeedKind::OfferRefused);
    assert_eq!(refusal.doer, ODRIS);
    assert_eq!(refusal.kind.weight(), (0, 0));

    // And Aud's read of Odris is unmoved by having been told no.
    let before = scene::society().standing(AUD, ODRIS);
    assert_eq!(society.standing(AUD, ODRIS).trust, before.trust);
}

#[test]
fn a_capability_refusal_is_about_the_work_and_not_the_asker() {
    let mut society = scene::society();
    let smithing = Work::new(Craft::Smithing, 2, 1);
    let answer = society
        .consider(&scene::offer_to(BRAM).on(Terms::new(2, 5)), Tick(8))
        .unwrap();
    assert_eq!(answer.verdict, Verdict::Accept);

    let mut offer = scene::offer_to(BRAM);
    offer.work = smithing;
    let answer = society.consider(&offer, Tick(9)).unwrap();
    assert_eq!(answer.verdict, Verdict::Refuse);
    assert!(answer.cited_deeds().is_empty());
    assert!(matches!(
        answer.premises.as_slice(),
        [Premise::Confidence { margin: -2, .. }]
    ));
}

fn kind_of(rulings: &[Ruling], index: usize) -> RulingKind {
    rulings[index].kind
}

#[test]
fn the_standing_agreement_runs_its_whole_life() {
    let mut society = scene::society();
    scene::answers(&mut society).unwrap();
    let rulings = scene::lifecycle(&mut society).unwrap();
    assert_eq!(rulings.len(), 5);

    let RulingKind::Formed(id) = kind_of(&rulings, 0) else {
        panic!("the arrangement was not formed: {:?}", rulings[0].kind);
    };
    assert_eq!(kind_of(&rulings, 1), RulingKind::Performed);
    assert_eq!(
        kind_of(&rulings, 2),
        RulingKind::Renegotiated {
            from: Terms::new(2, 5),
            to: Terms::new(3, 3),
        }
    );
    assert_eq!(kind_of(&rulings, 3), RulingKind::OutsideTerms);
    assert_eq!(kind_of(&rulings, 4), RulingKind::Ended(EndReason::WorkDone));

    // Exercised once, and before any renegotiation: routine work inside the
    // arrangement went through on the terms already agreed.
    let agreement = society.agreement(id).unwrap();
    assert_eq!(agreement.exercises(), 1);
    let shape: Vec<AgreementChange> = agreement.history.iter().map(|event| event.what).collect();
    assert_eq!(
        shape,
        vec![
            AgreementChange::Formed,
            AgreementChange::Exercised,
            AgreementChange::Renegotiated {
                from: Terms::new(2, 5),
                to: Terms::new(3, 3)
            },
            AgreementChange::Ended(EndReason::WorkDone),
        ]
    );

    // Every transition is a deed, and every deed is the one the event points at.
    for event in &agreement.history {
        let deed = society.log().get(event.deed).unwrap();
        assert_eq!(deed.at, event.at);
        let expected = match event.what {
            AgreementChange::Formed => DeedKind::AgreementFormed(id),
            AgreementChange::Exercised => DeedKind::PerformedUnderAgreement(id),
            AgreementChange::Renegotiated { .. } => DeedKind::AgreementRenegotiated(id),
            AgreementChange::Ended(_) => DeedKind::AgreementEnded(id),
        };
        assert_eq!(deed.kind, expected);
    }

    // The changed term bit, and the ending carried its reason.
    assert!(rulings[3].premises.iter().any(|premise| matches!(
        premise,
        Premise::AgreementTerm {
            danger_cap: 3,
            covers: false,
            ..
        }
    )));
    assert!(rulings[4].premises.iter().any(|premise| matches!(
        premise,
        Premise::Ending {
            why: EndReason::WorkDone,
            ..
        }
    )));
    assert!(!rulings[4].cited_deeds().is_empty());

    assert_eq!(
        society.exercise(id, &THE_WORK, Tick(14)),
        Err(SocialError::AlreadyEnded(id))
    );
}

#[test]
fn a_standing_agreement_is_not_a_command() {
    // The canary, as a test. An arrangement makes routine work routine; it
    // does not make it compulsory, and a holder whose premises have changed
    // says so under one.
    let mut society = scene::society();
    let formed = society.form(&scene::offer_to(BRAM), Tick(9)).unwrap();
    let RulingKind::Formed(id) = formed.kind else {
        panic!("the arrangement was not formed");
    };

    let left = society.record(Tick(10), AUD, Some(BRAM), DeedKind::Abandoned);
    let ruling = society.exercise(id, &THE_WORK, Tick(11)).unwrap();

    assert_eq!(ruling.kind, RulingKind::Declined);
    assert_eq!(ruling.recorded, None);
    assert!(
        ruling.cited_deeds().contains(&left),
        "the decline did not name what changed"
    );
    assert!(
        ruling
            .premises
            .iter()
            .any(|premise| matches!(premise, Premise::TrustAsked { met: false, .. }))
    );
    assert!(society.agreement(id).unwrap().standing());
}

#[test]
fn the_same_history_gives_the_same_answers_and_the_same_premises() {
    let run = || {
        let mut society = scene::society();
        let answers = scene::answers(&mut society).unwrap();
        let rulings = scene::lifecycle(&mut society).unwrap();
        let bytes = encode(&society).expect("a society always encodes");
        (answers, rulings, hash_bytes(&bytes), society)
    };

    let (first_answers, first_rulings, first_hash, first) = run();
    let (second_answers, second_rulings, second_hash, second) = run();

    assert_eq!(first_answers, second_answers);
    assert_eq!(first_rulings, second_rulings);
    assert_eq!(first_hash, second_hash);
    assert_eq!(first, second);
    println!(
        "society hash {first_hash:#018x}, {} deeds",
        first.log().len()
    );
}

#[test]
fn a_society_survives_a_round_trip() {
    let mut society = scene::society();
    scene::answers(&mut society).unwrap();
    scene::lifecycle(&mut society).unwrap();

    let bytes = encode(&society).unwrap();
    let restored: Society = decode(&bytes).unwrap();
    assert_eq!(restored, society);
}

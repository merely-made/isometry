// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S2's receipt: the negotiated home.
//!
//! The gate's done-condition, machine-checkable half: a refusal and a
//! counteroffer occur for reasons traceable to premises and deeds, and an
//! accepted agreement changes where someone lives and what they do daily.
//! The residence assertions all read through [`Settlement::daily`] and
//! [`Settlement::home_of`], which are derived from agreement state, so the
//! move-out tests are really tests that no second copy of residence exists.

use mesocosm_core::snapshot::{decode, encode, hash_bytes};
use paredros_identity::Tick;
use paredros_social::agreement::EndReason;
use paredros_social::scene::{AUD, BRAM, ODRIS, SELA};
use paredros_social::settling::{self, THE_GATEHOUSE, THE_STILL_ROOM};
use paredros_social::{
    DailyRound, DeedKind, Premise, RulingKind, SettleError, Settlement, Society, Terms, Verdict,
};

fn settled() -> (Society, Settlement) {
    let mut society = settling::society();
    let mut settlement = settling::settlement();
    settling::answers(&mut society).unwrap();
    settling::housings(&mut society, &mut settlement).unwrap();
    (society, settlement)
}

#[test]
fn a_home_is_taken_a_home_is_refused_and_a_home_is_bargained() {
    let mut society = settling::society();
    let answers = settling::answers(&mut society).unwrap();

    assert_eq!(answers[0].by, BRAM);
    assert_eq!(answers[0].verdict, Verdict::Accept);
    assert_eq!(answers[1].by, ODRIS);
    assert_eq!(answers[1].verdict, Verdict::Refuse);
    assert_eq!(answers[2].by, SELA);
    assert_eq!(answers[2].verdict, Verdict::Counteroffer(Terms::new(3, 3)));

    // Odris's refusal is about Aud and provably not about the work: his
    // watching exactly meets the grade, the trust gate failed, and the
    // deciding evidence is the abandonment, cited by id.
    assert!(
        answers[1]
            .premises
            .iter()
            .any(|premise| matches!(premise, Premise::Confidence { margin: 0, .. }))
    );
    assert!(
        answers[1]
            .premises
            .iter()
            .any(|premise| matches!(premise, Premise::TrustAsked { met: false, .. }))
    );
    assert!(
        !answers[1]
            .premises
            .iter()
            .any(|premise| matches!(premise, Premise::DangerWeighed { .. })),
        "a gate never reached was claimed"
    );
    let cited = answers[1].cited_deeds();
    assert!(cited.iter().any(|id| {
        let deed = society.log().get(*id).unwrap();
        deed.doer == AUD && deed.toward == Some(ODRIS) && deed.kind == DeedKind::Abandoned
    }));

    // Sela's counter is the danger gate speaking: trust met, weight not.
    assert!(
        answers[2]
            .premises
            .iter()
            .any(|premise| matches!(premise, Premise::TrustAsked { met: true, .. }))
    );
    assert!(
        answers[2]
            .premises
            .iter()
            .any(|premise| matches!(premise, Premise::DangerWeighed { borne: false, .. }))
    );
}

#[test]
fn an_accepted_agreement_changes_where_someone_lives_and_what_they_do() {
    let mut society = settling::society();
    let mut settlement = settling::settlement();
    settling::answers(&mut society).unwrap();

    // Before: nobody lives anywhere and nobody has a daily round.
    for subject in [BRAM, ODRIS, SELA] {
        assert_eq!(settlement.daily(&society, subject), DailyRound::default());
    }

    settling::housings(&mut society, &mut settlement).unwrap();

    // After: the two who agreed live where they agreed and do what they
    // agreed, and the one who refused is exactly as he was.
    let bram = settlement.daily(&society, BRAM);
    assert_eq!(bram.home, Some(THE_GATEHOUSE));
    assert_eq!(bram.does.unwrap(), settling::BRAM_ASK);

    let sela = settlement.daily(&society, SELA);
    assert_eq!(sela.home, Some(THE_STILL_ROOM));
    // Her round is the bargained one, not the one first asked: danger 3
    // under her own cap, not the 4 she refused to carry.
    assert_eq!(sela.does.unwrap(), settling::sela_settled_ask().work);

    assert_eq!(settlement.daily(&society, ODRIS), DailyRound::default());

    // Every home traces to the bargain under it: the tenancy names a formed,
    // standing agreement whose formation is a deed in the log.
    for tenancy in settlement.tenancies() {
        let agreement = society.agreement(tenancy.under).unwrap();
        assert!(agreement.standing());
        assert_eq!(agreement.holder, tenancy.tenant);
        let formed = agreement.history[0];
        assert_eq!(
            society.log().get(formed.deed).unwrap().kind,
            DeedKind::AgreementFormed(tenancy.under)
        );
    }
}

#[test]
fn moving_out_is_the_agreement_ending() {
    let (mut society, mut settlement) = settled();
    let under = settlement
        .tenancies()
        .iter()
        .find(|tenancy| tenancy.tenant == SELA)
        .unwrap()
        .under;

    society
        .end(under, SELA, EndReason::Resigned, Tick(13))
        .unwrap();

    // No call was made on the settlement, and she has still moved out,
    // because residence is derived from the agreement rather than stored
    // beside it.
    assert_eq!(settlement.home_of(&society, SELA), None);
    assert_eq!(settlement.daily(&society, SELA), DailyRound::default());
    assert_eq!(settlement.tenant_of(&society, THE_STILL_ROOM), None);

    // Bram's roof does not move when Sela's does.
    assert_eq!(settlement.home_of(&society, BRAM), Some(THE_GATEHOUSE));

    // And the tenancy is history rather than gone: the line of tenants
    // stays readable after the tenant leaves.
    assert!(
        settlement
            .tenancies()
            .iter()
            .any(|tenancy| tenancy.tenant == SELA && tenancy.dwelling == THE_STILL_ROOM)
    );

    // The vacated dwelling can be offered again.
    let again = settlement
        .offer_home(
            &mut society,
            &settling::sela_settled_ask(),
            THE_STILL_ROOM,
            Tick(14),
        )
        .unwrap();
    assert!(matches!(again.kind, RulingKind::Formed(_)));
    assert_eq!(settlement.tenant_of(&society, THE_STILL_ROOM), Some(SELA));
}

#[test]
fn a_standing_home_refuses_a_second_tenant() {
    let (mut society, mut settlement) = settled();

    // Occupancy is checked before anyone weighs anything: no answer is asked
    // of Sela for a roof that is not free to offer.
    let deeds_before = society.log().len();
    assert_eq!(
        settlement.offer_home(
            &mut society,
            &settling::sela_settled_ask(),
            THE_GATEHOUSE,
            Tick(13),
        ),
        Err(SettleError::Occupied(THE_GATEHOUSE, BRAM))
    );
    assert_eq!(society.log().len(), deeds_before);
}

#[test]
fn a_home_nobody_agreed_to_cannot_be_created() {
    let mut society = settling::society();
    let mut settlement = settling::settlement();

    // Odris's ask, put as a housing offer directly: he refuses, the ruling
    // says declined, and no tenancy exists to show for it.
    let ruling = settlement
        .offer_home(
            &mut society,
            &settling::ask_of(ODRIS),
            THE_STILL_ROOM,
            Tick(8),
        )
        .unwrap();
    assert_eq!(ruling.kind, RulingKind::Declined);
    assert!(settlement.tenancies().is_empty());
    assert_eq!(settlement.home_of(&society, ODRIS), None);
}

#[test]
fn the_settling_replays_to_the_same_homes() {
    let run = || {
        let mut society = settling::society();
        let mut settlement = settling::settlement();
        let answers = settling::answers(&mut society).unwrap();
        settling::housings(&mut society, &mut settlement).unwrap();
        let bytes = encode(&(&society, &settlement)).expect("the pair always encodes");
        (answers, hash_bytes(&bytes), society, settlement)
    };

    let (first_answers, first_hash, first_society, first_settlement) = run();
    let (second_answers, second_hash, second_society, second_settlement) = run();

    assert_eq!(first_answers, second_answers);
    assert_eq!(first_hash, second_hash);
    assert_eq!(first_society, second_society);
    assert_eq!(first_settlement, second_settlement);
    println!(
        "settlement hash {first_hash:#018x}, {} deeds, {} tenancies",
        first_society.log().len(),
        first_settlement.tenancies().len()
    );
}

#[test]
fn a_settlement_survives_a_round_trip() {
    let (society, settlement) = settled();
    let restored: Settlement = decode(&encode(&settlement).unwrap()).unwrap();
    assert_eq!(restored, settlement);
    // And the derived reads agree after the trip, since they are functions
    // of the two states rather than caches inside either.
    assert_eq!(
        restored.daily(&society, BRAM),
        settlement.daily(&society, BRAM)
    );
}

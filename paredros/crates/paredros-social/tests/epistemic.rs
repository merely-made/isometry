// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! F3a receipt: pointable, actor-scoped claims about accepted deeds.

use mesocosm_core::snapshot::{decode, encode, hash_bytes};
use paredros_identity::{SubjectId, Tick};
use paredros_social::{
    ClaimHistory, ClaimId, DeedId, DeedKind, DeedLog, EpistemicEntry, EpistemicError, EpistemicId,
    EpistemicLog, EpistemicRecord, EventReading, Evidence, ObservationId, Proposition, ReportId,
};

const AUD: SubjectId = SubjectId(1);
const BRAM: SubjectId = SubjectId(2);
const SELA: SubjectId = SubjectId(3);
const HELPED: EventReading = EventReading::Helped;
const BETRAYED: EventReading = EventReading::Betrayed;

fn deeds() -> (DeedLog, DeedId) {
    let mut deeds = DeedLog::new();
    let event = deeds.record(Tick(5), AUD, Some(BRAM), DeedKind::StoodBy);
    (deeds, event)
}

fn proposition(event: DeedId, reading: EventReading) -> Proposition {
    Proposition { event, reading }
}

fn history() -> (DeedLog, EpistemicLog, DeedId, ClaimId) {
    let (deeds, event) = deeds();
    let helped = proposition(event, HELPED);
    let betrayed = proposition(event, BETRAYED);
    let mut log = EpistemicLog::new();
    let aud_saw = log.observe(&deeds, Tick(6), AUD, event).unwrap();
    let claim = log
        .claim(Tick(7), AUD, helped, &[Evidence::Observation(aud_saw)])
        .unwrap();
    log.report(Tick(8), AUD, SELA, claim, helped).unwrap();
    log.correct(
        Tick(8),
        AUD,
        claim,
        betrayed,
        &[Evidence::Observation(aud_saw)],
    )
    .unwrap();
    (deeds, log, event, claim)
}

#[test]
fn two_witnesses_keep_supported_readings_and_a_report_does_not_believe_for_its_hearer() {
    let (deeds, event) = deeds();
    let helped = proposition(event, HELPED);
    let betrayed = proposition(event, BETRAYED);
    let mut log = EpistemicLog::new();
    let aud_saw = log.observe(&deeds, Tick(6), AUD, event).unwrap();
    let bram_saw = log.observe(&deeds, Tick(6), BRAM, event).unwrap();
    let aud_claim = log
        .claim(Tick(7), AUD, helped, &[Evidence::Observation(aud_saw)])
        .unwrap();
    let bram_claim = log
        .claim(Tick(7), BRAM, betrayed, &[Evidence::Observation(bram_saw)])
        .unwrap();
    let report = log.report(Tick(8), AUD, SELA, aud_claim, helped).unwrap();

    assert_eq!(log.beliefs_of(AUD)[0].claim, aud_claim);
    assert_eq!(log.beliefs_of(BRAM)[0].claim, bram_claim);
    assert_eq!(log.beliefs_of(AUD)[0].proposition, helped);
    assert_eq!(log.beliefs_of(BRAM)[0].proposition, betrayed);
    assert_eq!(log.received_by(SELA).count(), 1);
    assert!(log.beliefs_of(SELA).is_empty());

    let sela_claim = log
        .claim(Tick(9), SELA, helped, &[Evidence::Report(report)])
        .unwrap();
    assert_eq!(log.beliefs_of(SELA)[0].claim, sela_claim);
}

#[test]
fn observations_require_real_deeds_at_or_after_their_tick() {
    let (deeds, event) = deeds();
    let mut log = EpistemicLog::new();
    let unchanged = log.clone();
    assert_eq!(
        log.observe(&deeds, Tick(5), AUD, DeedId(99)),
        Err(EpistemicError::UnknownEvent(DeedId(99)))
    );
    assert_eq!(
        log.observe(&deeds, Tick(4), AUD, event),
        Err(EpistemicError::ObservationBeforeEvent {
            event,
            deed_at: Tick(5),
            observation_at: Tick(4)
        })
    );
    assert_eq!(log, unchanged);
    assert_eq!(
        log.observe(&deeds, Tick(5), AUD, event),
        Ok(ObservationId(0))
    );
}

#[test]
fn evidence_is_owned_by_the_claimant_and_report_propositions_are_exact() {
    let (deeds, event) = deeds();
    let helped = proposition(event, HELPED);
    let betrayed = proposition(event, BETRAYED);
    let mut log = EpistemicLog::new();
    let aud_saw = log.observe(&deeds, Tick(6), AUD, event).unwrap();
    let before_foreign_claim = log.clone();
    assert_eq!(
        log.claim(Tick(7), BRAM, helped, &[Evidence::Observation(aud_saw)]),
        Err(EpistemicError::ForeignEvidence(Evidence::Observation(
            aud_saw
        )))
    );
    assert_eq!(log, before_foreign_claim);
    let claim = log
        .claim(Tick(7), AUD, helped, &[Evidence::Observation(aud_saw)])
        .unwrap();
    assert_eq!(claim, ClaimId(0));
    let report = log.report(Tick(8), AUD, SELA, claim, helped).unwrap();
    let before_bad_report_evidence = log.clone();
    assert_eq!(
        log.claim(Tick(9), BRAM, helped, &[Evidence::Report(report)]),
        Err(EpistemicError::ForeignEvidence(Evidence::Report(report)))
    );
    assert_eq!(
        log.claim(Tick(9), SELA, betrayed, &[Evidence::Report(report)]),
        Err(EpistemicError::MismatchedSupport(Evidence::Report(report)))
    );
    assert_eq!(log, before_bad_report_evidence);
}

#[test]
fn reports_require_a_real_transmission_chain_and_keep_its_proposition() {
    let (deeds, event) = deeds();
    let helped = proposition(event, HELPED);
    let betrayed = proposition(event, BETRAYED);
    let mut log = EpistemicLog::new();
    let aud_saw = log.observe(&deeds, Tick(6), AUD, event).unwrap();
    let claim = log
        .claim(Tick(7), AUD, helped, &[Evidence::Observation(aud_saw)])
        .unwrap();
    let before_rejections = log.clone();
    assert_eq!(
        log.report(Tick(8), BRAM, SELA, claim, helped),
        Err(EpistemicError::CannotReport {
            reporter: BRAM,
            claim,
            proposition: helped
        })
    );
    assert_eq!(
        log.report(Tick(8), AUD, SELA, claim, betrayed),
        Err(EpistemicError::CannotReport {
            reporter: AUD,
            claim,
            proposition: betrayed
        })
    );
    assert_eq!(log, before_rejections);
    assert_eq!(
        log.report(Tick(8), AUD, SELA, claim, helped),
        Ok(ReportId(0))
    );
    assert_eq!(
        log.report(Tick(9), SELA, BRAM, claim, helped),
        Ok(ReportId(1))
    );
}

#[test]
fn only_the_claimant_can_change_the_current_fold_without_erasing_history() {
    let (deeds, log, event, claim) = history();
    let helped = proposition(event, HELPED);
    let betrayed = proposition(event, BETRAYED);
    let mut log = log;
    let aud_saw = ObservationId(0);
    let before_rejections = log.clone();
    assert_eq!(
        log.correct(
            Tick(10),
            BRAM,
            claim,
            betrayed,
            &[Evidence::Observation(aud_saw)]
        ),
        Err(EpistemicError::NotClaimant {
            corrector: BRAM,
            claim
        })
    );
    assert_eq!(
        log.correct(
            Tick(10),
            AUD,
            claim,
            betrayed,
            &[Evidence::Observation(aud_saw)]
        ),
        Err(EpistemicError::UnchangedCorrection(claim))
    );
    assert_eq!(log, before_rejections);
    let belief = &log.beliefs_of(AUD)[0];
    assert_eq!(belief.proposition, betrayed);
    assert_eq!(belief.revision, EpistemicId(3));
    assert_eq!(
        log.history_of(claim).unwrap(),
        ClaimHistory {
            claim,
            reports: vec![ReportId(0)],
            corrections: vec![EpistemicId(3)]
        }
    );
    assert!(
        matches!(&log.entries()[1].record, EpistemicRecord::Claim { proposition, .. } if *proposition == helped)
    );
    drop(deeds);
}

#[test]
fn decoded_entries_replay_through_validation_and_preserve_same_tick_order() {
    let (deeds, log, event, claim) = history();
    let bytes = encode(&log.entries().to_vec()).unwrap();
    let entries: Vec<EpistemicEntry> = decode(&bytes).unwrap();
    assert_eq!(entries[2].at, entries[3].at);
    assert!(entries[2].id < entries[3].id);
    let replayed = EpistemicLog::replay(&deeds, &entries).unwrap();
    assert_eq!(replayed, log);
    assert_eq!(replayed.beliefs_of(AUD), log.beliefs_of(AUD));
    assert_eq!(
        hash_bytes(&encode(&replayed.entries().to_vec()).unwrap()),
        hash_bytes(&bytes)
    );

    let mut tampered = entries;
    let EpistemicRecord::Correction { replacement, .. } = &mut tampered[3].record else {
        panic!("expected correction")
    };
    *replacement = proposition(event, HELPED);
    assert_eq!(
        EpistemicLog::replay(&deeds, &tampered),
        Err(EpistemicError::UnchangedCorrection(claim))
    );
}

#[test]
fn replay_refuses_reidentified_entries_and_specialized_ids() {
    let (deeds, log, _, _) = history();
    let mut entries = log.entries().to_vec();
    entries[0].id = EpistemicId(1);
    assert_eq!(
        EpistemicLog::replay(&deeds, &entries),
        Err(EpistemicError::ReplayEntryId {
            expected: EpistemicId(0),
            found: EpistemicId(1)
        })
    );

    let mut entries = log.entries().to_vec();
    let EpistemicRecord::Observation { observation, .. } = &mut entries[0].record else {
        panic!("expected observation")
    };
    *observation = ObservationId(4);
    assert_eq!(
        EpistemicLog::replay(&deeds, &entries),
        Err(EpistemicError::ReplayObservationId {
            expected: ObservationId(0),
            found: ObservationId(4)
        })
    );
}

#[test]
fn replay_refuses_support_lists_that_would_be_silently_normalized() {
    let (deeds, log, _, _) = history();
    let mut entries = log.entries().to_vec();
    let EpistemicRecord::Claim { supports, .. } = &mut entries[1].record else {
        panic!("expected claim")
    };
    supports.push(supports[0]);
    assert_eq!(
        EpistemicLog::replay(&deeds, &entries),
        Err(EpistemicError::ReplayRecordMismatch(EpistemicId(1)))
    );
}

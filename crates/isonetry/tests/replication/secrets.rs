//! W0: a GM-only fact stays host-side until it is revealed.
//!
//! A peer's snapshot bytes provably contain no unrevealed secret, a reveal
//! publishes it to every journal as an ordinary logged event, and a restored
//! host reconciles whatever was in flight.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

/// The W0 done-condition (worldbuilding plan, decision 8): a GM-only
/// fact lives host-side only, a peer's snapshot bytes provably contain
/// no unrevealed secret, a reveal publishes it to every journal as an
/// ordinary logged event, and a client cannot fabricate a fact.
#[test]
fn secrets_stay_host_side_until_revealed() {
    use isometry_campaign::{RevealCondition, SecretFact};

    const SECRET_TEXT: &str = "OATHBOUND-RIVER-SECRET";

    let mut host = HostSession::new(snapshot());
    // The GM layer belongs to the host session, never inside its snapshot.
    host.campaign_mut().insert_secret(SecretFact {
        id: "sword-01.curse".to_owned(),
        text: SECRET_TEXT.to_owned(),
        tags: vec!["item:sword-01".to_owned()],
        reveal: RevealCondition::Identify,
    });

    let mut sim = Sim::new(host);
    sim.connect(PeerId(10));
    sim.connect(PeerId(11));

    // Unrevealed: no peer's replicated state carries the secret, byte-wise.
    let needle = SECRET_TEXT.as_bytes();
    for (peer, client) in &sim.clients {
        let bytes = postcard::to_allocvec(client.state().unwrap()).unwrap();
        assert!(
            !bytes.windows(needle.len()).any(|w| w == needle),
            "client {peer:?} snapshot bytes contain the unrevealed secret"
        );
        assert!(client.state().unwrap().journal.is_empty());
    }

    // A client cannot make something true by proposing it.
    sim.client_intent(
        PeerId(10),
        GameEvent::Fact(isometry_campaign::WorldFact {
            id: "forged".to_owned(),
            kind: "reveal".to_owned(),
            text: "the king owes me gold".to_owned(),
            tags: Vec::new(),
        }),
    );
    assert!(sim.host.state().journal.is_empty());
    assert_converged(&sim);

    // The DM reveals through the host transaction. The private record only
    // finalizes after its public fact commits.
    sim.host_reveal_secret("sword-01.curse")
        .expect("secret exists");
    assert!(
        sim.host.campaign().is_empty(),
        "revealed fact left the GM layer"
    );

    assert_eq!(sim.host.state().journal.len(), 1);
    assert_eq!(sim.host.state().journal[0].text, SECRET_TEXT);
    for client in sim.clients.values() {
        assert_eq!(client.state().unwrap().journal.len(), 1);
        assert_eq!(client.state().unwrap().journal[0].kind, "reveal");
    }
    assert_converged(&sim);
}

#[test]
fn failed_reveal_restores_the_private_secret() {
    use isometry_campaign::{RevealCondition, SecretFact, WorldFact};

    let mut host = HostSession::new(snapshot());
    host.local_event(GameEvent::Fact(WorldFact {
        id: "sword-01.curse".to_owned(),
        kind: "history".to_owned(),
        text: "A different public fact already owns this id.".to_owned(),
        tags: Vec::new(),
    }));
    host.campaign_mut().insert_secret(SecretFact {
        id: "sword-01.curse".to_owned(),
        text: "The sword is cursed.".to_owned(),
        tags: Vec::new(),
        reveal: RevealCondition::Manual,
    });

    assert!(host.reveal_secret("sword-01.curse").is_err());
    assert!(host.campaign().secret("sword-01.curse").is_some());
    assert!(host.campaign().pending_reveal("sword-01.curse").is_none());
}

#[test]
fn restored_host_reconciles_a_pending_reveal() {
    use isometry_campaign::{CampaignStore, RevealCondition, SecretFact};

    let mut campaign = CampaignStore::new();
    campaign.insert_secret(SecretFact {
        id: "sword-01.curse".to_owned(),
        text: "The sword is cursed.".to_owned(),
        tags: Vec::new(),
        reveal: RevealCondition::Manual,
    });
    campaign
        .begin_reveal("sword-01.curse")
        .expect("secret exists");

    let mut host = HostSession::with_campaign(snapshot(), campaign);
    let out = host
        .reconcile_pending_reveals()
        .expect("reconciles pending fact");
    assert_eq!(out.len(), 1);
    assert_eq!(host.state().journal.len(), 1);
    assert!(host.campaign().is_empty());
}

#[test]
fn restored_history_rebuilds_sequence_and_convergence_hash() {
    use isometry_campaign::CampaignStore;

    let mut host = HostSession::new(snapshot());
    host.local_event(mv(1, (2, 1)));
    host.local_event(GameEvent::TurnAdd(TokenId(1)));
    let state = host.state().clone();
    let history = host.history().clone();
    let hash = host.log_hash();

    let mut restored = HostSession::with_history(state, CampaignStore::new(), history);
    assert_eq!(restored.seq(), 2);
    assert_eq!(restored.log_hash(), hash);

    restored.local_event(GameEvent::TurnAdvance);
    assert_eq!(restored.seq(), 3);
    assert_eq!(restored.history().len(), 3);
}

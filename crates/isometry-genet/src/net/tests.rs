use super::*;
use isometry_campaign::{GenValue, GeneratorRequest};
use isometry_core::{MapDocument, TurnList};
use std::collections::BTreeMap;
use std::time::Instant;

const ACTOR_DEADLINE: Duration = Duration::from_secs(30);

#[test]
fn campaign_party_context_crosses_the_actor_bridge() {
    let mut bridge = NetBridge::spawn(
        Role::Host {
            state: snapshot(),
            campaign: CampaignStore::new(),
            history: Journal::new(),
        },
        std::sync::Arc::new(|| {}),
    );
    wait_for(&mut bridge, "host readiness", |bridge| {
        bridge.latest().is_some()
    });
    let catalog = isometry_system::GeneratorCatalog::discover([std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    ))
    .join("../isometry-system/examples/packs/watchtower")]);
    let mut tape = isometry_campaign::EntropyTape::from_seed(91);
    let record = catalog
        .generate(
            "party-start".to_owned(),
            &GeneratorRequest {
                generator: "watchtower:ruined_tower".to_owned(),
                args: GenValue::Text {
                    value: "ash-and-bells".to_owned(),
                },
                locks: BTreeMap::new(),
            },
            &mut tape,
            isometry_system::GeneratorLimits::default(),
        )
        .expect("bundled watchtower draft");
    bridge
        .commit_campaign(record, None, Some("player".to_owned()))
        .unwrap();
    wait_for(&mut bridge, "party placement", |bridge| {
        bridge.latest().is_some_and(|state| {
            state.world.party_at("player") == Some("watchtower:ruined-watchtower")
        })
    });
    let state = bridge.latest().unwrap();
    assert_eq!(state.world.party_node.len(), 1);
    assert_eq!(state.world.overmap_for("player").nodes.len(), 2);
    bridge.submit(GameEvent::Map(isometry_core::SessionEvent::TokenMoved {
        id: TokenId(1),
        to: (1, 7),
    }));
    bridge
        .travel(
            TokenId(1),
            isonetry::RequestId::host(901),
            "player".to_owned(),
        )
        .expect("travel command queued after the move");
    wait_for(&mut bridge, "regional arrival", |bridge| {
        bridge.latest().is_some_and(|state| {
            state.active_map.as_deref() == Some("watchtower:forest-region")
                && state.world.party_at("player") == Some("watchtower:forest-region")
        })
    });
    let state = bridge.latest().unwrap();
    assert_eq!(state.world.overmap_for("player").nodes.len(), 4);
    assert_eq!(state.world.party_node.len(), 1);
}

fn wait_for(bridge: &mut NetBridge, what: &str, mut ready: impl FnMut(&NetBridge) -> bool) {
    let deadline = Instant::now() + ACTOR_DEADLINE;
    loop {
        bridge.poll();
        if ready(bridge) {
            return;
        }
        if let Some(error) = bridge.take_failure() {
            panic!("{what} failed: {error}");
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn snapshot() -> GameSnapshot {
    GameSnapshot {
        map: MapDocument::new("bridge", 2, 2),
        turns: TurnList::new(),
        roll_log: Vec::new(),
        journal: Vec::new(),
        inventories: Default::default(),
        generations: Vec::new(),
        maps: Default::default(),
        active_map: None,
        world: Default::default(),
        clocks: Default::default(),

        party_cap: isonetry::default_party_cap(),
        last_beats: Vec::new(),
        beat_seq: 0,
        applied_actions: Default::default(),
    }
}

#[test]
fn host_bridge_delivers_actor_state_to_the_kernel() {
    let mut bridge = NetBridge::spawn(
        Role::Host {
            state: snapshot(),
            campaign: CampaignStore::new(),
            history: Journal::new(),
        },
        // The windowless test drives `poll` itself, so the wake has nowhere
        // to go: the event loop it would schedule a turn on does not exist.
        std::sync::Arc::new(|| {}),
    );

    wait_for(&mut bridge, "host actor readiness", |bridge| {
        bridge.ticket().is_some() && bridge.latest().is_some()
    });
    assert!(
        bridge.ticket().is_some(),
        "host actor bound and published a ticket"
    );
    assert_eq!(bridge.latest(), Some(snapshot()));

    let version = bridge.version();
    bridge.submit(GameEvent::TurnAdvance);
    wait_for(&mut bridge, "host command actor update", |bridge| {
        bridge.version() > version
    });
}

#[test]
fn rejected_campaign_is_correlated_without_failing_the_actor() {
    let mut bridge = NetBridge::spawn(
        Role::Host {
            state: snapshot(),
            campaign: CampaignStore::new(),
            history: Journal::new(),
        },
        // The windowless test drives `poll` itself, so the wake has nowhere
        // to go: the event loop it would schedule a turn on does not exist.
        std::sync::Arc::new(|| {}),
    );
    for _ in 0..100 {
        bridge.poll();
        if bridge.ticket().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let record = GenerationRecord {
        id: "not-a-campaign".to_owned(),
        request: GeneratorRequest {
            generator: "demo:text".to_owned(),
            args: GenValue::Text {
                value: "text".to_owned(),
            },
            locks: BTreeMap::new(),
        },
        entropy: 1,
        proposal: GenValue::Text {
            value: "text".to_owned(),
        },
    };
    let request = bridge
        .commit_campaign(record, None, None)
        .expect("actor accepts the command");

    for _ in 0..100 {
        bridge.poll();
        let outcomes = bridge.take_campaign_outcomes();
        if let Some(outcome) = outcomes.into_iter().next() {
            assert_eq!(outcome.request, request);
            assert!(outcome.value.is_err());
            assert!(bridge.take_failure().is_none());
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("campaign outcome did not return through the actor update channel");
}

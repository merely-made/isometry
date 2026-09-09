//! Campaign installation with an explicit, replicated party start.

use super::*;

fn campaign(start_places: usize) -> GenerationRecord {
    let mut world = CampaignWorld::default();
    for index in 0..start_places {
        let id = if index == 0 {
            "tower".to_owned()
        } else {
            format!("tower-{index}")
        };
        world.places.insert(
            id.clone(),
            WorldPlace {
                id,
                name: "Ash-Bell Watchtower".into(),
                tags: vec!["ruin".into()],
                map: Some("watchtower".into()),
                position: None,
            },
        );
    }
    world.places.insert(
        "forest".into(),
        WorldPlace {
            id: "forest".into(),
            name: "Bellwood Reach".into(),
            tags: vec!["forest".into()],
            map: Some("forest-map".into()),
            position: None,
        },
    );
    if start_places > 0 {
        world.routes.insert(
            "road".into(),
            WorldRoute {
                id: "road".into(),
                from: "tower".into(),
                to: "forest".into(),
                tags: vec!["road".into()],
                weight: 1,
            },
        );
    }
    world.storylets.insert(
        "finale".into(),
        StoryletProposal {
            key: "finale".into(),
            entry: "Ring the bell".into(),
            tags: vec![],
            requirements: Default::default(),
            roles: vec![],
            effects: vec![],
        },
    );
    GenerationRecord {
        id: "generated.watchtower.start".into(),
        request: GeneratorRequest {
            generator: "watchtower:ruined_tower".into(),
            args: GenValue::Text {
                value: "ash-and-bells".into(),
            },
            locks: BTreeMap::new(),
        },
        entropy: 91,
        proposal: GenValue::Campaign {
            campaign: CampaignDraft {
                id: "watchtower".into(),
                name: "Ash-Bell Watchtower".into(),
                world,
                maps: vec![DraftMap {
                    scale: MapScale::Local,
                    map: LocalMapProposal {
                        id: "watchtower".into(),
                        name: "Ash-Bell Watchtower".into(),
                        width: 3,
                        height: 3,
                        default_ground: "grass".into(),
                        cells: vec![],
                        spawn_zones: vec![],
                        transitions: vec![],
                        encounter_anchors: vec![],
                    },
                    inhabitants: vec![],
                }],
                secrets: vec![],
                rewards: vec![],
                starting_map: "watchtower".into(),
                final_storylet: "finale".into(),
            },
        },
    }
}

#[test]
fn campaign_start_places_and_reveals_the_named_party_over_the_existing_wire() {
    let mut host = HostSession::new(snapshot());
    host.local_event(GameEvent::World(WorldEvent::PartyMoved {
        party: "dm".into(),
        node: "before".into(),
    }));
    let mut client = ClientSession::new();
    let snapshot = host.on_connect(PeerId(41)).pop().unwrap().1;
    assert!(client.on_message(snapshot).is_empty());

    let out = host
        .commit_campaign_for_party(campaign(1), None, "forest-walker")
        .unwrap();
    for (_, message) in out {
        let json = serde_json::to_string(&message).unwrap();
        let from_json: NetMessage = serde_json::from_str(&json).unwrap();
        let bytes = postcard::to_allocvec(&from_json).unwrap();
        let from_wire: NetMessage = postcard::from_bytes(&bytes).unwrap();
        assert!(client.on_message(from_wire).is_empty());
    }

    let world = &host.state().world;
    assert_eq!(world.party_at("forest-walker"), Some("tower"));
    assert!(world.knows("forest-walker", "tower"));
    assert!(world.knows("forest-walker", "forest"));
    assert_eq!(world.party_at("dm"), Some("before"));
    assert!(world.knows("dm", "before"));
    assert_eq!(client.state(), Some(host.state()));
    assert_eq!(client.log_hash(), host.log_hash());
}

#[test]
fn campaign_start_preserves_the_exact_party_identity() {
    let mut host = HostSession::new(snapshot());
    host.commit_campaign_for_party(campaign(1), None, " forest-walker ")
        .unwrap();
    assert_eq!(
        host.state().world.party_at(" forest-walker "),
        Some("tower")
    );
    assert_eq!(host.state().world.party_at("forest-walker"), None);
}

#[test]
fn rejected_campaign_start_keeps_state_history_and_private_store_unchanged() {
    for (record, party) in [
        (campaign(1), "   "),
        (campaign(0), "forest-walker"),
        (campaign(2), "forest-walker"),
    ] {
        let mut host = HostSession::new(snapshot());
        let state = host.state().clone();
        let private = host.campaign().clone();
        let history = host.history().clone();
        let seq = host.seq();
        let hash = host.log_hash();

        assert!(host.commit_campaign_for_party(record, None, party).is_err());
        assert_eq!(host.state(), &state);
        assert_eq!(host.campaign(), &private);
        assert_eq!(host.history(), &history);
        assert_eq!(host.seq(), seq);
        assert_eq!(host.log_hash(), hash);
    }
}

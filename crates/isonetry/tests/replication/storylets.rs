//! Storylets and campaign drafts: private fact in, public commit out.
//!
//! A storylet matches on facts the players cannot see, casts an existing role,
//! and commits its effects to every journal. A campaign commit applies its
//! public draft while its secrets stay host-side.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn storylet_matches_private_fact_casts_existing_role_and_commits_effects() {
    let mut host = HostSession::new(snapshot());
    host.campaign_mut().insert_secret(SecretFact {
        id: "ford.secret".into(),
        text: "The ford remembers a drowned oath.".into(),
        tags: vec!["river".into()],
        reveal: RevealCondition::Manual,
    });
    host.local_event(GameEvent::World(WorldEvent::Faction(WorldFaction {
        id: "tide".into(),
        name: "Tide Court".into(),
        tags: vec!["river".into()],
        claims: vec![],
    })));
    host.local_event(GameEvent::World(WorldEvent::Character(WorldCharacter {
        id: "mara".into(),
        name: "Mara".into(),
        tags: vec!["warden".into()],
        faction: Some("tide".into()),
        place: None,
    })));
    host.local_event(GameEvent::World(WorldEvent::Law(WorldLaw {
        id: "iron-remembers".into(),
        name: "Iron remembers".into(),
        text: "Iron keeps its maker's name.".into(),
        tags: vec!["magic".into()],
        parameters: BTreeMap::new(),
    })));
    let encounter = LocalMapProposal {
        id: "oath-encounter".into(),
        name: "Drowned Ford".into(),
        width: 3,
        height: 3,
        default_ground: "water".into(),
        cells: vec![],
        spawn_zones: vec![],
        transitions: vec![],
        encounter_anchors: vec![],
    };
    host.local_event(GameEvent::World(WorldEvent::Storylet(StoryletProposal {
        key: "drowned-oath".into(),
        entry: "The drowned oath surfaces.".into(),
        tags: vec!["encounter".into()],
        requirements: StoryletRequirements {
            faction_tags: vec!["river".into()],
            hidden_facts: vec!["ford.secret".into()],
            world_laws: vec!["iron-remembers".into()],
        },
        roles: vec![RoleSlot {
            key: "warden".into(),
            tags: vec!["warden".into()],
        }],
        effects: vec![
            StoryletEffect::History {
                event: HistoryEvent {
                    id: "oath-returned".into(),
                    time: 4,
                    kind: "omen".into(),
                    text: "The oath returned.".into(),
                    participants: vec!["mara".into()],
                    place: None,
                    tags: vec![],
                },
            },
            StoryletEffect::Item {
                item: ItemProposal {
                    template: "demo:oath-blade".into(),
                    name: "Oath Blade".into(),
                    tags: vec!["weapon".into()],
                },
            },
            StoryletEffect::LocalMap { map: encounter },
            StoryletEffect::Fact {
                fact: WorldFact {
                    id: "oath.public".into(),
                    kind: "storylet".into(),
                    text: "The oath has returned.".into(),
                    tags: vec!["river".into()],
                },
            },
        ],
    })));

    host.commit_storylet("drowned-oath", Some(TokenId(1)))
        .unwrap();
    assert_eq!(host.state().world.history[0].id, "oath-returned");
    assert!(host.state().inventories[&TokenId(1)]
        .items
        .values()
        .any(|item| item.name == "Oath Blade"));
    assert!(host.state().maps.contains_key("oath-encounter"));
    assert!(host
        .state()
        .journal
        .iter()
        .any(|fact| fact.id == "oath.public"));

    // A storylet re-lights while its requirements hold, so it can be played
    // again. The Item effect must not collide with its first grant: a second
    // play yields a second blade rather than failing the whole commit.
    host.commit_storylet("drowned-oath", Some(TokenId(1)))
        .expect("a repeat play must not error on a duplicate item id");
    let blades = host.state().inventories[&TokenId(1)]
        .items
        .values()
        .filter(|item| item.name == "Oath Blade")
        .count();
    assert_eq!(blades, 2, "each play grants a fresh instance");
}

#[test]
fn campaign_commit_keeps_secrets_private_and_applies_public_draft() {
    const SECRET_TEXT: &str = "The witness lied.";
    let mut world = CampaignWorld::default();
    world.factions.insert(
        "tide".into(),
        WorldFaction {
            id: "tide".into(),
            name: "Tide Court".into(),
            tags: vec!["river".into()],
            claims: vec![],
        },
    );
    world.storylets.insert(
        "finale".into(),
        StoryletProposal {
            key: "finale".into(),
            entry: "The oath returns.".into(),
            tags: vec![],
            requirements: Default::default(),
            roles: vec![],
            effects: vec![],
        },
    );
    let draft = CampaignDraft {
        id: "oath".into(),
        name: "River Oath".into(),
        world,
        maps: vec![DraftMap {
            scale: MapScale::Region,
            map: LocalMapProposal {
                id: "march".into(),
                name: "River March".into(),
                width: 3,
                height: 2,
                default_ground: "grass".into(),
                cells: vec![],
                spawn_zones: vec![],
                transitions: vec![],
                encounter_anchors: vec![],
            },
            inhabitants: vec![MapInhabitant {
                id: 31,
                name: "Watchtower Keeper".into(),
                sprite: "keeper".into(),
                at: MapPoint { col: 2, row: 1 },
                system: "demo".into(),
                stats: BTreeMap::from([("vigilance".into(), 3)]),
                owner: None,
            }],
        }],
        secrets: vec![SecretFact {
            id: "oath.secret".into(),
            text: SECRET_TEXT.into(),
            tags: vec![],
            reveal: RevealCondition::Manual,
        }],
        rewards: vec![ItemProposal {
            template: "demo:witness".into(),
            name: "Witness Blade".into(),
            tags: vec!["weapon".into()],
        }],
        starting_map: "march".into(),
        final_storylet: "finale".into(),
    };
    let record = GenerationRecord {
        id: "generated.campaign.1".into(),
        request: GeneratorRequest {
            generator: "demo:campaign".into(),
            args: GenValue::Text {
                value: "river".into(),
            },
            locks: BTreeMap::new(),
        },
        entropy: 7,
        proposal: GenValue::Campaign { campaign: draft },
    };
    let mut host = HostSession::new(snapshot());
    let peer = PeerId(31);
    let mut client = ClientSession::new();
    let snapshot = host.on_connect(peer).pop().unwrap().1;
    assert!(client.on_message(snapshot).is_empty());
    let out = host.commit_campaign(record, Some(TokenId(1))).unwrap();
    for (recipient, message) in out {
        assert_eq!(recipient, Recipient::All);
        let bytes = postcard::to_allocvec(&message).unwrap();
        assert!(
            !bytes
                .windows(SECRET_TEXT.len())
                .any(|window| window == SECRET_TEXT.as_bytes()),
            "the public event stream must never contain a campaign secret"
        );
        let replayed: NetMessage = postcard::from_bytes(&bytes).unwrap();
        assert!(client.on_message(replayed).is_empty());
    }

    assert!(host.campaign().secret("oath.secret").is_some());
    assert!(host.state().world.factions.contains_key("tide"));
    assert_eq!(host.state().active_map.as_deref(), Some("march"));
    let keeper = host.state().map.token(TokenId(31)).unwrap();
    assert_eq!(keeper.sprite, "keeper");
    assert_eq!(
        host.state()
            .map
            .sheet(TokenId(31))
            .unwrap()
            .int("vigilance"),
        Some(3)
    );
    assert_eq!(host.state().turns.entries(), &[TokenId(31)]);
    assert_eq!(client.state(), Some(host.state()));
    assert!(
        host.state().inventories[&TokenId(1)]
            .items
            .values()
            .any(|item| item.name == "Witness Blade")
    );
    assert!(
        host.state()
            .journal
            .iter()
            .all(|fact| fact.text != SECRET_TEXT)
    );
    let public_history = postcard::to_allocvec(host.history().entries()).unwrap();
    assert!(
        !public_history
            .windows(SECRET_TEXT.len())
            .any(|window| window == SECRET_TEXT.as_bytes()),
        "the durable public history must not retain a campaign secret"
    );
    let late_snapshot = host.on_connect(PeerId(32)).pop().unwrap().1;
    let late_snapshot_bytes = postcard::to_allocvec(&late_snapshot).unwrap();
    assert!(
        !late_snapshot_bytes
            .windows(SECRET_TEXT.len())
            .any(|window| window == SECRET_TEXT.as_bytes()),
        "a late joiner must not receive a campaign secret"
    );
}

#[test]
fn campaign_commit_refuses_invalid_inhabitants_without_changing_public_or_private_state() {
    let mut host = HostSession::new(snapshot());
    let state_before = host.state().clone();
    let campaign_before = host.campaign().clone();
    let seq_before = host.seq();
    let log_hash_before = host.log_hash();
    let record = GenerationRecord {
        id: "generated.bad-campaign.1".into(),
        request: GeneratorRequest {
            generator: "demo:campaign".into(),
            args: GenValue::Text {
                value: "tower".into(),
            },
            locks: BTreeMap::new(),
        },
        entropy: 9,
        proposal: GenValue::Campaign {
            campaign: CampaignDraft {
                id: "bad-tower".into(),
                name: "Bad Tower".into(),
                world: CampaignWorld {
                    storylets: BTreeMap::from([(
                        "finale".into(),
                        StoryletProposal {
                            key: "finale".into(),
                            entry: "Finish".into(),
                            tags: vec![],
                            requirements: Default::default(),
                            roles: vec![],
                            effects: vec![],
                        },
                    )]),
                    ..Default::default()
                },
                maps: vec![DraftMap {
                    scale: MapScale::Local,
                    map: LocalMapProposal {
                        id: "tower".into(),
                        name: "Ruined Watchtower".into(),
                        width: 2,
                        height: 2,
                        default_ground: "stone".into(),
                        cells: vec![],
                        spawn_zones: vec![],
                        transitions: vec![],
                        encounter_anchors: vec![],
                    },
                    inhabitants: vec![
                        MapInhabitant {
                            id: 4,
                            name: "Warden".into(),
                            sprite: "warden".into(),
                            at: MapPoint { col: 0, row: 0 },
                            system: "demo".into(),
                            stats: BTreeMap::new(),
                            owner: None,
                        },
                        MapInhabitant {
                            id: 4,
                            name: "Other Warden".into(),
                            sprite: "warden".into(),
                            at: MapPoint { col: 1, row: 1 },
                            system: "demo".into(),
                            stats: BTreeMap::new(),
                            owner: None,
                        },
                    ],
                }],
                secrets: vec![SecretFact {
                    id: "bad-tower.secret".into(),
                    text: "must not persist".into(),
                    tags: vec![],
                    reveal: RevealCondition::Manual,
                }],
                rewards: vec![],
                starting_map: "tower".into(),
                final_storylet: "finale".into(),
            },
        },
    };

    assert!(host.commit_campaign(record, None).is_err());
    assert_eq!(host.state(), &state_before);
    assert_eq!(host.campaign(), &campaign_before);
    assert_eq!(host.seq(), seq_before);
    assert_eq!(host.log_hash(), log_hash_before);
}

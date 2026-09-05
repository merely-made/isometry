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
        }],
        secrets: vec![SecretFact {
            id: "oath.secret".into(),
            text: "The witness lied.".into(),
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
    host.commit_campaign(record, Some(TokenId(1))).unwrap();

    assert!(host.campaign().secret("oath.secret").is_some());
    assert!(host.state().world.factions.contains_key("tide"));
    assert_eq!(host.state().active_map.as_deref(), Some("march"));
    assert!(host.state().inventories[&TokenId(1)]
        .items
        .values()
        .any(|item| item.name == "Witness Blade"));
    assert!(host
        .state()
        .journal
        .iter()
        .all(|fact| fact.text != "The witness lied."));
}

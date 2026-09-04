//! End-to-end replication over the in-process sim: the same routing the
//! iroh transport performs, minus the wire. Proves the I4 done-condition
//! (mid-session join, no divergence: state and log hashes match) without
//! two machines.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use isometry_campaign::{
    CampaignDraft, CampaignMap, CampaignWorld, DraftMap, EncounterAnchor, EntropyTape,
    EquipmentSlot, GenValue, GenerationRecord, GeneratorRequest, HiddenItemModifier, HistoryEvent,
    Inventory, ItemId, ItemInstance, ItemModifier, ItemModifierKind, ItemProposal,
    LocalMapProposal, MapCellProposal, MapPoint, MapScale, MapTransition, RevealCondition,
    RoleSlot, SecretFact, SpawnZone, StoryletEffect, StoryletProposal, StoryletRequirements,
    WorldCharacter, WorldEvent, WorldFact, WorldFaction, WorldLaw, WorldPlace, WorldRoute,
};
use isometry_core::{
    Beat, Facing, MapDocument, RollRecord, SessionEvent, SheetData, SheetDelta, Token, TokenId,
    TurnList,
};
use isometry_net::sim::Sim;
use isometry_net::{
    ActionIntent, ActionResolved, GameError, GameEvent, GameSnapshot, HostSession, PeerId,
    RequestId, TransitionResolved,
};

// The 2026-09-04 split moved the tests into the modules below; this file
// keeps the shared imports and the fixtures they all read through
// `use super::*`.
mod adjudication;
mod authority;
mod convergence;
mod generation;
mod items;
mod secrets;
mod storylets;
mod travel;
mod world;

fn snapshot() -> GameSnapshot {
    let mut map = MapDocument::new("net demo", 8, 8);
    let grass = map.intern_tile_kind("grass");
    for r in 0..8 {
        for c in 0..8 {
            map.ground.set(c, r, grass);
        }
    }
    map.tokens.push(Token {
        id: TokenId(1),
        at: (1, 1),
        facing: Facing::South,
        sprite: "knight".to_owned(),
        owner: Some("A".to_owned()),
    });
    map.tokens.push(Token {
        id: TokenId(2),
        at: (6, 6),
        facing: Facing::North,
        sprite: "goblin".to_owned(),
        owner: Some("B".to_owned()),
    });
    GameSnapshot {
        map,
        turns: TurnList::new(),
        roll_log: Vec::new(),
        journal: Vec::new(),
        inventories: Default::default(),
        generations: Vec::new(),
        maps: Default::default(),
        active_map: None,
        world: Default::default(),
        clocks: Default::default(),

        party_cap: isometry_net::default_party_cap(),
        last_beats: Vec::new(),
        beat_seq: 0,
        applied_actions: Default::default(),
    }
}

/// A fresh request id, standing in for the authority's stamp: in play the host
/// numbers each ask, and a fixture that reused one number would be asking the
/// same question twice (and the second answer would rightly be a no-op).
fn next_request() -> RequestId {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    RequestId::host(NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Bind a sheet to a token so it can take part in an adjudicated action.
fn sheet(name: &str, hp: i64, ac: i64) -> SheetData {
    let mut s = SheetData::new("5e-srd");
    s.set_text("name", name);
    s.set_int("hp_current", hp);
    s.set_int("hp_max", hp);
    s.set_int("ac", ac);
    s
}

/// A hit for `damage` on the goblin, shaped exactly as the system resolver
/// produces it. The net crate never builds one of these itself: it only carries
/// what the rules decided.
fn attack_hit(damage: i64) -> GameEvent {
    GameEvent::ActionResolved(ActionResolved {
        request: next_request(),
        actor: TokenId(1),
        target: TokenId(2),
        action_key: "attack".to_owned(),
        label: "Attack".to_owned(),
        attack: RollRecord {
            by: "Knight".to_owned(),
            expr: "1d20+5".to_owned(),
            dice: vec![14],
            total: 19,
        },
        hit: true,
        damage: Some(RollRecord {
            by: "Knight".to_owned(),
            expr: "1d8+3".to_owned(),
            dice: vec![4],
            total: damage as i32,
        }),
        deltas: vec![SheetDelta {
            token: TokenId(2),
            key: "hp_current".to_owned(),
            add: -damage,
        }],
        beats: vec![
            Beat::new(TokenId(1), "strike"),
            Beat::new(TokenId(2), "recoil"),
        ],
        defeated: Vec::new(),
        displaced: Vec::new(),
        conditions: Vec::new(),
        mobility: Vec::new(),
        owner_changes: Vec::new(),
        turn_counters: Vec::new(),
    })
}

fn generation_record(id: &str) -> GenerationRecord {
    GenerationRecord {
        id: id.to_owned(),
        request: GeneratorRequest {
            generator: "demo:forge-item".to_owned(),
            args: GenValue::Text {
                value: "river".to_owned(),
            },
            locks: Default::default(),
        },
        entropy: EntropyTape::from_seed(7).draw(),
        proposal: GenValue::Item {
            item: ItemProposal {
                template: "demo:river-blade".to_owned(),
                name: "River Blade".to_owned(),
                tags: vec!["fixture".to_owned()],
            },
        },
    }
}

/// Every connected client holds exactly the host's state, hash, and seq.
fn assert_converged(sim: &Sim) {
    for (peer, client) in &sim.clients {
        assert_eq!(
            client.state(),
            Some(sim.host.state()),
            "client {peer:?} state diverged"
        );
        assert_eq!(
            client.log_hash(),
            sim.host.log_hash(),
            "client {peer:?} log hash diverged"
        );
        assert_eq!(
            client.applied(),
            sim.host.seq(),
            "client {peer:?} seq diverged"
        );
    }
}

fn mv(id: u32, to: (i32, i32)) -> GameEvent {
    GameEvent::Map(SessionEvent::TokenMoved {
        id: TokenId(id),
        to,
    })
}

fn sword_inventory() -> Inventory {
    let sword = ItemInstance {
        id: ItemId::new("reward-03.sword"),
        template: "srd5e:longsword".to_owned(),
        name: "Fine Longsword".to_owned(),
        quantity: 1,
        tags: vec!["weapon".to_owned()],
        modifiers: Vec::new(),
        appearance_layers: vec!["weapon:longsword".to_owned()],
    };
    let mut inventory = Inventory::default();
    inventory.insert(sword).unwrap();
    inventory
        .equip(EquipmentSlot::MainHand, ItemId::new("reward-03.sword"))
        .unwrap();
    inventory
}

/// Two prepared maps joined by a door: `field` (the snapshot's demo board,
/// promoted to a stored map) and `hut`, whose entry door faces back.
fn two_map_snapshot() -> GameSnapshot {
    let mut snap = snapshot();
    let field = CampaignMap {
        id: "field".to_owned(),
        scale: MapScale::Local,
        document: snap.map.clone(),
        spawn_zones: Vec::new(),
        transitions: vec![MapTransition {
            id: "field-gate".to_owned(),
            at: MapPoint { col: 3, row: 3 },
            target_map: "hut".to_owned(),
            target_entry: Some("hut-door".to_owned()),
        }],
        encounter_anchors: Vec::new(),
    };
    let mut hut_doc = MapDocument::new("hut", 6, 6);
    let floor = hut_doc.intern_tile_kind("stone");
    for r in 0..6 {
        for c in 0..6 {
            hut_doc.ground.set(c, r, floor);
        }
    }
    let hut = CampaignMap {
        id: "hut".to_owned(),
        scale: MapScale::Local,
        document: hut_doc,
        spawn_zones: Vec::new(),
        transitions: vec![MapTransition {
            id: "hut-door".to_owned(),
            at: MapPoint { col: 1, row: 1 },
            target_map: "field".to_owned(),
            target_entry: Some("field-gate".to_owned()),
        }],
        encounter_anchors: Vec::new(),
    };
    snap.maps.insert("field".to_owned(), field);
    snap.maps.insert("hut".to_owned(), hut);
    snap.active_map = Some("field".to_owned());
    snap
}

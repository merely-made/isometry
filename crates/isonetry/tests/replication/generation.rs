//! W2 commit-result mode: peers store the host-selected typed output.
//!
//! They do not execute the pack script, a client cannot forge a generation
//! record, the host refuses a malformed one before it enters history, and a
//! generated map replicates as result data.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

/// W2 commit-result mode: peers store the host-selected typed output but do
/// not execute its pack script. A client cannot forge a generation record.
#[test]
fn committed_generation_replicates_without_client_authority() {
    let record = generation_record("generated.river-blade.1");
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));

    sim.host_event(GameEvent::Generation(record.clone()));
    assert_eq!(sim.host.state().generations, vec![record.clone()]);
    assert_eq!(
        sim.clients[&PeerId(10)].state().unwrap().generations,
        vec![record.clone()]
    );

    sim.connect(PeerId(20));
    assert_eq!(
        sim.clients[&PeerId(20)].state().unwrap().generations,
        vec![record.clone()],
        "a late joiner receives committed results in its snapshot"
    );

    sim.client_intent(
        PeerId(10),
        GameEvent::Generation(generation_record("forged")),
    );
    assert_eq!(sim.host.state().generations, vec![record]);
    assert_converged(&sim);
}

#[test]
fn host_rejects_malformed_generation_before_it_enters_history() {
    let mut host = HostSession::new(snapshot());
    let mut malformed = generation_record("");
    malformed.request.generator.clear();

    assert!(host.commit_generation(malformed).is_err());
    assert!(host.state().generations.is_empty());
    assert!(host.history().is_empty());
}

#[test]
fn generated_map_stores_activates_edits_and_replicates_as_result_data() {
    let map = LocalMapProposal {
        id: "demo:river-cache".to_owned(),
        name: "River Cache".to_owned(),
        width: 5,
        height: 4,
        default_ground: "grass".to_owned(),
        cells: vec![MapCellProposal {
            col: 2,
            row: 2,
            ground: Some("stone".to_owned()),
            prop: None,
            elevation: Some(1),
        }],
        spawn_zones: vec![SpawnZone {
            id: "party".to_owned(),
            cells: vec![MapPoint { col: 0, row: 1 }],
        }],
        transitions: Vec::new(),
        encounter_anchors: vec![EncounterAnchor {
            id: "guardian".to_owned(),
            at: MapPoint { col: 3, row: 2 },
            tags: vec!["guardian".to_owned()],
        }],
    }
    .lower(MapScale::Local)
    .unwrap();
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));

    sim.host_event(GameEvent::MapStored(map.clone()));
    sim.host_event(GameEvent::MapActivated { id: map.id.clone() });
    assert_eq!(
        sim.host.state().active_map.as_deref(),
        Some("demo:river-cache")
    );
    assert_eq!(sim.host.state().map.ground.width(), 5);

    let stone = sim
        .host
        .state()
        .map
        .tile_kinds
        .iter()
        .position(|kind| kind == "stone")
        .unwrap() as u16;
    sim.host_event(GameEvent::Map(SessionEvent::TilePlaced {
        layer: isometry_core::Layer::Ground,
        at: (1, 1),
        kind: isometry_core::TileKindId(stone),
    }));
    assert_eq!(
        sim.host.state().maps["demo:river-cache"]
            .document
            .ground
            .get(1, 1),
        Some(&isometry_core::TileKindId(stone))
    );

    sim.connect(PeerId(20));
    assert_eq!(
        sim.clients[&PeerId(20)].state().unwrap().maps,
        sim.host.state().maps
    );
    sim.client_intent(PeerId(10), GameEvent::MapStored(map));
    assert_eq!(sim.host.seq(), 3, "client map authoring entered the log");
    assert_converged(&sim);
}

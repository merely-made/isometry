//! The campaign world over the wire: faction turns, discovery, overmap travel.
//!
//! Everything that moves the party between places rather than across a board.
//! The clock, the discovery set, and the faction channel are all host
//! verdicts, and every peer lives in the world they changed.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn a_faction_turn_commits_and_every_peer_lives_in_the_changed_world() {
    let mut snap = snapshot();
    snap.world.factions.insert(
        "tide".to_owned(),
        WorldFaction {
            id: "tide".into(),
            name: "Tide Court".into(),
            tags: vec!["river".into()],
            claims: vec![],
        },
    );
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));

    // The DM rolls a downtime tick from the host's own world and tape, then
    // commits the batch. (A real DM edits it first; here we commit as rolled.)
    let mut tape = EntropyTape::from_seed(7);
    let moves = sim.host.state().world.faction_turn(4, &mut tape);
    assert_eq!(moves.len(), 1, "one move for the one faction");
    let logged: usize = moves.iter().map(|m| m.clone().into_events().len()).sum();
    sim.host_faction_turn(moves).expect("the tick commits");

    // Every peer holds the faction-turn history at the tick, identically: a
    // faction acting on the world is ordinary replicated truth, not a host-only
    // record. And the whole batch (each move's history plus its change) reached
    // the ordered log.
    let meanwhile = |s: &GameSnapshot| {
        s.world
            .history
            .iter()
            .filter(|h| h.kind == "faction-turn" && h.time == 4)
            .count()
    };
    assert_eq!(meanwhile(sim.host.state()), 1);
    assert_eq!(meanwhile(sim.clients[&PeerId(10)].state().unwrap()), 1);
    assert_eq!(
        sim.host.seq() as usize,
        logged,
        "every move event entered the log"
    );
    assert_converged(&sim);
}

#[test]
fn discovery_replicates_as_the_party_travels() {
    let mut snap = snapshot();
    snap.world.places.insert(
        "village".into(),
        WorldPlace {
            id: "village".into(),
            name: "Village".into(),
            tags: vec![],
            map: None,
            position: None,
        },
    );
    snap.world.places.insert(
        "forest".into(),
        WorldPlace {
            id: "forest".into(),
            name: "Forest".into(),
            tags: vec![],
            map: None,
            position: None,
        },
    );
    snap.world.routes.insert(
        "r".into(),
        WorldRoute {
            id: "r".into(),
            from: "village".into(),
            to: "forest".into(),
            tags: vec![],
            weight: 2,
        },
    );
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));

    // Nobody knows the map yet.
    assert!(!sim.host.state().world.knows("A", "village"));
    // The party sets out; arriving discovers the village and the forest one step
    // on -- and the party's map is shared truth, so the client learns it too.
    sim.host_event(GameEvent::World(WorldEvent::PartyMoved {
        party: "A".into(),
        node: "village".into(),
    }));
    let knows = |s: &GameSnapshot, node: &str| s.world.knows("A", node);
    assert!(knows(sim.host.state(), "village") && knows(sim.host.state(), "forest"));
    assert!(
        knows(sim.clients[&PeerId(10)].state().unwrap(), "forest"),
        "the client shares the party's discovered map"
    );
    assert_converged(&sim);
}

#[test]
fn a_party_travels_the_overmap_and_every_peer_agrees() {
    let mut snap = snapshot();
    // A tiny overmap projected from two places joined by a route.
    snap.world.places.insert(
        "village".into(),
        WorldPlace {
            id: "village".into(),
            name: "Village".into(),
            tags: vec![],
            map: None,
            position: None,
        },
    );
    snap.world.places.insert(
        "forest".into(),
        WorldPlace {
            id: "forest".into(),
            name: "Forest".into(),
            tags: vec![],
            map: None,
            position: None,
        },
    );
    snap.world.routes.insert(
        "r1".into(),
        WorldRoute {
            id: "r1".into(),
            from: "village".into(),
            to: "forest".into(),
            tags: vec![],
            weight: 2,
        },
    );
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));

    // The party sets out, then travels the edge to the forest.
    sim.host_event(GameEvent::World(WorldEvent::PartyMoved {
        party: "A".into(),
        node: "village".into(),
    }));
    sim.host_event(GameEvent::World(WorldEvent::PartyMoved {
        party: "A".into(),
        node: "forest".into(),
    }));

    // Where the party stands on the overmap is shared truth: every peer agrees,
    // the way they agree on which tactical map is active.
    let at = |s: &GameSnapshot| s.world.party_at("A").map(str::to_owned);
    assert_eq!(at(sim.host.state()).as_deref(), Some("forest"));
    assert_eq!(
        at(sim.clients[&PeerId(10)].state().unwrap()).as_deref(),
        Some("forest"),
        "the client holds the same overmap position"
    );
    assert_converged(&sim);
}

#[test]
fn a_resolved_travel_moves_the_party_and_ticks_the_clock_on_every_peer() {
    let mut snap = snapshot();
    snap.world.places.insert(
        "village".into(),
        WorldPlace {
            id: "village".into(),
            name: "Village".into(),
            tags: vec![],
            map: None,
            position: None,
        },
    );
    // The forest is a site (a tactical map), so arriving there advances its clock.
    snap.world.places.insert(
        "forest".into(),
        WorldPlace {
            id: "forest".into(),
            name: "Forest".into(),
            tags: vec![],
            map: Some("forest-map".into()),
            position: None,
        },
    );
    snap.world.party_node.insert("A".into(), "village".into());
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));

    // The host applies a travel verdict the system rolled: 5 ticks, lost the
    // way, and the long leg tolled the party exhaustion 2 (token 1 is A's).
    sim.host_event(GameEvent::TravelResolved {
        party: "A".into(),
        to: "forest".into(),
        ticks: 5,
        roll: RollRecord {
            by: "Scout".into(),
            expr: "1d20".into(),
            dice: vec![7],
            total: 7,
        },
        lost: true,
        exhaustion: 2,
        encounter: false,
        forage: 3,
    });

    let at = |s: &GameSnapshot| s.world.party_at("A").map(str::to_owned);
    let clock = |s: &GameSnapshot| s.clocks.get("forest-map").copied().unwrap_or(0);
    let tired = |s: &GameSnapshot| s.map.condition_value(TokenId(1), "exhaustion");
    let food = |s: &GameSnapshot| s.world.party_resource("A", "food");
    assert_eq!(
        at(sim.host.state()).as_deref(),
        Some("forest"),
        "the party arrived"
    );
    assert_eq!(
        clock(sim.host.state()),
        5,
        "arriving advanced the destination's clock"
    );
    assert_eq!(
        tired(sim.host.state()),
        2,
        "the march exhausted the party member"
    );
    assert_eq!(
        at(sim.clients[&PeerId(10)].state().unwrap()).as_deref(),
        Some("forest")
    );
    assert_eq!(
        clock(sim.clients[&PeerId(10)].state().unwrap()),
        5,
        "every peer holds the same arrival time"
    );
    assert_eq!(
        tired(sim.clients[&PeerId(10)].state().unwrap()),
        2,
        "and the same exhaustion -- attrition is replicated truth"
    );
    assert_eq!(
        food(sim.host.state()),
        3,
        "the foraged food joined the party's stores"
    );
    assert_eq!(
        food(sim.clients[&PeerId(10)].state().unwrap()),
        3,
        "and the client holds the same stores"
    );
    assert_eq!(
        sim.host.state().roll_log.len(),
        1,
        "the navigation roll reached the log"
    );
    assert_converged(&sim);
}

#[test]
fn an_encounter_on_the_road_drops_the_party_onto_the_map() {
    // two_map_snapshot registers the "field" and "hut" tactical maps.
    let mut snap = two_map_snapshot();
    snap.world.places.insert(
        "green".into(),
        WorldPlace {
            id: "green".into(),
            name: "Green".into(),
            tags: vec![],
            map: Some("field".into()),
            position: None,
        },
    );
    snap.world.places.insert(
        "hollow".into(),
        WorldPlace {
            id: "hollow".into(),
            name: "Hollow".into(),
            tags: vec![],
            map: Some("hut".into()),
            position: None,
        },
    );
    snap.world.routes.insert(
        "r".into(),
        WorldRoute {
            id: "r".into(),
            from: "green".into(),
            to: "hollow".into(),
            tags: vec![],
            weight: 3,
        },
    );
    snap.world.party_node.insert("A".into(), "green".into());
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));

    // The party travels to the hollow and the road throws a peril: it is dropped
    // onto the hollow's map to fight rather than arriving in peace.
    sim.host_event(GameEvent::TravelResolved {
        party: "A".into(),
        to: "hollow".into(),
        ticks: 3,
        roll: RollRecord {
            by: "?".into(),
            expr: "1d20".into(),
            dice: vec![10],
            total: 10,
        },
        lost: false,
        exhaustion: 0,
        encounter: true,
        forage: 0,
    });

    let active = |s: &GameSnapshot| s.active_map.clone();
    assert_eq!(
        active(sim.host.state()).as_deref(),
        Some("hut"),
        "the peril dropped the party onto the destination's map"
    );
    assert_eq!(
        active(sim.clients[&PeerId(10)].state().unwrap()).as_deref(),
        Some("hut"),
        "on every peer -- an encounter is game truth"
    );
    assert_converged(&sim);
}

#[test]
fn a_client_cannot_pronounce_its_own_travel() {
    let mut snap = snapshot();
    snap.world.places.insert(
        "village".into(),
        WorldPlace {
            id: "village".into(),
            name: "Village".into(),
            tags: vec![],
            map: None,
            position: None,
        },
    );
    snap.world.places.insert(
        "forest".into(),
        WorldPlace {
            id: "forest".into(),
            name: "Forest".into(),
            tags: vec![],
            map: None,
            position: None,
        },
    );
    snap.world.party_node.insert("A".into(), "village".into());
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));
    let seq = sim.host.seq();

    // Proposing a travel verdict is forging one; the host refuses, and the party
    // stays put.
    sim.client_intent(
        PeerId(10),
        GameEvent::TravelResolved {
            party: "A".into(),
            to: "forest".into(),
            ticks: 1,
            roll: RollRecord {
                by: "?".into(),
                expr: "1d20".into(),
                dice: vec![20],
                total: 20,
            },
            lost: false,
            exhaustion: 0,
            encounter: false,
            forage: 0,
        },
    );
    assert_eq!(
        sim.host.seq(),
        seq,
        "a forged travel verdict entered the log"
    );
    assert_eq!(
        sim.host.state().world.party_at("A").as_deref(),
        Some("village"),
        "the party did not move"
    );
    assert_converged(&sim);
}

#[test]
fn a_granted_player_plays_a_faction_and_a_stranger_may_not() {
    let mut snap = snapshot();
    // The goblin (token 2) is the Tide Court's furniture, not any player's.
    snap.map.tokens[1].owner = Some("tide".to_owned());
    snap.world.factions.insert(
        "tide".to_owned(),
        WorldFaction {
            id: "tide".into(),
            name: "Tide Court".into(),
            tags: vec!["river".into()],
            claims: vec![],
        },
    );
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));
    sim.client_hello(PeerId(10), "B");

    // Ungranted, B cannot even emote the faction's token: it is not B's, and no
    // channel has been handed over.
    sim.client_intent(
        PeerId(10),
        GameEvent::Emoted {
            token: TokenId(2),
            beat: "cheer".to_owned(),
        },
    );
    assert!(
        sim.host.state().last_beats.is_empty(),
        "a stranger cannot command a faction's token"
    );

    // The DM grants B the Tide Court's channel. Now B plays the faction: a
    // faction is an owner name, and the grant is the per-channel permission.
    sim.host_event(GameEvent::World(WorldEvent::FactionControlSet {
        faction: "tide".to_owned(),
        player: Some("B".to_owned()),
    }));
    sim.client_intent(
        PeerId(10),
        GameEvent::Emoted {
            token: TokenId(2),
            beat: "cheer".to_owned(),
        },
    );
    assert_eq!(
        sim.host.state().last_beats,
        &[Beat::new(TokenId(2), "cheer")],
        "a granted player commands the faction's token as its own"
    );

    // Revoke, and the faction returns to the DM: B is a stranger again, so a
    // fresh emote is refused and the last beat is still the granted one.
    sim.host_event(GameEvent::World(WorldEvent::FactionControlSet {
        faction: "tide".to_owned(),
        player: None,
    }));
    sim.client_intent(
        PeerId(10),
        GameEvent::Emoted {
            token: TokenId(2),
            beat: "taunt".to_owned(),
        },
    );
    assert_eq!(
        sim.host.state().last_beats,
        &[Beat::new(TokenId(2), "cheer")],
        "after revoke the channel is the DM's again, so the taunt never played"
    );
    assert_converged(&sim);
}

#[test]
fn banked_time_makes_a_bigger_tick_and_the_commit_empties_the_bank() {
    let mut snap = snapshot();
    snap.world.factions.insert(
        "tide".to_owned(),
        WorldFaction {
            id: "tide".into(),
            name: "Tide Court".into(),
            tags: vec!["river".into()],
            claims: vec![],
        },
    );
    // The table spent a long scene away: 25 units banked toward this faction.
    snap.world.faction_sheets.insert(
        "tide".to_owned(),
        BTreeMap::from([("banked_time".to_owned(), 25)]),
    );
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));

    let mut tape = EntropyTape::from_seed(3);
    let moves = sim.host.state().world.faction_turn(5, &mut tape);
    assert_eq!(
        moves.len(),
        3,
        "banked 25 => one baseline plus two earned moves"
    );
    sim.host_faction_turn(moves).expect("the tick commits");

    // The tick was proportional (3 faction-turn history events), and acting
    // emptied the bank -- on every peer, so the same time cannot be spent twice.
    let banked = |s: &GameSnapshot| {
        s.world
            .faction_sheet("tide")
            .and_then(|m| m.get("banked_time"))
            .copied()
    };
    let logged = |s: &GameSnapshot| {
        s.world
            .history
            .iter()
            .filter(|h| h.kind == "faction-turn" && h.time == 5)
            .count()
    };
    assert_eq!(logged(sim.host.state()), 3);
    assert_eq!(
        banked(sim.host.state()),
        Some(0),
        "the bank emptied on the host"
    );
    assert_eq!(
        banked(sim.clients[&PeerId(10)].state().unwrap()),
        Some(0),
        "and on the client -- the spend is replicated truth"
    );
    assert_converged(&sim);
}

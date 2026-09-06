//! A bounded two-machine protocol-v4 receipt for a character arrival and a
//! forest journey over real Iroh QUIC.
//!
//! Host:
//! `cargo run -p isonetry --features iroh --example forest_session -- host --ticket-file host.ticket --receipt-file host.receipt`
//!
//! Client (after copying the `ticket=` value to the second machine):
//! `cargo run -p isonetry --features iroh --example forest_session -- join --ticket TICKET --receipt-file client.receipt --release-file client.release`
//!
//! Both receipt files are line-oriented `key=value` records. The controller
//! compares their `seq` and `hash` after both commands exit successfully.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use isometry_campaign::{EntropyTape, GenValue, GenerationRecord, GeneratorRequest};
use isometry_core::{Facing, FieldValue, MapDocument, SheetData, Token, TokenId, TurnList};
use isometry_system::{GeneratorCatalog, GeneratorLimits};
use isonetry::iroh_link::{ClientNet, HostNet};
use isonetry::{
    ActionIntent, GameEvent, GameSnapshot, PROTOCOL_VERSION, RequestId, resolve_transition,
};

const CLIENT_NAME: &str = "player";
const DEADLINE: Duration = Duration::from_secs(45);

fn empty_snapshot() -> GameSnapshot {
    let mut map = MapDocument::new("forest-session", 6, 6);
    let grass = map.intern_tile_kind("grass");
    for row in 0..6 {
        for column in 0..6 {
            map.ground.set(column, row, grass);
        }
    }

    GameSnapshot {
        map,
        turns: TurnList::new(),
        roll_log: Vec::new(),
        journal: Vec::new(),
        inventories: BTreeMap::new(),
        generations: Vec::new(),
        maps: BTreeMap::new(),
        active_map: None,
        world: Default::default(),
        clocks: BTreeMap::new(),
        party_cap: isonetry::default_party_cap(),
        last_beats: Vec::new(),
        beat_seq: 0,
        applied_actions: Default::default(),
    }
}

fn generated_watchtower() -> Result<(GenerationRecord, (i32, i32), String), String> {
    let pack =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../isometry-system/examples/packs/watchtower");
    let catalog = GeneratorCatalog::discover([pack]);
    if !catalog.diagnostics().is_empty() {
        return Err(format!(
            "watchtower pack diagnostics: {:?}",
            catalog.diagnostics()
        ));
    }
    let request = GeneratorRequest {
        generator: "watchtower:ruined_tower".into(),
        args: GenValue::Text {
            value: "ash-and-bells".into(),
        },
        locks: BTreeMap::new(),
    };
    let mut tape = EntropyTape::from_seed(91);
    let record = catalog
        .generate(
            "forest-session.watchtower.91",
            &request,
            &mut tape,
            GeneratorLimits::default(),
        )
        .map_err(|error| format!("generate watchtower: {error}"))?;
    let GenValue::Campaign { campaign } = &record.proposal else {
        return Err("watchtower pack did not return a campaign".into());
    };
    let start = campaign
        .maps
        .iter()
        .find(|map| map.map.id == campaign.starting_map)
        .ok_or_else(|| "watchtower starting map missing".to_owned())?;
    let door = start
        .map
        .transitions
        .first()
        .ok_or_else(|| "watchtower starting map has no forest door".to_owned())?;
    let doorway = (door.at.col as i32, door.at.row as i32);
    let forest_map = door.target_map.clone();
    Ok((record, doorway, forest_map))
}

fn character(at: (i32, i32)) -> GameEvent {
    let mut sheet = SheetData::new("5e-srd");
    sheet
        .fields
        .insert("name".into(), FieldValue::Text("Mara".into()));
    sheet.set_int("hp_current", 11);
    sheet.set_int("hp_max", 11);
    GameEvent::CharacterCreated {
        token: Token {
            id: TokenId(41),
            at,
            facing: Facing::East,
            sprite: "traveler".into(),
            owner: Some(CLIENT_NAME.into()),
        },
        sheet,
    }
}

fn assert_character(map: &MapDocument) -> Result<(), String> {
    let token = map
        .token(TokenId(41))
        .ok_or_else(|| "Mara token missing".to_owned())?;
    if token.owner.as_deref() != Some(CLIENT_NAME) || token.sprite != "traveler" {
        return Err("Mara token did not retain its owned character identity".into());
    }
    let sheet = map
        .sheets
        .get(&TokenId(41))
        .ok_or_else(|| "Mara sheet missing".to_owned())?;
    if sheet.system != "5e-srd"
        || sheet.text("name") != Some("Mara")
        || sheet.int("hp_current") != Some(11)
        || sheet.int("hp_max") != Some(11)
    {
        return Err("Mara sheet did not arrive atomically with the token".into());
    }
    Ok(())
}

fn assert_final(state: &GameSnapshot, forest_map: &str) -> Result<(), String> {
    let forest = state
        .maps
        .get(forest_map)
        .ok_or_else(|| "generated forest destination is not in the campaign registry".to_owned())?;
    assert_character(&forest.document)?;
    if forest.document.token(TokenId(41)).is_none()
        || !forest.document.sheets.contains_key(&TokenId(41))
    {
        return Err("Mara did not arrive in the generated forest map".into());
    }
    let tower = state
        .maps
        .get("watchtower:ruined-watchtower")
        .ok_or_else(|| "generated watchtower is not in the campaign registry".to_owned())?;
    if tower.document.token(TokenId(41)).is_some() || tower.document.token(TokenId(5)).is_none() {
        return Err("doorway travel corrupted the tower's separate residents".into());
    }
    let copies = state
        .maps
        .values()
        .flat_map(|map| &map.document.tokens)
        .filter(|token| token.id == TokenId(41))
        .count();
    if copies != 1
        || state.active_map.as_deref() != Some("watchtower:ruined-watchtower")
        || state.map.token(TokenId(41)).is_some()
        || state
            .map
            .token(TokenId(1))
            .and_then(|token| token.owner.as_deref())
            != Some(CLIENT_NAME)
    {
        return Err("split-party identity or active tower board changed".into());
    }
    Ok(())
}

async fn wait_for_ack(host: &HostNet, action_key: &str, actor: TokenId) -> Result<(), String> {
    let deadline = Instant::now() + DEADLINE;
    while Instant::now() < deadline {
        if host.take_action_intents().await.iter().any(|intent| {
            intent.action_key == action_key && intent.actor == actor && intent.target == actor
        }) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(format!(
        "timed out waiting for client {action_key} acknowledgement after {}s",
        DEADLINE.as_secs()
    ))
}

fn option(args: &[String], key: &str) -> Result<String, String> {
    args.windows(2)
        .find(|pair| pair[0] == key)
        .map(|pair| pair[1].clone())
        .ok_or_else(|| format!("missing {key}"))
}

fn write_record(
    path: &str,
    role: &str,
    status: &str,
    ticket: Option<&str>,
    seq: u64,
    hash: u64,
) -> Result<(), String> {
    let mut record = format!(
        "role={role}\nstatus={status}\nprotocol={PROTOCOL_VERSION}\nseq={seq}\nhash={hash:016x}\n"
    );
    if let Some(ticket) = ticket {
        record.push_str("ticket=");
        record.push_str(ticket);
        record.push('\n');
    }
    fs::write(Path::new(path), record).map_err(|error| format!("write {path}: {error}"))
}

fn write_snapshot(path: &str, state: &GameSnapshot) -> Result<(), String> {
    let bytes = postcard::to_allocvec(state).map_err(|error| error.to_string())?;
    fs::write(format!("{path}.snapshot"), bytes).map_err(|error| error.to_string())
}

async fn host(args: &[String]) -> Result<(), String> {
    let ticket_file = option(args, "--ticket-file")?;
    let receipt_file = option(args, "--receipt-file")?;
    let (record, doorway, forest_map) = generated_watchtower()?;
    let host = HostNet::bind(empty_snapshot()).await?;
    host.spawn_accept();
    let ticket = host.ticket().await;
    write_record(&ticket_file, "host", "waiting", Some(&ticket), 0, 0)?;
    println!("role=host status=waiting protocol={PROTOCOL_VERSION} ticket={ticket}");

    let deadline = Instant::now() + DEADLINE;
    while !host
        .player_names()
        .await
        .iter()
        .any(|name| name == CLIENT_NAME)
    {
        if Instant::now() >= deadline {
            return Err(format!(
                "timed out waiting for client protocol hello after {}s",
                DEADLINE.as_secs()
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    host.commit_campaign(record, None).await?;
    host.local_event(character(doorway)).await;
    assert_character(&host.snapshot().await.map)?;
    wait_for_ack(&host, "receipt-character", TokenId(41)).await?;
    let crossing = resolve_transition(&host.snapshot().await, TokenId(41), RequestId::host(1))
        .map_err(|error| format!("resolve generated forest doorway: {error:?}"))?;
    if crossing.to_map != forest_map {
        return Err("watchtower doorway target changed while resolving".into());
    }
    host.local_event(GameEvent::TransitionResolved(crossing))
        .await;
    let state = host.snapshot().await;
    assert_final(&state, &forest_map)?;
    let seq = host.seq().await;
    let hash = host.log_hash().await;
    wait_for_ack(&host, "receipt-forest", TokenId(1)).await?;
    write_snapshot(&receipt_file, &state)?;
    write_record(&receipt_file, "host", "passed", None, seq, hash)?;
    println!("role=host status=passed protocol={PROTOCOL_VERSION} seq={seq} hash={hash:016x}");
    Ok(())
}

async fn join(args: &[String]) -> Result<(), String> {
    let ticket = option(args, "--ticket")?;
    let receipt_file = option(args, "--receipt-file")?;
    let release_file = option(args, "--release-file")?;
    let client = tokio::time::timeout(DEADLINE, ClientNet::join(&ticket, CLIENT_NAME))
        .await
        .map_err(|_| format!("timed out joining host after {}s", DEADLINE.as_secs()))??;
    let deadline = Instant::now() + DEADLINE;
    loop {
        if let Some(state) = client.state().await {
            if state.map.token(TokenId(41)).is_some() {
                assert_character(&state.map)?;
                client
                    .action(ActionIntent::new(
                        TokenId(41),
                        TokenId(41),
                        "receipt-character",
                    ))
                    .await?;
                break;
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "timed out waiting for atomic character after {}s",
                DEADLINE.as_secs()
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let deadline = Instant::now() + DEADLINE;
    loop {
        if let Some(state) = client.state().await {
            if state.maps.contains_key("watchtower:forest-region")
                && state
                    .maps
                    .get("watchtower:forest-region")
                    .is_some_and(|map| map.document.token(TokenId(41)).is_some())
            {
                assert_final(&state, "watchtower:forest-region")?;
                let seq = client.applied().await;
                let hash = client.log_hash().await;
                client
                    // Mira remains on the active board; off-map actors cannot
                    // send actions through the current session API.
                    .action(ActionIntent::new(TokenId(1), TokenId(1), "receipt-forest"))
                    .await?;
                write_snapshot(&receipt_file, &state)?;
                write_record(&receipt_file, "client", "passed", None, seq, hash)?;
                let deadline = Instant::now() + DEADLINE;
                while !Path::new(&release_file).exists() {
                    if Instant::now() >= deadline {
                        return Err(format!(
                            "timed out waiting for controller release after {}s",
                            DEADLINE.as_secs()
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                println!(
                    "role=client status=passed protocol={PROTOCOL_VERSION} seq={seq} hash={hash:016x}"
                );
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "timed out waiting for the resolved forest crossing after {}s",
                DEADLINE.as_secs()
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("host") => host(&args[1..]).await,
        Some("join") => join(&args[1..]).await,
        _ => Err("usage: forest_session host --ticket-file FILE --receipt-file FILE | join --ticket TICKET --receipt-file FILE --release-file FILE".into()),
    }
}

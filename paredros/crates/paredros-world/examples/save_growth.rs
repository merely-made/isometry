// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measures full-history save growth for one reversible equipment operation.
//!
//! This is a narrow history-amplification baseline. It is not a forecast for
//! years of play, multiplayer, memory use, or general performance.

use std::time::Instant;

use mesocosm_core::{PartId, places::WALKER_HEIGHT};
use paredros_identity::{BodyRevisionId, SubjectId};
use paredros_world::fixtures::three_lives::wetland_body;
use paredros_world::{
    GameIntent, GameState, ItemId, ItemKind, ItemLocation, Name, World, WorldConfig,
};

const SUBJECT: SubjectId = SubjectId(1);
const PAIRS: &[usize] = &[0, 100, 1_000, 10_000];
const PART: PartId = PartId(1);

fn main() -> Result<(), String> {
    let (mut game, dressing) = bootstrap()?;
    let baseline_world = game.world().clone();
    let baseline_body = game
        .bodies()
        .get(SUBJECT)
        .cloned()
        .ok_or_else(|| "generated subject disappeared".to_owned())?;
    let baseline_items = game.items().clone();

    println!("pairs,intents,serialized_save_bytes,save_ms,restore_ms,restored_equality");
    let mut completed = 0;
    for &checkpoint in PAIRS {
        while completed < checkpoint {
            attach(&mut game, dressing)?;
            detach(&mut game, dressing)?;
            completed += 1;
        }
        verify_even_state(
            &game,
            dressing,
            &baseline_world,
            &baseline_body,
            &baseline_items,
        )?;

        let save_started = Instant::now();
        let bytes = game.save().map_err(debug)?;
        let save_ms = save_started.elapsed().as_secs_f64() * 1_000.0;
        let restore_started = Instant::now();
        let restored = GameState::restore(&bytes).map_err(debug)?;
        let restore_ms = restore_started.elapsed().as_secs_f64() * 1_000.0;
        let equal = restored == game;
        if !equal {
            return Err(format!("checkpoint {checkpoint}: restored state differs"));
        }
        verify_even_state(
            &restored,
            dressing,
            &baseline_world,
            &baseline_body,
            &baseline_items,
        )?;

        println!(
            "{checkpoint},{},{},{save_ms:.3},{restore_ms:.3},{equal}",
            game.intents().len(),
            bytes.len()
        );
    }
    Ok(())
}

fn bootstrap() -> Result<(GameState, ItemId), String> {
    let mut game = GameState::new(World::generate(4242, WorldConfig::default()).map_err(debug)?);
    let (dressing, at) = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at)
                if item.kind == ItemKind::Dressing
                    && game.world().ground().stands(at, WALKER_HEIGHT) =>
            {
                Some((item.id, at))
            },
            _ => None,
        })
        .ok_or_else(|| "no accessible dressing location".to_owned())?;
    let tick = game.next_tick();
    apply(
        &mut game,
        GameIntent::Generate {
            tick,
            subject: SUBJECT,
            body_seed: 7,
            at,
        },
    )?;
    let tick = game.next_tick();
    apply(
        &mut game,
        GameIntent::Name {
            tick,
            subject: SUBJECT,
            name: Name::new("Keeper").map_err(debug)?,
        },
    )?;
    let tick = game.next_tick();
    apply(
        &mut game,
        GameIntent::AdmitAnatomy {
            tick,
            subject: SUBJECT,
            revision: BodyRevisionId(0),
            document: Box::new(wetland_body()),
        },
    )?;
    let tick = game.next_tick();
    apply(
        &mut game,
        GameIntent::Take {
            tick,
            subject: SUBJECT,
            item: dressing,
        },
    )?;
    Ok((game, dressing))
}

fn attach(game: &mut GameState, item: ItemId) -> Result<(), String> {
    let revision = game
        .bodies()
        .get(SUBJECT)
        .ok_or_else(|| "missing subject before attachment".to_owned())?
        .revision;
    let tick = game.next_tick();
    apply(
        game,
        GameIntent::AttachItem {
            tick,
            subject: SUBJECT,
            item,
            part: PART,
            revision,
        },
    )
}

fn detach(game: &mut GameState, item: ItemId) -> Result<(), String> {
    let tick = game.next_tick();
    apply(
        game,
        GameIntent::DetachItem {
            tick,
            subject: SUBJECT,
            item,
        },
    )
}

fn apply(game: &mut GameState, intent: GameIntent) -> Result<(), String> {
    game.apply(intent).map(|_| ()).map_err(debug)
}

fn verify_even_state(
    game: &GameState,
    dressing: ItemId,
    world: &World,
    body: &paredros_world::Body,
    items: &paredros_world::Items,
) -> Result<(), String> {
    if game.world() != world
        || game.bodies().get(SUBJECT) != Some(body)
        || game.items() != items
        || game.items().get(dressing).map(|item| item.location)
            != Some(ItemLocation::Carried(SUBJECT))
    {
        return Err("even checkpoint changed current world, body, or carried dressing".to_owned());
    }
    Ok(())
}

fn debug(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}

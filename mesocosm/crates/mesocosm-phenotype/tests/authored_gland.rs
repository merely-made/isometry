// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD4: the authored gland is the packed gland is PD2's gland.
//!
//! One test per done-condition, named for it. What the gate asks:
//!
//! 1. Piccolo can propose the same accepted allocation the native fixture
//!    produced — as a proposal, and as the instruction it lowers to;
//! 2. the same context and the same entropy produce the same proposal and the
//!    same draw trace;
//! 3. contrasting developmental contexts produce different phenotypes from one
//!    body plan;
//! 4. an unknown id, an invalid part, excessive output and exhausted fuel each
//!    refuse cleanly, naming the boundary;
//! 5. Lua has no direct world mutation path.
//!
//! The first three are here; 4 and 5 are in `authored_gland/refusals.rs`, split
//! out at the 600-line ceiling and sharing this file's body plan and contexts.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mesocosm_core::{
    Arrangement, Attachment, BodyPhenotype, Intent, Organism, ProcessId, Provenance, Registry,
    Stage, VolumeRef, World, Yaw,
};
use mesocosm_phenotype::admit_dir;
use mesocosm_phenotype::express::{
    Ambient, DRAWS, Entropy, Expression, Fixture, Policy, Request, Runner, lower,
};

// ---------------------------------------------------------------------------
// The shipped pack, its script, and its fixtures
// ---------------------------------------------------------------------------

fn shipped_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the repository root")
        .join("packs")
        .join("mesocosm")
}

fn packed() -> Registry {
    admit_dir(&shipped_root()).expect("the shipped pack admits")
}

/// The declared script, opened through the door that refuses an undeclared
/// path — so this test could not read a file the manifest does not name.
fn script() -> String {
    let root = shipped_root();
    let manifest = mesocosm_phenotype::discover(&root).expect("the manifest reads");
    let path = mesocosm_phenotype::asset(&root, &manifest, "expression/gland.lua")
        .expect("the manifest declares it");
    std::fs::read_to_string(path).expect("the script reads")
}

fn fixture(name: &str) -> Fixture {
    let root = shipped_root();
    let manifest = mesocosm_phenotype::discover(&root).expect("the manifest reads");
    let relative = format!("fixtures/{name}.json");
    let path =
        mesocosm_phenotype::asset(&root, &manifest, &relative).expect("the manifest declares it");
    Fixture::read(&path).expect("the fixture reads")
}

fn gland() -> ProcessId {
    ProcessId::new("mesocosm", "secrete")
}

// ---------------------------------------------------------------------------
// The body plan both fixtures are frozen from
// ---------------------------------------------------------------------------

/// The wide plate PD2's fixtures grow: `[6, 4, 1]` lattices to twelve cells.
const FROND: [i32; 3] = [6, 4, 1];

/// A world whose played critter is a plain mature consumer of a known size —
/// the same fixture `tests/packed_gland.rs` and `tests/embodied.rs` open with.
fn bulk_world(seed: u64, founders: u32) -> World {
    let mut world = World::new(seed, founders);
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            me,
            species,
            mesocosm_core::Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    world
}

fn frond_on(world: &mut World) -> mesocosm_core::PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root")
}

/// **One body plan**, and the two fixtures are frozen from exactly this. A
/// bulk consumer carrying one twelve-cell frond.
fn body_plan() -> (BodyPhenotype, mesocosm_core::PartId) {
    let mut world = bulk_world(4_242, 24);
    let part = frond_on(&mut world);
    (world.phenotype().expect("embodied").clone(), part)
}

/// The frozen context the fixtures declare: one body plan, one granted
/// candidate, one integer budget, and the ground under the body.
fn context(phenotype: &BodyPhenotype, ground_mg: i64) -> Request {
    Request::frozen(
        &packed(),
        phenotype,
        vec![gland().qualified()],
        1_500,
        vec![Ambient {
            name: "ground_mg".to_owned(),
            value: ground_mg,
        }],
    )
}

/// The rich and lean grounds the two fixtures declare.
///
/// Neither is a magic number: the frond prices a cell at 23 mg, so a five-cell
/// gland holds 115 mg, and the script's rule is whether the ground under the
/// body can charge what is being asked for. 400 can; 20 cannot.
const RICH_GROUND: i64 = 400;
const LEAN_GROUND: i64 = 20;

/// The ground a fixture declares it was recorded under.
fn ground_of(fixture: &Fixture) -> i64 {
    fixture
        .request()
        .conditions
        .iter()
        .find(|ambient| ambient.name == "ground_mg")
        .expect("every declared context names the ground")
        .value
}

fn hunger() -> mesocosm_core::ConditionId {
    mesocosm_core::discovery::conditions()
        .into_iter()
        .find(|condition| condition.name == "mesocosm:endured-hunger")
        .expect("the table holds it")
        .id()
}

/// Holds the played body under the starved line, with a hand on it, until the
/// endurance condition lands.
fn endure(world: &mut World, ticks: u64) {
    for _ in 0..ticks {
        let Some(me) = world.controlled_id() else {
            return;
        };
        let upkeep = world.controlled().expect("alive").upkeep_mg();
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == me)
            .expect("in the roster")
            .energy_mg = upkeep * (mesocosm_core::STARVED_UPKEEP_TICKS - 1);
        world.apply(Intent::Resume);
    }
}

// ---------------------------------------------------------------------------
// 1. The same accepted allocation the native fixture produced
// ---------------------------------------------------------------------------

#[test]
fn lua_proposes_the_same_accepted_allocation_as_the_native_fixture() {
    // **The parity claim, twice over.** First that the two proposals lower to
    // the same allocation, then that the one validator turns both into the
    // same instruction — same revision, same sites, same cost, same resulting
    // digest. Anything less would leave "the same allocation" meaning "looked
    // similar".
    let registry = Arc::new(packed());
    let (phenotype, part) = body_plan();

    // The native fixture: what the discovery grants, proposed by the game.
    let native = mesocosm_core::discovery::conditions()
        .into_iter()
        .find(|condition| condition.id() == hunger())
        .expect("the table holds it")
        .grants
        .propose(&phenotype, Arrangement::Automatic)
        .expect("the frond is somewhere to put it");

    // The authored path: the same body, on ground that can charge a full
    // gland.
    let mut runner = Runner::load(&script(), Policy::default()).expect("the script loads");
    let entropy = Entropy::from_seed(fixture("gland_rich_ground").seed);
    let proposal = runner
        .propose(&context(&phenotype, RICH_GROUND), &entropy)
        .expect("the script proposes");
    assert_eq!(
        proposal.sites,
        vec![Expression {
            part: part.0,
            process: gland().qualified(),
            cells: 5,
        }],
        "the same five cells of the same frond the native fixture asked for"
    );

    let authored = lower(&registry, &phenotype, &proposal).expect("it lowers");
    assert_eq!(
        authored.sites, native.sites,
        "the same complete desired state"
    );
    assert_eq!(authored.parts, native.parts);

    let mut by_script = phenotype.clone();
    let mut by_game = phenotype.clone();
    let scripted = by_script.develop(&registry, &authored).expect("valid");
    let gamed = by_game.develop(&registry, &native).expect("valid");
    assert_eq!(
        scripted.instruction, gamed.instruction,
        "one validator, one instruction: revision, sites, cost and digest all"
    );
    assert_eq!(scripted.instruction.cost_cells, 5);
    assert_eq!(by_script, by_game, "and the same body afterwards");
}

#[test]
fn the_authored_proposal_walks_the_played_door_to_the_same_development() {
    // The same claim through a live world rather than a frozen context: the
    // request is built from what the world holds, and the ground the script
    // reads is the column the body is actually standing on. The body enriches
    // it with an ordinary `Deposit` first, which is the only reason a gland is
    // worth its rent there.
    let mut world = bulk_world(4_242, 24);
    endure(&mut world, mesocosm_core::discovery::HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the condition landed");
    let part = frond_on(&mut world);
    // Enduring left the body under the starved line by construction. Fed again,
    // it can enrich the column it is standing on, which is the ordinary game
    // verb that makes a full gland worth its rent there.
    let me = world.controlled_id().expect("embodied");
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("in the roster")
        .energy_mg = 1_500;
    assert_eq!(
        world.apply(Intent::Deposit { mass_mg: 400 }),
        mesocosm_core::Outcome::Deposited { organism: me }
    );

    let request = Request::of(&world, hunger()).expect("embodied and discovered");
    assert_eq!(
        request.candidates,
        vec![gland().qualified()],
        "the request shows what the line came to, by name"
    );
    assert!(
        request
            .conditions
            .iter()
            .any(|ambient| ambient.name == "ground_mg" && ambient.value >= RICH_GROUND),
        "the enriched column is what the script reads"
    );

    let mut runner = Runner::load(&script(), Policy::default()).expect("the script loads");
    let proposal = runner
        .propose(
            &request,
            &Entropy::from_seed(fixture("gland_rich_ground").seed),
        )
        .expect("the script proposes");

    let registry = world.admitted();
    let phenotype = world.phenotype().expect("embodied").clone();
    let allocation = lower(&registry, &phenotype, &proposal).expect("it lowers");
    let mut candidate = phenotype.clone();
    let development = candidate.develop(&registry, &allocation).expect("valid");

    assert_eq!(development.instruction.cost_cells, 5);
    assert!(
        candidate
            .glands()
            .iter()
            .any(|(on, cells)| *on == part && *cells == 5),
        "five cells of that frond are a gland now"
    );
    // And the world was not touched by any of it: the script proposed, the
    // validator ruled, and nothing has been committed to the roster.
    assert_eq!(world.phenotype().expect("embodied").revision(), 0);
}

// ---------------------------------------------------------------------------
// 2. The same context and entropy produce the same proposal and draw trace
// ---------------------------------------------------------------------------

#[test]
fn the_same_context_and_entropy_produce_the_same_proposal_and_draw_trace() {
    let (phenotype, _) = body_plan();
    let request = context(&phenotype, RICH_GROUND);
    let recorded = fixture("gland_rich_ground");

    // Asserted twice, in two fresh sandboxes, so the answer cannot be a
    // leftover of the first run's VM state.
    let mut first = Runner::load(&script(), Policy::default()).expect("loads");
    let mut second = Runner::load(&script(), Policy::default()).expect("loads");
    let one = Entropy::from_seed(recorded.seed);
    let two = Entropy::from_seed(recorded.seed);

    let here = first.propose(&request, &one).expect("proposes");
    let there = second.propose(&request, &two).expect("proposes");
    assert_eq!(here, there, "the same proposal");
    assert_eq!(one.draws, two.draws, "and the same draw trace");
    assert_eq!(one.draws.len(), DRAWS, "the whole tape, every call");

    // And against the recorded fixture, which is the claim that survives this
    // session: the exact proposal and the exact draws.
    assert_eq!(here, recorded.expected);
    assert_eq!(one.draws, recorded.draws);
    recorded
        .check(&script(), Policy::default())
        .expect("the recorded fixture holds");

    // A different seed is a different tape, and this script reads it, so the
    // draws are load-bearing rather than decorative.
    let other = Entropy::from_seed(recorded.seed.wrapping_add(1));
    assert_ne!(other.draws, recorded.draws);
}

// ---------------------------------------------------------------------------
// 3. Contrasting contexts, one body plan
// ---------------------------------------------------------------------------

#[test]
fn contrasting_developmental_contexts_grow_different_phenotypes_from_one_plan() {
    // **The done-condition, stated as a difference rather than as two runs.**
    // One body plan, one script, one seed; two declared grounds; two valid
    // phenotypes that are not each other.
    let registry = Arc::new(packed());
    let (phenotype, part) = body_plan();
    let rich = fixture("gland_rich_ground");
    let lean = fixture("gland_lean_ground");
    assert_eq!(
        rich.request.expect, lean.request.expect,
        "the same body plan, so the difference is the context and nothing else"
    );
    assert_eq!(rich.seed, lean.seed, "and the same entropy");
    // The declared contexts, and the only thing that differs between them.
    assert_eq!(ground_of(&rich), RICH_GROUND);
    assert_eq!(ground_of(&lean), LEAN_GROUND);

    rich.check(&script(), Policy::default())
        .expect("the rich fixture holds");
    lean.check(&script(), Policy::default())
        .expect("the lean fixture holds");

    assert_eq!(
        rich.expected.sites[0].cells, 5,
        "rich ground charges a gland"
    );
    assert_eq!(
        lean.expected.sites[0].cells, 1,
        "lean ground charges a token one, and the frond keeps fixing"
    );

    let mut on_rich = phenotype.clone();
    let mut on_lean = phenotype.clone();
    on_rich
        .develop(
            &registry,
            &lower(&registry, &phenotype, &rich.expected).expect("lowers"),
        )
        .expect("valid");
    on_lean
        .develop(
            &registry,
            &lower(&registry, &phenotype, &lean.expected).expect("lowers"),
        )
        .expect("valid");

    assert_eq!(on_rich.glands(), vec![(part, 5)]);
    assert_eq!(on_lean.glands(), vec![(part, 1)]);
    assert_ne!(
        on_rich.digest(),
        on_lean.digest(),
        "two phenotypes, and the digest says so"
    );
    // Both are real bodies, not one body and one refusal: the lean one still
    // fixes on the tissue it kept.
    let fix = packed()
        .get(&ProcessId::new("mesocosm", "fix"))
        .expect("declared")
        .reference();
    assert!(on_lean.expresses_on(part, fix), "eleven cells still fix");
    assert!(
        on_rich.expresses_on(part, fix),
        "and on rich ground, seven do"
    );
    assert_eq!(
        on_rich.explain(part).expect("a living part").free,
        on_lean.explain(part).expect("a living part").free,
        "neither context left tissue idle; they spent the frond differently"
    );
}

#[path = "authored_gland/refusals.rs"]
mod refusals;

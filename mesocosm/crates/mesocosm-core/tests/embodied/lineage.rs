// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P4 and PD5: a line commits a program, and its descendants are born under it.
//!
//! One test per claim: the founding program changes no birth; a revision is
//! immutable; a birth expresses it through the one validator and pays for it;
//! a refusal is a named fact; a founder preview is deterministic; one program
//! grows two bodies on two grounds; an unplayed lineage takes the same path;
//! incorporation, acquisition and filial expression are three records; and the
//! commit's own refusals, by name.
//!
//! What the epoch boundary does with all of this is PE3a's, next door in
//! `round.rs`.

use mesocosm_core::discovery::HUNGER_TICKS;
use mesocosm_core::history::Event;
use mesocosm_core::program::{Conditions, Founder, RevisionId};
use mesocosm_core::rules::{EpochRule, WorldRules};
use mesocosm_core::{
    Appendage, ConditionId, Founding, Intent, OrganismId, Outcome, Recipe, Rejection, SpeciesId,
    Stage, Tagma, Unrevised, World,
};

use super::bulk_world;
use super::discovery::{endure, hunger};
use super::gland::{frond_on, gland};

/// A recipe whose bodies grow one plate: the only shape that admits a gland.
///
/// The declared site names a *shape*, so what a descendant needs is a body
/// plan that grows one — which is PE2's residue answered: a bulk consumer has
/// nowhere to put a gland until its line grows a plate.
fn plate_recipe() -> Recipe {
    Recipe::of(vec![Tagma::new(1, Appendage::Plate)])
}

/// A recipe whose bodies are bulk and nothing else.
fn bare_recipe() -> Recipe {
    Recipe::of(vec![Tagma::new(1, Appendage::None)])
}

/// Stands a world at its lineage checkpoint, which since PE3 is the only place
/// a revision is admitted.
///
/// The epoch rule is a world rule, so a fixture shortens the budget the way it
/// picks a seed rather than idling a thousand ticks to reach a boundary the
/// test is not about. A one-tick budget makes every tick a boundary; the
/// scoring window comes down with it so the rounds it fires are cheap.
fn at_the_checkpoint(world: World) -> World {
    let mut world = world.with_rules(
        WorldRules::native()
            .ending(EpochRule::Timed { ticks: 1 })
            .scoring_over(2),
    );
    world.apply(Intent::Idle);
    assert!(world.at_boundary(), "a one-tick budget is spent every tick");
    world
}

/// A world whose played critter has come through the starvation horizon, so
/// its line holds the gland candidate — standing at its lineage checkpoint.
fn discovered_world(seed: u64) -> World {
    let mut world = bulk_world(seed, 24);
    frond_on(&mut world);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(
        world.discovered(hunger()),
        "the line came through the horizon"
    );
    at_the_checkpoint(world)
}

/// Puts a body in reach of the ordinary breeding gate, which is itself
/// untouched — mature and past gestation, as `succession.rs` and `flows.rs` do.
fn ready_to_breed(world: &mut World, id: OrganismId) {
    let organism = world
        .organisms
        .iter_mut()
        .find(|o| o.id == id)
        .expect("in the roster");
    organism.stage = Stage::Mature;
    organism.since_offspring = u32::MAX;
    organism.energy_mg = 100_000;
    assert!(
        organism.can_reproduce(),
        "the ecology's own gate is satisfied"
    );
}

/// Steps until `parent` bears a child, returning it and the events of that
/// tick.
fn bear(world: &mut World, parent: OrganismId) -> (OrganismId, Vec<Event>) {
    for _ in 0..40 {
        world.apply(Intent::Idle);
        let events: Vec<Event> = world
            .drain_events()
            .into_iter()
            .map(|recorded| recorded.record)
            .collect();
        if let Some(child) = events.iter().find_map(|event| match *event {
            Event::Born {
                organism,
                parent: Some(who),
                ..
            } if who == parent => Some(organism),
            _ => None,
        }) {
            return (child, events);
        }
    }
    panic!("no birth in forty ticks");
}

fn revise(world: &mut World, condition: ConditionId) -> RevisionId {
    match world.apply(Intent::Revise { condition }) {
        Outcome::Revised { revision, .. } => revision,
        other => panic!("the commit was refused: {other:?}"),
    }
}

#[test]
fn the_founding_program_declares_nothing_and_changes_no_birth() {
    // **The founding revision is what a birth does today**: allocation seeded
    // from geometry. It is stored nowhere, so a world that has committed
    // nothing pays exactly one varint per lineage for the field and writes no
    // filial record at all.
    let mut world = bulk_world(4_242, 24);
    for species in world.lineages().all() {
        assert!(
            species.program().is_empty(),
            "a founded lineage holds the founding program"
        );
        assert_eq!(species.program().digest(), 0);
    }
    let empty = mesocosm_core::snapshot::encode(&mesocosm_core::Program::default()).unwrap();
    assert_eq!(empty.len(), 1, "an uncommitted program is one byte");

    let me = world.controlled_id().expect("embodied");
    ready_to_breed(&mut world, me);
    let (child, events) = bear(&mut world, me);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::Inherited { .. } | Event::Unexpressed { .. })),
        "a birth under the founding revision writes neither record"
    );
    let born = world.living().find(|o| o.id == child).expect("alive");
    assert!(
        !born.phenotype.expresses(gland()),
        "and grows exactly what geometry seeded"
    );
}

#[test]
fn a_second_commit_appends_and_leaves_the_first_untouched() {
    // Epoch-boundary plan §2: every committed adaptation creates an immutable
    // child revision; nothing edits the parent and nothing merges two.
    let mut world = discovered_world(9_001);
    let species = world.controlled().expect("embodied").species;

    let first = revise(&mut world, hunger());
    let before = world
        .lineages()
        .get(species)
        .and_then(|line| line.program().get(first))
        .cloned()
        .expect("committed");

    // A second commit on the same discovery: the line says it again, and the
    // record grows rather than being rewritten.
    let second = revise(&mut world, hunger());
    let program = world.lineages().get(species).expect("the line").program();
    assert_ne!(first, second);
    assert_eq!(program.len(), 2);
    assert_eq!(
        program.get(first),
        Some(&before),
        "the parent revision is byte-identical after a child was committed"
    );
    assert_eq!(program.get(second).unwrap().parent, Some(first));
    assert_eq!(program.current().map(|r| r.id), Some(second));
    assert_eq!(
        program.get(first).unwrap().cites.condition,
        hunger(),
        "and it cites the discovery it was committed against"
    );
}

#[test]
fn a_birth_expresses_its_lines_revision_and_pays_for_it() {
    // The gate's headline. A line comes to a gland, commits a program that
    // declares one, and the next descendant of that line is *born* with it —
    // through `BodyPhenotype::develop`, out of the child's own reserve, into
    // the column under it.
    let mut world = discovered_world(4_242);
    let me = world.controlled_id().expect("embodied");
    let species = world.controlled().expect("embodied").species;
    world.lineages_mut().set_recipe(species, plate_recipe());
    let revision = revise(&mut world, hunger());

    ready_to_breed(&mut world, me);
    let (child, events) = bear(&mut world, me);

    let inherited = events
        .iter()
        .find_map(|event| match *event {
            Event::Inherited {
                organism,
                revision: under,
                part,
                cost_mg,
                ..
            } if organism == child => Some((under, part, cost_mg)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no filial record among {events:?}"));
    assert_eq!(inherited.0, revision, "the record names the revision");
    assert!(inherited.2 > 0, "and what expressing it cost");

    let born = world.living().find(|o| o.id == child).expect("alive");
    assert!(
        born.phenotype.expresses(gland()),
        "the descendant is born expressing the admitted option"
    );
    assert_eq!(
        born.phenotype.glands().first().map(|(part, _)| *part),
        Some(inherited.1),
        "on the part the record names"
    );
    assert!(
        born.phenotype.revision() > 0,
        "which is a committed development, not a seeding"
    );
}

#[test]
fn a_child_with_nowhere_to_put_it_is_born_anyway_and_the_record_says_why() {
    // PE2's residue, one generation down: a candidate that cannot be taken is
    // the ordinary case. The birth happens under geometry seeding and the
    // record names the revision it could not express — never a silent
    // fallback, which would make an inherited program unfalsifiable.
    let mut world = discovered_world(4_242);
    let me = world.controlled_id().expect("embodied");
    let species = world.controlled().expect("embodied").species;
    world.lineages_mut().set_recipe(species, bare_recipe());
    let revision = revise(&mut world, hunger());

    ready_to_breed(&mut world, me);
    let (child, events) = bear(&mut world, me);

    let why = events
        .iter()
        .find_map(|event| match *event {
            Event::Unexpressed {
                organism,
                revision: under,
                why,
                ..
            } if organism == child => Some((under, why)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no refusal record among {events:?}"));
    assert_eq!(why.0, revision, "which revision");
    assert_eq!(
        why.1,
        mesocosm_core::Unexpressed::NoSite {
            role: mesocosm_core::Role::Plate
        },
        "and why, by name"
    );
    let born = world.living().find(|o| o.id == child).expect("alive");
    assert!(!born.phenotype.expresses(gland()));
}

#[test]
fn a_founder_preview_is_the_same_body_twice() {
    // A preview is a prediction and an explanation receipt (phenotype plan
    // §3), so the same program under the same declared inputs must reproduce
    // it. No world is touched by either call.
    let mut world = discovered_world(7_007);
    let species = world.controlled().expect("embodied").species;
    world.lineages_mut().set_recipe(species, plate_recipe());
    revise(&mut world, hunger());

    let before = mesocosm_core::state_hash(&world);
    let line = world.lineages().get(species).expect("the line");
    let founder = Founder {
        mass_mg: 1_000,
        palette: Founding::default().palette(),
        conditions: Conditions {
            ground_mg: 400,
            material_mg: 1_500,
        },
    };
    let once = line.preview(world.ruleset(), founder, 2).expect("realizes");
    let twice = line.preview(world.ruleset(), founder, 2).expect("realizes");

    assert_eq!(once.phenotype.digest(), twice.phenotype.digest());
    assert_eq!(once.program, twice.program);
    assert!(once.filial.expect("a revision to express").is_ok());
    assert_eq!(
        mesocosm_core::state_hash(&world),
        before,
        "a preview mutates no world"
    );
}

#[test]
fn one_program_grows_two_bodies_on_rich_and_lean_ground() {
    // **The variance the ruling asks for**: another descendant may realize
    // differently under different world conditions, and that is expression of
    // one inherited program rather than an implicit mutation. Same program
    // digest, two phenotype digests.
    //
    // The two grounds are PD4's own fixtures' — `gland_rich_ground` at 400 mg
    // and `gland_lean_ground` at 20 mg — and the rule is the game's own
    // dormancy rule: a line does not grow more gland than the column under it
    // could charge.
    let mut world = discovered_world(7_007);
    let species = world.controlled().expect("embodied").species;
    world.lineages_mut().set_recipe(species, plate_recipe());
    revise(&mut world, hunger());
    let line = world.lineages().get(species).expect("the line");

    let under = |ground_mg: u64| {
        line.preview(
            world.ruleset(),
            Founder {
                mass_mg: 1_000,
                palette: Founding::default().palette(),
                conditions: Conditions {
                    ground_mg,
                    material_mg: 1_500,
                },
            },
            2,
        )
        .expect("realizes")
    };
    let rich = under(400);
    let lean = under(20);

    assert_eq!(
        rich.program, lean.program,
        "one program: the line committed once"
    );
    assert_ne!(
        rich.phenotype.digest(),
        lean.phenotype.digest(),
        "and two bodies"
    );
    let cells = |preview: &mesocosm_core::Preview| {
        preview
            .phenotype
            .glands()
            .first()
            .map(|(_, cells)| *cells)
            .unwrap_or(0)
    };
    assert!(
        cells(&rich) > cells(&lean),
        "rich {} should out-grow lean {}",
        cells(&rich),
        cells(&lean)
    );
    assert_eq!(cells(&lean), 1, "lean ground keeps a token gland");
}

#[test]
fn an_unplayed_lineage_takes_the_same_path() {
    // No policy chooses this — whether an NPC line ever revises is playable
    // ecology plan §6 ruling 5 and still open. What is settled is that when
    // something does, it takes `World::revise` and its descendants are
    // developed by the identical code, with nobody embodied in them.
    let mut world = discovered_world(3_300);
    let me = world.controlled_id().expect("embodied");
    let mine = world.controlled().expect("embodied").species;
    let (npc, breeder) = world
        .living()
        .find(|o| o.species != mine)
        .map(|o| (o.species, o.id))
        .expect("the enclosure holds another line");

    world.lineages_mut().set_recipe(npc, plate_recipe());
    let revision = world.revise(npc, hunger()).expect("committed");
    assert_ne!(breeder, me, "nobody is holding this one");

    ready_to_breed(&mut world, breeder);
    let (child, events) = bear(&mut world, breeder);
    assert!(
        events.iter().any(|event| matches!(
            *event,
            Event::Inherited { organism, revision: under, .. }
                if organism == child && under == revision
        )),
        "an unplayed descendant is born expressing it too: {events:?}"
    );
    assert!(
        world
            .living()
            .find(|o| o.id == child)
            .expect("alive")
            .phenotype
            .expresses(gland())
    );
}

#[test]
fn incorporation_discovery_and_filial_expression_are_three_records() {
    // The distinction PD5 exists to keep. One line walks all three: it comes
    // to the gland by enduring, expresses it on the body it is standing in,
    // commits it to the line, and its next descendant arrives already
    // expressing it. Four events, four different references, and no two of
    // them are the same fact.
    let mut world = discovered_world(4_242);
    let me = world.controlled_id().expect("embodied");
    let species = world.controlled().expect("embodied").species;

    // 1. Dormant acquisition. Already landed by enduring; drained here so the
    //    later ticks are read cleanly.
    let acquired: Vec<Event> = world
        .drain_events()
        .into_iter()
        .map(|recorded| recorded.record)
        .collect();
    let condition = acquired
        .iter()
        .find_map(|event| match *event {
            Event::Discovered { condition, .. } => Some(condition),
            _ => None,
        })
        .expect("the discovery is a record of its own");
    assert_eq!(condition, hunger());

    // 2. Somatic incorporation: the body you are standing in changes, and it
    //    pays. Nothing about this is heritable.
    let somatic = match world.apply(Intent::Express { condition }) {
        Outcome::Expressed { part, cost_mg, .. } => (part, cost_mg),
        other => panic!("the played body could not express it: {other:?}"),
    };
    let expressed = world
        .drain_events()
        .into_iter()
        .find_map(|recorded| match recorded.record {
            Event::Expressed { organism, part, .. } if organism == me => Some(part),
            _ => None,
        })
        .expect("a somatic development is a record of its own");
    assert_eq!(expressed, somatic.0);
    assert!(somatic.1 > 0, "and it cost the body milligrams");

    // 3. The lineage commit, which is what makes the next one possible.
    world.lineages_mut().set_recipe(species, plate_recipe());
    let revision = revise(&mut world, hunger());
    assert!(
        world.drain_events().into_iter().any(
            |recorded| matches!(recorded.record, Event::Revised { by: Some(who), .. } if who == me)
        ),
        "committing is on the hand's own causal line"
    );

    // 4. Filial expression: a different body, which never ate and never
    //    endured, arrives already expressing it.
    ready_to_breed(&mut world, me);
    let (child, events) = bear(&mut world, me);
    let inherited = events
        .iter()
        .find_map(|event| match *event {
            Event::Inherited {
                organism,
                revision: under,
                ..
            } if organism == child => Some(under),
            _ => None,
        })
        .expect("filial expression is a record of its own");

    assert_ne!(
        child, me,
        "three records, and the third is a different body"
    );
    assert_eq!(inherited, revision);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::Discovered { .. } | Event::Expressed { .. })),
        "a birth under a revision is neither a discovery nor a meal"
    );
    let line = world.lineages().get(species).expect("the line");
    assert_eq!(
        line.program()
            .get(revision)
            .expect("committed")
            .cites
            .condition,
        condition,
        "and the revision still resolves to the discovery it came from"
    );
}

#[test]
fn the_commit_is_gated_to_the_lineage_checkpoint_and_says_so_in_one_place() {
    // **The placeholder replaced** (PE3). Bodies change between epochs and not
    // during them, so the commit consults `revision_admitted_now`, that reads
    // `World::at_boundary`, and every other tick is refused `NotYet`. The
    // played door and the unplayed one are the same transaction, so they are
    // gated together and neither can be a second way for a program to move.
    let mut world = bulk_world(9_001, 24);
    frond_on(&mut world);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()));
    assert!(
        !world.revision_admitted_now(),
        "the default budget is nowhere near spent"
    );
    assert_eq!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Rejected(Rejection::Unrevised(Unrevised::NotYet)),
        "and mid-epoch it is refused by name"
    );

    let mut world = at_the_checkpoint(world);
    assert!(world.revision_admitted_now(), "inside, it is admitted");
    assert!(matches!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Revised { .. }
    ));
}

#[test]
fn revising_a_condition_the_line_has_not_come_to_is_refused_by_name() {
    let mut world = at_the_checkpoint(bulk_world(4_242, 24));
    assert_eq!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Rejected(Rejection::Unrevised(Unrevised::Undiscovered(hunger())))
    );
    assert_eq!(
        world.revise(SpeciesId(9_999), hunger()),
        Err(Unrevised::NoSuchSpecies(SpeciesId(9_999))),
        "and a line this world never heard of is its own answer"
    );
}

#[test]
fn a_revision_this_world_could_never_express_is_refused_at_the_commit() {
    // The gland removed from the admitted set. The condition table is native,
    // so the line still comes to a candidate citing it — and every descendant's
    // development would then refuse `UnknownProcess` forever. So the commit
    // refuses once instead: a program that can never be expressed is not a
    // program, and this is the honest place to say so.
    let mut defs: Vec<_> = mesocosm_core::Registry::native().all().cloned().collect();
    defs.retain(|def| def.id.name != "secrete");
    let without = std::sync::Arc::new(mesocosm_core::Registry::admit(defs).expect("no collision"));
    let mut world =
        World::founded_on(4_242, 24, Founding::default(), without).expect("the palette is valid");

    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = mesocosm_core::Organism {
        stage: Stage::Mature,
        ..mesocosm_core::Organism::founding(
            me,
            species,
            mesocosm_core::Kingdom::Consumer,
            mesocosm_core::VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the line came to it anyway");

    let mut world = at_the_checkpoint(world);
    assert_eq!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Rejected(Rejection::Unrevised(Unrevised::Nothing))
    );
    assert!(
        world
            .lineages()
            .get(species)
            .expect("the line")
            .program()
            .is_empty(),
        "and nothing was committed"
    );
}

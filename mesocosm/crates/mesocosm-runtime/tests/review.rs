// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **The lineage checkpoint, driven.** PE3b's runtime half: the played line's
//! turn arrives with the boundary, carries the reckoning and the trend beside
//! the table, and goes away again when the world resumes.
//!
//! The second proposal source lives here too, because it is the one half of a
//! review that reads a disk: a pack declares an expression script, the review
//! runs it through the bounded runner with the host's own entropy, and the row
//! carries both proposals marked by name.

use std::path::{Path, PathBuf};

use mesocosm_core::discovery::HUNGER_TICKS;
use mesocosm_core::rules::{EpochRule, WorldRules};
use mesocosm_core::{
    Appendage, Attachment, Intent, Provenance, Recipe, Tagma, Trend, VolumeRef, World, Yaw,
};
use mesocosm_runtime::{Authored, Occasion, Review, Runtime, Source};

/// The shipped pack, from this crate's own location.
fn pack_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|repo| repo.join("packs").join("mesocosm"))
        .expect("the workspace root is two above this crate")
}

fn hunger() -> mesocosm_core::ConditionId {
    mesocosm_core::discovery::conditions()
        .into_iter()
        .find(|condition| {
            mesocosm_core::discovery::name_of(condition.id())
                .is_some_and(|name| name == "mesocosm:endured-hunger")
        })
        .expect("the table holds it")
        .id()
}

/// Holds the played body under the starved line, with a hand on it.
///
/// `Intent::Resume` is the free verb that keeps the hand on: it moves nothing
/// and resets the idle run, which is exactly what it was built for.
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

/// A world whose played critter is a plain bulk root, as the core's own
/// embodied fixtures build one: the DC4 archetype is already an actuator, and
/// these claims are about what a *line* is offered rather than about what
/// worldgen happens to hand it.
fn bulk_world(seed: u64, founders: u32) -> World {
    let mut world = World::new(seed, founders);
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
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

/// A plate on the played critter, held up in a canopy position — the only
/// shape that admits a gland, and what the pack's script goes looking for.
fn frond_on(world: &mut World) {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            [6, 4, 1],
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root");
}

/// A world whose played line has come through hunger, grows the shape that
/// carries the reward, and is standing at its lineage checkpoint.
fn at_a_boundary(seed: u64) -> World {
    let mut world = bulk_world(seed, 24);
    frond_on(&mut world);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the line came through");
    let species = world.controlled().expect("embodied").species;
    world
        .lineages_mut()
        .set_recipe(species, Recipe::of(vec![Tagma::new(1, Appendage::Plate)]));

    let mut world = world.with_rules(
        WorldRules::native()
            .ending(EpochRule::Timed { ticks: 1 })
            .scoring_over(4),
    );
    world.apply(Intent::Idle);
    assert!(world.at_boundary());
    world
}

fn review_of(world: &World, authored: Option<&Authored>) -> Review {
    Review::of(world, &[], Trend::default(), authored).expect("somebody is embodied")
}

#[test]
fn the_review_stands_only_while_the_world_holds_at_a_lineage_checkpoint() {
    // The driver's half: it arrives with the boundary and goes away when the
    // world resumes. An ordinary tick of play never has one.
    let mut rt = Runtime::new(4_242, 40, 10).with_max_steps(u64::MAX);
    rt.queue(Intent::Resume);
    rt.step(1);
    assert!(rt.review().is_none(), "an ordinary tick asks nothing");

    // The default epoch budget, driven with a hand on the critter throughout.
    // A run that long meets births and deaths on the way, and those are the
    // *individual* checkpoint: they are answered and the run carries on, which
    // is also what shows that the review belongs to one of the three occasions
    // and not to the machinery they share.
    for _ in 0..mesocosm_core::rules::DEFAULT_EPOCH_TICKS * 4 {
        match rt.checkpoint().map(|held| held.occasion) {
            Some(Occasion::Epoch(_)) => break,
            Some(_) => {
                assert!(
                    rt.review().is_none(),
                    "an individual question is not a review"
                );
                let answer = rt
                    .checkpoint()
                    .and_then(|held| held.heir())
                    .map(|organism| Intent::TakeControl { organism })
                    .unwrap_or(Intent::Resume);
                rt.queue(answer);
            },
            // A boundary is only ever *asked* of a hand, so a run that lost
            // its body and its line takes another one. Ordinary
            // `TakeControl` through the ordinary eligibility gate — the same
            // door a player walks through after a line dies out.
            None if !rt.world().is_embodied() => {
                let Some(next) = rt
                    .world()
                    .living()
                    .find(|organism| rt.world().is_eligible(organism.id))
                    .map(|organism| organism.id)
                else {
                    panic!("the enclosure emptied");
                };
                rt.queue(Intent::TakeControl { organism: next });
            },
            None => rt.queue(Intent::Resume),
        }
        rt.step(1);
    }
    let checkpoint = rt.checkpoint().expect("the epoch's budget was spent");
    assert!(
        matches!(checkpoint.occasion, Occasion::Epoch(_)),
        "the lineage checkpoint, not a birth: {checkpoint:?}"
    );

    let review = rt.review().expect("the played line's turn").clone();
    assert_eq!(review.tick, rt.world().tick);
    assert_eq!(
        review.lineage,
        rt.world().controlled().expect("embodied").species
    );
    assert!(!review.rows.is_empty(), "the status quo is always a row");
    assert_eq!(review.rows[0].offer.candidate, None);
    assert!(
        review.rows[0].sources.is_empty(),
        "and the status quo proposes nothing"
    );

    // Held: the driver does not step, and the review does not change under it.
    assert_eq!(rt.step(4), 0, "the world is stopped");
    assert_eq!(rt.review(), Some(&review), "and asking again asks the same");

    // A revision answers the question without closing it — the one answer that
    // leaves the player at the board — and the reading is taken again rather
    // than left describing the program the line may no longer have. This one is
    // refused (a default line has come to nothing), which is the point: the
    // hold and the re-read are the driver's and do not depend on the commit
    // landing.
    rt.queue(Intent::Revise {
        condition: hunger(),
    });
    assert_eq!(
        rt.step(1),
        1,
        "a revision is an answer, so the world stepped"
    );
    assert!(
        rt.checkpoint()
            .is_some_and(|held| matches!(held.occasion, Occasion::Epoch(_))),
        "and the question is still standing"
    );
    assert!(rt.review().is_some(), "with the board re-read under it");

    rt.queue(Intent::Resume);
    rt.step(1);
    assert!(rt.review().is_none(), "resuming puts it away");
}

#[test]
fn a_review_built_twice_is_the_same_review_scripts_included() {
    // The determinism the whole panel rests on, this time with the authored
    // source in it: the entropy is drawn off the world without moving it, and
    // each call loads its own runner, so a second reading is the first.
    let world = at_a_boundary(7_007);
    let authored = Authored::load(&pack_root()).expect("the shipped pack admits");
    assert!(!authored.is_empty(), "it declares an expression script");

    let once = review_of(&world, Some(&authored));
    let twice = review_of(&world, Some(&authored));
    assert_eq!(once, twice);
}

#[test]
fn a_pack_expression_appears_beside_the_discovered_proposal_and_is_marked() {
    // **Two proposal sources over one validator** (PD4's `expression` arm, in
    // its first production consumer). The game builds one proposal from the
    // discovery; the pack's script builds another; the row shows both, named.
    let world = at_a_boundary(7_007);
    let authored = Authored::load(&pack_root()).expect("the shipped pack admits");

    let review = review_of(&world, Some(&authored));
    let row = review
        .rows
        .iter()
        .find(|row| row.offer.candidate == Some(hunger()))
        .expect("the line came to something");

    let sources: Vec<Source> = row.sources.iter().map(|proposed| proposed.source).collect();
    assert_eq!(sources, [Source::Discovery, Source::Authored]);
    for proposed in &row.sources {
        assert_eq!(proposed.refused, None, "neither refused: {proposed:?}");
        let (part, cells) = proposed.site.expect("each proposes a site");
        assert!(cells > 0, "on real tissue");
        assert!(
            world
                .phenotype()
                .expect("embodied")
                .mosaic(mesocosm_core::PartId(part))
                .is_some(),
            "and on a part this body has"
        );
    }

    // Without the pack, the same row simply has one source. That is what "if no
    // pack expression applies, the row simply has one source" means.
    let alone = review_of(&world, None);
    let row = alone
        .rows
        .iter()
        .find(|row| row.offer.candidate == Some(hunger()))
        .expect("the same candidate");
    assert_eq!(row.sources.len(), 1);
    assert_eq!(row.sources[0].source, Source::Discovery);
}

#[test]
fn the_review_only_offers_a_commit_for_a_row_the_world_would_admit() {
    // The board's commit key cannot send a revision the world would only
    // reject: the status quo commits nothing, and neither does a candidate
    // carrying a reason.
    let world = at_a_boundary(3_300);
    let review = review_of(&world, None);

    assert_eq!(review.commit(0), None, "the status quo commits nothing");
    assert_eq!(
        review.commit(99),
        None,
        "and neither does a row that is not there"
    );
    for (index, row) in review.rows.iter().enumerate() {
        assert_eq!(
            review.commit(index).is_some(),
            row.offer.takeable(),
            "row {index}: {row:?}"
        );
    }
    assert_eq!(
        review.takeable().count(),
        1,
        "one candidate on this table can be taken"
    );
}

#[test]
fn the_authored_source_is_absent_when_a_pack_declares_no_expression() {
    // A pack with nothing to say about development contributes nothing, rather
    // than an empty row that reads as a refusal.
    let empty = Authored::default();
    assert!(empty.is_empty());
    let world = at_a_boundary(7_007);
    for row in review_of(&world, Some(&empty)).rows {
        assert!(row.sources.len() <= 1, "{row:?}");
    }
}

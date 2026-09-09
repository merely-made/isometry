// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The epoch boundary, read against the ledger. (PE3a)
//!
//! Split out of `flows.rs` at the 600-line ceiling, the same way
//! `transfers.rs` was. Ending an epoch runs a whole adaptation round *inside*
//! a tick — every unplayed line grows its candidates in copies of this world
//! and the winners commit — and none of that may move a milligram of the
//! enclosure. The harness is next door and shared, because two files
//! reconciling ticks two different ways is how they would come to disagree.

use mesocosm_core::{Intent, World};

use super::stepped;

#[test]
fn matter_is_conserved_across_an_epoch_boundary_and_an_npc_commit() {
    // **PE3a's conservation case.** Ending an epoch runs an adaptation round
    // inside the tick: every unplayed line scores its candidates by growing
    // them in copies of this world, and the winners commit. None of that may
    // move a milligram of the enclosure — a copy is a copy, and a committed
    // revision is a program entry rather than tissue. `stepped` reconciles
    // every compartment against the stream on each tick, so this is that claim
    // asked at exactly the ticks a boundary lands on.
    let mut world = World::new(4_242, 24);
    let me = world.controlled_id().expect("a played critter");
    let organism = world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("here");
    let (species, position) = (organism.species, organism.position);
    *organism = mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
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
    // Through the starvation horizon, so the enclosure holds a candidate for
    // the unplayed lines to weigh. Without one, every round is empty and this
    // test would prove only that an empty round moves nothing.
    for _ in 0..=mesocosm_core::discovery::HUNGER_TICKS {
        let upkeep = world.controlled().expect("alive").upkeep_mg();
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == me)
            .expect("in the roster")
            .energy_mg = upkeep * (mesocosm_core::STARVED_UPKEEP_TICKS - 1);
        world.apply(Intent::Resume);
    }
    assert!(!world.discoveries().is_empty(), "there is one to weigh");
    let opening_matter = world.total_matter_mg();

    let mut world = world.with_rules(
        mesocosm_core::WorldRules::native()
            .ending(mesocosm_core::rules::EpochRule::Timed { ticks: 4 })
            .scoring_over(4),
    );
    // To the next boundary. The budget runs from the tick the epoch began on,
    // not from a multiple of the tick count, so this asks the world rather
    // than counting.
    for _ in 0..8 {
        if world.at_boundary() {
            break;
        }
        world.apply(Intent::Idle);
    }
    assert!(world.at_boundary(), "the budget is spent");
    let weighed = world
        .initiative()
        .into_iter()
        .filter(|line| world.candidates(*line).len() > 1)
        .count();
    assert!(weighed > 0, "unplayed lines had something to weigh");

    // An unplayed line commits, through the identical transaction the round
    // uses. Whether the round's own ordering happens to take it in this
    // enclosure is a question about income; whether committing one moves a
    // milligram is this test's, and it must not.
    let npc = world
        .living()
        .map(|o| o.species)
        .find(|line| *line != species && world.candidates(*line).len() > 1)
        .expect("an unplayed line can express it");
    let condition = world.discoveries()[0].condition;
    world
        .revise(npc, condition)
        .expect("committed at the lineage checkpoint");

    let opened = world.epoch;
    for tick in 1..=40 {
        stepped(
            &mut world,
            Intent::Idle,
            &format!("on boundary tick {tick}"),
        );
    }
    assert!(
        world.epoch > opened + 1,
        "several epochs ended inside the reconciled run"
    );
    assert_eq!(
        world.total_matter_mg(),
        opening_matter,
        "and the enclosure holds exactly what it held"
    );
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **A refused transaction reaches neither record.**
//!
//! Split out of `flows.rs` at the six-hundred-line ceiling, beside `boundary`,
//! `transfers` and `dev`, and sharing the same harness for the reason they all
//! do: two files reconciling ticks two different ways is how they would come to
//! disagree.
//!
//! It is one claim, and it is the other half of the file's own sentence.
//! Accepted and refused intents cannot disagree with the stream because they
//! share a commit point: the accepted branch emits both records, the refused one
//! returns before reaching either. `dev.rs` next door makes the same claim
//! about DT3's four.

use mesocosm_core::flow::Process;
use mesocosm_core::{Intent, Placement, World};

use super::stepped;

#[test]
fn an_accepted_deposit_is_in_the_stream_and_a_refused_one_is_not() {
    // Accepted and refused transactions cannot disagree with the stream,
    // because they share a commit point: the accepted branch emits, the refused
    // one returns before reaching it.
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);

    let flows = stepped(
        &mut world,
        Intent::Deposit { mass_mg: 60 },
        "on the deposit",
    );
    let deposits: Vec<u64> = flows
        .iter()
        .filter(|f| f.record.process == Process::Deposit)
        .map(|f| f.record.amount_mg)
        .collect();
    assert_eq!(deposits, vec![60], "one deposit, for what was deposited");

    let refused = stepped(
        &mut world,
        Intent::Deposit { mass_mg: u64::MAX },
        "on the refused deposit",
    );
    assert!(
        !refused.iter().any(|f| f.record.process == Process::Deposit),
        "a refusal moved nothing, so it recorded nothing"
    );
}

#[test]
fn a_refused_meal_leaves_the_prey_out_of_the_stream() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);

    let here = world.position().expect("embodied");
    let far = world
        .living()
        .filter(|o| Some(o.id) != world.controlled_id())
        .max_by_key(|o| (0..3).map(|a| (o.position[a] - here[a]).abs()).max())
        .map(|o| o.id)
        .expect("something is out of reach in a wide enclosure");

    let flows = stepped(
        &mut world,
        Intent::Metabolize {
            organism: far,
            placement: Placement::Planned,
        },
        "on the refused meal",
    );
    assert!(
        world.living().any(|o| o.id == far),
        "the refusal left it alive"
    );
    assert!(
        !flows.iter().any(|f| f.record.process == Process::Feeding
            && f.record.from.is_some_and(|s| s.organism == far)),
        "nothing was taken out of it, so nothing was recorded"
    );
}

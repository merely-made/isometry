// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Honest signals, bluffs, traps, and what gives a mimic away.
//!
//! Split out of `ecology/tests.rs` on 2026-08-29, with the carrion tests next
//! door, to bring that file back under the six-hundred-line ceiling.

use super::*;

#[test]
fn an_honest_organism_does_not_lie() {
    let plain = organism(Kingdom::Producer, 100);
    assert!(!plain.is_mimic());
    assert!(!plain.signals_falsely());

    let honestly_armed = Organism {
        signal: Signal::Warning,
        venom_mg: 80,
        ..organism(Kingdom::Producer, 100)
    };
    assert!(
        !honestly_armed.signals_falsely(),
        "a real warning is not a lie"
    );
}

#[test]
fn a_bluffer_warns_without_a_bite() {
    let bluffer = Organism {
        signal: Signal::Warning,
        venom_mg: 0,
        ..organism(Kingdom::Producer, 100)
    };
    assert!(bluffer.signals_falsely());
    assert!(bluffer.is_mimic());
}

#[test]
fn a_trap_looks_plain_and_bites() {
    let trap = Organism {
        signal: Signal::Plain,
        venom_mg: 120,
        guise: Kingdom::Producer,
        ..organism(Kingdom::Consumer, 100)
    };
    assert!(trap.is_mimic());
    assert!(trap.signals_falsely());
    assert!(
        trap.betrays_itself(),
        "unfair is fine, unknowable is not: a trap must leave a tell"
    );
}

#[test]
fn the_tell_is_that_a_trap_does_not_grow() {
    let mut honest = vec![organism(Kingdom::Producer, 200)];
    let mut trap = vec![Organism {
        guise: Kingdom::Producer,
        signal: Signal::Plain,
        venom_mg: 120,
        ..organism(Kingdom::Consumer, 200)
    }];

    run(&mut honest, 30);
    run(&mut trap, 30);
    assert!(honest[0].biomass_mg() > 200, "a real plant fixes energy");
    assert!(
        trap[0].biomass_mg() <= 200,
        "a plant that does not photosynthesise does not put on weight"
    );
    assert!(
        honest[0].biomass_mg() > trap[0].biomass_mg(),
        "and the gap between them is the tell"
    );
}

#[test]
fn a_lie_is_heritable() {
    let mut world = vec![Organism {
        signal: Signal::Warning,
        venom_mg: 0,
        ..organism(Kingdom::Producer, 400)
    }];
    run(&mut world, gestation_for_mass(400) + 10);
    let child = world.iter().find(|o| o.id.0 >= 100).expect("an offspring");
    assert_eq!(child.signal, Signal::Warning);
    assert_eq!(child.venom_mg, 0);
    assert!(
        child.is_mimic(),
        "a mimic lineage is learnable, not a coin flip"
    );
}

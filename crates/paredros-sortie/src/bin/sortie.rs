// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The sortie, printed.
//!
//! `cargo run -p paredros-sortie --bin sortie`. S3's judgment surface: the
//! march as a story a playtester can follow, ending with the same ask S2
//! saw counteroffered, answered differently, for a reason on the page.

use paredros_identity::Tick;
use paredros_social::Verdict;
use paredros_social::scene::SELA;
use paredros_sortie::scene;
use paredros_sortie::sortie::SortieEvent;

fn main() {
    let (answers, sortie) = scene::played_through();
    let name = |subject| sortie.society.name_of(subject).to_owned();

    println!("The muster:");
    for answer in &answers {
        println!("  {} {}.", name(answer.by), answer.verdict.name());
    }
    println!();

    for event in &sortie.events {
        match event {
            SortieEvent::Departed { at } => println!("t{}: the party departs.", at.0),
            SortieEvent::Fell { who, at, dropped } => {
                println!(
                    "t{}: {} falls {} down the scarp.",
                    at.0,
                    name(*who),
                    dropped
                )
            },
            SortieEvent::Wounded { who, at, revision } => println!(
                "t{}: {} is wounded (body revision {}).",
                at.0,
                name(*who),
                revision.0
            ),
            SortieEvent::Downed { who, at } => println!("t{}: {} is down.", at.0, name(*who)),
            SortieEvent::PactInvoked { at, .. } => {
                println!("t{}: the pact is invoked.", at.0)
            },
            SortieEvent::TaggedIn { at, to } => {
                println!("t{}: the player becomes {}.", at.0, name(*to))
            },
            SortieEvent::Tended { at, by, whom } => {
                println!("t{}: {} tends {}.", at.0, name(*by), name(*whom))
            },
            SortieEvent::TaggedOut { at } => println!("t{}: the player returns home.", at.0),
            SortieEvent::Dug { at, removed } => {
                println!("t{}: digging; {} voxels hewn.", at.0, removed)
            },
            SortieEvent::Took { at, .. } => println!("t{}: the salvage is taken.", at.0),
            SortieEvent::SharedOut { at, with } => {
                println!("t{}: the salvage is shared with {}.", at.0, name(*with))
            },
            SortieEvent::Returned { at } => println!("t{}: home again.", at.0),
        }
    }

    println!(
        "\n{} wounds, {} voxels hewn, hash {:#018x}.",
        sortie.wounds.len(),
        sortie.hewn,
        sortie.hash()
    );

    // The ask S2 saw counteroffered, put once more.
    let mut society = sortie.society;
    let after = scene::ask_again(&mut society, Tick(400));
    println!(
        "\nThe same ask that was counteroffered before the sortie: {} {}.",
        society.name_of(SELA),
        after.verdict.name()
    );
    if after.verdict == Verdict::Accept {
        for line in society.explain(&after.premises) {
            println!("    {line}");
        }
    }
}

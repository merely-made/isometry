// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The refusal scene, printed.
//!
//! `cargo run -p paredros-social --bin refusal`. This is the half of S1's
//! done-condition a machine cannot check: whether a playtester can say why each
//! of the three answered as they did, from what they were shown. The premises
//! below are the same values the tests assert on, put into words.

use paredros_social::scene;
use paredros_social::society::Society;

fn main() {
    let mut society = scene::society();

    println!("Aud asks three people to scout ahead.");
    println!(
        "  scouting at grade {}, danger {}, share {}, up to danger {}\n",
        scene::THE_WORK.grade,
        scene::THE_WORK.danger,
        scene::THE_TERMS.share,
        scene::THE_TERMS.danger_cap
    );

    let answers = scene::answers(&mut society).expect("the scene admits everyone it asks");
    for answer in &answers {
        println!("{} {}.", society.name_of(answer.by), answer.verdict.name());
        if let paredros_social::Verdict::Counteroffer(terms) = answer.verdict {
            println!(
                "  her terms: share {}, up to danger {}",
                terms.share, terms.danger_cap
            );
        }
        say(&society, &answer.premises);
    }

    println!("Aud and Bram make it a standing arrangement.\n");
    let rulings = scene::lifecycle(&mut society).expect("the arrangement holds");
    for ruling in &rulings {
        println!("{} {}.", society.name_of(ruling.by), ruling.kind.name());
        say(&society, &ruling.premises);
    }

    println!(
        "{} deeds in the log. Every answer above is one of them, and every \
         line above was read back out of them.",
        society.log().len()
    );
}

fn say(society: &Society, premises: &[paredros_social::Premise]) {
    for line in society.explain(premises) {
        println!("    {line}");
    }
    println!();
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The settling scene, printed.
//!
//! `cargo run -p paredros-social --bin home`. S2's judgment surface: whether
//! a playtester can say why each answer came back as it did, and can see
//! that an accepted agreement is what changed where someone lives and what
//! they do daily. The lines are the same premises the tests assert on.

use paredros_social::society::Society;
use paredros_social::{Premise, settling};

fn main() {
    let mut society = settling::society();
    let mut settlement = settling::settlement();

    println!("Aud offers a roof and daily work to each of the three.\n");
    let answers = settling::answers(&mut society).expect("the scene admits everyone it asks");
    for answer in &answers {
        let ask = settling::ask_of(answer.by);
        println!(
            "To {}: {} at grade {}, danger {}. {} {}.",
            society.name_of(answer.by),
            ask.work.craft.name(),
            ask.work.grade,
            ask.work.danger,
            society.name_of(answer.by),
            answer.verdict.name()
        );
        if let paredros_social::Verdict::Counteroffer(terms) = answer.verdict {
            println!(
                "  her terms: share {}, up to danger {}",
                terms.share, terms.danger_cap
            );
        }
        say(&society, &answer.premises);
    }

    println!("Two arrangements form; the homes follow from them.\n");
    settling::housings(&mut society, &mut settlement).expect("the settled offers hold");

    let names: Vec<(paredros_identity::SubjectId, String)> = society
        .companions()
        .map(|companion| (companion.subject, companion.name.clone()))
        .collect();
    for (subject, name) in names {
        let round = settlement.daily(&society, subject);
        match round.home.and_then(|id| settlement.dwelling(id)) {
            Some(dwelling) => println!(
                "{} lives in {} and daily does {} at grade {}.",
                name,
                dwelling.name,
                round
                    .does
                    .map(|work| work.craft.name())
                    .unwrap_or("nothing"),
                round.does.map(|work| work.grade).unwrap_or(0),
            ),
            None => println!("{name} lives as before."),
        }
    }

    println!(
        "\n{} deeds, {} tenancies. Every roof above traces to an agreement, \
         and ending that agreement is how a tenant moves out.",
        society.log().len(),
        settlement.tenancies().len()
    );
}

fn say(society: &Society, premises: &[Premise]) {
    for line in society.explain(premises) {
        println!("    {line}");
    }
    println!();
}

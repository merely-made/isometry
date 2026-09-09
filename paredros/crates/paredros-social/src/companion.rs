// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A companion, their declared crafts, and their own read of a piece of work.
//!
//! Skills here are what someone says they can do, not what a fight would prove.
//! The willingness owner never sees combat numbers; that is the point of the
//! two-owners rule, and it is why a grade is a plain declared integer.
//!
//! The played character is admitted here too. Everyone in this game is a peer,
//! and the player merely inhabits one of them, so there is no second kind of
//! record for the one being played.

use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};

use crate::offer::Work;

/// Plain words for plain work. Small vocabulary, kept small.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Craft {
    Scouting,
    Smithing,
    Healing,
    Hauling,
    Watching,
}

impl Craft {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Scouting => "scouting",
            Self::Smithing => "smithing",
            Self::Healing => "healing",
            Self::Hauling => "hauling",
            Self::Watching => "watching",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub craft: Craft,
    pub grade: u8,
}

impl Skill {
    pub fn new(craft: Craft, grade: u8) -> Self {
        Self { craft, grade }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Companion {
    pub subject: SubjectId,
    pub name: String,
    /// Declared, ordered, and theirs. Not combat stats.
    pub skills: Vec<Skill>,
    /// How much a given danger costs this person. Personality, one integer:
    /// a bold companion is negative, a careful one positive.
    pub caution: i16,
}

impl Companion {
    pub fn new(subject: SubjectId, name: &str, skills: Vec<Skill>, caution: i16) -> Self {
        Self {
            subject,
            name: name.to_owned(),
            skills,
            caution,
        }
    }

    pub fn grade_in(&self, craft: Craft) -> u8 {
        self.skills
            .iter()
            .filter(|skill| skill.craft == craft)
            .map(|skill| skill.grade)
            .max()
            .unwrap_or(0)
    }

    /// Their own read of whether this work is within them. Their read, not a
    /// verdict on them: nothing else in this crate asks whether they were right.
    pub fn read(&self, work: &Work) -> Confidence {
        Confidence {
            craft: work.craft,
            demanded: work.grade,
            held: self.grade_in(work.craft),
        }
    }
}

/// A companion's own read of a piece of work against their declared craft.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confidence {
    pub craft: Craft,
    pub demanded: u8,
    pub held: u8,
}

impl Confidence {
    pub fn margin(&self) -> i16 {
        i16::from(self.held) - i16::from(self.demanded)
    }

    /// Whether they believe they can do it at all. Below this they refuse, and
    /// the refusal is about the work rather than about the asker.
    pub fn sufficient(&self) -> bool {
        self.margin() >= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_craft_reads_as_grade_zero() {
        let bram = Companion::new(
            SubjectId(2),
            "Bram",
            vec![Skill::new(Craft::Scouting, 4)],
            0,
        );
        assert_eq!(bram.grade_in(Craft::Smithing), 0);

        let smithing = Work {
            craft: Craft::Smithing,
            grade: 2,
            danger: 0,
        };
        let read = bram.read(&smithing);
        assert_eq!(read.margin(), -2);
        assert!(!read.sufficient());
    }
}

// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One sortie and return: the joint receipt's machine.
//!
//! Every tick moves the played body toward its goal and lets each companion
//! move by their agreed part. There is no call that moves a companion
//! directly: the drivers read agreements, positions, and the ground, never
//! an order, which is the puppeteering canary held at the API as usual.
//!
//! The world itself is the hazard. `near::step`'s own law says a drop past
//! `COMFORT_DROP` is taken only when nothing else remains; this crate rules
//! that such a fall is a wound, uniformly, for anyone. What downs a body is
//! narrower: only the *played* body downs, because downing is a control
//! fact — it is what the pact watches for — while a wound is a body fact
//! and lands on whoever fell. Recorded as a finding; S4 may widen it.
//!
//! An injury is a body-revision fact: the wound bumps the subject's
//! [`BodyRevisionId`] in the shared facets, and the wound record names the
//! revision it created. What a revision means stays the wing phenotype
//! contract's business; Paredros only points at it.

use std::collections::BTreeMap;

use mesocosm_core::places::{Ground, ROCK};
use mesocosm_core::snapshot::{encode, hash_bytes};
use paredros_identity::{BodyRevisionId, Control, ControlIntent, Facets, SubjectId, Tick};
use paredros_social::agreement::EndReason;
use paredros_social::deed::DeedKind;
use paredros_social::offer::Work;
use paredros_social::response::RulingKind;
use paredros_social::society::Society;
use serde::{Deserialize, Serialize};

use crate::march;
use crate::party::{Pact, Part};

/// A fall this far is taken in stride; past it is a wound. Paredros's own
/// ruling about bodies, not a mechanical import: it coincides with the
/// near tier's `COMFORT_DROP` today (which that module keeps private), but
/// falls would hurt here even if the walker's willingness to take them
/// changed upstream.
pub const SAFE_FALL: i32 = 4;

/// How far ahead the scout ranges before holding for the others.
pub const LEAD: i32 = 8;
/// How close the healer keeps to the played body. Wide enough that a
/// rescue is a walk rather than a reach from where she already stands.
pub const TRAIL: i32 = 6;
/// Tending reaches one column over and a cliff's height down: the healer
/// works from the ledge rather than taking the same fall.
pub const TEND_REACH: i32 = 1;
pub const TEND_DROP: i32 = SAFE_FALL + 2;
/// Ticks without progress before the played body starts digging. The
/// grown terrain has pockets a one-voxel climb can never leave; the wing's
/// own carve verb is how a party gets out, and the hewn rock comes home
/// with them.
pub const STUCK: u64 = 4;
/// A sortie that has not come home in this many ticks has failed loudly.
pub const MAX_TICKS: u64 = 512;

/// A fall past comfort, and the body revision it created.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wound {
    pub subject: SubjectId,
    pub at: Tick,
    pub fell: i32,
    pub revision: BodyRevisionId,
}

/// What came back. `what` is the hillside's rock; which rock is a later
/// gate's question, and the position is the provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Salvage {
    pub what: u8,
    pub from: [i32; 3],
}

/// The sortie's own record, one entry per thing that happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortieEvent {
    Departed {
        at: Tick,
    },
    Fell {
        who: SubjectId,
        at: Tick,
        dropped: i32,
    },
    Wounded {
        who: SubjectId,
        at: Tick,
        revision: BodyRevisionId,
    },
    Downed {
        who: SubjectId,
        at: Tick,
    },
    PactInvoked {
        at: Tick,
        under: paredros_social::agreement::AgreementId,
    },
    TaggedIn {
        at: Tick,
        to: SubjectId,
    },
    Tended {
        at: Tick,
        by: SubjectId,
        whom: SubjectId,
    },
    TaggedOut {
        at: Tick,
    },
    /// The played body carved rock out of its way. `removed` is voxels.
    Dug {
        at: Tick,
        removed: u32,
    },
    Took {
        at: Tick,
        what: u8,
    },
    SharedOut {
        at: Tick,
        with: SubjectId,
    },
    Returned {
        at: Tick,
    },
}

pub struct Sortie {
    ground: Ground,
    pub society: Society,
    pub facets: Facets,
    pub control: Control,
    positions: BTreeMap<SubjectId, [i32; 3]>,
    parts: Vec<(SubjectId, Part)>,
    pact: Option<Pact>,
    outing: Option<paredros_social::agreement::AgreementId>,
    tend: Work,
    home: [i32; 3],
    site: [i32; 3],
    /// The waypoints of the return, ending at home. A scarp descended on
    /// the way out cannot be climbed back (the near tier lifts one voxel),
    /// so the way home is routed, not retraced.
    way_home: Vec<[i32; 2]>,
    way_index: usize,
    downed: Option<SubjectId>,
    /// Which way each walker last shouldered around an obstacle. Movement
    /// memory, not history: the trail is the record.
    shoulders: BTreeMap<SubjectId, i8>,
    /// The played body's best distance toward its goal, and how long since
    /// it improved. When it stops improving for [`STUCK`] ticks, digging
    /// starts.
    progress: (i32, u64),
    tick: u64,
    pub wounds: Vec<Wound>,
    pub carried: Option<Salvage>,
    /// Voxels the played body hewed out of the world getting there and
    /// back. Comes home with the salvage.
    pub hewn: u32,
    pub events: Vec<SortieEvent>,
    trail: Vec<Vec<[i32; 3]>>,
    done: bool,
}

impl Sortie {
    /// Musters at home. Everyone stands at the settlement whether or not
    /// they march: a refused companion is still a person with a position.
    #[allow(clippy::too_many_arguments)]
    pub fn muster(
        ground: Ground,
        society: Society,
        facets: Facets,
        control: Control,
        everyone: &[SubjectId],
        parts: Vec<(SubjectId, Part)>,
        pact: Option<Pact>,
        outing: Option<paredros_social::agreement::AgreementId>,
        tend: Work,
        home: [i32; 3],
        site: [i32; 3],
        way_home: Vec<[i32; 2]>,
        at: Tick,
    ) -> Self {
        let positions = everyone.iter().map(|subject| (*subject, home)).collect();
        let mut sortie = Self {
            ground,
            society,
            facets,
            control,
            positions,
            parts,
            pact,
            outing,
            tend,
            home,
            site,
            way_home,
            way_index: 0,
            downed: None,
            shoulders: BTreeMap::new(),
            progress: (i32::MAX, 0),
            tick: at.0,
            wounds: Vec::new(),
            carried: None,
            hewn: 0,
            events: vec![SortieEvent::Departed { at }],
            trail: Vec::new(),
            done: false,
        };
        sortie.push_trail();
        sortie
    }

    pub fn tick(&self) -> Tick {
        Tick(self.tick)
    }

    pub fn done(&self) -> bool {
        self.done
    }

    pub fn at(&self, subject: SubjectId) -> [i32; 3] {
        self.positions[&subject]
    }

    pub fn trail(&self) -> &[Vec<[i32; 3]>] {
        &self.trail
    }

    fn goal(&self) -> [i32; 2] {
        if self.carried.is_none() {
            [self.site[0], self.site[2]]
        } else {
            self.way_home[self.way_index]
        }
    }

    fn in_tend_reach(&self, healer: SubjectId, fallen: SubjectId) -> bool {
        let (at, down) = (self.positions[&healer], self.positions[&fallen]);
        march::apart(at, down) <= TEND_REACH && (at[1] - down[1]).abs() <= TEND_DROP
    }

    fn push_trail(&mut self) {
        let row = self.positions.values().copied().collect();
        self.trail.push(row);
    }

    /// A step taken, and the wound law applied to whoever took it.
    fn walk(&mut self, subject: SubjectId, goal: [i32; 2]) {
        let from = self.positions[&subject];
        let mut shoulder = self.shoulders.get(&subject).copied().unwrap_or(0);
        let to = march::toward(&self.ground, from, goal, &mut shoulder);
        self.shoulders.insert(subject, shoulder);
        if to == from {
            return;
        }
        self.positions.insert(subject, to);
        let dropped = from[1] - to[1];
        if dropped > SAFE_FALL {
            self.events.push(SortieEvent::Fell {
                who: subject,
                at: self.tick(),
                dropped,
            });
            self.wound(subject);
            // Only the home body downs. Downing is a control fact — it is
            // what the pact watches for — and a tagged-in successor or a
            // companion grits through the same wound, or the rescue could
            // strand two bodies at the cliff base with nobody to answer.
            if subject == self.control.home() && self.downed.is_none() {
                self.downed = Some(subject);
                self.events.push(SortieEvent::Downed {
                    who: subject,
                    at: self.tick(),
                });
            }
        }
    }

    fn wound(&mut self, subject: SubjectId) {
        let worn = self.facets.body_of(subject).map(|r| r.0).unwrap_or(0);
        let revision = BodyRevisionId(worn + 1);
        self.facets.wears(subject, revision);
        let fell = self
            .events
            .iter()
            .rev()
            .find_map(|event| match event {
                SortieEvent::Fell { who, dropped, .. } if *who == subject => Some(*dropped),
                _ => None,
            })
            .unwrap_or(0);
        self.wounds.push(Wound {
            subject,
            at: self.tick(),
            fell,
            revision,
        });
        self.events.push(SortieEvent::Wounded {
            who: subject,
            at: self.tick(),
            revision,
        });
    }

    /// One tick of the march. The only input is time passing: the played
    /// body walks toward the leg's goal, companions walk by their parts,
    /// and the standing rules do the rest.
    pub fn advance(&mut self) -> bool {
        if self.done || self.tick - self.events_first_tick() >= MAX_TICKS {
            return false;
        }
        self.tick += 1;

        // The pact watches for the played body going down. It fires only
        // while its agreement stands, one tick after the fall.
        if let (Some(down), Some(pact)) = (self.downed, self.pact)
            && !self.control.tagged_in()
            && down == self.control.played()
            && self
                .society
                .agreement(pact.under)
                .is_some_and(|agreement| agreement.standing())
        {
            self.events.push(SortieEvent::PactInvoked {
                at: self.tick(),
                under: pact.under,
            });
            self.control
                .apply(ControlIntent::TagIn {
                    to: pact.successor,
                    at: self.tick(),
                })
                .expect("the pact fires only when nobody is tagged in");
            self.events.push(SortieEvent::TaggedIn {
                at: self.tick(),
                to: pact.successor,
            });
        }

        // The played body moves first, toward the leg's goal, or toward the
        // downed body while tagged in. The rescuer stops at reach: walking
        // onto the fallen's column would take the same cliff.
        let played = self.control.played();
        if Some(played) != self.downed {
            match (self.control.tagged_in(), self.downed) {
                (true, Some(down)) => {
                    let at = self.positions[&down];
                    if !self.in_tend_reach(played, down) {
                        self.walk(played, [at[0], at[2]]);
                    }
                },
                _ => {
                    let goal = self.goal();
                    let before = Self::distance_to(self.positions[&played], goal);
                    if before < self.progress.0 {
                        self.progress = (before, 0);
                    }
                    if self.progress.1 >= STUCK {
                        self.dig(played, goal);
                        self.progress.1 = 0;
                    } else {
                        self.walk(played, goal);
                        let after = Self::distance_to(self.positions[&played], goal);
                        if after < self.progress.0 {
                            self.progress = (after, 0);
                        } else {
                            self.progress.1 += 1;
                        }
                    }
                },
            }
        }

        // Tending: within reach of the downed, under a standing agreement.
        if let (true, Some(down)) = (self.control.tagged_in(), self.downed) {
            let healer = self.control.played();
            if self.in_tend_reach(healer, down) {
                let under = self
                    .pact
                    .expect("tag-in happened, so the pact exists")
                    .under;
                let tended = self
                    .society
                    .exercise(under, &self.tend, self.tick())
                    .expect("the pact's agreement exists");
                if tended.kind == RulingKind::Performed {
                    self.society
                        .record(self.tick(), healer, Some(down), DeedKind::StoodBy);
                    self.events.push(SortieEvent::Tended {
                        at: self.tick(),
                        by: healer,
                        whom: down,
                    });
                    self.downed = None;
                    self.control
                        .apply(ControlIntent::TagOut { at: self.tick() })
                        .expect("tagged in, so tag-out holds");
                    self.events.push(SortieEvent::TaggedOut { at: self.tick() });
                }
            }
        }

        // Companions, by their parts, in id order. A companion with no part
        // stands where they stand: nothing else can move them.
        let anchor = self.positions[&self.control.home()];
        let goal = self.goal();
        let movers: Vec<(SubjectId, Part)> = self.parts.clone();
        for (subject, part) in movers {
            if subject == self.control.played() || Some(subject) == self.downed {
                continue;
            }
            match part {
                Part::Scout { .. } => {
                    // Holds only when more than a lead ahead on the route,
                    // measured toward the goal. Held by plain distance he
                    // would freeze forever the first time the party took a
                    // drop he did not.
                    let mine = Self::distance_to(self.positions[&subject], goal);
                    let theirs = Self::distance_to(anchor, goal);
                    if mine + LEAD > theirs && !march::arrived(self.positions[&subject], goal) {
                        self.walk(subject, goal);
                    }
                },
                Part::Healer { .. } => {
                    if let Some(down) = self.downed {
                        if !self.in_tend_reach(subject, down) {
                            let fallen = self.positions[&down];
                            self.walk(subject, [fallen[0], fallen[2]]);
                        }
                    } else if march::apart(self.positions[&subject], anchor) > TRAIL {
                        let a = anchor;
                        self.walk(subject, [a[0], a[2]]);
                    }
                },
            }
        }

        // The find, and the turn for home.
        let leader = self.positions[&self.control.home()];
        if self.carried.is_none() && march::arrived(leader, [self.site[0], self.site[2]]) {
            debug_assert!(self.ground.solid([leader[0], leader[1] - 1, leader[2]]));
            let salvage = Salvage {
                what: ROCK,
                from: self.site,
            };
            self.carried = Some(salvage);
            self.events.push(SortieEvent::Took {
                at: self.tick(),
                what: salvage.what,
            });
            self.progress = (i32::MAX, 0);
        }

        // Waypoints of the way home fall as the leader reaches them.
        while self.carried.is_some()
            && self.way_index + 1 < self.way_home.len()
            && march::arrived(
                self.positions[&self.control.home()],
                self.way_home[self.way_index],
            )
        {
            self.way_index += 1;
            self.progress = (i32::MAX, 0);
        }

        // Home again: the share, the settled accounts, and the end.
        if self.carried.is_some()
            && !self.control.tagged_in()
            && self.downed.is_none()
            && self.way_index + 1 == self.way_home.len()
            && march::arrived(
                self.positions[&self.control.home()],
                [self.home[0], self.home[2]],
            )
        {
            let leader = self.control.home();
            let with: Vec<SubjectId> = self.parts.iter().map(|(who, _)| *who).collect();
            for companion in with {
                self.society
                    .record(self.tick(), leader, Some(companion), DeedKind::Shared);
                self.events.push(SortieEvent::SharedOut {
                    at: self.tick(),
                    with: companion,
                });
            }
            if let Some(outing) = self.outing {
                self.society
                    .end(outing, leader, EndReason::WorkDone, self.tick())
                    .expect("the expedition agreement stands until now");
            }
            self.events.push(SortieEvent::Returned { at: self.tick() });
            self.done = true;
        }

        self.push_trail();
        true
    }

    /// Chebyshev distance from a body to a goal column.
    fn distance_to(at: [i32; 3], goal: [i32; 2]) -> i32 {
        (at[0] - goal[0]).abs().max((at[2] - goal[1]).abs())
    }

    /// One carve toward the goal, at head height and one column ahead: the
    /// notch the near tier's own one-voxel lift can climb into. Mesocosm's
    /// verb, consumed; the hewn rock comes home with the party.
    fn dig(&mut self, subject: SubjectId, goal: [i32; 2]) {
        let at = self.positions[&subject];
        let heading = [(goal[0] - at[0]).signum(), (goal[1] - at[2]).signum()];
        let removed = self
            .ground
            .carve([at[0] + heading[0], at[1] + 2, at[2] + heading[1]], 1);
        if removed > 0 {
            self.hewn += removed;
            self.events.push(SortieEvent::Dug {
                at: self.tick(),
                removed,
            });
        }
    }

    fn events_first_tick(&self) -> u64 {
        match self.events.first() {
            Some(SortieEvent::Departed { at }) => at.0,
            _ => 0,
        }
    }

    /// Runs to the end or to the loud failure of never getting there.
    pub fn run(&mut self) {
        while !self.done && self.advance() {}
    }

    /// The replay witness: everything that happened, hashed. Two runs of
    /// the same scene must agree on this number.
    pub fn hash(&self) -> u64 {
        let ground = hash_bytes(&encode(&self.ground).expect("ground always encodes"));
        let receipt = (
            &self.trail,
            self.control.log(),
            &self.wounds,
            &self.carried,
            self.hewn,
            ground,
            &self.events,
            &self.society,
            &self.facets,
        );
        hash_bytes(&encode(&receipt).expect("a sortie receipt always encodes"))
    }
}

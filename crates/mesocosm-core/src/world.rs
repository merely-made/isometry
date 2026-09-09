// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! The world: an enclosure holding one critter and the loose matter it can metabolize.
//! Integer state and seeded randomness make the same seed and trace replay exactly.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;
use crate::organism::{Organism, OrganismId};
use crate::places::{PlaceId, Places};
use crate::record::WorldRecord;
use crate::rng::Rng;
mod act;
mod adapt;
mod consume;
mod descendant_practice;
mod dev;
mod discover;
mod express;
mod expression_practice;
mod family_clearing;
mod family_practice;
mod filial;
pub mod generation;
mod genesis;
mod graft;
mod intent;
mod read;
mod records;
mod review;
mod revise;
mod terrarium;
pub use adapt::{Round, Score, Turn};
pub use descendant_practice::{DescendantPractice, DescendantStage};
pub use dev::PLACE_MATTER_MAX_MG;
pub use express::ExpressionPreview;
pub use family_practice::FamilyPracticeIds;
pub use genesis::Founding;
pub use graft::{Graft, GraftPreview};
pub use intent::{Ineligible, Intent, Outcome, Placement, Rejection, Route};
pub use read::Gland;
pub use review::{Offer, Prospect, Untakeable};
pub use revise::Unrevised;
pub use terrarium::{TERRARIUM_EXTENT, TERRARIUM_ROUTE_BUDGET, TerrariumBounds, TerrariumHabitat};
/// How far the enclosure reaches from its middle, in voxels.
///
/// **Sixty-four since S1** (scale plan, 2026-08-29): a 129-voxel span, 15.3x
/// the floor area of the 33-voxel world that shipped before it. Mark ruled
/// scale a feature in its own right, and the same day read the captures and
/// asked why the creatures looked so big — one number answers both. A palette
/// limb is 9 voxels long, which was **27%** of the old world's width and is
/// **7%** of this one's; the fix is the world rather than the bodies, because
/// `part_ceiling_mg` prices voxel volume and shrinking half-extents would drag
/// the economy TD6 and TD7 just tuned.
///
/// Terrain is O(area), never O(volume) — [`crate::places::SURFACE_BAND`] caps
/// height — so this costs bricks and soil columns quadratically and nothing
/// cubically.
pub const ENCLOSURE: i32 = 64;

/// Non-played founders, scaled to the enclosure's floor area.
///
/// **Density is what stays fixed.** The 33-voxel world founded 60 over its
/// 1,089 columns; a 129-voxel world founding the same 60 would be that
/// terrarium fifteen times emptier — big instead of diffuse is the failure the
/// scale plan names, and it names it at every rung. Derived rather than typed,
/// so widening the enclosure again carries the cohort with it. At
/// `ENCLOSURE = 64` this is 916, and the played critter makes 917.
pub const FOUNDERS: u32 = {
    let side = (2 * ENCLOSURE + 1) as u32;
    let reference_side: u32 = 33;
    side * side * 60 / (reference_side * reference_side)
};

/// Regions to a side. Nine is coarser than a crowding cell on purpose: a place
/// is somewhere you can be, not a bucket for counting neighbours.
///
/// **Deliberately unchanged by S1.** Growing it reorders every RNG draw and is
/// S3's whole subject (a spatial index, a distance-capped far tier, cohorts as
/// an execution path); S1 is constants and scaling only. The consequence is
/// measured and reported in the scale plan's S1 entry: nine regions over a
/// 129-voxel span makes each region 43 voxels across, so the near/far line —
/// tuned as `demote_hops = 2` against a diameter-2 graph — now falls tens of
/// voxels away instead of ten.
pub const PLACE_SIDE: u16 = 3;

/// Places draw from their own stream, derived from the world's seed.
///
/// Not from `World`'s, which the ecology spends every tick: dividing the
/// enclosure would shift every draw after it, so adding regions to a world would
/// silently rearrange the creatures in it.
pub const PLACE_SALT: u64 = 0x504C_4143_4553_0001;

/// Body recipes draw from their own stream too, per lineage.
pub const RECIPE_SALT: u64 = 0x5245_4349_5045_0001;

/// Tissue domains draw from their own stream too, per lineage.
///
/// Its own salt for the reason every other one has: assigning a lineage what it
/// is made of must not move a single creature in the enclosure.
pub const GRAFT_SALT: u64 = 0x4752_4146_5453_0001;

/// Individual developmental variation has its own worldgen stream.
///
/// It must not consume the ecology stream: adding a body detail must not move
/// every organism or change the timing of every later birth.
pub const DEVELOPMENT_SALT: u64 = 0x4445_5645_4C4F_5001;

/// How long a hand may be still before its critter goes back to its instincts,
/// in ticks.
///
/// Three seconds at the canonical ten ticks a second — long enough that a
/// pause to look at something is not read as walking away, short enough that
/// walking away is answered while you are still watching. **The idle terrarium
/// is the feature**: an ant farm nobody is touching is the way its dynamics
/// are seen, so the resting state of a controlled critter is its own drives,
/// not a statue. (TD4, ruled 2026-08-29.)
pub const INSTINCT_IDLE_TICKS: u32 = 30;

/// Ticks of upkeep left in the budget below which a meal burns instead of
/// building — **for every organism in the enclosure**, not just the played one
/// (TD5, ruled 2026-08-29; the ecology's `earn` reads it too).
///
/// A hundred: ten seconds of standing still at the canonical tempo, and about
/// a third of a 1,000 mg starter's 333-tick budget. **Wide on purpose.** The
/// ecology's own hunger horizon is eight ticks (`rates::HUNGRY_UPKEEP_TICKS`),
/// which is the point a body starts eating itself; routing a meal there would
/// mean every meal grew you until the tick before you died, and the burn half
/// of the verb would never be seen. This is instead the width of a state you
/// can notice, play out of, and be caught by.
pub const STARVED_UPKEEP_TICKS: u64 = 100;

/// This build's own definitions, as a world carries them. (PD4)
///
/// Built once. A world founded without an explicitly admitted pack runs the
/// natives, and a world decoded outside [`crate::snapshot::restore_under`] gets
/// them too — that raw door is documented as a same-process round trip, and the
/// checked one replaces this with whatever the caller admitted.
pub fn native_ruleset() -> std::sync::Arc<crate::process::Registry> {
    static NATIVE: std::sync::LazyLock<std::sync::Arc<crate::process::Registry>> =
        std::sync::LazyLock::new(|| {
            std::sync::Arc::new(crate::process::Registry::native().clone())
        });
    std::sync::Arc::clone(&NATIVE)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub epoch: u64,
    /// The rules this world realized, as digests. (PD3)
    ///
    /// **A saved fact, because a seed is not enough** (playable ecology plan
    /// §2): the code that reads a seed can change, so a durable world records
    /// which admitted biology it was actually running. One component today —
    /// the process ruleset — and it is serialized and hashed with everything
    /// else, so two worlds under different rulesets cannot agree about a
    /// state hash and a restore that offers the wrong one is refused by name
    /// through [`crate::snapshot::restore_under`].
    #[serde(default)]
    rules: crate::rules::WorldRules,
    /// The definitions those rules *are*. (PD4)
    ///
    /// **The set, beside the identity.** PD3 gave a world its ruleset digest
    /// and left [`BodyPhenotype::develop`](crate::BodyPhenotype::develop)
    /// resolving against `Registry::native()`, which was honest only while the
    /// shipped pack lowered to exactly that registry. This is the other half:
    /// the validator resolves against what this world admitted, so a body
    /// citing a definition this world does not hold is refused by name rather
    /// than validated against somebody else's biology.
    ///
    /// **Not serialized, and deliberately.** A world records the identity, not
    /// a copy (see [`crate::rules::WorldRules`]) — inlining five definitions
    /// into every snapshot would make the pack's own file the second place a
    /// rule lives. So the set is runtime carriage that a save re-enters
    /// through [`crate::snapshot::restore_under`], which is the door that
    /// checks the digest before it attaches anything. Skipping it also means
    /// PD4 moved no state hash.
    #[serde(skip, default = "native_ruleset")]
    ruleset: std::sync::Arc<crate::process::Registry>,
    rng: Rng,
    /// Which organism the player is, if any.
    ///
    /// **Control is a pointer, not a shape.** Switching lineage moves this and
    /// reconstructs nothing; a played critter and an unplayed one serialize
    /// identically; and no rule can branch on who is being played, because
    /// there is no flag to branch on.
    ///
    /// `None` is a first-class state, not a stale pointer. A world does not
    /// require anyone to be in it, and the difference between *nobody is
    /// embodied* and *this id no longer names anything* is one a caller has to
    /// be able to see.
    controlled: Option<OrganismId>,
    /// Who the world stopped being able to play on the most recent tick.
    control_lost: Option<OrganismId>,
    /// Consecutive [`Intent::Idle`] applications. Any other intent resets it.
    ///
    /// **World state, and hashed like the rest of it.** It is a pure function
    /// of the trace — count the idles at the end — so putting it here costs a
    /// replay nothing and buys the rule that reads it (`INSTINCT_IDLE_TICKS`)
    /// the one thing a host-side timer could never have: the same answer in
    /// every host, at every frame rate, on every replay. A wall clock in the
    /// host would have made the ecology's behaviour depend on how fast the
    /// machine drew.
    #[serde(default)]
    idle_run: u32,
    /// Lineages the player has inhabited.
    ///
    /// What "unlocked" means for the complexity frontier. Ordered, so the
    /// frontier is deterministic.
    #[serde(default)]
    unlocked: std::collections::BTreeSet<SpeciesId>,
    /// The most elaborate thing the player has ever held.
    ///
    /// **A high-water mark, not a current reading.** An earlier cut computed
    /// this from living organisms of unlocked species, which meant a lineage
    /// dying out collapsed the frontier to zero and left the world permanently
    /// uninhabitable. That contradicts the ruling that disembodiment is a seam
    /// rather than a dead end: losing a body must not unearn what reaching it
    /// cost.
    #[serde(default)]
    frontier: i32,
    /// Which lineages exist, what they are called, and what they came from.
    #[serde(default)]
    lineages: crate::species::Lineages,
    /// Part vocabulary admitted by this world for developmental realization.
    /// Recipes travel; this palette is the local interpretation they grow in.
    #[serde(default)]
    development_palette: crate::development::PartPalette,
    /// The enclosure, divided. Derived from the seed and fixed for a world's
    /// life: regions do not move, or nothing could be said to have happened in
    /// one.
    #[serde(default)]
    places: Places,
    /// The ground: brick truth raised from the same seed's landscape.
    /// Serialized whole, so carves live inside the replay hash (G1).
    #[serde(default)]
    ground: crate::places::Ground,
    /// The enclosure's matter, per voxel column.
    ///
    /// **The closed cycle's other half** (TD6). Producers draw out of it,
    /// bodies return to it, and the player's deposit enriches it, so the sum
    /// of soil, living substance, carrion and reserves is a constant of the
    /// run. World state like everything else here: serialized, hashed, and a
    /// pure function of the seed and the trace.
    #[serde(default)]
    soil: crate::places::Soil,
    /// Everywhere each lineage has been.
    ///
    /// **A high-water set**, for the same reason the frontier is a high-water
    /// mark: where a creature has been is not readable from where it is, and a
    /// lineage that withdrew from half the enclosure still reached it. Union is
    /// commutative, associative, and idempotent, so two worlds' ranges join the
    /// way their records do, without a protocol.
    #[serde(default)]
    ranges: BTreeMap<SpeciesId, BTreeSet<PlaceId>>,
    /// What this world has seen anyone do.
    #[serde(default)]
    record: WorldRecord,
    /// Ordered by id, so iteration never depends on hashing.
    pub organisms: Vec<Organism>,
    next_organism: u32,
    last_tally: crate::organism::Tally,
    /// What this line has come to, in order. (PE2)
    ///
    /// **World state, because a candidate is not a view.** A discovery decides
    /// what a body may be developed into, so it has to survive a save and come
    /// back the same on a replay — which it does, being a pure function of the
    /// seed and the trace like everything else here. Bounded by the condition
    /// table: a condition discovers once and never again.
    #[serde(default)]
    discoveries: Vec<crate::discovery::Discovery>,
    /// The most recent evidence a condition was offered, and what every
    /// condition made of it.
    ///
    /// One, not a log — a log of meals is what [`History`] is for. It is kept
    /// because evidence that unlocked nothing is still evidence, and "a meal
    /// supplies evidence without unlocking an incompatible candidate" is a
    /// claim about a record.
    #[serde(default)]
    last_observation: Option<crate::discovery::Observation>,
    /// This world's directed graft affinity over tissue domains. (P3)
    ///
    /// **World data, because the ruling says so.** A default world holds the
    /// three-domain cycle; a generated one arrives by holding a different
    /// table. It is serialized and hashed like every other rule a world
    /// realized, so two worlds that disagree about an edge cannot agree about a
    /// graft.
    #[serde(default)]
    affinity: crate::graft::Affinity,
    /// The most recent branch transfer this world's played body took. (P3)
    ///
    /// One, not a log — the same arrangement `last_observation` uses, and for
    /// the same reason: a running list of every branch a line ever took is a
    /// journal. What each transferred part *is* survives on its own provenance,
    /// which is durable; this is the transaction, so a receipt can say which
    /// crossing was taken and what the world's table said about it.
    #[serde(default)]
    last_graft: Option<graft::Graft>,
    /// Consecutive ticks a hand has held a body whose budget is under the
    /// starved line.
    ///
    /// **The authoritative bounded accumulator** a sustained-stress condition
    /// reads, and the reason such a condition does not have to read a
    /// presentation trend. One integer, world state, hashed and replayed —
    /// exactly the arrangement `idle_run` uses, and for the same reason: a
    /// host-side counter would give a different answer at a different frame
    /// rate.
    #[serde(default)]
    hunger_run: u32,
    /// What happened on the most recent tick, waiting to be drained.
    ///
    /// **One tick, never more.** History is derivable from a seed and ordered
    /// intents, so keeping it in the world would grow every snapshot without
    /// bound and cost the cheap whole-state capture the wing's rollback
    /// thinking rests on. A caller drains this into a [`History`], which lives
    /// beside the world and persists in its own slot.
    /// Not skipped when empty: postcard is positional, so a field written
    /// conditionally cannot be read back. That trap already cost one decode
    /// failure here.
    #[serde(default)]
    pending: Vec<crate::flow::RecordedEvent>,
    /// What moved on the most recent tick, waiting to be reduced.
    ///
    /// **Beside the world rather than in it** (PE0). Dense per-tick flow in a
    /// snapshot is exactly what the plan's stop rules forbid, so this is
    /// `serde(skip)` and transparent to equality — the `drain_ground_dirty`
    /// arrangement, for the same reason: a host that drains readings every frame
    /// and a headless replay that never drains still hash identically.
    ///
    /// Opened at the top of every tick rather than accumulating like `pending`,
    /// because a stream this dense must be bounded whether or not anyone is
    /// listening. `pending` keeps its old contract: sparse, and a caller's to
    /// drain.
    #[serde(skip)]
    flows: crate::flow::Ledger,
    /// The tick the current epoch began on. (PE3)
    ///
    /// Stored rather than derived from `tick % ticks`, because only the Timed
    /// rule is periodic: a Gated epoch ends when its conditions are met, and a
    /// world that had inferred its boundaries from arithmetic would have to be
    /// rewritten to admit one.
    #[serde(default)]
    epoch_began: u64,
    /// Whether this world is standing at its lineage checkpoint. (PE3)
    ///
    /// **A one-tick fact that a hold makes last.** It is set when the epoch's
    /// budget is spent and cleared by the next tick that is not a revision, so
    /// a headless enclosure gets exactly one tick's window and does not use it,
    /// while a driver that stops to ask leaves the world at the same tick for
    /// as long as the player thinks — the `control_lost` arrangement, and the
    /// same reason succession keeps the pause in the driver. It is what
    /// [`World::revision_admitted_now`] answers.
    #[serde(default)]
    at_boundary: bool,
    /// What the most recent adaptation round came to. (PE3)
    ///
    /// One, not a log — the arrangement `last_observation` and `last_graft`
    /// already use. What each line weighed is a reading of a decision, and a
    /// running list of every turn a world ever took is a journal; what the
    /// decisions *did* survives as `Event::Revised` in the past.
    #[serde(default)]
    last_round: adapt::Round,
    /// A forced birth, waiting to join the roster. (DT3)
    ///
    /// **A handoff inside one `apply`, and empty at every tick boundary.**
    /// `Intent::ForceBirth` runs the ordinary birth while the intent is being
    /// resolved, which is before the tick's own passes; a child appended there
    /// would be aged, rented and dispersed in the tick it was born in, and a
    /// natural newborn is not, because the birth pass appends after all of
    /// that. So the child waits here and joins where `ecology::breed`'s
    /// newborns do. Not serialized: nothing outside [`Self::apply`] can observe
    /// it holding anything, so it cannot move a state hash.
    #[serde(skip)]
    forced_birth: Option<Organism>,
    /// Whether this world is a scoring copy. (P4b)
    ///
    /// **Not serialized, never true in a world anyone holds.** A copy grown to
    /// score a candidate must not end epochs of its own: a round inside a round
    /// is a different game, and an unbounded one. Every real world answers
    /// `false`, so this cannot move a state hash.
    #[serde(skip)]
    scoring: bool,
}

/// The half-extent of an organism, by its volume tag.
///
/// Shapes vary so that [`crate::plan::classify`] finds real roles: a world of
/// identical cubes can only ever grow mass. A host's volumes must match, since
/// this is what placement and physics read.
pub fn organism_extent(tag: u8) -> [i32; 3] {
    match tag % 4 {
        0 => [3, 1, 1],
        1 => [1, 3, 1],
        2 => [3, 1, 3],
        _ => [2, 2, 2],
    }
}

impl World {
    /// Applies one intent and advances the tick. This is the only way world
    /// state changes.
    pub fn apply(&mut self, intent: Intent) -> Outcome {
        let actor = self.controlled;
        // Read before the intent is consumed. A revision is taken *at* the
        // lineage checkpoint and does not leave it, so it is the one intent
        // that does not close the boundary below.
        let revising = matches!(intent, Intent::Revise { .. });
        // Counted before the act, so a hand that is acting right now is
        // already holding at zero when the ecology below reads it.
        self.idle_run = if matches!(intent, Intent::Idle) {
            self.idle_run.saturating_add(1)
        } else {
            0
        };
        // One tick's flows, and only one. Opened before anything can move
        // matter, so the act and the ecology write into the same stamped
        // stream.
        self.flows.open(self.tick);
        // Read before anything can allocate an id, so both routes to a birth —
        // the ecology's pass and DT3's forced one, inside `resolve` — leave
        // their newborns addressable to the filial pass below. (PD5, DT3.)
        let before_births = self.next_organism;
        // Where the act's own consequences begin. Resolving can record
        // events of its own (learning a word from a meal), and those follow
        // from the act rather than preceding it, so the act is inserted at
        // this boundary rather than pushed after them.
        let boundary = self.pending.len();
        let outcome = self.resolve(intent);
        // Whether a hand asked for the boundary below. Read off the outcome
        // rather than kept as a flag: only an accepted `Intent::EndEpoch`
        // produces this, and a refused one is a rejection like any other. (DT3)
        let demanded = matches!(outcome, Outcome::EpochEnded { .. });

        // Recorded before the ecology steps, because that is the order these
        // happened in and a history that reversed them would let a creature's
        // death precede its last act. It also means the actor is still whoever
        // acted, even if this tick kills them.
        if let Some(event) = records::event_for(&outcome, actor) {
            let place = self.acted_at(actor);
            self.pending.insert(
                boundary,
                crate::flow::Envelope::new(self.tick, place, event),
            );
        }

        // The enclosure lives whether or not the player acted. This is what
        // separates an ecology from a field of pickups: things grow, breed,
        // starve, and rot on their own schedule.
        let focus = self.position();
        let held = self.held();
        let tick = self.tick;
        let mut records =
            crate::flow::Records::new(tick, Some(&self.places), &mut self.pending, &mut self.flows);
        self.last_tally = crate::organism::ecology::step_with_ground(
            &mut self.organisms,
            &mut self.next_organism,
            &mut self.rng,
            &mut records,
            &self.lineages,
            self.development_palette,
            &mut self.soil,
            &self.places,
            &self.ground,
            focus,
            held,
        );

        // A forced birth joins the roster here: the same point `breed`'s own
        // newborns reach it, so a child a hand asked for is not run through the
        // tick it was born in and a natural one is not either. (DT3)
        if let Some(child) = self.forced_birth.take() {
            self.organisms.push(child);
            self.last_tally.born += 1;
        }

        // **Filial expression** (PD5). A descendant of a line that has
        // committed a revision is developed under it here, in the tick it was
        // born in and before anything reads the world: the ecology owns making
        // a body, and what that body's program says it should be doing is the
        // world's, because it needs the admitted ruleset, the soil and both
        // records. A line with no revision reaches nothing below.
        self.express_filially(before_births);

        // What you reach, you keep. The frontier rises with the body you are
        // holding and never falls, so a lineage dying out costs you that body
        // rather than everything it earned.
        if let Some(held) = self.controlled().map(|o| self.intricacy(o)) {
            self.frontier = self.frontier.max(held);
        }

        // Control ends with the life it was attached to. Nothing exempts the
        // played critter from the ecology, so this is where a run can lose its
        // body without the world ending.
        self.control_lost = match self.controlled {
            Some(id) if !self.is_eligible(id) => {
                self.controlled = None;
                Some(id)
            },
            _ => None,
        };

        // The one accumulator a discovery condition may read, advanced after
        // the ecology has settled what this body is: whether it is starved and
        // whether it is still a body are both this tick's answers, not last
        // tick's. Event-driven — it offers evidence at the crossing and never
        // sweeps the roster. (PE2)
        self.endure();

        // Where each lineage has been, unioned in after the ecology has moved
        // everything. Reading it before the step would record last tick's
        // positions against this tick's roster.
        for organism in self.organisms.iter().filter(|o| o.is_alive()) {
            if let Some(place) = self.places.at(organism.position) {
                self.ranges
                    .entry(organism.species)
                    .or_default()
                    .insert(place);
            }
        }

        self.tick += 1;

        // **The epoch's own budget** (PE3). The world ends its epoch, because
        // the rule that ends it is a versioned world rule and a headless
        // enclosure has to obey it too — a boundary that only existed inside
        // one driver would be a second authority over when bodies may change.
        //
        // The *reckoning* is not here. It reads the past, and history lives
        // beside a world rather than inside it, so whoever holds the past does
        // that half through [`Self::reckon`].
        // **Or a hand asked for it** (DT3). `Intent::EndEpoch` runs this exact
        // path and adds nothing to it: the rule that admits the demand is the
        // world's own (`EpochRule::admits_demand`), and what follows is the
        // round, the boundary and the reckoning PE3a built.
        if !revising {
            self.at_boundary = false;
        }
        if !self.scoring && (demanded || self.rules.epoch.spent(self.tick - self.epoch_began)) {
            self.epoch += 1;
            self.epoch_began = self.tick;
            // Set before the round, so every unplayed line commits through the
            // same gate the player's revision passes.
            self.at_boundary = true;
            self.last_round = self.adapt_round();
        }
        outcome
    }
    /// The rules this world realized. (PD3)
    ///
    /// What a save cites, what a peer compares, and what a replay is checked
    /// against. See [`crate::rules::WorldRules`].
    pub fn rules(&self) -> crate::rules::WorldRules {
        self.rules
    }
    /// The definitions this world admitted. (PD4)
    ///
    /// The set [`Self::rules`] is the identity of, and the only ruleset any
    /// development on a body in this world is validated against.
    pub fn ruleset(&self) -> &crate::process::Registry {
        &self.ruleset
    }

    /// The same set, shareable, for a host that has to hand it somewhere.
    pub fn admitted(&self) -> std::sync::Arc<crate::process::Registry> {
        std::sync::Arc::clone(&self.ruleset)
    }

    /// Re-attaches a decoded world's definitions. **Crate-private, and only
    /// [`crate::snapshot::restore_under`] calls it** — which has already
    /// compared the digest, so this cannot swap a living world's biology.
    pub(crate) fn reattach_ruleset(&mut self, ruleset: std::sync::Arc<crate::process::Registry>) {
        self.ruleset = ruleset;
    }

    /// What the most recent tick did to the enclosure.
    pub fn last_tally(&self) -> crate::organism::Tally {
        self.last_tally
    }

    /// Living organisms, in id order.
    pub fn living(&self) -> impl Iterator<Item = &Organism> {
        self.organisms.iter().filter(|o| o.is_alive())
    }
}

#[cfg(test)]
mod behavior_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tg1_tests;
#[cfg(test)]
mod typed_soil_tests;

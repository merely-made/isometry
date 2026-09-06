// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The vocabulary a host speaks: what it may ask, and what it is told back.
//!
//! Split out of `world.rs` at the 600-line ceiling on 2026-09-01, when PD2
//! added the developmental verb. Nothing here mutates anything — `world::act`
//! is still the only room behind the one door — so this file is exactly the
//! contract between a host and the world, in one place.

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;
use crate::body::{PartId, Yaw};
use crate::organism::OrganismId;
use crate::phenotype::Refusal;
use crate::process::Unmet;

/// How an incorporated part finds its site.
///
/// A **policy**, not a destination. Placing a part explicitly is a different
/// way of growing, not a different thing to do with a meal, and folding it in
/// beside `Burn` conflated the two questions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Placement {
    /// The body plan decides. **The default**: growth is automatic and
    /// symmetric, and the player shapes the plan rather than the placement.
    Planned,
    /// An explicit site. The editor path: total control is possible, but it is
    /// never the resting state.
    Explicit {
        parent: PartId,
        offset: [i32; 3],
        yaw: Yaw,
    },
}

/// Where a meal goes.
///
/// **This is the game's central question, and before it existed there was no
/// question.** Eating used to grant a part *and* half the mass as energy, so
/// the most important verb asked the player nothing. Splitting the destination
/// is what makes every meal ask: live now, or grow later?
///
/// **The question is no longer put to the player's fingers.** Mark rejected
/// the hotkey pair (2026-08-28: "not a workable ui") and ruled the answer
/// diegetic on 2026-08-29: a starved body burns its meal, a provisioned one
/// builds with it, and the state that decides is already on the vitals panel.
/// So this survives as what the body *concluded*, resolved inside
/// [`World::apply`] — never as something an intent carries, which is why
/// replays cannot disagree about it.
///
/// Later destinations arrive when the systems that receive them do:
/// provisioning reproduction, depositing or building a niche, and cultivating
/// something outside the skin. Each is a different answer to *where does this
/// capability live: in me, in a relationship, or in the world?*
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// Burn it now. Full mass becomes usable energy and no part is kept.
    Burn,
    /// Commit it to growth. Yields **no** immediate energy, which is the whole
    /// tradeoff.
    Incorporate { placement: Placement },
}

/// What a host may ask the world to do. Hosts send intents; they never mutate
/// world state directly.
///
/// `Clone` rather than `Copy`, because naming a lineage carries a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Steer the critter toward a local voxel offset.
    ///
    /// One intent resolves to one legal near-tier step over the ground. The
    /// requested offset supplies the heading and vertical preference; it is
    /// never a licence to cross solid voxels or teleport across a slope.
    Move { delta: [i32; 3] },
    /// Eat something. **The one verb.**
    ///
    /// Metabolize means routing matter through the organism rather than
    /// swallowing it. What the meal *becomes* is not carried here: the body
    /// decides, burning when its budget is starved and building when it is
    /// provisioned (see [`Route`] and [`STARVED_UPKEEP_TICKS`]). What is
    /// carried is the other question entirely — *where a kept part goes* —
    /// which is a growth policy rather than a destination, and stays with the
    /// intent because an editor has to be able to say it.
    Metabolize {
        organism: OrganismId,
        placement: Placement,
    },
    /// Take **one organ** off a carcass. (PE2)
    ///
    /// A different act from [`Metabolize`], not a parameter on it. Eating a
    /// whole body is calories and the body routes them; this names an organ on
    /// something that has stopped being alive, settles exactly what that organ
    /// weighs, and writes down which part of which line it came off — the first
    /// time `Origin::Incorporated`'s `from_part` has been anything but the
    /// donor's root.
    ///
    /// It attaches one part and never a branch: the subtree under the organ
    /// stays on the corpse, because live subtree transfer is phenotype P3's
    /// gate and this proof deliberately stops short of it.
    ///
    /// [`Metabolize`]: Intent::Metabolize
    Consume { organism: OrganismId, part: PartId },
    /// Take a whole **branch** off a carcass and set it on this body. (P3)
    ///
    /// The transfer [`Consume`] deliberately stops short of. It names one part
    /// of a body that has stopped, and everything hanging off that part comes
    /// with it: the ids are freshly allocated here, the branch's own joints and
    /// parent relations are preserved, every arriving part's provenance names
    /// the exact part it came off, and the source loses the branch.
    ///
    /// **The crossing is the player's, the verdict is the world's.** Whether
    /// the branch can be *carried* — arriving with the arrangement it had — is
    /// what this world's graft affinity says about the two lines' tissue
    /// domains; regrowing it here is feasible whatever the table says, and does
    /// not promise the donor's arrangement.
    ///
    /// [`Consume`]: Intent::Consume
    Graft {
        organism: OrganismId,
        part: PartId,
        crossing: crate::graft::Crossing,
    },
    /// Return mass to the enclosure as carrion.
    Deposit { mass_mg: u64 },
    /// Split the line you are in, and name it.
    ///
    /// **The act that makes a species.** Splitting is not a threshold anybody
    /// crosses; it is something a player does, and the name is the doing.
    /// Takes the creature you are holding and nothing else, so a new line
    /// begins with one founder and its former kin keep the old one.
    Speciate { name: String },
    /// Inhabit another critter.
    ///
    /// **A recorded intent, not a side door.** Lineage switching is gameplay,
    /// and ordered intents are the only way world state changes. A control
    /// change made outside this path would replay every fact about a run
    /// except who was living it.
    TakeControl { organism: OrganismId },
    /// Advance one tick without acting.
    Idle,
    /// Answer a checkpoint by continuing as you are.
    ///
    /// **The other half of [`TakeControl`], and the reason the pair is a
    /// choice.** At a reproduction checkpoint the two answers are *take the
    /// offspring* and *stay in the parent*; at a control loss they are *continue
    /// through a descendant* and *let the line go*. Staying is a decision, and a
    /// decision has to be sayable, or the record cannot tell a hand that chose
    /// to carry on from a hand that was not there.
    ///
    /// To the world it is [`Idle`] that admits to being a hand: nothing moves,
    /// but the idle run resets, because somebody answered. Everything a
    /// checkpoint *is* — when one opens, what it says, how long it holds — lives
    /// in the driver, since a bounded pause is a question about when to step and
    /// the world only ever knows what is. (PE1.)
    ///
    /// [`TakeControl`]: Intent::TakeControl
    /// [`Idle`]: Intent::Idle
    Resume,
    /// Carve a pocket of air around a nearby point. Recorded like every
    /// mutation, so a burrow is part of the world's replayable history.
    /// The energetics of digging await the metabolize-earth ruling; for
    /// now legality is embodiment plus reach.
    Carve { at: [i32; 3], radius: i32 },
    /// Express a candidate this line has discovered. (PD3)
    ///
    /// **The bounded developmental verb**, and the door that replaced PD2's
    /// `Intent::Rearrange`. It names a condition rather than an arrangement:
    /// what the development *is* comes from the admitted ruleset and the
    /// discovery record, so a host cannot author an allocation over the wire,
    /// and nothing outside the game's own rules decides what tissue moves.
    ///
    /// Where the definition came from is the packed door: `mesocosm-phenotype`
    /// admits it out of pack data, the discovery grants a
    /// [`Candidate`](crate::discovery::Candidate) that cites it by content
    /// address, and [`Candidate::propose`] lowers that to the same
    /// [`AllocationProposal`] a hand once drew. One validator underneath,
    /// unchanged since PD1b.
    ///
    /// [`Candidate::propose`]: crate::discovery::Candidate::propose
    /// [`AllocationProposal`]: crate::phenotype::AllocationProposal
    Express {
        condition: crate::discovery::ConditionId,
    },
    /// Commit what this line has come to, so its descendants are born with it.
    /// (P4)
    ///
    /// **The lineage verb, and it is not [`Express`].** Expressing changes the
    /// body you are standing in and costs that body a development; revising
    /// changes what the *line* grows and costs the run nothing except that
    /// every descendant now arrives under it. Phenotype plan §3 keeps the two
    /// apart in so many words: an eaten limb is not automatically a heritable
    /// limb.
    ///
    /// It names a condition for [`Express`]'s reason: what the revision is
    /// comes from the admitted ruleset and the line's own discovery record, so
    /// a host cannot author a program over the wire.
    ///
    /// **When one may be committed is PE3's**, and
    /// [`World::revision_admitted_now`](crate::World::revision_admitted_now)
    /// is the one function standing in for that ruling until the epoch trigger
    /// is chosen.
    ///
    /// [`Express`]: Intent::Express
    Revise {
        condition: crate::discovery::ConditionId,
    },
    /// End the epoch now. **A dev tool** (DT3), and the built meaning of
    /// [`EpochRule::PlayerTriggered`].
    ///
    /// Admitted only where this world's [`EpochRule`] says so
    /// ([`EpochRule::admits_demand`], which carries the ruling and its
    /// reasons): a Timed epoch ends early and restarts its budget from this
    /// tick, a PlayerTriggered one ends *only* here, and a Gated one refuses.
    ///
    /// It runs exactly the boundary PE3a built and adds nothing to it — the
    /// same adaptation round, the same `at_boundary`, and the same reckoning
    /// by whoever holds the past. The one thing that is different about it is
    /// that a hand asked.
    ///
    /// [`EpochRule`]: crate::rules::EpochRule
    /// [`EpochRule::PlayerTriggered`]: crate::rules::EpochRule::PlayerTriggered
    /// [`EpochRule::admits_demand`]: crate::rules::EpochRule::admits_demand
    EndEpoch,
    /// Bear an offspring from this body now. **A dev tool** (DT3).
    ///
    /// **The ordinary birth, with the clock taken off it.** What it skips is
    /// the ecology's *timing* gate — adult stage, gestation, brood mass — and
    /// nothing else: the transaction is
    /// [`ecology::bear`](crate::organism::ecology::bear), the one the birth
    /// pass runs, so the child is realized under the same filial seed, scattered
    /// by the same draw, provisioned out of the same two accounts, recorded as
    /// the same `Event::Born`, and developed under its line's revision by the
    /// same filial pass.
    ///
    /// **Provisioning still binds.** A parent that cannot pay for its line's
    /// recipe out of a quarter of its own body is refused with
    /// [`Rejection::InsufficientMass`], which is exactly the condition a
    /// natural birth waits on rather than a second rule written for this door.
    ForceBirth { organism: OrganismId },
    /// End this body's life now. **A dev tool** (DT3).
    ///
    /// The ordinary death, through
    /// [`ecology::perish`](crate::organism::ecology::perish): the body becomes
    /// carrion holding exactly the substance it had, its reserve goes back into
    /// the column under it as a `Process::Death` flow, and the record gets the
    /// same `Event::Died` a starved or aged body writes. Nothing about the
    /// corpse it leaves says how it died, which is the point — a dev-caused
    /// death has to read as a natural one or the tool is lying about the world.
    ///
    /// A body that is already carrion, spent, or absent is refused by name.
    /// Killing the *controlled* critter is allowed and loses control exactly as
    /// any other death does.
    Kill { organism: OrganismId },
    /// Put matter into the ground at a cell. **A dev tool** (DT3).
    ///
    /// The one route by which the enclosure's matter total changes, and it is
    /// a recorded transfer rather than a hole: the milligrams come out of
    /// [`Account::Dev`](crate::flow::Account::Dev) and into the soil, so the
    /// flow record still accounts for every account and a conservation check
    /// subtracts what that source issued instead of tolerating it.
    ///
    /// Bounded by [`PLACE_MATTER_MAX_MG`](crate::world::PLACE_MATTER_MAX_MG)
    /// per intent, and refused off the grid rather than clamped onto its edge:
    /// `Soil::column_at` clamps as insurance against a leak, and a dev tool
    /// that leaned on it would silently pile matter against the wall.
    PlaceMatter { at: [i32; 3], mass_mg: u64 },
}

impl Intent {
    /// Whether this is one of DT3's world-changing dev intents.
    ///
    /// **The world does not branch on it**, and nothing here does either: each
    /// of the four is applied, refused and recorded like every other intent.
    /// What reads this is a receipt, so a run that used one is labelled
    /// assisted (dev tools plan §2, principle 5).
    pub fn is_dev(&self) -> bool {
        matches!(
            self,
            Self::EndEpoch | Self::ForceBirth { .. } | Self::Kill { .. } | Self::PlaceMatter { .. }
        )
    }
}

/// Why an intent could not be applied. Rejections are part of the recorded
/// outcome, so a replay that rejects the same intents is still identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// Nobody may inhabit that, and this says why.
    Ineligible(Ineligible),
    /// A critter cannot eat itself.
    ///
    /// Only expressible since the played critter joined the organism
    /// vector: before that it was not a thing anyone could target.
    Itself,
    /// The controlled body's declared ports do not admit this meal.
    Inedible(OrganismId),
    /// Nobody is being played, so there is nothing to act with. A world
    /// running with no one in it is a legitimate state.
    Disembodied,
    NoSuchOrganism(OrganismId),
    NoSuchParent(PartId),
    /// That body has no such part. (PE2)
    NoSuchPart(PartId),
    /// Its organs are not on offer, because it is still using them. Taking one
    /// would be live dismemberment, which phenotype P3 owns and PE2's bounded
    /// part-meal proof does not open.
    StillLiving(OrganismId),
    /// That part is the whole creature. A body without a root is not an
    /// injured body, so the root is not a branch anybody can take; eating it is
    /// the verb for wanting all of it. (P3)
    WholeBody(PartId),
    /// This world's graft affinity refuses to carry that line's tissue into
    /// this one. Regrowing it here is the feasible route, which is what the
    /// wing contract requires of an incompatible carry: refused or redirected,
    /// never silently rewritten. (P3)
    Incompatible {
        from: crate::graft::Domain,
        into: crate::graft::Domain,
    },
    /// The part is severed or already taken, so there is nothing there to
    /// settle. A severed part's milligrams have already left the conservation
    /// account; eating one would create matter.
    NothingLeft(PartId),
    /// The played critter could not touch it, and this says why: no actuator
    /// at all, or one that does not extend far enough.
    OutOfReach(Unmet),
    /// This line has not come to that, so there is nothing to express. (PD3)
    ///
    /// A discovery is what makes a candidate available; asking for one that
    /// never landed is refused rather than granted quietly, which is the whole
    /// reason the bounded door names a condition instead of an arrangement.
    Undiscovered(crate::discovery::ConditionId),
    /// The line has come to it and this body has nowhere to put it. (PD3)
    ///
    /// **A real state, not a failure.** A candidate is available before it is
    /// expressible: a consumer that has never grown a plate cannot express a
    /// gland until it does, and the difference between having the option and
    /// being able to take it is one a player is owed.
    Nowhere(crate::discovery::ConditionId),
    InsufficientMass,
    /// The body plan found nowhere for a part of this shape to go, or the
    /// resulting live body would not fit its current Ground stance. Refusing
    /// keeps both body topology and terrain occupancy honest.
    NoRoom,
    /// The development would not validate, and this is the boundary that
    /// failed. (PD2)
    ///
    /// Carried whole rather than flattened into a handful of world-level
    /// words: PD1b made the refusal order part of the contract precisely so
    /// two callers submitting the same invalid candidate get the same answer,
    /// and re-encoding fifteen named boundaries as three would throw that away
    /// at the one door a player actually knocks on.
    Refused(Refusal),
    /// The lineage revision would not commit, and this is why. (P4)
    ///
    /// Carried whole for [`Self::Refused`]'s reason: the commit door names its
    /// own boundaries, and re-encoding them here would give the same refusal
    /// two vocabularies.
    Unrevised(super::Unrevised),
    /// This world's epoch rule does not admit a demand. (DT3)
    ///
    /// Carries the rule, because *which* rule refused is the fact worth having:
    /// a Gated world says so, and the answer would be different in a world
    /// founded under either of the other two.
    EpochNotOnDemand(crate::rules::EpochRule),
    /// That body is not alive, so it cannot bear and cannot be killed. (DT3)
    ///
    /// The mirror of [`Self::StillLiving`], and both dev intents that name a
    /// body share it: a corpse has no offspring to provision, and a corpse
    /// cannot die twice.
    NotLiving(OrganismId),
    /// That cell is outside the enclosure. (DT3)
    ///
    /// Refused rather than clamped. `Soil::column_at` clamps as insurance
    /// against a leak at the wall, and a dev intent that leaned on it would
    /// silently pile every mistaken placement into one edge column.
    OffGrid([i32; 3]),
    /// More matter than one placement may carry. (DT3)
    ///
    /// Says both numbers, because a bound a caller cannot read is a bound it
    /// cannot work inside.
    OverBound {
        mass_mg: u64,
        max_mg: u64,
    },
}

/// Why an organism cannot be inhabited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ineligible {
    NoSuchOrganism,
    /// A carcass is not a critter you can play.
    NotAlive,
    /// More elaborate than anything you have earned.
    ///
    /// Stepping *down* into a newly viable niche is the point of switching;
    /// stepping across into an unearned peer is what this refuses.
    AboveTheFrontier {
        frontier: i32,
        target: i32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Moved,
    /// Ground removed around a point.
    Carved {
        at: [i32; 3],
        removed: u32,
    },
    /// A meal became energy and nothing else.
    Burned {
        organism: OrganismId,
        energy_mg: u64,
    },
    Incorporated {
        part: PartId,
    },
    /// A bilateral plan grew a mirrored pair from one meal, splitting its mass.
    IncorporatedPair {
        part: PartId,
        mirror: PartId,
    },
    /// One organ came off a carcass and onto this body. (PE2)
    ///
    /// Says both ends of the provenance, because that is the whole point of
    /// the verb: which part it became here, and which part of which body it
    /// was. `mass_mg` is exactly what that part weighed — no split, no spill.
    Consumed {
        part: PartId,
        from: OrganismId,
        from_part: PartId,
        mass_mg: u64,
    },
    /// A branch came off a carcass and onto this body, with its topology and
    /// its source addresses intact. (P3)
    ///
    /// Says both ends and the terms: which part it became here, which part of
    /// which body it was, how many parts came with it, what they weigh, which
    /// crossing was taken and what this world's table said about it. The full
    /// remapping is on the arriving parts' own provenance, which is a durable
    /// record rather than an outcome.
    Grafted {
        root: PartId,
        parts: u32,
        from: OrganismId,
        from_part: PartId,
        mass_mg: u64,
        crossing: crate::graft::Crossing,
        verdict: crate::graft::Verdict,
    },
    Deposited {
        organism: OrganismId,
    },
    /// Control moved to another critter.
    Inhabited {
        organism: OrganismId,
    },
    /// A line split, and was named.
    Speciated {
        species: SpeciesId,
        from: SpeciesId,
        founder: OrganismId,
    },
    Idled,
    /// A checkpoint was answered by carrying on unchanged.
    Resumed,
    /// A discovered candidate was expressed on a part, and paid for. (PD2's
    /// transaction, through PD3's door.)
    ///
    /// `cost_mg` is what the development cost: the cells whose expression
    /// changed, priced at what a cell of that part's tissue is worth. It left
    /// the body's reserve and went into the ground under it, because nothing
    /// evaporates.
    Expressed {
        part: PartId,
        cost_mg: u64,
        /// The phenotype revision this development created, so a receipt can
        /// point at the ordering rather than only at the fact.
        revision: u32,
    },
    /// A line committed a revision of its development program. (P4)
    ///
    /// It cost the body nothing, which is the ruling: founding a continuation
    /// *is* the prize (epoch-boundary plan §2). What it costs is that every
    /// descendant of this line now arrives under it and the line can be held
    /// to it.
    Revised {
        species: SpeciesId,
        revision: crate::program::RevisionId,
        condition: crate::discovery::ConditionId,
    },
    /// The epoch was ended on demand. (DT3)
    ///
    /// Names the epoch that **closed**, which is the one a reader was watching;
    /// the world is in the next one by the time anybody sees this.
    EpochEnded {
        epoch: u64,
    },
    /// A birth was taken now rather than when the ecology would have taken it.
    /// (DT3)
    ///
    /// Both ends, because that is what a birth is. No `Event` follows from this
    /// outcome: the transaction writes the ordinary `Event::Born` inside
    /// itself, so the record has one writer — the arrangement
    /// [`Self::Revised`] already uses.
    Bore {
        parent: OrganismId,
        offspring: OrganismId,
    },
    /// A body's life was ended now. (DT3)
    ///
    /// `substance_mg` is what the corpse it left weighs and `reserve_mg` is
    /// what its death put back into the ground — the two halves of the ordinary
    /// death, said out loud so a receipt can check them against a natural one
    /// rather than re-deriving them. The `Event::Died` is written inside the
    /// transaction, for [`Self::Bore`]'s reason.
    Killed {
        organism: OrganismId,
        substance_mg: u64,
        reserve_mg: u64,
    },
    /// Matter entered the ground at a cell, out of the dev source. (DT3)
    Placed {
        at: [i32; 3],
        mass_mg: u64,
    },
    Rejected(Rejection),
}

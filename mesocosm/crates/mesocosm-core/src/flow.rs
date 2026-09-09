// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where the matter went.
//!
//! [`history`](crate::history) is biography: birth, feeding, growth, death, the
//! sparse causal line of a creature. Trophic visibility needs the other record —
//! exact, frequent resource movement — and putting every milligram of upkeep and
//! soil draw into the permanent causal log would make the wrong record carry the
//! wrong frequency (PE0, playable ecology plan §2).
//!
//! So there are two records and one commit point. `World::apply` emits both from
//! the same transaction: an accepted mutation emits its flows, a refused one
//! emits nothing at all. **A resource mutation cannot be visible to the state
//! while absent from the flow record** — `tests/flows.rs` reconciles the stream
//! against every compartment, tick by tick, which is that sentence made
//! executable.
//!
//! # Three compartments and one carrier
//!
//! TD6's conserved quantity is `soil + Σ(substance + reserve)`, so those are the
//! three [`Account`]s a milligram can sit in and every flow is a transfer
//! between two of them. [`Carrier`] has one variant today because matter is the
//! only thing the enclosure moves; PE4's generated materials are what it exists
//! to be extended by.
//!
//! DT3 added a fourth account, [`Account::Dev`], and it sits **outside** that
//! sum rather than inside it: a dev tool that puts matter into a column is the
//! one route by which the enclosure's total changes, and the account is what
//! makes the change a recorded transfer instead of a silent one. The conserved
//! quantity is therefore the three compartments less what that account issued,
//! which is [`Account::issued_mg`] and is checked with no tolerance at all.
//!
//! # The envelope carries when and where
//!
//! [`Envelope`] stamps a record with its tick and its region rather than copying
//! those two fields into every event variant. `History` therefore finally has a
//! tick on it, which is what a bounded window over births and deaths needs.
//!
//! # It lives beside the world, never in it
//!
//! [`Ledger`] is the world's one-tick buffer, and it is `serde(skip)` and
//! transparent to equality — the [`drain_ground_dirty`] precedent. Draining
//! readings therefore cannot move the state hash, and a dense per-tick stream
//! cannot leak into a snapshot to serve a panel.
//!
//! [`drain_ground_dirty`]: crate::World::drain_ground_dirty

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;
use crate::organism::{Kingdom, Organism, OrganismId};
use crate::places::{PlaceId, Places};

mod accounts;

pub use accounts::Accounts;

/// When and where a record happened.
///
/// One envelope for both record types, because tick and place are the two facts
/// every record needs and no record variant should have to carry them itself.
/// `place` is optional because a position can fall outside the place division,
/// and a record that guessed a region would be worse than one that says it does
/// not know.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub tick: u64,
    pub place: Option<PlaceId>,
    pub record: T,
}

impl<T> Envelope<T> {
    pub fn new(tick: u64, place: Option<PlaceId>, record: T) -> Self {
        Self {
            tick,
            place,
            record,
        }
    }
}

/// A causal event, stamped.
pub type RecordedEvent = Envelope<crate::history::Event>;

/// A resource movement, stamped.
pub type RecordedFlow = Envelope<FlowEvent>;

/// What is moving.
///
/// One variant, deliberately. The enclosure moves matter and nothing else, and
/// a carrier field with a single value is the seam PE4's generated material
/// vocabulary arrives through rather than a decoration: a flow that could not
/// say what it carried would have to be rewritten to admit a second material.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Carrier {
    #[default]
    Matter,
}

/// Where matter sits when it is not moving.
///
/// Exactly TD6's conserved sum, split: the ground store, what a body weighs,
/// and what it has banked. A transfer names two of these, so summing the stream
/// per account reproduces what the compartments did.
///
/// [`Dev`](Self::Dev) is the fourth, and it is outside the enclosure rather
/// than in it — see its own note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Account {
    /// The enclosure's per-column matter store.
    Soil,
    /// A body's substance: what its parts weigh.
    Substance,
    /// A body's banked budget.
    Reserve,
    /// The dev source: matter a dev tool put into the enclosure from outside
    /// it. (DT3)
    ///
    /// **An account, not an exception.** `Intent::PlaceMatter` could have
    /// written milligrams straight into a column and left the flow record
    /// silent, and then the enclosure's total would rise with nothing in the
    /// stream saying why — exactly the failure `tests/flows.rs` exists
    /// against. Naming the source instead keeps the reconciliation exact: the
    /// soil's gain is claimed by a transfer, and the transfer says it came
    /// from outside.
    ///
    /// Its balance is not held anywhere, because it does not need to be: the
    /// account's whole history is the stream, so what it has issued over a run
    /// is the sum of the transfers that name it, and
    /// [`Account::issued_mg`] is that sum. Nothing about it enters a snapshot,
    /// so a world that had matter placed in it hashes as a world holding that
    /// much matter — which is what it is.
    Dev,
}

/// Why matter moved.
///
/// Named `Process` after the plan's field and kept behind the `flow::` qualifier
/// because [`crate::process::Process`] is a different noun — what a *part* does.
/// The two converge later rather than colliding: when PE4 admits generated
/// transformations, this is where a `ProcessDef` identity lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Process {
    /// A producer drawing out of the ground.
    Uptake,
    /// Rent: what a body spends per tick simply existing.
    Upkeep,
    /// A meal. The one verb, whichever body took it.
    Feeding,
    /// Travel, paid in substance into the ground it was covered over.
    Travel,
    /// A parent provisioning an offspring.
    Birth,
    /// A body releasing what it was carrying when it stopped being able to.
    Death,
    /// A corpse returning to the column it lies on.
    Decay,
    /// The player enriching the ground.
    Deposit,
    /// What a body could not hold, and what a bite of venom cost, going back
    /// where it came from. Nothing evaporates. (TD6)
    Spill,
    /// A branch crossing from one body to another: the exact substance of a
    /// transferred subtree, out of the source and into the recipient. (P3)
    ///
    /// Distinct from [`Self::Feeding`] on purpose. A meal is matter a body
    /// took apart; a graft is matter that arrived still shaped like what it
    /// was, and a reading that could not tell them apart could not say whether
    /// a lineage is eating its neighbours or wearing them.
    Graft,
    /// Rebuilding an organ: the tissue a development moved, paid out of the
    /// reserve and into the ground under the body. (PD2)
    ///
    /// Distinct from [`Self::Upkeep`] on purpose. Rent is what carrying a
    /// gland costs every tick; this is the one-off price of installing one,
    /// and a reading that could not tell them apart could not say whether a
    /// lineage is building or merely standing still.
    Develop,
    /// A dev tool putting matter into a column, out of [`Account::Dev`]. (DT3)
    ///
    /// Distinct from [`Self::Deposit`] on purpose, and the distinction is the
    /// whole point of the account: a deposit is the *player* moving matter the
    /// enclosure already held, and this is matter arriving from outside it. A
    /// reading that could not tell them apart could not say whether a run was
    /// assisted.
    Place,
}

/// One side of a flow, when that side is a body.
///
/// The organism is the identity; the lineage and the kingdom are the two keys
/// the readings window by, and neither is recoverable from an [`Account`] alone.
/// The **true** kingdom, read off anatomy — a mimic's `guise` is what it claims,
/// and a ledger that believed a claim would be the wrong kind of record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Subject {
    pub organism: OrganismId,
    pub lineage: SpeciesId,
    pub kingdom: Kingdom,
}

impl Subject {
    /// Reads a subject off a body. Costs one anatomy read, so a hot loop takes
    /// it once per organism per tick rather than once per flow.
    pub fn of(organism: &Organism) -> Self {
        Self {
            organism: organism.id,
            lineage: organism.species,
            kingdom: organism.kingdom(),
        }
    }
}

/// One transfer of one carrier between two accounts.
///
/// The smallest record that can reconcile the current matter cycle: what moved,
/// how much, out of where and into where, and which bodies were on each side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowEvent {
    pub process: Process,
    pub carrier: Carrier,
    pub source: Account,
    pub destination: Account,
    pub amount_mg: u64,
    /// Whose account the matter left, when it left a body.
    pub from: Option<Subject>,
    /// Whose account it reached, when it reached one.
    pub to: Option<Subject>,
}

impl FlowEvent {
    /// Out of the ground and into a body.
    pub fn uptake(to: Subject, into: Account, amount_mg: u64) -> Self {
        Self {
            process: Process::Uptake,
            carrier: Carrier::Matter,
            source: Account::Soil,
            destination: into,
            amount_mg,
            from: None,
            to: Some(to),
        }
    }

    /// Out of a body and back into the ground.
    pub fn returned(process: Process, from: Subject, out_of: Account, amount_mg: u64) -> Self {
        Self {
            process,
            carrier: Carrier::Matter,
            source: out_of,
            destination: Account::Soil,
            amount_mg,
            from: Some(from),
            to: None,
        }
    }

    /// Out of the dev source and into the ground. (DT3)
    ///
    /// Names no [`Subject`] on either end, exactly as [`Self::uptake`] names
    /// none on the soil end: neither account belongs to a body, so a
    /// reconciliation over bodies passes this by and the soil's own claim is
    /// the whole of it.
    pub fn placed(amount_mg: u64) -> Self {
        Self {
            process: Process::Place,
            carrier: Carrier::Matter,
            source: Account::Dev,
            destination: Account::Soil,
            amount_mg,
            from: None,
            to: None,
        }
    }

    /// Between two bodies: a meal, or a parent provisioning a child.
    pub fn between(
        process: Process,
        from: Subject,
        out_of: Account,
        to: Subject,
        into: Account,
        amount_mg: u64,
    ) -> Self {
        Self {
            process,
            carrier: Carrier::Matter,
            source: out_of,
            destination: into,
            amount_mg,
            from: Some(from),
            to: Some(to),
        }
    }

    /// The signed effect of this flow on one account, in milligrams.
    ///
    /// A transfer between two of the same account nets to nothing, which is what
    /// makes soil-to-soil transport honest to leave unrecorded.
    pub fn net_on(&self, account: Account) -> i128 {
        let into = i128::from(self.destination == account);
        let out = i128::from(self.source == account);
        (into - out) * i128::from(self.amount_mg)
    }
}

/// The world's one-tick flow buffer.
///
/// **Beside the world, not inside it.** `World` holds this behind `serde(skip)`
/// and this type's [`PartialEq`] is unconditionally true, so the buffer reaches
/// neither the snapshot nor the state hash: a host that drains every frame and a
/// headless replay that never drains at all still compare and hash equal. Same
/// arrangement, and the same reason, as `Ground`'s dirty set.
///
/// Bounded by construction. `World::apply` opens the ledger at the top of each
/// tick, which drops whatever the previous tick left, so nothing accumulates
/// whether or not anybody is listening.
#[derive(Clone, Debug, Default)]
pub struct Ledger {
    tick: u64,
    records: Vec<RecordedFlow>,
}

/// Never part of a world's identity; see the type's own note.
impl PartialEq for Ledger {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Ledger {}

impl Ledger {
    /// Starts a tick, discarding anything the last one left undrained.
    pub fn open(&mut self, tick: u64) {
        self.tick = tick;
        self.records.clear();
    }

    /// Records a flow that happened in a region.
    ///
    /// The region is the acting body's, which is where a transfer is attributed
    /// even when the ground it drew from lies a column or two away: a root's
    /// forage radius reaches past its own column, so a per-region soil ledger
    /// would need the column rather than the body. Compartment reconciliation is
    /// exact regardless, because the two columns are the same account.
    ///
    /// A transfer of nothing is not a record. Emitting it would put a flood of
    /// empty rows in front of every reducer for no fact.
    pub fn record(&mut self, place: Option<PlaceId>, flow: FlowEvent) {
        if flow.amount_mg == 0 {
            return;
        }
        self.records.push(Envelope::new(self.tick, place, flow));
    }

    /// This tick's records, without taking them.
    pub fn records(&self) -> &[RecordedFlow] {
        &self.records
    }

    /// Takes this tick's records.
    pub fn take(&mut self) -> Vec<RecordedFlow> {
        std::mem::take(&mut self.records)
    }
}

/// The tick's two record streams, and the stamp they share.
///
/// **One accepted transaction emits both, at one commit point.** Holding the
/// writing end of the causal log and the flow ledger together is what makes that
/// sentence structural rather than a convention: nothing inside the tick can
/// move matter through a route that reaches one stream and not the other,
/// because there is one thing to reach.
///
/// The two buffers stay separately owned by the world, because only one of them
/// belongs in a snapshot.
pub struct Records<'a> {
    tick: u64,
    places: Option<&'a Places>,
    events: &'a mut Vec<RecordedEvent>,
    flows: &'a mut Ledger,
}

impl<'a> Records<'a> {
    pub fn new(
        tick: u64,
        places: Option<&'a Places>,
        events: &'a mut Vec<RecordedEvent>,
        flows: &'a mut Ledger,
    ) -> Self {
        Self {
            tick,
            places,
            events,
            flows,
        }
    }

    fn place_of(&self, position: [i32; 3]) -> Option<PlaceId> {
        self.places.and_then(|places| places.at(position))
    }

    /// Records that something happened to somebody, and where.
    pub fn event(&mut self, position: [i32; 3], event: crate::history::Event) {
        let place = self.place_of(position);
        self.events.push(Envelope::new(self.tick, place, event));
    }

    /// Records that matter moved, and where.
    pub fn flow(&mut self, position: [i32; 3], flow: FlowEvent) {
        let place = self.place_of(position);
        self.flows.record(place, flow);
    }
}

/// Consecutive ticks the stand must read short before the game says so.
///
/// **One whole judgement window, unbroken**: sixty ticks during every one of
/// which the trailing sixty read short. Six seconds at the canonical tempo.
///
/// Measured rather than picked. Eight seeds of the shipping roster were run two
/// thousand ticks apiece, untouched, and their longest shortfall streaks read
/// `0, 0, 0, 0, 83, 153, 169, 367`. Four enclosures never crossed the line at
/// all; the other four crossed it and stayed over — and those are the seeds
/// whose stand really is declining, which is the population instrument's
/// standing verdict for this world (`thins`, not `breathes`). So the number is
/// not chosen to keep the warning quiet in a shrinking enclosure. It is chosen
/// so that an enclosure holding its stand never raises it, which the same eight
/// runs show, while an induced overdraw on those same seeds reaches 60 to 884.
/// The receipt is `mesocosm-runtime/tests/readings.rs`; widening the judgement
/// window to 120, 180 or 240 was measured too and moved neither side.
pub const WARN_AFTER_TICKS: u64 = 60;

/// What the bounded windows currently read.
///
/// Facts and their windows, never a verdict: every number here says what moved
/// and over how many ticks, which is the shape the readings contract rules
/// (plan §3). The reduction that fills it lives in `mesocosm-runtime`; this type
/// is here so a view can render it without depending on the driver.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trend {
    /// Ticks the replacement counts cover.
    pub replacement_ticks: u64,
    /// Creatures that came of age in that window.
    pub matured: u32,
    /// Creatures that died in it.
    pub died: u32,
    /// Ticks the stand reading covers.
    pub stand_ticks: u64,
    /// What the enclosure's standing plant matter did over that window.
    ///
    /// **A stock trend, not a flow ratio**, and the round found out why the
    /// hard way: producers pay enormous rent, so their gross draw out of the
    /// ground is mostly treadmill and dwarfs anything a mouth takes, while the
    /// net of the two sits on zero at equilibrium and its sign is decided by
    /// noise. What a support path is actually short of is *stock*, so that is
    /// what the reading watches.
    pub stand_change_mg: i64,
    /// How much matter mouths took out of producers in the same window. Stated
    /// beside the change rather than divided into it: grazing is one of several
    /// ways the stand loses matter, and a ratio would claim it was the cause.
    pub grazed_mg: u64,
    /// Consecutive ticks the stand has read short over `stand_ticks`.
    pub shortfall_ticks: u64,
}

impl Trend {
    /// Maturation against mortality, per mille. `None` when nothing died, which
    /// is not an infinite ratio but an absent one.
    pub fn replacement_permille(&self) -> Option<u64> {
        (self.died > 0).then(|| u64::from(self.matured) * 1_000 / u64::from(self.died))
    }

    /// Whether the stand has been shrinking long enough to be worth saying.
    pub fn warns(&self) -> bool {
        self.shortfall_ticks >= WARN_AFTER_TICKS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: OrganismId = OrganismId(1);

    fn subject() -> Subject {
        Subject {
            organism: A,
            lineage: SpeciesId(3),
            kingdom: Kingdom::Producer,
        }
    }

    #[test]
    fn a_transfer_nets_out_of_one_account_and_into_the_other() {
        let flow = FlowEvent::uptake(subject(), Account::Substance, 40);
        assert_eq!(flow.net_on(Account::Soil), -40);
        assert_eq!(flow.net_on(Account::Substance), 40);
        assert_eq!(flow.net_on(Account::Reserve), 0);
    }

    #[test]
    fn a_ledger_holds_one_tick_and_drops_the_last() {
        let mut ledger = Ledger::default();
        ledger.open(7);
        ledger.record(None, FlowEvent::uptake(subject(), Account::Reserve, 5));
        assert_eq!(ledger.records().len(), 1);
        assert_eq!(ledger.records()[0].tick, 7);

        ledger.open(8);
        assert!(
            ledger.records().is_empty(),
            "an undrained tick does not accumulate"
        );
    }

    #[test]
    fn a_ledger_is_transparent_to_equality() {
        // The whole reason draining readings cannot move a state hash: two
        // ledgers holding different streams are still the same world.
        let mut drained = Ledger::default();
        let mut held = Ledger::default();
        held.open(1);
        held.record(None, FlowEvent::uptake(subject(), Account::Reserve, 9));
        assert_eq!(drained, held);
        assert_eq!(drained.take(), Vec::new());
    }

    /// The dev source is an account: it names no body, it lands in the soil,
    /// and what it has issued is the sum of the stream. (DT3)
    #[test]
    fn the_dev_source_is_an_account_outside_the_enclosure() {
        let placed = FlowEvent::placed(400);
        assert_eq!(placed.net_on(Account::Soil), 400);
        assert_eq!(placed.net_on(Account::Dev), -400);
        assert_eq!(placed.net_on(Account::Substance), 0);
        assert!(placed.from.is_none() && placed.to.is_none());
        assert!(!Account::Dev.is_body(), "and it is not a body's account");
        assert!(!Account::Soil.is_body());
        assert!(Account::Substance.is_body() && Account::Reserve.is_body());

        let stream = vec![
            Envelope::new(1, None, placed),
            Envelope::new(1, None, FlowEvent::placed(75)),
            // An ordinary transfer inside the enclosure issues nothing.
            Envelope::new(1, None, FlowEvent::uptake(subject(), Account::Reserve, 900)),
        ];
        assert_eq!(Account::issued_mg(&stream), 475);
    }

    #[test]
    fn a_zero_transfer_is_not_a_record() {
        // Nothing moved, so nothing happened. Recording it would put a flood of
        // empty rows in front of every reducer for no fact.
        let mut ledger = Ledger::default();
        ledger.open(1);
        ledger.record(None, FlowEvent::uptake(subject(), Account::Reserve, 0));
        assert!(ledger.records().is_empty());
    }

    #[test]
    fn a_trend_states_its_windows_rather_than_a_verdict() {
        let trend = Trend {
            replacement_ticks: 240,
            matured: 6,
            died: 4,
            stand_ticks: 60,
            stand_change_mg: -7_930,
            grazed_mg: 15_771,
            shortfall_ticks: WARN_AFTER_TICKS,
        };
        assert_eq!(trend.replacement_permille(), Some(1_500));
        assert!(trend.warns());

        let quiet = Trend {
            shortfall_ticks: WARN_AFTER_TICKS - 1,
            ..trend
        };
        assert!(!quiet.warns(), "the window is a threshold, not a rounding");

        let empty = Trend::default();
        assert_eq!(empty.replacement_permille(), None, "nothing died");
        assert!(!empty.warns());
    }
}

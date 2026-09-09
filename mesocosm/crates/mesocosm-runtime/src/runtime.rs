// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host-neutral driver.
//!
//! A host owns the window, the device, and the frame loop. It hands elapsed
//! time in, queues intents, and reads the world back to draw it. It never
//! touches world state, and every step it causes is recorded, so any run can
//! be replayed from its seed and trace alone.

use std::collections::VecDeque;

use mesocosm_core::{
    Accounts, DevelopmentError, History, Intent, OrganismId, Outcome, Reading, Trend, World,
    state_hash,
};

use crate::clock::Clock;
use crate::readings::FlowWindows;
use crate::review::{Authored, Review};
use crate::succession::{Checkpoint, Occasion};

mod expression_practice;
mod terrarium;

/// Default ceiling on steps authorised by one `advance` call. A stalled host
/// resuming after a long pause catches up over several frames rather than in
/// one burst that would itself cause another stall.
pub const DEFAULT_MAX_STEPS_PER_ADVANCE: u64 = 8;

/// Drives a [`World`] at a fixed rate from a host's uneven frame delivery.
pub struct Runtime {
    world: World,
    clock: Clock,
    queued: VecDeque<Intent>,
    /// The ordered trace of every intent actually applied.
    trace: Vec<Intent>,
    /// What happened, drained tick by tick.
    ///
    /// The world buffers one tick and a caller drains it, so somebody has to be
    /// that caller or a shipped run has no past at all. This is the caller: the
    /// driver already records every intent, and recording every consequence
    /// belongs beside it.
    history: History,
    /// The bounded ecology windows, reduced from the same two streams.
    ///
    /// Here rather than in the world for the reason the history is here: it is
    /// derivable from a seed and a trace, so keeping it in the snapshot would
    /// put a presentation reading inside the replay hash. `Runtime::replayed`
    /// rebuilds it, which is that claim made executable.
    readings: FlowWindows,
    /// The one body whose own accounts are being reduced beside the ecology's,
    /// and what they read. (DT2)
    ///
    /// **One body, because an inspector looks at one.** A per-organism ring for
    /// every creature in the enclosure would be a reading nobody asked for at
    /// nine hundred times the cost; this is three integers and an id. Reset
    /// whenever the id changes, so a window always covers the body it names.
    ///
    /// Beside the world like the readings above it: nothing here is written
    /// back, so watching a body cannot move the trace or the hash. A caller
    /// that never watches pays one `Option` comparison per tick.
    watched: Option<OrganismId>,
    accounts: Accounts,
    /// The question the world is holding at, if it is holding at one.
    ///
    /// **The pause lives here and nowhere else.** Stepping is the driver's job,
    /// so not stepping is too; the world stays a pure function of its seed and
    /// its trace and never learns that anybody stopped to think. See
    /// [`crate::succession`].
    checkpoint: Option<Checkpoint>,
    /// The played line's own turn, while the world is holding at a lineage
    /// checkpoint. (PE3b)
    ///
    /// **Built when the question opens and again after a commit**, never per
    /// frame: every row costs a bounded scoring run, which is the price an
    /// unplayed line's turn pays and not one a redraw should. Presentation like
    /// the readings beside it — nothing here is written back, so the trace and
    /// the hash never learn it exists.
    review: Option<Review>,
    /// The pack's declared expression scripts, when a host supplied them. The
    /// review's second proposal source; `None` leaves every row with one.
    authored: Option<Authored>,
    /// The epoch this driver has already reckoned. (PE3)
    ///
    /// The world ends its own epochs, because the rule that ends them is a
    /// world rule; the *reckoning* reads the past, which lives here, so this
    /// is how the driver notices there is one to do.
    ///
    /// **It is how the driver notices every boundary, and there is only one
    /// kind** (DT4). A budget that spent itself and a hand that asked through
    /// `Intent::EndEpoch` both come through `World::apply`'s boundary block and
    /// both are found here; the manual `Runtime::end_epoch` that used to reckon
    /// a boundary of its own is deleted. See `mesocosm_core`'s `world::adapt`
    /// module docs.
    epoch_seen: u64,
    /// What the most recent boundary came to.
    reckoning: Vec<Reading>,
    /// World-changing dev intents this run actually applied. (DT3)
    ///
    /// **A fact about how the run was driven, not about what it did.** The
    /// world does not branch on it and cannot see it; what reads it is the
    /// receipt, so a playtest that ended an epoch, forced a birth, killed
    /// something or placed matter is labelled assisted rather than passing for
    /// an unaided one (dev tools plan §2, principle 5).
    ///
    /// Counted here rather than in a host because it is a pure function of the
    /// trace and the world's answers — which is why a replay of a trace that
    /// carries dev intents arrives at the same number the recording did.
    /// Refused ones do not count: nothing was applied.
    dev_intents: u64,
    last: Vec<Outcome>,
    max_steps: u64,
    seed: u64,
    organisms: u32,
}

/// The reckoning, at whatever tick the world ended an epoch on.
///
/// **A driven run and a replay must both do this**, or their world records
/// diverge and so do their hashes. It is a free function for exactly that
/// reason: there is one of it, and both callers reach it.
fn reckon_if_ended(world: &mut World, history: &History, seen: &mut u64) -> Option<Vec<Reading>> {
    (world.epoch != *seen).then(|| {
        *seen = world.epoch;
        world.reckon(history)
    })
}

impl Runtime {
    pub fn new(seed: u64, organisms: u32, ticks_per_second: u32) -> Self {
        Self::from_world(
            World::new(seed, organisms),
            seed,
            organisms,
            ticks_per_second,
        )
    }

    /// Builds the shipping roster under an explicitly admitted developmental
    /// palette. The palette remains world state, so hosts can resolve the
    /// body's immutable content from the same admitted references used at
    /// founding and replay.
    pub fn with_development_palette(
        seed: u64,
        organisms: u32,
        ticks_per_second: u32,
        development_palette: mesocosm_core::PartPalette,
    ) -> Result<Self, DevelopmentError> {
        Self::with_founding_palette(
            seed,
            organisms,
            ticks_per_second,
            mesocosm_core::Founding::Roster,
            development_palette,
        )
    }

    /// Admit the declared recipe set as well as its immutable part vocabulary.
    pub fn with_founding_palette(
        seed: u64,
        organisms: u32,
        ticks_per_second: u32,
        founding: mesocosm_core::Founding,
        development_palette: mesocosm_core::PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let world = World::founded_with_palette(seed, organisms, founding, development_palette)?;
        Ok(Self::from_world(world, seed, organisms, ticks_per_second))
    }

    pub fn generated_start(
        selection: &mesocosm_core::world::generation::Selection,
        palette: mesocosm_core::PartPalette,
        ticks_per_second: u32,
    ) -> Result<Self, mesocosm_core::world::generation::Error> {
        let world = selection.enter(palette)?;
        Ok(Self::from_world(
            world,
            selection.request.seed,
            selection.request.organisms,
            ticks_per_second,
        ))
    }

    fn from_world(world: World, seed: u64, organisms: u32, ticks_per_second: u32) -> Self {
        Self {
            world,
            clock: Clock::new(ticks_per_second),
            queued: VecDeque::new(),
            trace: Vec::new(),
            history: History::new(),
            readings: FlowWindows::new(),
            watched: None,
            accounts: Accounts::default(),
            checkpoint: None,
            review: None,
            authored: None,
            epoch_seen: 0,
            reckoning: Vec::new(),
            dev_intents: 0,
            last: Vec::new(),
            max_steps: DEFAULT_MAX_STEPS_PER_ADVANCE,
            seed,
            organisms,
        }
    }

    pub fn with_max_steps(mut self, max_steps: u64) -> Self {
        assert!(max_steps > 0, "at least one step per advance");
        self.max_steps = max_steps;
        self
    }

    /// Gives the review a second proposal source: a pack's declared expression
    /// scripts. (PE3b)
    ///
    /// Presentation only, like the review itself. A script proposes and the
    /// world never hears about it, so this cannot move a hash or a trace, and a
    /// driver without it simply shows one source per row.
    pub fn with_authored(mut self, authored: Authored) -> Self {
        self.authored = Some(authored);
        self
    }

    /// Queues an intent for the next step. Intents are consumed in order, one
    /// per step; a step with nothing queued runs [`Intent::Idle`], so the
    /// simulation advances at a fixed rate whether or not the player acts.
    pub fn queue(&mut self, intent: Intent) {
        self.queued.push_back(intent);
    }

    pub fn queued_len(&self) -> usize {
        self.queued.len()
    }

    /// Runs whatever steps the elapsed time authorises. Returns how many ran.
    ///
    /// **A held world does not bank time.** While a [`checkpoint`] stands and
    /// nothing queued answers it, the elapsed microseconds are dropped rather
    /// than accumulated in the clock — otherwise a player who took ten seconds
    /// over the question would be answered by ten seconds of ecology sprinting
    /// past in one frame.
    ///
    /// [`checkpoint`]: Self::checkpoint
    pub fn advance(&mut self, elapsed_us: u64) -> u64 {
        if self.held_at_a_question() {
            self.last.clear();
            return 0;
        }
        let advance = self.clock.advance(elapsed_us, self.max_steps);
        self.last.clear();
        let mut taken = 0;
        for _ in 0..advance.steps {
            if !self.step_once() {
                break;
            }
            taken += 1;
        }
        taken
    }

    /// Whether the world is stopped at a question nothing queued answers.
    fn held_at_a_question(&self) -> bool {
        self.checkpoint.as_ref().is_some_and(|checkpoint| {
            !self
                .queued
                .front()
                .is_some_and(|intent| checkpoint.answers(intent))
        })
    }

    /// Runs up to `steps` steps, ignoring the clock. Returns how many ran, which
    /// is fewer than asked when a checkpoint holds the world.
    pub fn step(&mut self, steps: u64) -> u64 {
        self.last.clear();
        let mut taken = 0;
        for _ in 0..steps {
            if !self.step_once() {
                break;
            }
            taken += 1;
        }
        taken
    }

    /// One tick, unless the world is holding at a question this tick's intent
    /// does not answer. `false` means nothing happened and nothing was consumed.
    fn step_once(&mut self) -> bool {
        if let Some(checkpoint) = &self.checkpoint {
            // An unanswered question is not a slow tick; it is a stopped world.
            // Note what is *queued* rather than substituting an answer: putting
            // one in on the player's behalf is exactly what "one recorded
            // choice" rules out.
            match self.queued.front() {
                // A revision is taken *at* the lineage checkpoint and leaves
                // the question standing, so the player is not thrown back into
                // the terrarium by committing. Every other answer closes it.
                Some(intent) if checkpoint.closed_by(intent) => self.checkpoint = None,
                Some(intent) if checkpoint.answers(intent) => {},
                _ => return false,
            }
        }
        // The hand, read before the tick: a critter that dies this tick is held
        // by nobody after it — and if it was *eaten* it is not even in the
        // roster to be asked what lineage it was. That is the moment the
        // question matters most, so both facts are taken while they still exist.
        let hand = self
            .world
            .held()
            .and_then(|id| self.world.controlled().map(|held| (id, held.species)));
        let intent = self.queued.pop_front().unwrap_or(Intent::Idle);
        // A commit at the boundary leaves the question standing and changes
        // what the review is about, so the reading is rebuilt rather than left
        // describing the program the line no longer has.
        let revised = matches!(intent, Intent::Revise { .. });
        // Read before the intent is consumed, and counted only where the world
        // accepted it: a refused dev intent applied nothing, so a run that only
        // ever asked for something impossible was not assisted. (DT3)
        let dev = intent.is_dev();
        let outcome = self.world.apply(intent.clone());
        if dev && !matches!(outcome, Outcome::Rejected(_)) {
            self.dev_intents += 1;
        }
        self.trace.push(intent);
        self.absorb(hand, revised);
        self.last.push(outcome);
        true
    }

    /// Takes the tick's two records: the causal half into the past, both halves
    /// into the windows, and both again to see whether this tick asked the
    /// player something. One call, because they are one tick's worth and a
    /// caller that drained one and forgot the other would have a past and a
    /// reading that disagreed.
    fn absorb(
        &mut self,
        hand: Option<(mesocosm_core::OrganismId, mesocosm_core::SpeciesId)>,
        revised: bool,
    ) {
        let events = self.world.drain_events();
        let flows = self.world.drain_flows();
        self.readings.absorb(&events, &flows);
        // The watched body's own half of the same tick, through core's split.
        // (DT2)
        if let Some(watched) = self.watched {
            self.accounts.absorb(watched, &flows);
        }
        // Recorded before the question is put, so a line that gained a
        // descendant on the same tick it lost its body can still be continued
        // through that descendant.
        self.history.record_all(events.iter().copied());
        // And before the reckoning, which reads the past this tick just added
        // to. (PE3)
        if let Some(readings) =
            reckon_if_ended(&mut self.world, &self.history, &mut self.epoch_seen)
        {
            self.reckoning = readings;
        }
        if self.checkpoint.is_none() {
            self.checkpoint =
                crate::succession::opened(&self.world, &self.history, hand, &events, &flows);
        }
        self.refresh_review(revised);
    }

    /// Keeps the played line's turn in step with the question. (PE3b)
    ///
    /// Built once when a lineage checkpoint opens, rebuilt after a commit, and
    /// dropped the moment the world is not holding at one — so an ordinary tick
    /// of play pays one match and nothing else.
    fn refresh_review(&mut self, revised: bool) {
        let at_boundary = matches!(
            self.checkpoint.as_ref().map(|held| held.occasion),
            Some(Occasion::Epoch(_))
        );
        if !at_boundary {
            self.review = None;
            return;
        }
        if self.review.is_some() && !revised {
            return;
        }
        self.review = Review::of(
            &self.world,
            &self.reckoning,
            self.readings.trend(),
            self.authored.as_ref(),
        );
    }

    /// What the most recent epoch boundary reckoned. Empty until one happens.
    pub fn reckoning(&self) -> &[Reading] {
        &self.reckoning
    }

    /// The played line's turn, while the world is holding at its lineage
    /// checkpoint. `None` at every other moment. (PE3b)
    pub fn review(&self) -> Option<&Review> {
        self.review.as_ref()
    }

    /// The question the world is holding at, if any.
    pub fn checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoint.as_ref()
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// Takes the ground bricks changed since the last drain, so a host's
    /// projection can upload regions rather than the whole map.
    ///
    /// The only `&mut World` route a host gets, and it is not a world change:
    /// the dirty set is outside the snapshot and outside equality, so a host
    /// that drains and a headless replay that never does still agree.
    pub fn drain_ground_dirty(&mut self) -> Vec<[i16; 3]> {
        self.world.drain_ground_dirty()
    }

    /// What this run has seen happen.
    pub fn history(&self) -> &History {
        &self.history
    }

    /// The bounded ecology windows over this run's flows.
    pub fn windows(&self) -> &FlowWindows {
        &self.readings
    }

    /// What those windows currently read: facts and the windows they cover.
    pub fn trend(&self) -> Trend {
        self.readings.trend()
    }

    /// Reduces one body's own accounts beside the ecology's windows. (DT2)
    ///
    /// Idempotent: watching the body already watched keeps the window it has,
    /// so a host may call this every frame. Naming a different body — or none —
    /// starts the window over, because figures carried across a change of
    /// subject would be somebody else's.
    ///
    /// Presentation only. It reduces the same drained stream the readings do
    /// and writes nothing back, so a run that watches a body and one that never
    /// does are the same world and the same state hash.
    pub fn watch(&mut self, organism: Option<OrganismId>) {
        if self.watched == organism {
            return;
        }
        self.watched = organism;
        self.accounts = Accounts::default();
    }

    /// Which body is being watched, if any.
    pub fn watched(&self) -> Option<OrganismId> {
        self.watched
    }

    /// What the watched body's accounts read, over the ticks since it was
    /// watched. All zero, over zero ticks, when nobody is.
    pub fn accounts(&self) -> Accounts {
        self.accounts
    }

    /// What each lineage has done, without noting any of it.
    ///
    /// The reading half on its own, so a host can show a standing without
    /// ending an epoch to find it out.
    pub fn readings(&self) -> Vec<Reading> {
        mesocosm_core::readings(&self.world, &self.history)
    }

    /// The ordered trace of applied intents. Together with the seed and organism
    /// count this reproduces the run exactly.
    pub fn trace(&self) -> &[Intent] {
        &self.trace
    }

    /// World-changing dev intents this run applied. (DT3)
    ///
    /// Zero for a run that used none, which is every run without `--dev` and
    /// most runs with it. A nonzero count is what makes a receipt say the run
    /// was assisted.
    pub fn dev_intents(&self) -> u64 {
        self.dev_intents
    }

    /// Outcomes from the most recent `advance` or `step`.
    pub fn last_outcomes(&self) -> &[Outcome] {
        &self.last
    }

    pub fn state_hash(&self) -> u64 {
        state_hash(&self.world)
    }

    /// The receipt a host probe compares: what this run was, and where it
    /// ended up.
    pub fn receipt(&self) -> Receipt {
        Receipt {
            seed: self.seed,
            organisms: self.organisms,
            // The trace is one entry per step actually applied, so it is the
            // step count whether the run was clocked or stepped by hand — and
            // it stays honest across a checkpoint, where the clock may have
            // authorised steps the held world never took.
            steps: self.trace.len() as u64,
            state_hash: self.state_hash(),
        }
    }

    /// Rebuilds a run from a seed and trace, without any host at all. Two hosts
    /// agree exactly when their traces replay to the same hash here.
    ///
    /// Returns the past as well as the world, because it has to: a driven run
    /// drains its events every tick and a replay that did not would end holding
    /// a tick of undrained ones, which is a difference in the snapshot and so a
    /// difference in the hash. Reproducing the history rather than discarding it
    /// is also the claim that keeps it out of the snapshot, made executable.
    pub fn replay(seed: u64, organisms: u32, trace: &[Intent]) -> (World, History) {
        let replayed = Self::replayed(seed, organisms, trace);
        (replayed.world, replayed.history)
    }

    /// The same replay, keeping the readings it rebuilt.
    ///
    /// **The done-condition made runnable.** The windows are not in the
    /// snapshot, so nothing forces them to agree; reducing the replay's own
    /// streams through the same reducer and comparing the encodings is what
    /// shows that a replayed run reads the same as the run it replays.
    pub fn replayed(seed: u64, organisms: u32, trace: &[Intent]) -> Replayed {
        let mut world = World::new(seed, organisms);
        let mut history = History::new();
        let mut readings = FlowWindows::new();
        let mut epoch_seen = 0;
        for intent in trace {
            world.apply(intent.clone());
            let events = world.drain_events();
            readings.absorb(&events, &world.drain_flows());
            history.record_all(events);
            // The same reckoning a driven run does, through the same function.
            // The world ends its own epochs, so a replay reaches every boundary
            // the run did; skipping the reckoning here would leave the replayed
            // world's record short and its hash different. (PE3)
            reckon_if_ended(&mut world, &history, &mut epoch_seen);
        }
        Replayed {
            world,
            history,
            readings,
        }
    }
}

/// What a replay reproduced: the world, its past, and its readings.
pub struct Replayed {
    pub world: World,
    pub history: History,
    pub readings: FlowWindows,
}

/// A run's identity, for comparing hosts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub seed: u64,
    pub organisms: u32,
    pub steps: u64,
    pub state_hash: u64,
}

#[cfg(test)]
mod tests;

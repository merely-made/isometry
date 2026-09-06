// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The bounded ecology reducer.
//!
//! The driver already drains a tick of causal events and rebuilds them on
//! replay, so it is where a replay-derived reduction of the flow stream belongs
//! (PE0, playable ecology plan §8). Nothing here is authority: it reads two
//! streams the world emitted and keeps fixed-size integer windows over them.
//!
//! # Bounded, and explicitly so
//!
//! One ring of [`RETENTION_TICKS`] per-tick totals. Two windows are read off it
//! — the whole ring for replacement, [`JUDGEMENT_TICKS`] for support — because
//! several resolutions may coexist but every retention length has to be stated
//! and tested. Nothing here grows with the length of a run.
//!
//! # Deterministic
//!
//! Integers only, summed over a fixed slice in a fixed order, from streams that
//! are themselves a function of the seed and the trace. So a replay reduces to
//! the same windows byte for byte, which `tests/readings.rs` asserts by encoding
//! both and comparing the bytes.

use mesocosm_core::Kingdom;
use mesocosm_core::flow::{Account, Process, RecordedEvent, RecordedFlow, Subject, Trend};
use mesocosm_core::history::Event;
use serde::{Deserialize, Serialize};

/// How many ticks of per-tick totals are retained.
///
/// Two hundred and forty: the plan's own example window, twenty-four seconds at
/// the canonical ten ticks a second. Long enough to hold a life-history event
/// or two of a starter body, short enough that the whole ring is one cache line
/// per field.
pub const RETENTION_TICKS: usize = 240;

/// The window the stand reading judges over.
///
/// Sixty ticks — six seconds. Wide enough that one corpse or one bite does not
/// decide the sign, narrow enough that the moment a stand starts losing ground
/// is not smeared across half a lifetime. How *long* the trouble has lasted is
/// carried by the shortfall streak instead, which is what the warning says.
pub const JUDGEMENT_TICKS: usize = 60;

/// One tick, reduced.
///
/// Four numbers, because PE0 ships two indicators: replacement (maturation
/// against mortality) and the support path (what the standing plant matter did,
/// and how much of it mouths took). Everything else the readings contract lists
/// waits for the phase that consumes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Totals {
    /// Net change in the substance of every producer-bodied organism.
    ///
    /// **The stock, not the throughput.** Producers pay rent proportional to
    /// what they carry, so their gross draw out of the ground is mostly a
    /// treadmill and outweighs any mouth several times over; the net of draw
    /// against rent sits on zero at equilibrium and its sign is decided by
    /// noise. What a support path runs short of is standing matter, so that is
    /// what is counted, straight off the flow record's own accounts.
    stand_change_mg: i64,
    /// Matter taken out of producer substance by something feeding.
    grazed_mg: u64,
    matured: u32,
    died: u32,
}

impl Totals {
    /// Reduces one tick of both streams.
    ///
    /// Maturation and mortality come from the causal record and the two matter
    /// sums from the flow record, which is the split the two records exist for:
    /// coming of age moves no matter, and a milligram of upkeep is nobody's
    /// biography.
    fn of(events: &[RecordedEvent], flows: &[RecordedFlow]) -> Self {
        let mut totals = Self::default();
        for event in events {
            match event.record {
                Event::Matured { .. } => totals.matured += 1,
                Event::Died { .. } => totals.died += 1,
                _ => {},
            }
        }
        let producer = |side: Option<Subject>| {
            side.is_some_and(|subject| subject.kingdom == Kingdom::Producer)
        };
        for flow in flows {
            let record = &flow.record;
            let amount = record.amount_mg as i64;
            // Every transfer touching producer substance, in either direction.
            // A birth is producer-to-producer and cancels, which is right: a
            // seedling is the stand rearranging itself, not growing.
            if record.source == Account::Substance && producer(record.from) {
                totals.stand_change_mg -= amount;
            }
            if record.destination == Account::Substance && producer(record.to) {
                totals.stand_change_mg += amount;
            }
            // What a mouth took, whoever the mouth belonged to. A spill from a
            // producer went to the ground rather than into a consumer, so it is
            // a loss to the stand but not a graze.
            if record.process == Process::Feeding
                && record.source == Account::Substance
                && producer(record.from)
            {
                totals.grazed_mg += record.amount_mg;
            }
        }
        totals
    }
}

/// Fixed-size integer windows over the world's two record streams.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowWindows {
    /// The ring. Grows to [`RETENTION_TICKS`] and never past it; a `Vec` rather
    /// than an array only because serde's blanket array impls stop at 32 and
    /// this has to be encodable to be compared byte for byte.
    ring: Vec<Totals>,
    /// Ticks absorbed, ever. Also the write cursor, modulo the ring.
    absorbed: u64,
    /// Consecutive ticks the judgement window has read short.
    shortfall_ticks: u64,
}

impl Default for FlowWindows {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowWindows {
    pub fn new() -> Self {
        Self {
            ring: Vec::with_capacity(RETENTION_TICKS),
            absorbed: 0,
            shortfall_ticks: 0,
        }
    }

    /// Reduces one tick, dropping whatever fell out of the ring.
    pub fn absorb(&mut self, events: &[RecordedEvent], flows: &[RecordedFlow]) {
        let totals = Totals::of(events, flows);
        let slot = (self.absorbed % RETENTION_TICKS as u64) as usize;
        if self.ring.len() < RETENTION_TICKS {
            self.ring.push(totals);
        } else {
            self.ring[slot] = totals;
        }
        self.absorbed += 1;

        // The streak is judged after this tick is in, so a warning is always
        // about a window that includes the tick it was raised on.
        self.shortfall_ticks = if self.stand().0 < 0 {
            self.shortfall_ticks + 1
        } else {
            0
        };
    }

    /// How many ticks the ring is holding.
    pub fn retained(&self) -> u64 {
        self.absorbed.min(RETENTION_TICKS as u64)
    }

    /// The most recent `ticks` entries, newest last. Never more than the ring.
    fn recent(&self, ticks: usize) -> impl Iterator<Item = &Totals> {
        let held = self.ring.len();
        let want = ticks.min(held);
        // Walk backwards from the write cursor so the order is the order the
        // ticks happened in, wherever the cursor currently sits.
        (0..want).map(move |back| {
            let index = ((self.absorbed % held as u64) as usize + held - back - 1) % held;
            &self.ring[index]
        })
    }

    /// What the stand did over the judgement window, and how much was grazed.
    fn stand(&self) -> (i64, u64) {
        self.recent(JUDGEMENT_TICKS)
            .fold((0, 0), |(change, grazed), tick| {
                (change + tick.stand_change_mg, grazed + tick.grazed_mg)
            })
    }

    /// What the windows currently read.
    pub fn trend(&self) -> Trend {
        let (matured, died) = self
            .recent(RETENTION_TICKS)
            .fold((0, 0), |(matured, died), tick| {
                (matured + tick.matured, died + tick.died)
            });
        let (stand_change_mg, grazed_mg) = self.stand();
        Trend {
            replacement_ticks: self.retained(),
            matured,
            died,
            stand_ticks: self.retained().min(JUDGEMENT_TICKS as u64),
            stand_change_mg,
            grazed_mg,
            shortfall_ticks: self.shortfall_ticks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::flow::{Envelope, FlowEvent};
    use mesocosm_core::{OrganismId, SpeciesId};

    fn subject(kingdom: Kingdom) -> Subject {
        Subject {
            organism: OrganismId(1),
            lineage: SpeciesId(1),
            kingdom,
        }
    }

    /// A tick in which the stand drew `grew` mg into substance and a mouth took
    /// `grazed` out of it.
    fn tick(grew: u64, grazed: u64) -> Vec<RecordedFlow> {
        vec![
            Envelope::new(
                0,
                None,
                FlowEvent::uptake(subject(Kingdom::Producer), Account::Substance, grew),
            ),
            Envelope::new(
                0,
                None,
                FlowEvent::between(
                    Process::Feeding,
                    subject(Kingdom::Producer),
                    Account::Substance,
                    subject(Kingdom::Consumer),
                    Account::Reserve,
                    grazed,
                ),
            ),
        ]
    }

    fn died() -> Vec<RecordedEvent> {
        vec![Envelope::new(
            0,
            None,
            Event::Died {
                organism: OrganismId(1),
                species: SpeciesId(1),
            },
        )]
    }

    #[test]
    fn the_ring_holds_its_stated_retention_and_no_more() {
        // The retention length is a claim, so it is asserted rather than
        // assumed: a run three times the window long still answers over one.
        let mut windows = FlowWindows::new();
        for _ in 0..RETENTION_TICKS * 3 {
            windows.absorb(&died(), &[]);
        }
        assert_eq!(windows.retained(), RETENTION_TICKS as u64);
        assert_eq!(windows.ring.len(), RETENTION_TICKS);
        assert_eq!(windows.trend().died, RETENTION_TICKS as u32);
    }

    #[test]
    fn a_tick_that_fell_out_of_the_window_stops_counting() {
        let mut windows = FlowWindows::new();
        windows.absorb(&died(), &[]);
        assert_eq!(windows.trend().died, 1);
        for _ in 0..RETENTION_TICKS {
            windows.absorb(&[], &[]);
        }
        assert_eq!(windows.trend().died, 0, "the ring forgot it, exactly once");
    }

    #[test]
    fn the_stand_window_is_the_shorter_of_the_two() {
        let mut windows = FlowWindows::new();
        for _ in 0..RETENTION_TICKS {
            windows.absorb(&[], &tick(10, 4));
        }
        let trend = windows.trend();
        assert_eq!(trend.replacement_ticks, RETENTION_TICKS as u64);
        assert_eq!(trend.stand_ticks, JUDGEMENT_TICKS as u64);
        assert_eq!(trend.stand_change_mg, 6 * JUDGEMENT_TICKS as i64);
        assert_eq!(trend.grazed_mg, 4 * JUDGEMENT_TICKS as u64);
    }

    #[test]
    fn the_streak_counts_consecutive_short_ticks_and_resets_on_one_good_one() {
        let mut windows = FlowWindows::new();
        for _ in 0..300 {
            windows.absorb(&[], &tick(1, 9));
        }
        assert_eq!(windows.trend().shortfall_ticks, 300);
        assert!(windows.trend().warns());

        // One tick of plenty is not enough to clear a window that is still
        // mostly short, which is the point of judging over a window at all.
        windows.absorb(&[], &tick(100, 0));
        assert!(windows.trend().shortfall_ticks > 0);

        for _ in 0..JUDGEMENT_TICKS {
            windows.absorb(&[], &tick(100, 0));
        }
        assert_eq!(windows.trend().shortfall_ticks, 0);
        assert!(!windows.trend().warns());
    }

    #[test]
    fn a_quiet_enclosure_never_warns() {
        let mut windows = FlowWindows::new();
        for _ in 0..RETENTION_TICKS * 2 {
            windows.absorb(&[], &tick(10, 3));
        }
        assert_eq!(windows.trend().shortfall_ticks, 0);
    }
}

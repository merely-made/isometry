// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The fixed-step clock.
//!
//! A host measures elapsed wall time however it likes and hands it here. The
//! clock converts it into a whole number of simulation steps and keeps the
//! remainder, so the number of steps taken depends only on total elapsed time
//! and never on how that time arrived.
//!
//! Time is integer microseconds. The core is integer-only for determinism, and
//! a float accumulator here would put the host platform's rounding behaviour
//! back into the step count.
//!
//! The accumulator is kept in *rational* form: elapsed microseconds are scaled
//! by the tick rate and compared against one second, rather than divided by a
//! precomputed interval. A precomputed interval would have to round (60 Hz is
//! 16666.67 microseconds, and 1_000_000 is not divisible by 60), and the
//! rounding error would accumulate as drift. This way every tick rate is
//! exact and no rate is refused.

use serde::{Deserialize, Serialize};

const ONE_SECOND_US: u64 = 1_000_000;

/// Converts elapsed wall time into fixed simulation steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clock {
    ticks_per_second: u64,
    /// Scaled by `ticks_per_second`; one step is due per `ONE_SECOND_US`.
    scaled_remainder: u64,
    /// Steps this clock has ever authorised. Monotonic.
    steps: u64,
}

/// What a single [`Clock::advance`] authorised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Advance {
    /// Steps to run now.
    pub steps: u64,
    /// Steps deferred to a later call because they exceeded the cap.
    pub deferred: u64,
}

impl Clock {
    pub fn new(ticks_per_second: u32) -> Self {
        assert!(ticks_per_second > 0, "tick rate must be non-zero");
        Self {
            ticks_per_second: ticks_per_second as u64,
            scaled_remainder: 0,
            steps: 0,
        }
    }

    pub fn ticks_per_second(&self) -> u64 {
        self.ticks_per_second
    }

    /// Nominal step length, rounded down. For a host that wants to pace
    /// itself; the clock itself never uses it.
    pub fn nominal_interval_us(&self) -> u64 {
        ONE_SECOND_US / self.ticks_per_second
    }

    pub fn steps_taken(&self) -> u64 {
        self.steps
    }

    /// Accepts elapsed microseconds and returns the steps to run.
    ///
    /// `max_steps` bounds how many steps one call may authorise, so a long
    /// stall cannot produce an unbounded burst. Deferred steps stay in the
    /// remainder and arrive on later calls, so **no step is ever dropped** and
    /// the total for a given elapsed time is fixed. That is what makes the
    /// simulation independent of frame delivery rather than merely tolerant of
    /// it.
    pub fn advance(&mut self, elapsed_us: u64, max_steps: u64) -> Advance {
        let scaled =
            (elapsed_us as u128 * self.ticks_per_second as u128).min(u64::MAX as u128) as u64;
        self.scaled_remainder = self.scaled_remainder.saturating_add(scaled);

        let due = self.scaled_remainder / ONE_SECOND_US;
        let steps = due.min(max_steps);
        self.scaled_remainder -= steps * ONE_SECOND_US;
        self.steps += steps;
        Advance {
            steps,
            deferred: due - steps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNCAPPED: u64 = u64::MAX;

    #[test]
    fn even_and_uneven_delivery_authorise_the_same_steps() {
        let mut even = Clock::new(60);
        let mut uneven = Clock::new(60);

        let mut even_total = 0;
        for _ in 0..60 {
            even_total += even.advance(16_666, UNCAPPED).steps;
        }

        let ragged = [1_000u64, 400_000, 33, 250_000, 99_567, 249_360];
        let mut uneven_total = 0;
        for chunk in ragged {
            uneven_total += uneven.advance(chunk, UNCAPPED).steps;
        }

        assert_eq!(ragged.iter().sum::<u64>(), 16_666 * 60);
        assert_eq!(even_total, uneven_total);
        assert_eq!(even.steps_taken(), uneven.steps_taken());
    }

    #[test]
    fn sixty_hertz_does_not_drift() {
        // The failure this design exists to avoid: a precomputed 16666 us
        // interval runs fast and gains a step roughly every 25 seconds.
        let mut clock = Clock::new(60);
        for _ in 0..60 {
            clock.advance(ONE_SECOND_US, UNCAPPED);
        }
        assert_eq!(clock.steps_taken(), 60 * 60, "exactly 60 steps per second");
    }

    #[test]
    fn awkward_tick_rates_are_exact_too() {
        let mut clock = Clock::new(7);
        for _ in 0..10 {
            clock.advance(ONE_SECOND_US, UNCAPPED);
        }
        assert_eq!(clock.steps_taken(), 70);
    }

    #[test]
    fn remainder_is_never_lost() {
        let mut clock = Clock::new(100);
        let mut total = 0;
        for _ in 0..10 {
            total += clock.advance(9_000, UNCAPPED).steps;
        }
        assert_eq!(total, 9);
    }

    #[test]
    fn a_cap_defers_rather_than_drops() {
        let mut clock = Clock::new(60);
        let first = clock.advance(ONE_SECOND_US, 4);
        assert_eq!(first.steps, 4);
        assert_eq!(first.deferred, 56);

        let mut recovered = first.steps;
        for _ in 0..14 {
            recovered += clock.advance(0, 4).steps;
        }
        assert_eq!(recovered, 60, "no step may be dropped by the cap");
    }

    #[test]
    fn zero_elapsed_authorises_nothing() {
        let mut clock = Clock::new(60);
        assert_eq!(clock.advance(0, UNCAPPED).steps, 0);
    }
}

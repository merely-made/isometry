// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Standing: one person's read of another, as two small integers.
//!
//! Standing is never authored. It is folded out of the deed log, and it carries
//! the ids of the deeds that produced it, so the explanation surface can point
//! at the evidence instead of asserting a number. Change the log and the
//! standing changes with it; there is no second copy to drift.

use std::collections::BTreeMap;

use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};

use crate::deed::{DeedId, DeedLog};

/// Trust is what they would risk on you. Affinity is whether they like you.
/// They move together often and not always, which is most of the character in
/// the model.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    pub trust: i16,
    pub affinity: i16,
    /// Oldest first, in log order.
    pub from_deeds: Vec<DeedId>,
}

impl Standing {
    pub fn stranger() -> Self {
        Self::default()
    }

    pub fn known(&self) -> bool {
        !self.from_deeds.is_empty()
    }
}

/// Keyed by (holder, toward): how the holder reads the other one. Asymmetric
/// on purpose, since being stood by and standing by are different facts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relations {
    standings: BTreeMap<(SubjectId, SubjectId), Standing>,
}

impl Relations {
    /// The whole fold. Every deed aimed at somebody moves that person's read
    /// of the doer, by the doer's own deed weights.
    pub fn derive(log: &DeedLog) -> Self {
        let mut standings: BTreeMap<(SubjectId, SubjectId), Standing> = BTreeMap::new();
        for deed in log.deeds() {
            let Some(toward) = deed.toward else { continue };
            if toward == deed.doer {
                continue;
            }
            let (trust, affinity) = deed.kind.weight();
            let standing = standings.entry((toward, deed.doer)).or_default();
            standing.trust += trust;
            standing.affinity += affinity;
            standing.from_deeds.push(deed.id);
        }
        Self { standings }
    }

    /// How `holder` reads `toward`. A stranger is a zero, not an absence, so
    /// callers never have to special-case a first meeting.
    pub fn of(&self, holder: SubjectId, toward: SubjectId) -> Standing {
        self.standings
            .get(&(holder, toward))
            .cloned()
            .unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&(SubjectId, SubjectId), &Standing)> {
        self.standings.iter()
    }

    pub fn len(&self) -> usize {
        self.standings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.standings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deed::DeedKind;
    use paredros_identity::Tick;

    const AUD: SubjectId = SubjectId(1);
    const ODRIS: SubjectId = SubjectId(3);

    #[test]
    fn standing_is_a_view_over_the_log_and_names_its_deeds() {
        let mut log = DeedLog::new();
        let shared = log.record(Tick(3), AUD, Some(ODRIS), DeedKind::Shared);
        let left = log.record(Tick(4), AUD, Some(ODRIS), DeedKind::Abandoned);

        let standing = Relations::derive(&log).of(ODRIS, AUD);
        assert_eq!(standing.trust, -4);
        assert_eq!(standing.affinity, -1);
        assert_eq!(standing.from_deeds, vec![shared, left]);

        // And the other direction is its own fact, untouched.
        assert!(!Relations::derive(&log).of(AUD, ODRIS).known());
    }
}

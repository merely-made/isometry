// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Controlled-subject session state: who the player is being.
//!
//! The pointer moves only through a recorded intent, with the same discipline
//! as Mesocosm's control pointer, and replaying the intents reproduces it
//! exactly. That is what makes tag-in and succession the same machinery seen
//! at two lengths.
//!
//! The puppeteering canary is held here by the shape of the state rather than
//! by discipline: [`Control::played`] is one subject, there is no list, and no
//! call in this crate acts on anyone. Two characters at once is not a rule
//! this type enforces, it is a state this type cannot represent.

use serde::{Deserialize, Serialize};

use crate::{IdentityError, SubjectId, Tick};

/// Every way the played subject may change, and the only ways.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlIntent {
    /// Opens the log. Exactly once, first.
    Begin { subject: SubjectId, at: Tick },
    /// Becomes a companion for a while. Home is remembered.
    TagIn { to: SubjectId, at: Tick },
    /// Returns to home.
    TagOut { at: Tick },
    /// Home itself changes: the played subject died, and a companion is now
    /// the one you are. Tag-in rehearsed this.
    Succeed { to: SubjectId, at: Tick },
}

/// Who the player is being, and how they came to be being them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Control {
    home: SubjectId,
    played: SubjectId,
    log: Vec<ControlIntent>,
}

impl Control {
    pub fn begin(subject: SubjectId, at: Tick) -> Self {
        Self {
            home: subject,
            played: subject,
            log: vec![ControlIntent::Begin { subject, at }],
        }
    }

    /// The subject the player is embodying right now. One, always.
    pub fn played(&self) -> SubjectId {
        self.played
    }

    /// The subject a tag-out returns to.
    pub fn home(&self) -> SubjectId {
        self.home
    }

    pub fn tagged_in(&self) -> bool {
        self.played != self.home
    }

    pub fn log(&self) -> &[ControlIntent] {
        &self.log
    }

    pub fn apply(&mut self, intent: ControlIntent) -> Result<(), IdentityError> {
        match intent {
            ControlIntent::Begin { .. } => return Err(IdentityError::AlreadyBegun),
            ControlIntent::TagIn { to, .. } => {
                if self.tagged_in() {
                    return Err(IdentityError::AlreadyTaggedIn);
                }
                self.played = to;
            },
            ControlIntent::TagOut { .. } => {
                if !self.tagged_in() {
                    return Err(IdentityError::NotTaggedIn);
                }
                self.played = self.home;
            },
            ControlIntent::Succeed { to, .. } => {
                self.home = to;
                self.played = to;
            },
        }
        self.log.push(intent);
        Ok(())
    }

    /// Rebuilds the session from its intents. A save carries the log; the
    /// pointer is derived, never stored on its own.
    pub fn replay(intents: &[ControlIntent]) -> Result<Self, IdentityError> {
        let Some(ControlIntent::Begin { subject, at }) = intents.first().copied() else {
            return Err(IdentityError::NotBegun);
        };
        let mut control = Self::begin(subject, at);
        for intent in &intents[1..] {
            control.apply(*intent)?;
        }
        Ok(control)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUD: SubjectId = SubjectId(1);
    const BRAM: SubjectId = SubjectId(2);

    #[test]
    fn a_tag_in_returns_home_and_a_succession_moves_it() {
        let mut control = Control::begin(AUD, Tick(0));
        control
            .apply(ControlIntent::TagIn {
                to: BRAM,
                at: Tick(4),
            })
            .unwrap();
        assert_eq!(control.played(), BRAM);
        assert_eq!(control.home(), AUD);

        control
            .apply(ControlIntent::TagOut { at: Tick(6) })
            .unwrap();
        assert_eq!(control.played(), AUD);

        control
            .apply(ControlIntent::Succeed {
                to: BRAM,
                at: Tick(9),
            })
            .unwrap();
        assert_eq!(control.played(), BRAM);
        assert_eq!(control.home(), BRAM);
        assert!(!control.tagged_in());
    }

    #[test]
    fn the_pointer_replays_from_its_intents() {
        let mut control = Control::begin(AUD, Tick(0));
        for intent in [
            ControlIntent::TagIn {
                to: BRAM,
                at: Tick(4),
            },
            ControlIntent::TagOut { at: Tick(6) },
            ControlIntent::Succeed {
                to: BRAM,
                at: Tick(9),
            },
        ] {
            control.apply(intent).unwrap();
        }
        assert_eq!(Control::replay(control.log()).unwrap(), control);
    }

    #[test]
    fn a_second_tag_in_is_refused() {
        let mut control = Control::begin(AUD, Tick(0));
        control
            .apply(ControlIntent::TagIn {
                to: BRAM,
                at: Tick(4),
            })
            .unwrap();
        assert_eq!(
            control.apply(ControlIntent::TagIn {
                to: SubjectId(3),
                at: Tick(5)
            }),
            Err(IdentityError::AlreadyTaggedIn)
        );
    }
}

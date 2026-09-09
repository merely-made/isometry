// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A declared expression fixture: the exact proposal, and the exact draws.
//! (PD4)
//!
//! Plan §4 asks for *exact fixtures for proposal, entropy draws, validation and
//! refusal*. This is the record. It pins both halves deliberately: a script
//! that changes **how many numbers it reads, or in what order** is a different
//! script even when it happens to return the same proposal today, and a fixture
//! that only checked the proposal would let that through.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{Entropy, Policy, Proposal, Refused, Request, Runner};

/// One recorded expression: a context, a seed, and what came back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    /// What this fixture is for, in the plain sentence a failure prints.
    pub name: String,
    /// The seed the host's tape was drawn from.
    pub seed: u64,
    /// The frozen context, exactly as the script saw it.
    pub request: Request,
    /// The proposal that must come back.
    pub expected: Proposal,
    /// The draw trace that must be recorded. Exact.
    pub draws: Vec<u64>,
}

/// Why a fixture did not hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mismatch {
    /// The script refused where the fixture expected a proposal.
    Refused(Refused),
    /// A different proposal came back.
    Proposal { found: Proposal },
    /// The same proposal, from different entropy. A changed number or order of
    /// draws is a changed script.
    Draws { found: Vec<u64> },
}

impl Fixture {
    /// Reads a fixture from a declared pack file.
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
    }

    /// Runs the script against this fixture's own context and checks both
    /// halves.
    pub fn check(&self, script: &str, policy: Policy) -> Result<(), Mismatch> {
        let mut runner = Runner::load(script, policy).map_err(Mismatch::Refused)?;
        let entropy = Entropy::from_seed(self.seed);
        let found = runner
            .propose(&self.request, &entropy)
            .map_err(Mismatch::Refused)?;
        if found != self.expected {
            return Err(Mismatch::Proposal { found });
        }
        if entropy.draws != self.draws {
            return Err(Mismatch::Draws {
                found: entropy.draws,
            });
        }
        Ok(())
    }

    /// The context this fixture declares, for a caller that wants to lower the
    /// proposal onto a real body rather than only compare it.
    pub fn request(&self) -> &Request {
        &self.request
    }
}

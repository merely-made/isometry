// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The developmental verb: a discovered candidate, previewed, expressed and
//! paid for. (PD3)

use crate::discovery::ConditionId;
use crate::flow::{Account, FlowEvent, Subject};

use super::{Outcome, World};

mod preview;

pub use preview::ExpressionPreview;

#[cfg(test)]
mod tests;

impl World {
    /// Checks a discovered development without changing the world.
    ///
    /// The returned phenotype is the checked candidate the expression would
    /// commit. This is the same preparation path as [`Self::express`], so a
    /// panel never quotes a different recipient, price, revision, or refusal
    /// from the one the intent will receive.
    pub fn preview_expression(
        &self,
        condition: ConditionId,
    ) -> Result<ExpressionPreview, super::Rejection> {
        self.prepare_expression(condition)
            .map(|prepared| prepared.preview)
    }

    /// Commits an expression after the shared preparation passed.
    pub(super) fn express(&mut self, condition: ConditionId) -> Outcome {
        let prepared = match self.prepare_expression(condition) {
            Ok(prepared) => prepared,
            Err(rejection) => return Outcome::Rejected(rejection),
        };
        let preview = prepared.preview;
        let (position, subject) = {
            let organism = self
                .organisms
                .iter()
                .find(|organism| organism.id == preview.recipient)
                .expect("the prepared recipient remains in the world");
            (organism.position, Subject::of(organism))
        };
        let organism = self
            .organisms
            .iter_mut()
            .find(|organism| organism.id == preview.recipient)
            .expect("the prepared recipient remains in the world");
        organism.phenotype = preview.phenotype;
        organism.energy_mg -= preview.cost_mg;

        let column = self.soil.column_at(position);
        self.soil.deposit(column, preview.cost_mg);
        self.flow(
            position,
            FlowEvent::returned(
                crate::flow::Process::Develop,
                subject,
                Account::Reserve,
                preview.cost_mg,
            ),
        );
        Outcome::Expressed {
            part: preview.part,
            cost_mg: preview.cost_mg,
            revision: preview.revision,
        }
    }
}

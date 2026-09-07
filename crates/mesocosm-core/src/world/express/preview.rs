// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Checked, unpublished expression candidates.

use crate::body::PartId;
use crate::discovery::ConditionId;
use crate::organism::OrganismId;
use crate::phenotype::{Arrangement, BodyPhenotype};

use super::super::{Rejection, World};

/// What an expression would commit, after every expression check passed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionPreview {
    pub recipient: OrganismId,
    pub condition: ConditionId,
    pub part: PartId,
    pub cost_mg: u64,
    pub revision: u32,
    pub phenotype: BodyPhenotype,
}

impl World {
    /// The one preparation path shared by preview and commit.
    pub(super) fn prepare_expression(
        &self,
        condition: ConditionId,
    ) -> Result<PreparedExpression, Rejection> {
        let Some(me) = self.controlled() else {
            return Err(Rejection::Disembodied);
        };
        if !self.discovered(condition) {
            return Err(Rejection::Undiscovered(condition));
        }
        // `Arrangement` is diagnostic; the validator does not inspect it.
        let Some(proposal) = self.candidate_proposal(condition, Arrangement::Direct) else {
            return Err(Rejection::Nowhere(condition));
        };
        // Candidates name one organ, priced in that organ's own tissue.
        let Some(&part) = proposal.parts.first() else {
            return Err(Rejection::Nowhere(condition));
        };
        let (recipient, energy_mg) = (me.id, me.energy_mg);
        let mut phenotype = me.phenotype.clone();
        let development = phenotype
            .develop(self.ruleset(), &proposal)
            .map_err(Rejection::Refused)?;
        let cost_mg = u64::from(development.instruction.cost_cells) * me.phenotype.cell_mg(part);
        if cost_mg > energy_mg {
            return Err(Rejection::InsufficientMass);
        }

        Ok(PreparedExpression {
            preview: ExpressionPreview {
                recipient,
                condition,
                part,
                cost_mg,
                revision: development.instruction.revision,
                phenotype,
            },
        })
    }
}

pub(super) struct PreparedExpression {
    pub(super) preview: ExpressionPreview,
}

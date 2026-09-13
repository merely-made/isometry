// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The declared tissue a lineage starts with before any ecological exchange.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::matter::{Material, Stock};

/// A lineage's initial tissue proportions, in [`Material::ALL`] order.
///
/// The weights are lineage state rather than a reading of the current body.
/// Feeding and growth change an organism's stored scruples; this recipe only
/// supplies a new, unparented founder. The default is only for hand-built
/// fixtures; every live worldgen path installs an explicit living material.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct InitialTissueRecipe {
    weights: [u64; 4],
}

/// Why an initial tissue recipe cannot found a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TissueRecipeError {
    Empty,
}

impl InitialTissueRecipe {
    /// Declares raw relative weights in [`Material::ALL`] order.
    pub fn try_new(weights: [u64; 4]) -> Result<Self, TissueRecipeError> {
        weights
            .iter()
            .any(|weight| *weight != 0)
            .then_some(Self { weights })
            .ok_or(TissueRecipeError::Empty)
    }

    /// Declares one pure founding tissue.
    pub const fn single(material: Material) -> Self {
        let mut weights = [0; 4];
        weights[material.index()] = 1;
        Self { weights }
    }

    /// The saved relative weights, for inspection and provenance displays.
    pub const fn weights(self) -> [u64; 4] {
        self.weights
    }

    /// Scales this declaration to an exact body mass.
    ///
    /// Fractional remainders are assigned by [`Material::ALL`] order, matching
    /// [`Stock::take`]'s deterministic tie rule. An empty declaration cannot
    /// silently relabel a nonempty founder as untyped matter.
    pub fn stock_for(self, mass_mg: u64) -> Result<Stock, TissueRecipeError> {
        if mass_mg == 0 {
            return Ok(Stock::EMPTY);
        }
        let total: u128 = self.weights.iter().map(|weight| u128::from(*weight)).sum();
        if total == 0 {
            return Err(TissueRecipeError::Empty);
        }
        let mut amounts = [0; 4];
        let mut remainders = [0; 4];
        let mut assigned = 0_u128;
        for material in Material::ALL {
            let index = material.index();
            let product = u128::from(self.weights[index]) * u128::from(mass_mg);
            amounts[index] = (product / total) as u64;
            remainders[index] = product % total;
            assigned += u128::from(amounts[index]);
        }
        let mut order = [0_usize, 1, 2, 3];
        order.sort_by(|left, right| remainders[*right].cmp(&remainders[*left]));
        for index in order
            .into_iter()
            .take((u128::from(mass_mg) - assigned) as usize)
        {
            amounts[index] += 1;
        }
        Ok(Stock::from_amounts(amounts))
    }
}

impl<'de> Deserialize<'de> for InitialTissueRecipe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Stored {
            weights: [u64; 4],
        }
        let stored = Stored::deserialize(deserializer)?;
        Self::try_new(stored.weights).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for TissueRecipeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("initial tissue recipe has no material"),
        }
    }
}

impl Default for InitialTissueRecipe {
    fn default() -> Self {
        Self::single(Material::Untyped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_weights_close_an_odd_body_exactly() {
        let stock = InitialTissueRecipe::try_new([0, 2, 1, 0])
            .unwrap()
            .stock_for(5)
            .unwrap();
        assert_eq!(stock.amounts(), [0, 3, 2, 0]);
        assert_eq!(stock.total(), 5);
    }

    #[test]
    fn empty_declaration_refuses_a_nonempty_founder() {
        assert_eq!(
            InitialTissueRecipe::try_new([0; 4]),
            Err(TissueRecipeError::Empty)
        );
        assert_eq!(
            InitialTissueRecipe::single(Material::Producer).stock_for(0),
            Ok(Stock::EMPTY),
        );
    }

    #[test]
    fn serde_rejects_empty_and_round_trips_large_weights() {
        assert!(serde_json::from_str::<InitialTissueRecipe>(r#"{"weights":[0,0,0,0]}"#).is_err());
        let recipe = InitialTissueRecipe::try_new([u64::MAX, 1, 0, u64::MAX]).unwrap();
        let encoded = serde_json::to_string(&recipe).unwrap();
        assert_eq!(
            serde_json::from_str::<InitialTissueRecipe>(&encoded).unwrap(),
            recipe
        );
        assert_eq!(
            recipe.stock_for(u64::MAX).unwrap().total(),
            u64::MAX as u128
        );
    }
}

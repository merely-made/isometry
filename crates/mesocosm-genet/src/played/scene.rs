// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// The host presentation/world fixture selected for a recorded run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneMode {
    #[default]
    Ecology,
    Terrarium,
    /// Authored practice fixture, distinct from a natural encounter.
    GraftPractice,
    /// Authored discovery prehistory ready for somatic expression.
    ExpressionPractice,
    /// Original authored family fixture retained for recorded play.
    FamilyPractice,
    /// Family fixture with a clearing, cave, tunnel and reserve provision.
    FamilyClearing,
    /// The same origin with excess parent reserve returned to local soil.
    FamilyClearingLean,
}

impl SceneMode {
    pub const fn is_ecology(&self) -> bool {
        matches!(self, Self::Ecology)
    }

    pub const fn is_family_clearing(self) -> bool {
        matches!(self, Self::FamilyClearing | Self::FamilyClearingLean)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Ecology => "ecology",
            Self::Terrarium => "terrarium",
            Self::GraftPractice => "graft-practice",
            Self::ExpressionPractice => "expression-practice",
            Self::FamilyPractice => "family-practice",
            Self::FamilyClearing => "family-clearing",
            Self::FamilyClearingLean => "family-clearing-lean",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "ecology" => Some(Self::Ecology),
            "terrarium" => Some(Self::Terrarium),
            "graft-practice" => Some(Self::GraftPractice),
            "expression-practice" => Some(Self::ExpressionPractice),
            "family-practice" => Some(Self::FamilyPractice),
            "family-clearing" => Some(Self::FamilyClearing),
            "family-clearing-lean" => Some(Self::FamilyClearingLean),
            _ => None,
        }
    }
}

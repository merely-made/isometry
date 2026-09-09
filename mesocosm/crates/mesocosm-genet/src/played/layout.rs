// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::Founding;
use serde::{Deserialize, Serialize};

/// A founding recipe set, independent of voxel content and drawing mode.
/// Saved traces select their own set; absent values retain historical anatomy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyLayout {
    #[default]
    Axial,
    Branching,
    Jointed,
    Spaced,
}

impl BodyLayout {
    pub const fn axial() -> Self {
        Self::Axial
    }

    pub fn founding(self) -> Founding {
        match self {
            Self::Axial => Founding::Roster,
            Self::Branching => Founding::BranchingRoster,
            Self::Jointed => Founding::JointedRoster,
            Self::Spaced => Founding::SpacedRoster,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Axial => "axial",
            Self::Branching => "branching",
            Self::Jointed => "jointed",
            Self::Spaced => "spaced",
        }
    }

    pub fn is_axial(&self) -> bool {
        *self == Self::Axial
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "axial" => Some(Self::Axial),
            "branching" => Some(Self::Branching),
            "jointed" => Some(Self::Jointed),
            "spaced" => Some(Self::Spaced),
            _ => None,
        }
    }
}

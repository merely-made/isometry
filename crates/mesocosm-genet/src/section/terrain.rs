// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host presentation presets. They never enter a world or a played trace.

use super::{Grade, PALETTE, Section};
use mesocosm_lens::TerrainAppearance;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerrainStyle {
    #[default]
    Auto,
    Classic,
    Habitat,
}

impl Section {
    pub fn configure_terrain(
        &mut self,
        style: TerrainStyle,
        section_centre: [f32; 3],
        clearing_y: f32,
    ) {
        if style.resolved(self.terrarium.is_some()) == TerrainStyle::Classic {
            self.terrain_appearance = None;
            self.grade = Grade::retro(PALETTE);
            return;
        }
        self.grade = Grade {
            fog_start: 1.0,
            ..Grade::clay()
        };
        self.terrain_appearance = Some(TerrainAppearance {
            soil: [0.30, 0.22, 0.17],
            rock: [0.25, 0.31, 0.34],
            unknown: [0.60, 0.23, 0.58],
            sky: [0.64, 0.73, 0.76],
            underground: [0.09, 0.12, 0.14],
            section_centre,
            clearing_y,
        });
    }
}

impl TerrainStyle {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "classic" => Some(Self::Classic),
            "habitat" => Some(Self::Habitat),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Classic => "classic",
            Self::Habitat => "habitat",
        }
    }

    pub fn resolved(self, terrarium: bool) -> Self {
        match self {
            Self::Auto if terrarium => Self::Habitat,
            Self::Auto => Self::Classic,
            explicit => explicit,
        }
    }
}

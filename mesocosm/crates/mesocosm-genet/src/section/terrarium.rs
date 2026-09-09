// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Fixed habitat volume and explicit, presentation-only interior exposure.

use super::{CameraMode, Section};
use mesocosm_core::{places::Ground, world::TerrariumHabitat};
use mesocosm_lens::{BrickMap, BrickProjectionRevision};

/// CP1 uses the existing locomotion unit conversion for all visible anatomy.
pub const BODY_SCALE: f32 = 1.0 / mesocosm_core::places::BODY_VOXELS_PER_GROUND_VOXEL as f32;

/// Frame admitted anatomy once at genesis. The host retains this volume while
/// organisms move; a longer founding plant cannot be clipped by a seed's crop.
pub fn framed_habitat(world: &mesocosm_core::World) -> TerrariumHabitat {
    let mut habitat = world.terrarium_habitat();
    for organism in &world.organisms {
        let body = organism.body().aabb();
        for axis in 0..3 {
            let floor = if axis == 1 { body.min[1] } else { 0 };
            let min = organism.position[axis] as f32 + (body.min[axis] - floor) as f32 * BODY_SCALE;
            let max = organism.position[axis] as f32 + (body.max[axis] - floor) as f32 * BODY_SCALE;
            habitat.bounds.min[axis] = habitat.bounds.min[axis].min(min.floor() as i32);
            habitat.bounds.max[axis] = habitat.bounds.max[axis].max(max.ceil() as i32);
        }
    }
    habitat
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cutaway {
    #[default]
    Occupied,
    Always,
    Never,
}
impl Cutaway {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "occupied" => Some(Self::Occupied),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Occupied => "occupied",
            Self::Always => "always",
            Self::Never => "never",
        }
    }
}

pub(super) struct TerrariumView {
    habitat: TerrariumHabitat,
    pitch: f32,
    reveal: bool,
    cached: Option<(u64, CameraMode, bool)>,
    revision: u64,
}
impl TerrariumView {
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        (
            self.habitat.bounds.min.map(|v| v as f32),
            self.habitat.bounds.max.map(|v| v as f32 + 1.0),
        )
    }
    pub fn depth(&self) -> f32 {
        let (min, max) = self.bounds();
        // Entire fixed volume fits inside every rotated ray envelope.
        (0..3)
            .map(|i| (max[i] - min[i]).powi(2))
            .sum::<f32>()
            .sqrt()
            * 2.0
    }
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn refresh(
        &mut self,
        ground: &Ground,
        mode: CameraMode,
    ) -> Result<Option<BrickMap>, String> {
        let key = (ground.revision(), mode, self.reveal);
        if self.cached == Some(key) {
            return Ok(None);
        }
        self.revision += 1;
        let bounds = self.habitat.bounds;
        let chamber = self.habitat.chamber;
        let forward = mode.forward();
        let horizontal = forward[0].hypot(forward[2]);
        let forward = forward.map(|v| v / horizontal);
        let reveal = self.reveal;
        let map = BrickMap::from_ground_filtered(
            ground,
            BrickProjectionRevision(self.revision),
            |at, _| {
                let inside = (0..3).all(|i| at[i] >= bounds.min[i] && at[i] <= bounds.max[i]);
                let foreground = (at[0] - chamber[0]) as f32 * forward[0]
                    + (at[2] - chamber[2]) as f32 * forward[2]
                    < 3.0;
                inside && !(reveal && foreground && at[1] >= chamber[1])
            },
        )
        .map_err(|error| error.to_string())?;
        self.cached = Some(key);
        Ok(Some(map))
    }
}

/// Occupancy is read from the controlled body's position in this connected
/// route. This prototype reveals current occupancy, not unexplored interiors.
pub fn occupied(habitat: &TerrariumHabitat, at: [i32; 3]) -> bool {
    at[1] < habitat.entrance[1]
        && habitat.route.iter().any(|point| {
            (point[0] - at[0]).abs() <= 3
                && (point[2] - at[2]).abs() <= 3
                && (point[1] - at[1]).abs() <= 2
        })
}

impl Section {
    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
    }

    pub fn configure_terrarium(
        &mut self,
        habitat: &TerrariumHabitat,
        pitch: f32,
        policy: Cutaway,
        at: [i32; 3],
    ) {
        self.bodies.scale = BODY_SCALE;
        self.bodies.ground_anatomy = true;
        let reveal =
            policy == Cutaway::Always || (policy == Cutaway::Occupied && occupied(habitat, at));
        let view = self.terrarium.get_or_insert_with(|| TerrariumView {
            habitat: habitat.clone(),
            pitch,
            reveal,
            cached: None,
            revision: 0,
        });
        view.pitch = pitch;
        view.reveal = reveal;
    }
}

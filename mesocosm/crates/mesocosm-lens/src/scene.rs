// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable presentation inputs for native/browser parity receipts.

use crate::{CritterPose, Flight, Grade, maps::BiomeMaps};

/// One complete lens projection, independent of a GPU or host.
///
/// This is presentation data rather than simulation authority. V1 serializes
/// it only to prove that native and browser hosts receive identical inputs.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LensScene {
    pub maps: BiomeMaps,
    pub flight: Flight,
    pub grade: Grade,
    pub pose: Option<CritterPose>,
}

impl LensScene {
    pub fn to_postcard(&self) -> Result<Vec<u8>, SceneCodecError> {
        postcard::to_allocvec(self).map_err(SceneCodecError::Encode)
    }

    pub fn from_postcard(bytes: &[u8]) -> Result<Self, SceneCodecError> {
        postcard::from_bytes(bytes).map_err(SceneCodecError::Decode)
    }
}

#[derive(Debug)]
pub enum SceneCodecError {
    Encode(postcard::Error),
    Decode(postcard::Error),
}

impl std::fmt::Display for SceneCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encode(error) => write!(f, "could not encode lens scene: {error}"),
            Self::Decode(error) => write!(f, "could not decode lens scene: {error}"),
        }
    }
}

impl std::error::Error for SceneCodecError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maps;

    #[test]
    fn a_scene_round_trips_as_the_same_projection() {
        let maps = maps::synthesize(41, 32);
        let scene = LensScene {
            grade: Grade::retro(maps.palette.len() as u32),
            maps,
            flight: Flight {
                eye: [8.0, 40.0, 8.0],
                yaw: 0.4,
                pitch: -0.2,
                fov: 0.9,
                far: 200.0,
            },
            pose: Some(CritterPose {
                bounds_radius: 2.0,
                tint: [0.3, 0.7, 0.4],
                ..Default::default()
            }),
        };
        let bytes = scene.to_postcard().expect("encode");
        assert_eq!(LensScene::from_postcard(&bytes).expect("decode"), scene);
    }
}

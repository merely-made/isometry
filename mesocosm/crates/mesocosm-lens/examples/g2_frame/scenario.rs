// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::places::{Ground, Places};
use mesocosm_lens::{
    BrickDiagnostics, BrickFrameInput, BrickMap, BrickRevision, BrickTracer, CritterPose, Grade,
    TraceCamera, critter::Capsule,
};

pub const INITIAL_SIZE: [u32; 2] = [960, 540];
pub const MIN_FRAMES: u32 = 2;
pub const WINDOW_TITLE: &str = "Mesocosm G2";

pub struct Scenario {
    map: BrickMap,
    revision: BrickRevision,
    camera: TraceCamera,
    grade: Grade,
    pose: CritterPose,
    bytes: Vec<u8>,
}

impl Scenario {
    pub fn new() -> Result<Self, String> {
        let ground = Ground::grow(&Places::grown(4_242, 4, 64), 64);
        let map = BrickMap::from_ground(&ground).map_err(|error| error.to_string())?;
        let body_top = ground
            .surface(4, 18)
            .ok_or("body fixture column is outside Ground")? as f32;
        let body = [4.5, body_top + 1.15, 18.5];
        let camera = TraceCamera::orthographic_slab(
            [body[0], body_top * 0.5 + 4.0, body[2]],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            20.0,
            16.0 / 9.0,
            16.0,
        )
        .ok_or("invalid terrarium camera")?;
        let pose = CritterPose::from_capsules(
            vec![Capsule {
                a: [body[0] - 0.7, body[1], body[2]],
                ra: 0.65,
                b: [body[0] + 0.7, body[1], body[2]],
                rb: 0.52,
            }],
            [
                [body[0] - 0.45, body[1] + 0.15, body[2] - 0.35, 0.10],
                [body[0] - 0.45, body[1] - 0.15, body[2] - 0.35, 0.10],
            ],
            [0.15, 0.86, 0.32],
        );
        let grade = Grade::retro(3);
        let bytes = postcard::to_allocvec(&(ground, camera, grade, pose.clone()))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            map,
            revision: BrickRevision(0),
            camera,
            grade,
            pose,
            bytes,
        })
    }

    pub fn encode(
        &mut self,
        tracer: &mut BrickTracer,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        _frame: u32,
    ) -> Result<BrickDiagnostics, String> {
        tracer
            .encode(
                encoder,
                target,
                BrickFrameInput::for_camera(&self.map, self.revision, self.camera, &self.grade)
                    .with_pose(&self.pose),
            )
            .map_err(|error| error.to_string())
    }

    pub fn map(&self) -> &BrickMap {
        &self.map
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

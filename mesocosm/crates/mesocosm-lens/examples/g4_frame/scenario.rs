// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use mesocosm_core::organism::FaunaDrive;
use mesocosm_core::places::{PlaceId, WALKER_HEIGHT};
use mesocosm_core::{History, Intent, Outcome, World, state_hash};
use mesocosm_lens::{
    BodyLensProjection, BodyPlacement, BrickDiagnostics, BrickFrameInput, BrickMap, BrickRevision,
    BrickTracer, Flight, Grade,
};

use crate::burrow_scenario::{self, HUNTER_ID, PLAYER_ID};

pub const INITIAL_SIZE: [u32; 2] = [960, 540];
pub const MIN_FRAMES: u32 = 4;
pub const WINDOW_TITLE: &str = "Mesocosm G4: generated crossing";

pub struct Scenario {
    world: World,
    twin: World,
    history: History,
    twin_history: History,
    map: BrickMap,
    route: Vec<[i32; 3]>,
    boundary_step: usize,
    from_place: PlaceId,
    to_place: PlaceId,
    trace: Vec<Intent>,
    player_positions: Vec<[i32; 3]>,
    hunter_positions: Vec<[i32; 3]>,
    flight: Flight,
    grade: Grade,
    body: BodyLensProjection,
    source_revision: u64,
    replay_hash: u64,
    scene_bytes: Vec<u8>,
    advanced_steps: usize,
}

impl Scenario {
    pub fn new() -> Result<Self, String> {
        let fixture = burrow_scenario::setup();
        let twin_fixture = burrow_scenario::setup();
        if state_hash(&fixture.world) != state_hash(&twin_fixture.world) {
            return Err("independent seed-0 worlds disagree before the G4 trace".into());
        }
        let player = position(&fixture.world, PLAYER_ID)?;
        let hunter = position(&fixture.world, HUNTER_ID)?;
        let map =
            BrickMap::from_ground(fixture.world.ground()).map_err(|error| error.to_string())?;
        let grade = Grade::retro(3);
        let flight = flight(player, hunter);
        let body = hunter_body(&fixture.world)?;
        let source_revision = fixture.world.ground().revision();
        let replay_hash = state_hash(&fixture.world);
        let mut scenario = Self {
            world: fixture.world,
            twin: twin_fixture.world,
            history: History::new(),
            twin_history: History::new(),
            map,
            route: fixture.route,
            boundary_step: fixture.boundary_step,
            from_place: fixture.from_place,
            to_place: fixture.to_place,
            trace: fixture.trace,
            player_positions: vec![player],
            hunter_positions: vec![hunter],
            flight,
            grade,
            body,
            source_revision,
            replay_hash,
            scene_bytes: Vec::new(),
            advanced_steps: 0,
        };
        scenario.refresh_scene_bytes()?;
        Ok(scenario)
    }

    pub fn encode(
        &mut self,
        tracer: &mut BrickTracer,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: u32,
    ) -> Result<BrickDiagnostics, String> {
        let wanted_steps = frame
            .saturating_sub(1)
            .min(u32::try_from(self.trace.len()).unwrap_or(u32::MAX))
            as usize;
        while self.advanced_steps < wanted_steps {
            self.advance()?;
        }
        let input = BrickFrameInput::new(
            &self.map,
            BrickRevision(self.world.ground().revision()),
            &self.flight,
            &self.grade,
        )
        .with_pose(&self.body.pose);
        let diagnostics = tracer
            .encode(encoder, target, input)
            .map_err(|error| error.to_string())?;
        if frame > 1 && diagnostics.brick_upload_bytes != 0 {
            return Err(format!(
                "static generated Ground uploaded {} bytes on movement frame {frame}",
                diagnostics.brick_upload_bytes
            ));
        }
        Ok(diagnostics)
    }

    fn advance(&mut self) -> Result<(), String> {
        let index = self.advanced_steps;
        let intent = self
            .trace
            .get(index)
            .cloned()
            .ok_or("the G4 host advanced past its ordered trace")?;
        let outcome = self.world.apply(intent.clone());
        let twin_outcome = self.twin.apply(intent);
        self.history.record_all(self.world.drain_events());
        self.twin_history.record_all(self.twin.drain_events());
        if outcome != twin_outcome {
            return Err(format!("G4 replay changed movement outcome {index}"));
        }
        if !matches!(outcome, Outcome::Moved) {
            return Err(format!("G4 movement {index} was {outcome:?}"));
        }

        let player = position(&self.world, PLAYER_ID)?;
        let hunter = position(&self.world, HUNTER_ID)?;
        if player != self.route[index + 2] {
            return Err(format!(
                "player stuttered at G4 step {index}: {player:?} != {:?}",
                self.route[index + 2]
            ));
        }
        if hunter != self.route[index + 1] {
            return Err(format!(
                "hunter left the generated route at G4 step {index}: {hunter:?} != {:?}",
                self.route[index + 1]
            ));
        }
        if !self.world.ground().stands(hunter, WALKER_HEIGHT) {
            return Err(format!("hunter lost footing at G4 step {index}"));
        }
        let drive = self
            .world
            .organisms
            .iter()
            .find(|organism| organism.id == HUNTER_ID)
            .and_then(|organism| organism.last_fauna_decision.as_ref())
            .map(|decision| decision.selected_drive);
        if drive != Some(FaunaDrive::Pursue) {
            return Err(format!("hunter selected {drive:?} at G4 step {index}"));
        }

        let replay_hash = state_hash(&self.world);
        if replay_hash != state_hash(&self.twin) || self.history != self.twin_history {
            return Err(format!("G4 replay diverged after movement {index}"));
        }
        self.player_positions.push(player);
        self.hunter_positions.push(hunter);
        self.flight = flight(player, hunter);
        self.body = hunter_body(&self.world)?;
        self.replay_hash = replay_hash;
        self.advanced_steps += 1;
        self.refresh_scene_bytes()
    }

    fn refresh_scene_bytes(&mut self) -> Result<(), String> {
        self.scene_bytes = postcard::to_allocvec(&(
            &self.world,
            self.flight,
            self.grade,
            &self.body,
            &self.trace,
            self.advanced_steps,
        ))
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn scene_bytes(&self) -> &[u8] {
        &self.scene_bytes
    }

    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub fn committed_revision(&self) -> u64 {
        self.world.ground().revision()
    }

    pub fn route(&self) -> &[[i32; 3]] {
        &self.route
    }

    pub fn boundary_step(&self) -> usize {
        self.boundary_step
    }

    pub fn origin_place(&self) -> PlaceId {
        self.from_place
    }

    pub fn destination_place(&self) -> PlaceId {
        self.to_place
    }

    pub fn ordered_intents(&self) -> usize {
        self.trace.len()
    }

    pub fn advanced_steps(&self) -> usize {
        self.advanced_steps
    }

    pub fn player_start(&self) -> [i32; 3] {
        self.player_positions[0]
    }

    pub fn player_after(&self) -> [i32; 3] {
        *self.player_positions.last().unwrap()
    }

    pub fn hunter_start(&self) -> [i32; 3] {
        self.hunter_positions[0]
    }

    pub fn hunter_after(&self) -> [i32; 3] {
        *self.hunter_positions.last().unwrap()
    }

    pub fn player_crossed_boundary(&self) -> bool {
        self.world.places().at(self.player_start()) == Some(self.from_place)
            && self.world.places().at(self.player_after()) == Some(self.to_place)
    }

    pub fn hunter_crossed_boundary(&self) -> bool {
        self.world.places().at(self.hunter_start()) == Some(self.from_place)
            && self.world.places().at(self.hunter_after()) == Some(self.to_place)
    }

    pub fn hunter_crossed_threshold(&self) -> bool {
        self.hunter_positions.first() == self.route.first()
            && self.hunter_positions.get(1) == self.route.get(1)
    }

    pub fn player_stutterless(&self) -> bool {
        unique(&self.player_positions)
    }

    pub fn hunter_stutterless(&self) -> bool {
        unique(&self.hunter_positions)
    }

    pub fn replay_hash(&self) -> u64 {
        self.replay_hash
    }

    pub fn history_events(&self) -> usize {
        self.history.log().len()
    }

    pub fn body_revision(&self) -> u64 {
        self.body.revision.0
    }

    pub fn body_capsules(&self) -> usize {
        self.body.pose.capsules.len()
    }
}

fn unique(positions: &[[i32; 3]]) -> bool {
    positions.iter().copied().collect::<BTreeSet<_>>().len() == positions.len()
}

fn position(world: &World, id: mesocosm_core::OrganismId) -> Result<[i32; 3], String> {
    world
        .organisms
        .iter()
        .find(|organism| organism.id == id)
        .map(|organism| organism.position)
        .ok_or_else(|| format!("G4 world lost organism {id:?}"))
}

fn flight(player: [i32; 3], hunter: [i32; 3]) -> Flight {
    let eye = [
        player[0] as f32 + 0.5,
        player[1] as f32 + 1.0,
        player[2] as f32 + 0.5,
    ];
    let target = [
        hunter[0] as f32 + 0.5,
        hunter[1] as f32 + 0.15,
        hunter[2] as f32 + 0.5,
    ];
    let view_dx = target[0] - eye[0];
    let view_dz = target[2] - eye[2];
    let horizontal = (view_dx * view_dx + view_dz * view_dz).sqrt();
    Flight {
        eye,
        yaw: f32::atan2(view_dx, view_dz),
        pitch: f32::atan2(target[1] - eye[1], horizontal),
        fov: 0.8,
        far: 24.0,
    }
}

fn hunter_body(world: &World) -> Result<BodyLensProjection, String> {
    let hunter = world
        .organisms
        .iter()
        .find(|organism| organism.id == HUNTER_ID)
        .ok_or("the G4 crossing has no hunter")?;
    BodyLensProjection::project(
        hunter.body(),
        BodyPlacement {
            ground: [
                hunter.position[0] as f32 + 0.5,
                hunter.position[1] as f32,
                hunter.position[2] as f32 + 0.5,
            ],
            scale: 0.06,
            tint: [0.92, 0.44, 0.24],
        },
    )
    .map_err(|error| format!("could not project the G4 hunter body: {error:?}"))
}

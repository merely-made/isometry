// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! T1's judgment: a deterministic cursor sweep over the G2 terrarium,
//! picked through the Conatus tactile world and judged against an exact
//! occupancy oracle.
//!
//! All tactile judgment happens on the CPU before the first frame: the
//! whole sweep runs twice from freshly grown worlds and must agree
//! bit for bit, every ground answer must match the oracle cell for cell,
//! and the mid-sweep carve must move its column's answer exactly as the
//! oracle says. The headed frames then present the already-judged sweep:
//! the terrarium, the cursor, and each answer's marker.
//!
//! The oracle is exact because the terrarium slab is axis-aligned: rays
//! travel `-z`, so the first solid cell along a column is an integer scan
//! of `Ground::solid`, no traversal arithmetic shared with Rapier.

use mesocosm_core::places::{Ground, Places};
use mesocosm_core::snapshot;
use mesocosm_core::voxel_profile::GroundVoxelProfile;
use mesocosm_lens::{
    BrickChange, BrickDiagnostics, BrickFrameInput, BrickMap, BrickRevision, BrickTracer,
    CritterPose, Grade, TraceCamera, critter::Capsule,
};
use mesocosm_runtime::{TactileCapsule, TactilePick, TactileWorld};

pub const INITIAL_SIZE: [u32; 2] = [960, 540];
pub const WINDOW_TITLE: &str = "Mesocosm T1: terrarium picking";
/// One frame per cursor stop (eleven before the carve, eleven after); the
/// app writes its receipt on the last.
pub const MIN_FRAMES: u32 = 22;

const SEED: u64 = 4_242;
const CRITTER_KEY: u64 = 7;
/// The slab camera's vertical half-extent and depth, as G2 ratified them.
const HALF_HEIGHT: f32 = 20.0;
const ASPECT: f32 = 16.0 / 9.0;
const DEPTH: f32 = 16.0;

/// What one cursor stop found, serialized into the replay hash.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Answer {
    Ground { cell: [i32; 3], distance_bits: u32 },
    Critter { key: u64, distance_bits: u32 },
    Nothing,
}

/// One judged cursor stop.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stop {
    /// World x and y the cursor names; the ray travels `-z` from the slab
    /// face.
    pub world: [f32; 2],
    pub answer: Answer,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SweepLog {
    pub before: Vec<Stop>,
    pub carved_cell: [i32; 3],
    pub ground_revision_before: u64,
    pub ground_revision_after: u64,
    pub synced_cells: usize,
    pub after: Vec<Stop>,
}

/// The scenario the frames present: the judged log plus the render state.
pub struct Scenario {
    map: BrickMap,
    revision: BrickRevision,
    camera: TraceCamera,
    grade: Grade,
    pose: CritterPose,
    /// Slab basis for cursor projection: centre of the near face.
    origin: [f32; 3],
    pub log: SweepLog,
    pub log_hash: u64,
    /// Every stop in frame order: `before`, then `after` (the carve lands
    /// between them at frame `before.len()`).
    pub stops: Vec<Stop>,
    dirty_slots: Option<Vec<u32>>,
    body_centre: [f32; 3],
}

struct World {
    ground: Ground,
    profile: GroundVoxelProfile,
    tactile: TactileWorld,
    body_centre: [f32; 3],
}

impl World {
    fn grow() -> Result<Self, String> {
        let ground = Ground::grow(&Places::grown(SEED, 4, 64), 64);
        let profile = GroundVoxelProfile::from_ground(&ground).map_err(|e| e.to_string())?;
        let mut tactile = TactileWorld::from_profile(&profile).map_err(|e| e.to_string())?;
        let body_top = ground
            .surface(4, 18)
            .ok_or("the critter fixture column is outside Ground")? as f32;
        let body_centre = [4.5, body_top + 1.15, 18.5];
        tactile
            .set_critter(CRITTER_KEY, &[capsule(body_centre)])
            .map_err(|e| e.to_string())?;
        Ok(Self {
            ground,
            profile,
            tactile,
            body_centre,
        })
    }

    /// Pick one cursor stop and judge it against the oracle in the same
    /// breath.
    fn judged_stop(&self, world: [f32; 2]) -> Result<Stop, String> {
        let origin = [world[0], world[1], slab_near_z()];
        let hit = self
            .tactile
            .pick(origin, [0.0, 0.0, -1.0], DEPTH)
            .map_err(|e| e.to_string())?;
        let oracle = oracle_ground(&self.ground, world);
        let answer = match hit {
            None => {
                if let Some((cell, _)) = oracle {
                    return Err(format!(
                        "tactile found nothing at {world:?} but the oracle sees {cell:?}"
                    ));
                }
                Answer::Nothing
            },
            Some(hit) => match hit.pick {
                TactilePick::Ground { cell } => {
                    let (expected, toi) = oracle.ok_or_else(|| {
                        format!("tactile picked ground at {world:?} where the oracle sees none")
                    })?;
                    if cell != expected {
                        return Err(format!(
                            "tactile cell {cell:?} disagrees with oracle {expected:?} at {world:?}"
                        ));
                    }
                    if (hit.distance - toi).abs() > 1e-3 {
                        return Err(format!(
                            "tactile distance {} disagrees with oracle {toi} at {world:?}",
                            hit.distance
                        ));
                    }
                    Answer::Ground {
                        cell,
                        distance_bits: hit.distance.to_bits(),
                    }
                },
                TactilePick::Critter { key } => {
                    // The stops aimed at the critter pass through its
                    // capsule core, so any radius model agrees; the ground
                    // behind it must be farther when it exists at all.
                    if let Some((_, toi)) = oracle
                        && toi < hit.distance
                    {
                        return Err(format!(
                            "tactile picked the critter behind nearer ground at {world:?}"
                        ));
                    }
                    Answer::Critter {
                        key,
                        distance_bits: hit.distance.to_bits(),
                    }
                },
            },
        };
        Ok(Stop { world, answer })
    }

    fn sweep(&self, stops: &[[f32; 2]]) -> Result<Vec<Stop>, String> {
        stops.iter().map(|world| self.judged_stop(*world)).collect()
    }
}

/// One full judged run from a freshly grown world.
fn run_once() -> Result<(SweepLog, [f32; 3]), String> {
    let mut world = World::grow()?;
    let cursor = cursor_stops(&world.ground, world.body_centre)?;
    let before = world.sweep(&cursor)?;

    // Carve the first cell the sweep actually picked, so the delta lands on
    // a swept column rather than a hand-chosen one.
    let carved_cell = before
        .iter()
        .find_map(|stop| match stop.answer {
            Answer::Ground { cell, .. } => Some(cell),
            _ => None,
        })
        .ok_or("the sweep never touched ground")?;
    let ground_revision_before = world.ground.revision();
    if world.ground.carve(carved_cell, 0) == 0 {
        return Err(format!("the carve at {carved_cell:?} removed nothing"));
    }
    let update = world
        .profile
        .sync(ground_revision_before, &world.ground)
        .map_err(|e| e.to_string())?;
    let synced_cells = world.tactile.sync(&update).map_err(|e| e.to_string())?;
    if synced_cells == 0 {
        return Err("the carve reached the tactile world as no edit".into());
    }
    // The same stale update again must refuse without moving anything.
    if world.tactile.sync(&update).is_ok() {
        return Err("a stale tactile sync was accepted".into());
    }

    let after = world.sweep(&cursor)?;
    // The cursor that produced the carved cell must answer differently
    // after the carve, and its new answer was already judged against the
    // oracle inside the sweep.
    let held_index = before
        .iter()
        .position(|stop| matches!(stop.answer, Answer::Ground { cell, .. } if cell == carved_cell))
        .expect("the carved cell came from the sweep");
    if before[held_index].answer == after[held_index].answer {
        return Err(format!(
            "the carve at {carved_cell:?} did not move the swept answer"
        ));
    }

    Ok((
        SweepLog {
            before,
            carved_cell,
            ground_revision_before,
            ground_revision_after: world.ground.revision(),
            synced_cells,
            after,
        },
        world.body_centre,
    ))
}

impl Scenario {
    pub fn new() -> Result<Self, String> {
        // The whole judgment runs twice from freshly grown worlds; the two
        // logs must agree bit for bit before anything is presented.
        let (log, body_centre) = run_once()?;
        let (again, _) = run_once()?;
        if log != again {
            return Err("two identical sweeps produced different logs".into());
        }
        let log_hash =
            snapshot::hash_bytes(&postcard::to_allocvec(&log).map_err(|e| e.to_string())?);

        let classes = |stops: &[Stop]| {
            let ground = stops
                .iter()
                .filter(|s| matches!(s.answer, Answer::Ground { .. }))
                .count();
            let critter = stops
                .iter()
                .filter(|s| matches!(s.answer, Answer::Critter { .. }))
                .count();
            let nothing = stops
                .iter()
                .filter(|s| matches!(s.answer, Answer::Nothing))
                .count();
            (ground, critter, nothing)
        };
        let (ground, critter, nothing) = classes(&log.before);
        if ground == 0 || critter == 0 || nothing == 0 {
            return Err(format!(
                "the sweep must exercise every class: {ground} ground, {critter} critter, {nothing} nothing"
            ));
        }

        // The render state replays the same story for the frames: the same
        // grown world, carved the same way, drawn by the shared tracer.
        let mut ground_render = Ground::grow(&Places::grown(SEED, 4, 64), 64);
        let mut map = BrickMap::from_ground(&ground_render).map_err(|e| e.to_string())?;
        let body_top = body_centre[1] - 1.15;
        let camera = TraceCamera::orthographic_slab(
            [body_centre[0], body_top * 0.5 + 4.0, body_centre[2]],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            HALF_HEIGHT,
            ASPECT,
            DEPTH,
        )
        .ok_or("invalid terrarium camera")?;
        assert!(ground_render.carve(log.carved_cell, 0) > 0);
        let dirty = ground_render.drain_dirty();
        let slots = map
            .refresh(&ground_render, dirty)
            .map_err(|e| e.to_string())?;
        let revision = BrickRevision(ground_render.revision());
        let pose = pose(body_centre);
        let mut stops = log.before.clone();
        stops.extend(log.after.iter().cloned());

        Ok(Self {
            map,
            revision,
            camera,
            grade: Grade::retro(3),
            pose,
            origin: [body_centre[0], body_top * 0.5 + 4.0, slab_near_z()],
            log,
            log_hash,
            stops,
            dirty_slots: Some(slots),
            body_centre,
        })
    }

    pub fn encode(
        &mut self,
        tracer: &mut BrickTracer,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        _frame: u32,
    ) -> Result<BrickDiagnostics, String> {
        let mut input =
            BrickFrameInput::for_camera(&self.map, self.revision, self.camera, &self.grade)
                .with_pose(&self.pose);
        if let Some(slots) = self.dirty_slots.take() {
            // First frame: publish the carved slots; later frames are
            // steady.
            return tracer
                .encode(encoder, target, input.changed(BrickChange::Slots(&slots)))
                .map_err(|e| e.to_string());
        }
        input = input.changed(BrickChange::Slots(&[]));
        tracer
            .encode(encoder, target, input)
            .map_err(|e| e.to_string())
    }

    /// The stop a frame presents (frames are 1-based).
    pub fn stop_for_frame(&self, frame: u32) -> &Stop {
        let last = self.stops.len() - 1;
        &self.stops[(frame as usize - 1).min(last)]
    }

    /// A world x/y as a pixel on the frame.
    pub fn pixel(&self, world: [f32; 2], size: [u32; 2]) -> [f32; 2] {
        let ndc_x = (world[0] - self.origin[0]) / (HALF_HEIGHT * ASPECT);
        let ndc_y = (world[1] - self.origin[1]) / HALF_HEIGHT;
        [
            (ndc_x * 0.5 + 0.5) * size[0] as f32,
            (1.0 - (ndc_y * 0.5 + 0.5)) * size[1] as f32,
        ]
    }

    pub const fn body_centre(&self) -> [f32; 3] {
        self.body_centre
    }
}

/// The G2 critter capsule as the tactile adapter wants it.
fn capsule(centre: [f32; 3]) -> TactileCapsule {
    TactileCapsule {
        a: [centre[0] - 0.7, centre[1], centre[2]],
        b: [centre[0] + 0.7, centre[1], centre[2]],
        ra: 0.65,
        rb: 0.52,
    }
}

/// The same capsule as the tracer's presentation pose.
fn pose(centre: [f32; 3]) -> CritterPose {
    CritterPose::from_capsules(
        vec![Capsule {
            a: [centre[0] - 0.7, centre[1], centre[2]],
            ra: 0.65,
            b: [centre[0] + 0.7, centre[1], centre[2]],
            rb: 0.52,
        }],
        [
            [centre[0] - 0.45, centre[1] + 0.15, centre[2] - 0.35, 0.10],
            [centre[0] - 0.45, centre[1] - 0.15, centre[2] - 0.35, 0.10],
        ],
        [0.15, 0.86, 0.32],
    )
}

/// Where the slab's near face sits: rays enter here travelling `-z`.
fn slab_near_z() -> f32 {
    18.5 + DEPTH * 0.5
}

/// The deterministic cursor stops, in world x/y. Terrain stops sit at cell
/// centres on the tallest column their x can see inside the slab, critter
/// stops pass through the capsule core, and the sky stop sits above every
/// surface the slab holds.
fn cursor_stops(ground: &Ground, body_centre: [f32; 3]) -> Result<Vec<[f32; 2]>, String> {
    let mut stops = Vec::new();
    let mut tallest_anywhere = i32::MIN;
    for offset in [-24, -17, -10, -4, 6, 12, 19, 26] {
        let x = 4 + offset;
        let mut tallest = i32::MIN;
        for z in 11..=26 {
            if let Some(top) = ground.surface(x, z) {
                tallest = tallest.max(top);
            }
        }
        if tallest == i32::MIN {
            return Err(format!("terrain column x={x} is outside Ground"));
        }
        tallest_anywhere = tallest_anywhere.max(tallest);
        stops.push([x as f32 + 0.5, tallest as f32 + 0.5]);
    }
    // Above everything the slab holds.
    stops.push([body_centre[0] - 20.0, tallest_anywhere as f32 + 4.5]);
    // Through the capsule's core: on its axis at the centre and toward one
    // end, where every radius model contains the ray. Last, so the receipt
    // capture ends on the critter answer.
    stops.push([body_centre[0], body_centre[1]]);
    stops.push([body_centre[0] + 0.4, body_centre[1]]);
    Ok(stops)
}

/// The exact answer for a `-z` slab ray: the first solid cell along the
/// column, with its time of impact, or nothing.
///
/// A solid cell containing the ray origin answers at distance zero, which
/// is also what a solid raycast reports from inside.
fn oracle_ground(ground: &Ground, world: [f32; 2]) -> Option<([i32; 3], f32)> {
    let x = world[0].floor() as i32;
    let y = world[1].floor() as i32;
    let near = slab_near_z();
    let origin_cell = near.floor() as i32;
    if ground.solid([x, y, origin_cell]) {
        return Some(([x, y, origin_cell], 0.0));
    }
    let first = (near - DEPTH).floor() as i32;
    for z in (first..origin_cell).rev() {
        if !ground.solid([x, y, z]) {
            continue;
        }
        let toi = near - (z + 1) as f32;
        if toi > DEPTH {
            return None;
        }
        return Some(([x, y, z], toi));
    }
    None
}

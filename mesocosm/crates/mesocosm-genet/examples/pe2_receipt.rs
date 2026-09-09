// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PE2's headed receipt: a discovery, with its evidence and route legible.
//!
//! Rasterized through the real cambium/netrender pipeline over a headless
//! device, so the pixels are the engine's own
//! rather than a description of them. It drives ordinary recorded intents —
//! `Resume`, `Metabolize`, `Consume`, `Express` — plus the one native
//! developmental fixture PD2 already permits, because no meal in this enclosure
//! grows a consumer a plate.
//!
//! Three captures, in `Code/testing/mesocosm/`:
//! - `pe2_discovery.png` — **the non-food route.** A hundred ticks under the
//!   starved line, come through alive: what was discovered, by what route, on
//!   what evidence, and what it grants. No meal appears anywhere in it.
//! - `pe2_meal_refused.png` — **a meal that supplies evidence and unlocks
//!   nothing.** The organ, the donor and the mass are on the record, and so is
//!   the condition that could not be reached by them and why.
//! - `pe2_candidate_taken.png` — the discovery turned into a body: the same
//!   candidate lowered through the one validator, with PD2's gland reading
//!   underneath it.
//!
//! ```text
//! cargo run -p mesocosm-genet --release --example pe2_receipt
//! ```

use std::path::Path;

use mesocosm_core::discovery::{self, Condition, HUNGER_TICKS};
use mesocosm_core::{
    Attachment, Intent, Kingdom, Organism, OrganismId, Outcome, PartId, Placement, Provenance,
    STARVED_UPKEEP_TICKS, SpeciesId, Stage, Trend, VolumeRef, World, Yaw,
};
use mesocosm_genet::chrome::Chrome;
use mesocosm_genet::vitals::VitalsChrome;
use mesocosm_genet::{hud, played};

/// Panel plus margin. Taller than PD2's because the panel is: a discovery
/// carries three rows and the evidence line wraps.
const FRAME: (u32, u32) = (330, 344);
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// The same twelve-cell plate PD2's fixtures grow.
const FROND: [i32; 3] = [6, 4, 1];
const SEED: u64 = 4_242;
const FOUNDERS: u32 = 24;

fn hunger() -> Condition {
    discovery::conditions()
        .into_iter()
        .find(|found| found.name == "mesocosm:endured-hunger")
        .expect("the table holds it")
}

/// The played critter, reset to a plain bulk consumer so the fixture below is
/// the only anatomy that matters.
fn bulk_world(seed: u64, founders: u32) -> World {
    let mut world = World::new(seed, founders);
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            me,
            species,
            Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    world
}

/// Holds the body under the starved line, with a hand on it, for `ticks`.
///
/// The budget is topped back to just short of the line each tick rather than
/// left at zero, because the claim is about **surviving** the stress.
/// `Intent::Resume` is the free verb that keeps a hand on: it moves nothing and
/// resets the idle run.
fn endure(world: &mut World, ticks: u64) {
    for _ in 0..ticks {
        let me = world.controlled_id().expect("still alive");
        let upkeep = world.controlled().expect("still alive").upkeep_mg();
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == me)
            .expect("in the roster")
            .energy_mg = upkeep * (STARVED_UPKEEP_TICKS - 1);
        world.apply(Intent::Resume);
    }
}

/// Grows a frond directly — the native developmental fixture PD2's receipts
/// already use, because no ordinary meal here grows a consumer a plate.
fn frond_on(world: &mut World) -> PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root")
}

/// Somebody bulky within reach, so the meal is a plain one: bulk teaches
/// nothing, which is the whole of the second capture.
fn a_neighbour(world: &mut World) -> OrganismId {
    let here = world.position().expect("embodied");
    let id = OrganismId(9_700);
    world.organisms.push(Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            id,
            SpeciesId(6),
            Kingdom::Decomposer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            [here[0] + 1, here[1], here[2]],
            260,
        )
    });
    id
}

fn main() {
    let out_dir = played::default_out_dir();
    std::fs::create_dir_all(&out_dir).expect("out dir");

    let chrome = pollster::block_on(headless_chrome());
    let mut vitals = VitalsChrome::new(&chrome);

    let mut world = bulk_world(SEED, FOUNDERS);

    // Capture 1: the non-food route. A hundred ticks under the starved line,
    // come through alive, and the record says what that bought.
    endure(&mut world, HUNGER_TICKS + 1);
    let discovery = *world
        .discoveries()
        .first()
        .expect("coming through the horizon is the condition");
    println!(
        "capture 1: {} by {:?} on {}, tick {}",
        discovery::name_of(discovery.condition).unwrap_or("?"),
        discovery.route,
        discovery.evidence.words(),
        world.tick
    );
    capture(
        &chrome,
        &mut vitals,
        &world,
        &[],
        &out_dir.join("pe2_discovery.png"),
    );

    // Capture 2: a meal that supplies evidence and unlocks nothing. The
    // organ, the donor and the mass are on the record; so is the condition
    // that never declared the meal lane, and why it could not be reached.
    let neighbour = a_neighbour(&mut world);
    let outcome = world.apply(Intent::Metabolize {
        organism: neighbour,
        placement: Placement::Planned,
    });
    let observation = world
        .last_observation()
        .expect("a meal is an observation")
        .clone();
    assert!(
        observation.matched.is_none(),
        "the fixture wants a meal that taught nothing"
    );
    println!(
        "capture 2: {outcome:?}; evidence {}, missed {:?}",
        observation.evidence.words(),
        observation.missed
    );
    capture(
        &chrome,
        &mut vitals,
        &world,
        &[outcome],
        &out_dir.join("pe2_meal_refused.png"),
    );

    // Capture 3: the candidate taken up. The discovery proposes; the one
    // validator lowers it; PD2's reading appears underneath because the body
    // now actually expresses it.
    let part = frond_on(&mut world);
    let intent = world
        .candidate_intent(hunger().id())
        .expect("the frond is somewhere to put it");
    let outcome = world.apply(intent);
    let Outcome::Expressed { cost_mg, .. } = outcome else {
        panic!("the discovered candidate did not validate: {outcome:?}");
    };
    let gland = world.gland().expect("it has one now");
    println!(
        "capture 3: expressed on part {}, cost {cost_mg} mg, {} cells, {} mg a bite, charged {}",
        part.0, gland.cells, gland.potency_mg, gland.charged
    );
    capture(
        &chrome,
        &mut vitals,
        &world,
        &[outcome],
        &out_dir.join("pe2_candidate_taken.png"),
    );
}

async fn headless_chrome() -> Chrome {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
            compatible_surface: None,
        })
        .await
        .expect("an adapter");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("pe2 receipt"),
            ..Default::default()
        })
        .await
        .expect("a device");
    Chrome::new(
        netrender::WgpuHandles {
            instance,
            adapter,
            device,
            queue,
        },
        FORMAT,
        hud::SIDE,
    )
    .expect("the chrome device")
}

/// Refreshes the panel off `world` and writes it, alone, to `path`.
fn capture(
    chrome: &Chrome,
    vitals: &mut VitalsChrome,
    world: &World,
    outcomes: &[Outcome],
    path: &Path,
) {
    vitals.refresh(chrome, world, outcomes, world.tick, &Trend::default());

    let device = chrome.device();
    let queue = chrome.queue();
    let (width, height) = FRAME;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pe2 receipt target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("pe2 receipt"),
    });
    {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pe2 receipt clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.07,
                        b: 0.05,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    vitals.capture_composite(chrome, FORMAT, &mut encoder, &view, FRAME);

    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pe2 receipt readback"),
        size: (padded * height) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll");
    let mapped = slice.get_mapped_range().expect("mapped range");
    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    for row in 0..height {
        let start = (row * padded) as usize;
        pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();

    played::write_png(path, width, height, &pixels).expect("write png");
    println!("wrote {}", path.display());
}

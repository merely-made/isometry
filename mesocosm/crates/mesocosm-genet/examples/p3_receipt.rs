// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P3's headed receipt: a body wearing a branch that used to be somebody else.
//!
//! **Two real pipelines, one sheet.** The body is rendered by
//! `mesocosm-render` off the authoritative anatomy, the way `grow` renders one;
//! the panel is rasterized through the real cambium/netrender chrome over a
//! headless device, the way `pe2_receipt` is. Neither half
//! is a drawing of what the code would do. They are stacked into one PNG here
//! because a mid-run host frame buries a digging critter in its own burrow,
//! which shows the transfer to nobody.
//!
//! `Code/testing/mesocosm/p3_graft.png` — the recipient's body with a
//! two-part branch off a carcass on it, and underneath, the provenance in the
//! words a player reads: which parts came off which part of which line, the
//! crossing that was taken, this world's verdict on it, and what the branch is
//! doing as a result.
//!
//! ```text
//! cargo run -p mesocosm-genet --release --example p3_receipt
//! ```

use mesocosm_core::{
    AllocationProposal, Arrangement, Attachment, CellId, Crossing, Domain, Intent, Kingdom,
    Organism, OrganismId, Outcome, PartId, Process, ProposedSite, Provenance, Registry, SpeciesId,
    Stage, Trend, VolumeRef, World, Yaw,
};
use mesocosm_mesh::{Volume, VolumeMap, mesh_body};
use mesocosm_render::{Camera, Renderer, SceneItem};

use mesocosm_genet::chrome::Chrome;
use mesocosm_genet::vitals::VitalsChrome;
use mesocosm_genet::{fixture, hud, played};

/// The body render, and the panel under it.
const BODY: (u32, u32) = (760, 560);
const PANEL: (u32, u32) = (330, 300);
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

const SEED: u64 = 4_242;
const FOUNDERS: u32 = 24;
const DONOR: OrganismId = OrganismId(9_700);
const DONOR_LINE: SpeciesId = SpeciesId(5);
/// PD2's frond: twelve cells, and the only shape that admits a gland.
const FROND: [i32; 3] = [6, 4, 1];
/// The limb hanging off it.
const TIP: [i32; 3] = [7, 1, 1];

/// Content addresses for the branch's two parts.
///
/// **Their own tags, and their own registered volumes.** The host's fixture map
/// registers the world's palette and a placeholder range; borrowing one of those
/// tags for a part of a different size would draw a block where the core placed
/// an organ, which is the one thing a body receipt must not do.
const FROND_VOLUME: u8 = 40;
const TIP_VOLUME: u8 = 41;

fn main() {
    let out_dir = played::default_out_dir();
    std::fs::create_dir_all(&out_dir).expect("out dir");

    let mut world = bulk_world();
    // A cross-domain edge this world favours, so the capture shows the verdict
    // that is worth looking at: a branch that is **on** the body, weighing what
    // it weighs, and expressing nothing until an adapter is grown on it.
    {
        let mine = world.controlled().expect("embodied").species;
        let lineages = world.lineages_mut();
        lineages.found(DONOR_LINE);
        lineages.set_domain(DONOR_LINE, Domain(1));
        lineages.set_domain(mine, Domain(2));
    }
    let (frond, tip) = donor(&mut world);
    println!(
        "verdict for that tissue: {:?}",
        world.verdict_between(DONOR_LINE, world.controlled().unwrap().species)
    );

    let outcome = world.apply(Intent::Graft {
        organism: DONOR,
        part: frond,
        crossing: Crossing::Carry,
    });
    let Outcome::Grafted {
        root,
        parts,
        mass_mg,
        verdict,
        ..
    } = outcome
    else {
        panic!("the branch did not land: {outcome:?}");
    };
    let graft = world.carried_branch().expect("the played body took it");
    println!(
        "grafted {parts} parts ({mass_mg} mg) onto part {}, {verdict:?}, cost {} mg, revision {}",
        root.0, graft.cost_mg, graft.revision
    );
    let body = world.body().expect("embodied");
    for part in &graft.parts {
        println!(
            "  part {} <- {:?}",
            part.0,
            body.part(*part).unwrap().provenance.origin
        );
    }
    println!(
        "  the corpse kept neither: frond living {}, tip living {}",
        world
            .organisms
            .iter()
            .find(|o| o.id == DONOR)
            .unwrap()
            .body()
            .is_living(frond),
        world
            .organisms
            .iter()
            .find(|o| o.id == DONOR)
            .unwrap()
            .body()
            .is_living(tip)
    );

    for part in body.living() {
        println!(
            "  living part {} half {:?} pivot {:?}",
            part.id.0,
            part.half_extent,
            body.world_pivot(part.id)
        );
    }

    let body_pixels = render_body(&world);
    let chrome = pollster::block_on(headless_chrome());
    let mut vitals = VitalsChrome::new(&chrome);
    let panel_pixels = render_panel(&chrome, &mut vitals, &world, &[outcome]);

    let path = out_dir.join("p3_graft.png");
    let (width, height) = (BODY.0, BODY.1 + PANEL.1);
    let mut sheet = vec![0u8; (width * height * 4) as usize];
    // The panel's own ground, carried across the whole strip under the body, so
    // the sheet reads as one thing rather than a screenshot on a mat.
    let ground: [u8; 4] = panel_pixels[0..4].try_into().expect("a panel pixel");
    for pixel in sheet.chunks_exact_mut(4) {
        pixel.copy_from_slice(&ground);
    }
    blit(&mut sheet, width, &body_pixels, BODY, (0, 0));
    blit(
        &mut sheet,
        width,
        &panel_pixels,
        PANEL,
        ((width - PANEL.0) / 2, BODY.1),
    );
    played::write_png(&path, width, height, &sheet).expect("write png");
    println!("wrote {}", path.display());
}

/// Copies one RGBA image into the sheet at `(x, y)`.
fn blit(sheet: &mut [u8], sheet_width: u32, src: &[u8], size: (u32, u32), at: (u32, u32)) {
    let (width, height) = size;
    for row in 0..height {
        let from = (row * width * 4) as usize;
        let to = (((at.1 + row) * sheet_width + at.0) * 4) as usize;
        sheet[to..to + (width * 4) as usize]
            .copy_from_slice(&src[from..from + (width * 4) as usize]);
    }
}

/// The played critter, reset to a plain bulk consumer so the only interesting
/// anatomy in the picture is the branch that arrived.
fn bulk_world() -> World {
    let mut world = World::new(SEED, FOUNDERS);
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

/// A carcass in reach carrying a frond with a limb hanging off it, and a gland
/// arranged on the frond — so the branch is a branch, and it arrives with an
/// arrangement that a cross-domain boundary will not let it keep.
fn donor(world: &mut World) -> (PartId, PartId) {
    let here = world.position().expect("embodied");
    let mut corpse = Organism {
        stage: Stage::Carrion,
        ..Organism::founding(
            DONOR,
            DONOR_LINE,
            Kingdom::Producer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            [here[0] + 1, here[1], here[2]],
            1_200,
        )
    };
    let root = corpse.body().root;
    let frond = corpse
        .phenotype
        .attach(
            VolumeRef::from_tag(FROND_VOLUME),
            400,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches");
    let tip = corpse
        .phenotype
        .attach(
            VolumeRef::from_tag(TIP_VOLUME),
            150,
            TIP,
            Attachment {
                parent: frond,
                offset: [13, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("and a limb hangs off it");
    let capacity = corpse.phenotype.mosaic(frond).expect("a mosaic").capacity();
    let proposal = AllocationProposal {
        expect: corpse.phenotype.digest(),
        source: Arrangement::Direct,
        parts: vec![frond],
        sites: vec![ProposedSite {
            part: frond,
            process: Registry::native().of_native(Process::Secrete).reference(),
            cells: (0..capacity).map(|cell| CellId(cell as u16)).collect(),
        }],
    };
    corpse
        .phenotype
        .develop(Registry::native(), &proposal)
        .expect("valid on the donor");
    world.organisms.push(corpse);
    (frond, tip)
}

/// The recipient's anatomy, through the real body renderer.
fn render_body(world: &World) -> Vec<u8> {
    let mut volumes = fixture::volumes();
    for (tag, half, material) in [(FROND_VOLUME, FROND, 3u8), (TIP_VOLUME, TIP, 6)] {
        let size = half.map(|axis| (axis * 2).max(1) as u32);
        volumes.insert(VolumeRef::from_tag(tag), Volume::solid(size, material));
    }
    let volumes: VolumeMap = volumes;
    let renderer = Renderer::headless(BODY.0, BODY.1).expect("a GPU adapter");
    let mesh = mesh_body(world.body().expect("embodied"), &volumes).expect("every part resolves");
    let (min, max) = mesh.bounds().expect("geometry");
    let camera = Camera::framing(min, max, 1.0);
    renderer
        .render_scene(&[SceneItem::new(&mesh, [0, 0, 0])], &camera)
        .expect("render")
        .pixels
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
            label: Some("p3 receipt"),
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

/// The vitals panel, rasterized off `world`.
fn render_panel(
    chrome: &Chrome,
    vitals: &mut VitalsChrome,
    world: &World,
    outcomes: &[Outcome],
) -> Vec<u8> {
    vitals.refresh(chrome, world, outcomes, world.tick, &Trend::default());

    let device = chrome.device();
    let queue = chrome.queue();
    let (width, height) = PANEL;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("p3 receipt target"),
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
        label: Some("p3 receipt"),
    });
    {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("p3 receipt clear"),
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
    vitals.capture_composite(chrome, FORMAT, &mut encoder, &view, PANEL);

    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("p3 receipt readback"),
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
    pixels
}

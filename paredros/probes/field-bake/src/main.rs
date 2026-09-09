// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! R2: bake a Burn field into a compacted renderling draw.
//!
//! CubeCL cannot import a renderling slab allocation in the live 0.10
//! backend. Extraction therefore stays in CubeCL-owned memory, followed by
//! one device-local buffer copy into a GPU-only renderling vertex range.

use std::{mem::size_of, path::PathBuf, sync::Arc};

mod compose;

use burn_tensor::{Tensor, TensorPrimitive};
use burn_wgpu::{RuntimeOptions, Wgpu, WgpuSetup, init_device};
use compose::{Composer, MASTER_FORMAT};
use cubecl::prelude::*;
use cubecl::wgpu::WgpuRuntime;
use netrender::{Scene, TenantNeeds, WgpuHandles, boot_on};
use renderling::{
    camera::Camera,
    context::{Context, RenderTarget},
    geometry::{Geometry, Vertices},
    glam::{Mat4, Vec3, Vec4},
    light::{AnalyticalLight, DirectionalLight, Lux},
    material::Material,
    primitive::Primitive,
    stage::Stage,
    types::GpuOnlyArray,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

type B = Wgpu<f32, i32>;

const SIZE: [u32; 2] = [960, 720];
const FIELD_WIDTH: u32 = 48;
const CELL_COUNT: usize = (FIELD_WIDTH * FIELD_WIDTH) as usize;
const VERTICES_PER_CELL: usize = 6;
const VERTEX_WORDS: usize = 26;
const CUBE_DIM: u32 = 64;
const CAPTURE_AFTER: u64 = 3;
const DEFAULT_CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\r2_field_bake.png";

#[cube(launch_unchecked)]
fn count_cells(field: &Array<f32>, counts: &mut Array<u32>, threshold: f32) {
    let i = ABSOLUTE_POS;
    if i < field.len() {
        if field[i] > threshold {
            counts[i] = VERTICES_PER_CELL as u32;
        } else {
            counts[i] = 0u32;
        }
    }
}

#[cube(launch_unchecked)]
fn prefix_counts(
    counts: &Array<u32>,
    offsets: &mut Array<u32>,
    stats: &mut Array<u32>,
    capacity: u32,
) {
    if ABSOLUTE_POS == 0 {
        let mut total = 0u32;
        let mut i = 0usize;
        while i < counts.len() {
            offsets[i] = total;
            total += counts[i];
            i += 1usize;
        }
        stats[0] = total;
        if total > capacity {
            stats[1] = 1u32;
        } else {
            stats[1] = 0u32;
        }
    }
}

#[cube]
fn store_float(output: &mut Array<u32>, index: usize, value: f32) {
    output[index] = u32::reinterpret(value);
}

#[cube(launch_unchecked)]
fn scatter_vertices(
    field: &Array<f32>,
    counts: &Array<u32>,
    offsets: &Array<u32>,
    output: &mut Array<u32>,
    width: u32,
    capacity: u32,
) {
    let invocation = ABSOLUTE_POS;
    let cell = invocation / VERTICES_PER_CELL;
    let corner = invocation % VERTICES_PER_CELL;
    if cell < field.len() && counts[cell] != 0u32 {
        let out_vertex = offsets[cell] + corner as u32;
        if out_vertex < capacity {
            let x = (cell % width as usize) as f32;
            let z = (cell / width as usize) as f32;
            let mut px = x;
            let mut pz = z;
            if corner == 1usize || corner == 2usize || corner == 5usize {
                px += 1.0f32;
            }
            if corner == 1usize || corner == 4usize || corner == 5usize {
                pz += 1.0f32;
            }
            px -= width as f32 * 0.5f32;
            pz -= width as f32 * 0.5f32;

            let height = field[cell] * 2.5f32;
            let tint = 0.45f32 + field[cell] * 0.18f32;
            let base = out_vertex as usize * VERTEX_WORDS;

            store_float(output, base, px);
            store_float(output, base + 1usize, height);
            store_float(output, base + 2usize, pz);
            store_float(output, base + 3usize, 0.12f32 + tint * 0.20f32);
            store_float(output, base + 4usize, 0.42f32 + tint * 0.45f32);
            store_float(output, base + 5usize, 0.18f32 + tint * 0.12f32);
            store_float(output, base + 6usize, 1.0f32);
            store_float(output, base + 7usize, 0.0f32);
            store_float(output, base + 8usize, 0.0f32);
            store_float(output, base + 9usize, 0.0f32);
            store_float(output, base + 10usize, 0.0f32);
            store_float(output, base + 11usize, 0.0f32);
            store_float(output, base + 12usize, 1.0f32);
            store_float(output, base + 13usize, 0.0f32);
            store_float(output, base + 14usize, 1.0f32);
            store_float(output, base + 15usize, 0.0f32);
            store_float(output, base + 16usize, 0.0f32);
            store_float(output, base + 17usize, 1.0f32);
            let mut word = 18usize;
            while word < VERTEX_WORDS {
                output[base + word] = 0u32;
                word += 1usize;
            }
        }
    }
}

struct Extracted {
    client: ComputeClient<WgpuRuntime>,
    output: cubecl::server::Handle,
    count: usize,
    capacity: usize,
    expected: usize,
}

fn launch_extract(
    client: &ComputeClient<WgpuRuntime>,
    field: cubecl::server::Handle,
    capacity: usize,
) -> (cubecl::server::Handle, cubecl::server::Handle) {
    let counts = client.empty(CELL_COUNT * size_of::<u32>());
    let offsets = client.empty(CELL_COUNT * size_of::<u32>());
    let output = client.empty(capacity * VERTEX_WORDS * size_of::<u32>());
    let stats = client.create_from_slice(bytemuck::cast_slice(&[0u32; 2]));
    let cell_cubes = CELL_COUNT.div_ceil(CUBE_DIM as usize) as u32;
    let vertex_invocations = CELL_COUNT * VERTICES_PER_CELL;
    let vertex_cubes = vertex_invocations.div_ceil(CUBE_DIM as usize) as u32;

    unsafe {
        count_cells::launch_unchecked::<WgpuRuntime>(
            client,
            CubeCount::Static(cell_cubes, 1, 1),
            CubeDim::new_1d(CUBE_DIM),
            ArrayArg::from_raw_parts(field.clone(), CELL_COUNT),
            ArrayArg::from_raw_parts(counts.clone(), CELL_COUNT),
            0.0f32,
        );
        prefix_counts::launch_unchecked::<WgpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(counts.clone(), CELL_COUNT),
            ArrayArg::from_raw_parts(offsets.clone(), CELL_COUNT),
            ArrayArg::from_raw_parts(stats.clone(), 2),
            capacity as u32,
        );
        scatter_vertices::launch_unchecked::<WgpuRuntime>(
            client,
            CubeCount::Static(vertex_cubes, 1, 1),
            CubeDim::new_1d(CUBE_DIM),
            ArrayArg::from_raw_parts(field, CELL_COUNT),
            ArrayArg::from_raw_parts(counts, CELL_COUNT),
            ArrayArg::from_raw_parts(offsets, CELL_COUNT),
            ArrayArg::from_raw_parts(output.clone(), capacity * VERTEX_WORDS),
            FIELD_WIDTH,
            capacity as u32,
        );
    }
    (output, stats)
}

fn read_stats(client: &ComputeClient<WgpuRuntime>, stats: cubecl::server::Handle) -> [u32; 2] {
    let bytes = client.read_one(stats).expect("CubeCL stats readback");
    [
        u32::from_le_bytes(bytes[0..4].try_into().expect("count bytes")),
        u32::from_le_bytes(bytes[4..8].try_into().expect("overflow bytes")),
    ]
}

fn extract(setup: WgpuSetup) -> Extracted {
    let device = init_device(setup, RuntimeOptions::default());
    let client = WgpuRuntime::client(&device);
    let seeds: Vec<f32> = (0..CELL_COUNT)
        .map(|cell| {
            let x = (cell % FIELD_WIDTH as usize) as f32;
            let z = (cell / FIELD_WIDTH as usize) as f32;
            x * 0.31 + (z * 0.47).cos() * 1.8
        })
        .collect();
    let expected = seeds.iter().filter(|seed| seed.sin() > 0.0).count() * VERTICES_PER_CELL;
    let field = Tensor::<B, 1>::from_floats(seeds.as_slice(), &device).sin();
    let TensorPrimitive::Float(field_primitive) = field.into_primitive() else {
        unreachable!("f32 field primitive")
    };
    assert!(std::ptr::eq(
        client.properties(),
        field_primitive.client.properties()
    ));
    let field_handle = field_primitive.handle;

    let overflow_capacity = expected.saturating_sub(1).max(1);
    let (_overflow_output, overflow_stats) =
        launch_extract(&client, field_handle.clone(), overflow_capacity);
    let overflow = read_stats(&client, overflow_stats);
    assert_eq!(expected as u32, overflow[0]);
    assert_eq!(1, overflow[1], "GPU overflow flag was not raised");

    let capacity = CELL_COUNT * VERTICES_PER_CELL;
    let (output, stats) = launch_extract(&client, field_handle, capacity);
    let final_stats = read_stats(&client, stats);
    assert_eq!(expected as u32, final_stats[0]);
    assert_eq!(0, final_stats[1]);
    println!(
        "extraction receipt: Burn field -> CubeCL count/prefix/scatter, {} compacted vertices (expected {}), overflow run refused capacity {}",
        final_stats[0], expected, overflow_capacity
    );

    Extracted {
        client,
        output,
        count: final_stats[0] as usize,
        capacity,
        expected,
    }
}

struct BakedScene {
    view: wgpu::TextureView,
    ctx: Context,
    stage: Stage,
    _camera: Camera,
    _sun: AnalyticalLight<DirectionalLight>,
    _material: Material,
    _primitive: Primitive,
    _vertices: Vertices<GpuOnlyArray>,
    _growth_ballast: Vertices<GpuOnlyArray>,
}

impl BakedScene {
    fn new(handles: &WgpuHandles, extracted: Extracted) -> Self {
        let target = handles.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("field bake target"),
            size: wgpu::Extent3d {
                width: SIZE[0],
                height: SIZE[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let ctx = Context::new(
            RenderTarget::from(target),
            handles.adapter.clone(),
            handles.device.clone(),
            handles.queue.clone(),
        );
        let stage = ctx
            .new_stage()
            .with_background_color([0.018, 0.025, 0.035, 1.0]);
        let camera = stage.new_camera();
        let eye = Vec3::new(42.0, 44.0, 52.0);
        camera.set_projection_and_view(
            Mat4::perspective_rh(
                std::f32::consts::FRAC_PI_4,
                SIZE[0] as f32 / SIZE[1] as f32,
                0.1,
                180.0,
            ),
            Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y),
        );
        stage.use_camera(&camera);
        let sun = stage
            .new_directional_light()
            .with_direction(Vec3::new(-0.5, -1.0, -0.3).normalize());
        sun.set_intensity(Lux::OUTDOOR_DIRECT_SUNLIGHT_HIGH);
        let material = stage
            .new_material()
            .with_albedo_factor(Vec4::ONE)
            .with_has_lighting(false);

        let geometry: &Geometry = stage.as_ref();
        let _ = geometry.commit();
        assert!(!geometry.slab_allocator().has_queued_updates());
        let vertices = stage.new_gpu_vertices(extracted.capacity);
        assert_eq!(extracted.capacity, vertices.capacity());
        assert_eq!(
            extracted.capacity * VERTEX_WORDS,
            vertices.array().into_u32_array().len(),
            "vertex capacity is not one contiguous range"
        );
        assert!(
            !geometry.slab_allocator().has_queued_updates(),
            "GPU vertex capacity staged CPU values"
        );

        let primitive = stage.new_primitive().with_material(&material);
        let overflow = primitive
            .set_vertices_with_count(&vertices, extracted.capacity + 1)
            .err()
            .expect("renderling accepted an oversized draw count");
        assert_eq!(extracted.capacity, overflow.capacity);
        primitive
            .set_vertices_with_count(&vertices, extracted.count)
            .expect("bounded compacted draw count");

        let initial = geometry.commit();
        let growth_ballast = stage.new_gpu_vertices(extracted.capacity * 8);
        let grown = geometry.commit();
        assert!(
            initial.is_invalid(),
            "slab growth did not invalidate old lease"
        );
        assert!(grown.is_new_this_commit());
        assert!(
            !wgpu::Buffer::eq(&initial, &grown),
            "slab growth retained the old buffer identity"
        );
        let attached = geometry.commit();
        assert!(
            wgpu::Buffer::eq(&grown, &attached),
            "publication buffer is not renderling's attached slab"
        );

        let managed = extracted
            .client
            .get_resource(extracted.output.clone())
            .expect("CubeCL output resource");
        let source = managed.resource();
        assert!(
            !wgpu::Buffer::eq(&source.buffer, &attached),
            "CubeCL unexpectedly imported the renderling slab"
        );
        let range = vertices.array().into_u32_array();
        let destination_offset = range.starting_index() as u64 * size_of::<u32>() as u64;
        let copy_size = extracted.count as u64 * VERTEX_WORDS as u64 * size_of::<u32>() as u64;
        assert!(copy_size <= source.size);
        let mut encoder = handles
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("CubeCL to renderling vertex publication"),
            });
        encoder.copy_buffer_to_buffer(
            &source.buffer,
            source.offset,
            &attached,
            destination_offset,
            copy_size,
        );
        let submission = handles.queue.submit([encoder.finish()]);
        handles
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .expect("vertex publication completion");

        let full_bytes = extracted.count * VERTEX_WORDS * size_of::<u32>();
        let compact_bytes = extracted.count * 10 * size_of::<u32>();
        println!(
            "allocation receipt: buffer identity attached after growth; {} vertices, {}-word contiguous capacity, zero CPU vertex staging",
            extracted.count,
            extracted.capacity * VERTEX_WORDS
        );
        println!(
            "publication receipt: CubeCL-owned source -> renderling slab by one device-local copy; two allocators, {} bytes, two-word CPU count readback",
            copy_size
        );
        println!(
            "ABI receipt: full Vertex {} bytes versus 10-word procedural {} bytes for this bake ({:.1}x); keep full Vertex for bounded bake output, not as a universal procedural ABI",
            full_bytes,
            compact_bytes,
            full_bytes as f32 / compact_bytes as f32
        );
        assert_eq!(extracted.expected, extracted.count);

        Self {
            view,
            ctx,
            stage,
            _camera: camera,
            _sun: sun,
            _material: material,
            _primitive: primitive,
            _vertices: vertices,
            _growth_ballast: growth_ballast,
        }
    }

    fn draw(&self) {
        let frame = self.ctx.get_next_frame().expect("field bake frame");
        self.stage.render(&frame.view());
        frame.present();
    }
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    handles: WgpuHandles,
    baked: BakedScene,
    composer: Composer,
}

struct App {
    instance: wgpu::Instance,
    capture_mode: bool,
    capture_path: PathBuf,
    live: Option<Live>,
    frames: u64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Paredros R2: GPU field bake")
                        .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1])),
                )
                .expect("field bake window"),
        );
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("field bake surface");
        let handles = boot_on(
            self.instance.clone(),
            Some(&surface),
            &TenantNeeds {
                optional_features: wgpu::Features::INDIRECT_FIRST_INSTANCE
                    | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
                    | wgpu::Features::VERTEX_WRITABLE_STORAGE
                    | wgpu::Features::CLEAR_TEXTURE,
                label: Some("field bake"),
                ..Default::default()
            },
        )
        .expect("one device for Burn, CubeCL, renderling, and presentation");
        println!("adapter: {}", handles.adapter.get_info().name);
        let setup = WgpuSetup {
            instance: self.instance.clone(),
            adapter: handles.adapter.clone(),
            device: handles.device.clone(),
            queue: handles.queue.clone(),
            backend: handles.adapter.get_info().backend,
        };
        let extracted = extract(setup);
        let baked = BakedScene::new(&handles, extracted);
        let composer = Composer::new(handles.clone(), SIZE);
        let capabilities = surface.get_capabilities(&handles.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .unwrap_or(capabilities.formats[0]);
        let mut live = Live {
            window,
            surface,
            format,
            handles,
            baked,
            composer,
        };
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live);
                }
            }
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }
}

impl App {
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        live.baked.draw();
        let chrome = chrome();
        let master = live.composer.compose(&chrome, &live.baked.view);
        let size = live.window.inner_size();
        use wgpu::CurrentSurfaceTexture as Acquired;
        match live.surface.get_current_texture() {
            Acquired::Success(frame) | Acquired::Suboptimal(frame) => {
                let target = frame.texture.create_view(&Default::default());
                live.composer.present(
                    &master,
                    &target,
                    live.format,
                    [size.width.max(1), size.height.max(1)],
                );
                live.window.pre_present_notify();
                live.handles.queue.present(frame);
            }
            Acquired::Outdated | Acquired::Lost => configure(live),
            Acquired::Timeout | Acquired::Occluded => {}
            Acquired::Validation => panic!("surface acquisition validation failure"),
        }
        self.frames += 1;

        if self.capture_mode && self.frames >= CAPTURE_AFTER {
            if let Some(parent) = self.capture_path.parent() {
                std::fs::create_dir_all(parent).expect("capture directory");
            }
            let capture = live.composer.capture(&master);
            capture
                .write_png(&self.capture_path)
                .expect("headed capture PNG");
            let (colours, lit) = image_receipt(&capture.pixels);
            println!(
                "headed receipt: {} colours, {:.1}% non-background -> {}",
                colours,
                lit * 100.0,
                self.capture_path.display()
            );
            assert!(colours > 32, "capture is visually flat");
            assert!((0.01..0.80).contains(&lit), "capture coverage {lit:.3}");
            live.handles
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("final device poll");
            event_loop.exit();
            return;
        }
        live.window.request_redraw();
    }
}

fn chrome() -> Scene {
    let mut scene = Scene::new(SIZE[0], SIZE[1]);
    scene.push_rect(0.0, 0.0, SIZE[0] as f32, 34.0, [0.03, 0.05, 0.07, 0.90]);
    scene.push_rect(14.0, 13.0, 260.0, 21.0, [0.35, 0.78, 0.52, 0.95]);
    scene
}

fn image_receipt(pixels: &[u8]) -> (usize, f32) {
    let row = SIZE[0] as usize * 4;
    let region = &pixels[row * 40..];
    let background = [region[0], region[1], region[2]];
    let mut colours = std::collections::HashSet::new();
    let mut lit = 0usize;
    for pixel in region.chunks_exact(4) {
        colours.insert([pixel[0], pixel[1], pixel[2]]);
        if pixel[..3] != background {
            lit += 1;
        }
    }
    (colours.len(), lit as f32 / (region.len() / 4) as f32)
}

fn configure(live: &mut Live) {
    let size = live.window.inner_size();
    live.surface.configure(
        &live.handles.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: live.format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
    let _ = MASTER_FORMAT;
}

fn main() {
    let capture_mode = std::env::var("FIELD_BAKE_CAPTURE").is_ok_and(|value| value != "0");
    let capture_path = std::env::var_os("FIELD_BAKE_CAPTURE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CAPTURE));
    let event_loop = EventLoop::new().expect("field bake event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        instance: wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        ),
        capture_mode,
        capture_path,
        live: None,
        frames: 0,
    };
    event_loop.run_app(&mut app).expect("field bake run");
}

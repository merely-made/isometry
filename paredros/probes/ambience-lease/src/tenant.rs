// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The renderling half: one instanced mote mesh, N transforms, a camera
//! that looks at the cloud from an angle so depth reads, and a sun.
//!
//! renderling addresses geometry through a craballoc slab rather than
//! through buffers a caller binds, so this is also where the lease's
//! consumer-side truth lives: which buffer the transforms are in, where
//! they begin, and whether the allocator recreated the buffer this
//! commit.

use netrender::WgpuHandles;
use renderling::camera::Camera;
use renderling::context::{Context, RenderTarget};
use renderling::geometry::{Geometry, Vertex, Vertices};
use renderling::glam::{Mat4, Vec3, Vec4};
use renderling::light::{AnalyticalLight, DirectionalLight, Lux};
use renderling::primitive::Primitive;
use renderling::stage::Stage;
use renderling::transform::Transform;

/// A `TransformDescriptor` is translation (3) + rotation (4) + scale
/// (3): ten u32 words in the slab, translation first. The publish
/// kernel writes those first three and never touches the rest.
pub const TRANSFORM_WORDS: u32 = 10;

pub struct Tenant {
    pub view: wgpu::TextureView,
    ctx: Context,
    stage: Stage,
    _camera: Camera,
    _sun: AnalyticalLight<DirectionalLight>,
    _motes: Vec<Primitive>,
    transforms: Vec<Transform>,
}

impl Tenant {
    pub fn new(handles: &WgpuHandles, size: [u32; 2], count: u32, extent: f32) -> Self {
        let target = handles.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ambience tenant target"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
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
            .with_background_color([0.02, 0.02, 0.04, 1.0]);

        let camera = stage.new_camera();
        let eye = Vec3::new(extent * 2.1, extent * 1.4, extent * 2.1);
        let projection = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_4,
            size[0] as f32 / size[1] as f32,
            1.0,
            extent * 12.0,
        );
        camera.set_projection_and_view(projection, Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y));
        stage.use_camera(&camera);

        // A sun, so the motes are shaded by orientation rather than
        // flat-filled. renderling shades through PBR and its intensities
        // are physical: overcast (1,000 lux) renders these nearly black,
        // so this is direct sunlight.
        let sun = stage
            .new_directional_light()
            .with_direction(Vec3::new(-0.6, -1.0, -0.35).normalize());
        sun.set_color(Vec4::new(1.0, 0.96, 0.90, 1.0));
        sun.set_intensity(Lux::OUTDOOR_DIRECT_SUNLIGHT_HIGH);

        let vertices = stage.new_vertices(mote_mesh());
        let material = stage
            .new_material()
            .with_albedo_factor(Vec4::new(0.62, 0.78, 1.0, 1.0));
        material.set_metallic_factor(0.0);
        material.set_roughness_factor(0.55);

        // Transforms first, in one unbroken pass, *then* the primitives.
        // Interleaving the two allocations puts a primitive descriptor
        // between every pair of transforms and the slab stride stops
        // being the transform's own size, which the adapter kernel
        // indexes by. The consumer's allocation order is part of the
        // contract, and this is the line that makes it so.
        let transforms: Vec<Transform> = (0..count).map(|_| stage.new_transform()).collect();
        let motes: Vec<Primitive> = transforms
            .iter()
            .map(|transform| {
                stage
                    .new_primitive()
                    .with_vertices(vertices.clone())
                    .with_material(&material)
                    .with_transform(transform)
            })
            .collect();

        Self {
            view,
            ctx,
            stage,
            _camera: camera,
            _sun: sun,
            _motes: motes,
            transforms,
        }
    }

    /// Commit the stage and report whether the slab buffer was
    /// *recreated* this commit. This is the lease's epoch, and the
    /// consumer supplies it: craballoc's `is_new_this_commit` exists
    /// precisely so downstream resources (bind groups) can be rebuilt.
    /// A producer that skips this writes into a dead buffer while the
    /// consumer reads a live one, and the picture silently freezes.
    pub fn commit(&self) -> (wgpu::Buffer, bool) {
        let geometry: &Geometry = self.stage.as_ref();
        let slab = geometry.commit();
        let grew = slab.is_new_this_commit();
        ((*slab).clone(), grew)
    }

    /// Force the allocator past its spare capacity, so it *recreates*
    /// the buffer rather than filling slack: the event the epoch exists
    /// for, provoked deliberately rather than waited for. One large
    /// allocation, because the first attempt at this (40,000 more
    /// transforms) fit in the slack and proved nothing while costing a
    /// 600 ms frame.
    pub fn grow(&self, vertices: usize) -> Vertices {
        self.stage.new_vertices(vec![Vertex::default(); vertices])
    }

    /// Where renderling actually put the transform descriptors, asked of
    /// renderling rather than assumed. Commit first: the slab buffer
    /// does not exist until the stage has staged its writes.
    pub fn slab_and_transform_base(&self) -> (wgpu::Buffer, u32) {
        let geometry: &Geometry = self.stage.as_ref();
        let slab = geometry.commit();
        let base = self
            .transforms
            .first()
            .expect("at least one mote")
            .id()
            .inner();
        // The allocator is expected to have laid them out contiguously;
        // the adapter kernel indexes by stride from the first, so a gap
        // would corrupt the publish. Check it rather than trust it.
        for (i, transform) in self.transforms.iter().enumerate() {
            assert_eq!(
                transform.id().inner(),
                base + i as u32 * TRANSFORM_WORDS,
                "transforms are not contiguous: the adapter's stride assumption fails"
            );
        }
        ((*slab).clone(), base)
    }

    pub fn draw(&self) {
        let frame = self.ctx.get_next_frame().expect("tenant frame");
        self.stage.render(&frame.view());
        frame.present();
    }
}

/// A small octahedron: cheap, closed, and shaded per face, so lighting
/// varies with orientation and the picture reads as solid.
fn mote_mesh() -> Vec<Vertex> {
    const R: f32 = 1.6;
    let top = Vec3::new(0.0, R, 0.0);
    let bottom = Vec3::new(0.0, -R, 0.0);
    let ring = [
        Vec3::new(R, 0.0, 0.0),
        Vec3::new(0.0, 0.0, R),
        Vec3::new(-R, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -R),
    ];
    let mut out = Vec::with_capacity(24);
    for i in 0..4 {
        let a = ring[i];
        let b = ring[(i + 1) % 4];
        for (apex, first, second) in [(top, a, b), (bottom, b, a)] {
            let normal = (first - apex).cross(second - apex).normalize_or_zero();
            let tint = 0.55 + 0.45 * normal.y.abs();
            for point in [apex, first, second] {
                out.push(
                    Vertex::default()
                        .with_position(point.to_array())
                        .with_normal(normal.to_array())
                        .with_color([0.60 * tint, 0.78 * tint, 1.0 * tint, 1.0]),
                );
            }
        }
    }
    out
}

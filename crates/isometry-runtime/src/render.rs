//! Product-owned rendering of Isometry's resident Conatus body positions.
//!
//! The tenant reads Quint's exact suballocation directly and renders a locked
//! isometric marker layer into its own same-device texture. Isometry owns the
//! projection and appearance. Netrender only receives the resulting texture
//! view for explicit composition.

use std::{error::Error, fmt, num::NonZeroU64};

use bytemuck::{Pod, Zeroable};
use quint::resident::{ChunkStamp, PlaneClass, PlaneElementType, RawKernelView};

const POSITION_WIDTH: usize = 4;
const POSITION_BYTES: u64 = 16;
const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const BODY_MARKER_WGSL: &str = include_str!("body_marker.wgsl");

/// Product settings for the locked isometric body-marker lens.
///
/// Each basis vector maps one Conatus world unit to target pixels. The
/// defaults form a conventional 2:1 isometric diamond, but the product host
/// may replace every projection and appearance value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IsometryBodyRenderConfig {
    pub target_size: [u32; 2],
    pub origin: [f32; 2],
    pub basis: [[f32; 2]; 3],
    pub marker_size: [f32; 2],
    pub color: [f32; 4],
}

impl Default for IsometryBodyRenderConfig {
    fn default() -> Self {
        Self {
            target_size: [256, 256],
            origin: [128.0, 32.0],
            basis: [[16.0, 8.0], [0.0, -16.0], [-16.0, 8.0]],
            marker_size: [10.0, 14.0],
            color: [0.25, 0.8, 1.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BodyRenderParams {
    target_origin: [f32; 4],
    basis_x_y: [f32; 4],
    basis_z_marker: [f32; 4],
    color: [f32; 4],
}

struct BoundPositions {
    view: RawKernelView,
    bind_group: wgpu::BindGroup,
}

/// Isometry's fixed-isometric body renderer tenant.
///
/// It owns its output texture and keeps the current Quint view alive. A fresh
/// view with the same stamp performs no render submission. An advancing stamp
/// reuses the bind group while the allocation is stable and rebuilds it only
/// after an explicit resident allocation replacement.
///
/// The supplied resident view must come from a `ResidentClient` registered on
/// this tenant's device and queue. wgpu exposes no portable device identity on
/// a buffer, so the product host enforces that construction invariant; wgpu
/// validation rejects a foreign-device view.
pub struct IsometryBodyTenant {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: IsometryBodyRenderConfig,
    capacity: usize,
    instance_count: u32,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    params: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    bound: Option<BoundPositions>,
}

impl IsometryBodyTenant {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: IsometryBodyRenderConfig,
        capacity: usize,
    ) -> Result<Self, IsometryBodyRenderError> {
        validate_config(config, capacity)?;
        let limits = device.limits();
        let required_bytes = (capacity as u64)
            .checked_mul(POSITION_BYTES)
            .ok_or(IsometryBodyRenderError::InvalidCapacity(capacity))?;
        if required_bytes > limits.max_storage_buffer_binding_size {
            return Err(IsometryBodyRenderError::CapacityExceedsDevice {
                required: required_bytes,
                limit: limits.max_storage_buffer_binding_size,
            });
        }
        if config
            .target_size
            .iter()
            .any(|dimension| *dimension > limits.max_texture_dimension_2d)
        {
            return Err(IsometryBodyRenderError::TargetExceedsDevice {
                requested: config.target_size,
                limit: limits.max_texture_dimension_2d,
            });
        }
        let instance_count = u32::try_from(capacity)
            .map_err(|_| IsometryBodyRenderError::InvalidCapacity(capacity))?;
        let binding_size = NonZeroU64::new(required_bytes)
            .ok_or(IsometryBodyRenderError::InvalidCapacity(capacity))?;

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Isometry resident body tenant target"),
            size: wgpu::Extent3d {
                width: config.target_size[0],
                height: config.target_size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let params_value = BodyRenderParams {
            target_origin: [
                config.target_size[0] as f32,
                config.target_size[1] as f32,
                config.origin[0],
                config.origin[1],
            ],
            basis_x_y: [
                config.basis[0][0],
                config.basis[0][1],
                config.basis[1][0],
                config.basis[1][1],
            ],
            basis_z_marker: [
                config.basis[2][0],
                config.basis[2][1],
                config.marker_size[0],
                config.marker_size[1],
            ],
            color: config.color,
        };
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Isometry resident body tenant params"),
            size: size_of::<BodyRenderParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });
        {
            let mut mapped = params
                .slice(..)
                .get_mapped_range_mut()
                .expect("new Isometry tenant parameter buffer is mapped");
            mapped.copy_from_slice(bytemuck::bytes_of(&params_value));
        }
        params.unmap();

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Isometry resident body tenant layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(binding_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(size_of::<BodyRenderParams>() as u64),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Isometry resident body tenant pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Isometry resident body tenant shader"),
            source: wgpu::ShaderSource::Wgsl(BODY_MARKER_WGSL.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Isometry resident body tenant pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TARGET_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            config,
            capacity,
            instance_count,
            target,
            target_view,
            params,
            layout,
            pipeline,
            bound: None,
        })
    }

    pub const fn config(&self) -> IsometryBodyRenderConfig {
        self.config
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn target_texture(&self) -> &wgpu::Texture {
        &self.target
    }

    pub fn frame(&self) -> Option<IsometryBodyRenderFrame<'_>> {
        self.bound.as_ref().map(|bound| IsometryBodyRenderFrame {
            view: &self.target_view,
            stamp: bound.view.stamp(),
            target_size: self.config.target_size,
        })
    }

    /// Render only when the resident publication stamp advances.
    ///
    /// Queue writes performed by `IsometryResidentBodies::apply_frame` are
    /// ordered before this submission on the same host queue. The target and
    /// current binding remain unchanged when validation refuses a view. The
    /// view must originate on the tenant's stored host device; see the type's
    /// construction invariant.
    pub fn render_if_changed(
        &mut self,
        positions: RawKernelView,
    ) -> Result<Option<IsometryBodyRenderUpdate>, IsometryBodyRenderError> {
        validate_view(&positions, self.capacity)?;
        let offered = positions.stamp();
        let allocation_changed = match &self.bound {
            Some(bound) => {
                let current = bound.view.stamp();
                if offered == current {
                    if positions.allocation() != bound.view.allocation() {
                        return Err(IsometryBodyRenderError::AllocationChangedWithoutStamp {
                            stamp: offered,
                        });
                    }
                    return Ok(None);
                }
                if offered.revision <= current.revision
                    || offered.valid_read_epoch.get() <= current.valid_read_epoch.get()
                {
                    return Err(IsometryBodyRenderError::RegressingStamp { current, offered });
                }
                positions.allocation() != bound.view.allocation()
            }
            None => false,
        };

        let replacement = if self.bound.is_none() || allocation_changed {
            Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Isometry resident body tenant binding"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(positions.binding()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.params.as_entire_binding(),
                    },
                ],
            }))
        } else {
            None
        };
        let bind_group = replacement
            .as_ref()
            .or_else(|| self.bound.as_ref().map(|bound| &bound.bind_group))
            .expect("the first publication always creates a tenant binding");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Isometry resident body tenant encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Isometry resident body tenant pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..6, 0..self.instance_count);
        }
        self.queue.submit([encoder.finish()]);

        if let Some(bind_group) = replacement {
            self.bound = Some(BoundPositions {
                view: positions,
                bind_group,
            });
        } else {
            self.bound
                .as_mut()
                .expect("a reused binding has resident state")
                .view = positions;
        }

        Ok(Some(IsometryBodyRenderUpdate {
            stamp: offered,
            rebound: allocation_changed,
        }))
    }
}

/// One submitted body-tenant update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IsometryBodyRenderUpdate {
    pub stamp: ChunkStamp,
    pub rebound: bool,
}

/// The same-device texture Netrender may explicitly compose.
#[derive(Clone, Copy)]
pub struct IsometryBodyRenderFrame<'a> {
    pub view: &'a wgpu::TextureView,
    pub stamp: ChunkStamp,
    pub target_size: [u32; 2],
}

#[cfg(feature = "netrender-tenant")]
impl IsometryBodyRenderFrame<'_> {
    pub fn netrender_composite(
        &self,
        placement: netrender::ExternalTexturePlacement,
        scene_op_boundary: usize,
    ) -> netrender::ExternalTextureComposite<'_> {
        netrender::ExternalTextureComposite::new(
            self.view,
            placement.with_alpha(netrender::SourceAlpha::Premultiplied),
        )
        .with_scene_op_boundary(scene_op_boundary)
    }
}

fn validate_config(
    config: IsometryBodyRenderConfig,
    capacity: usize,
) -> Result<(), IsometryBodyRenderError> {
    if capacity == 0 || u32::try_from(capacity).is_err() {
        return Err(IsometryBodyRenderError::InvalidCapacity(capacity));
    }
    if config.target_size.contains(&0) {
        return Err(IsometryBodyRenderError::InvalidConfig(
            "body render target dimensions must be nonzero",
        ));
    }
    if config
        .origin
        .iter()
        .chain(config.basis.iter().flatten())
        .any(|value| !value.is_finite())
    {
        return Err(IsometryBodyRenderError::InvalidConfig(
            "body render projection values must be finite",
        ));
    }
    if config
        .marker_size
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(IsometryBodyRenderError::InvalidConfig(
            "body marker dimensions must be finite and positive",
        ));
    }
    if config
        .color
        .iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(IsometryBodyRenderError::InvalidConfig(
            "body marker color components must be finite and between zero and one",
        ));
    }
    Ok(())
}

fn validate_view(view: &RawKernelView, capacity: usize) -> Result<(), IsometryBodyRenderError> {
    let layout = view.layout();
    let expected_shape = [capacity, POSITION_WIDTH, 1];
    if layout.shape != expected_shape {
        return Err(IsometryBodyRenderError::UnexpectedShape {
            expected: expected_shape,
            actual: layout.shape,
        });
    }
    if layout.element_type != PlaneElementType::F32 {
        return Err(IsometryBodyRenderError::UnexpectedElementType(
            layout.element_type,
        ));
    }
    if view.class() != PlaneClass::Derived {
        return Err(IsometryBodyRenderError::UnexpectedPlaneClass(view.class()));
    }
    let lease = view.lease();
    if !lease.fits() {
        return Err(IsometryBodyRenderError::LeaseTooSmall {
            described: lease.byte_len(),
            available: lease.size,
        });
    }
    Ok(())
}

#[derive(Debug)]
pub enum IsometryBodyRenderError {
    InvalidCapacity(usize),
    InvalidConfig(&'static str),
    CapacityExceedsDevice {
        required: u64,
        limit: u64,
    },
    TargetExceedsDevice {
        requested: [u32; 2],
        limit: u32,
    },
    UnexpectedShape {
        expected: [usize; 3],
        actual: [usize; 3],
    },
    UnexpectedElementType(PlaneElementType),
    UnexpectedPlaneClass(PlaneClass),
    LeaseTooSmall {
        described: u64,
        available: u64,
    },
    RegressingStamp {
        current: ChunkStamp,
        offered: ChunkStamp,
    },
    AllocationChangedWithoutStamp {
        stamp: ChunkStamp,
    },
}

impl fmt::Display for IsometryBodyRenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCapacity(capacity) => {
                write!(formatter, "body render capacity {capacity} is invalid")
            }
            Self::InvalidConfig(message) => formatter.write_str(message),
            Self::CapacityExceedsDevice { required, limit } => write!(
                formatter,
                "body render positions require a {required}-byte storage binding; device limit is {limit}"
            ),
            Self::TargetExceedsDevice { requested, limit } => write!(
                formatter,
                "body render target {requested:?} exceeds the device's {limit}-pixel 2D limit"
            ),
            Self::UnexpectedShape { expected, actual } => write!(
                formatter,
                "body tenant expected resident shape {expected:?}, found {actual:?}"
            ),
            Self::UnexpectedElementType(actual) => write!(
                formatter,
                "body tenant expected an F32 resident plane, found {actual:?}"
            ),
            Self::UnexpectedPlaneClass(actual) => write!(
                formatter,
                "body tenant expected a derived resident plane, found {actual:?}"
            ),
            Self::LeaseTooSmall {
                described,
                available,
            } => write!(
                formatter,
                "body tenant view describes {described} bytes in a {available}-byte lease"
            ),
            Self::RegressingStamp { current, offered } => write!(
                formatter,
                "body tenant stamp {offered:?} does not advance {current:?}"
            ),
            Self::AllocationChangedWithoutStamp { stamp } => write!(
                formatter,
                "body tenant allocation changed without advancing stamp {stamp:?}"
            ),
        }
    }
}

impl Error for IsometryBodyRenderError {}

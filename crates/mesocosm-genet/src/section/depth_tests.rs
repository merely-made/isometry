use mesocosm_core::places::{Ground, Places};
use mesocosm_core::{PartId, VolumeRef};
use mesocosm_lens::{
    BrickFrameInput, BrickMap, BrickRevision, BrickTracer, FRAME_FORMAT, Grade, TraceCamera,
};
use mesocosm_mesh::{BodyMesh, Volume};
use mesocosm_render::{LiveBody, LiveBodyRenderer};

use super::bodies::clip_from_world;
use super::*;

fn ground() -> Ground {
    let places = Places::grown(4_242, 4, 4);
    Ground::grow(&places, 4)
}

fn read_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let row = width * 4;
    let padded = row.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("section depth test readback"),
        size: u64::from(padded) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("section depth test readback"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
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
    queue.submit([encoder.finish()]);
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("readback poll");
    let mapped = slice.get_mapped_range().expect("mapped readback");
    let mut pixels = Vec::with_capacity((row * height) as usize);
    for y in 0..height {
        let start = (y * padded) as usize;
        pixels.extend_from_slice(&mapped[start..start + row as usize]);
    }
    drop(mapped);
    staging.unmap();
    pixels
}

fn differing_pixels(a: &[u8], b: &[u8]) -> usize {
    a.chunks_exact(4)
        .zip(b.chunks_exact(4))
        .filter(|(left, right)| left[..3] != right[..3])
        .count()
}

#[test]
fn voxel_body_and_brick_tracer_share_depth_in_both_orders() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("no adapter; skipping voxel depth join receipt");
        return;
    };
    let Ok((device, queue)) = pollster::block_on(adapter.request_device(&Default::default()))
    else {
        eprintln!("no device; skipping voxel depth join receipt");
        return;
    };
    let (width, height) = (96, 64);
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("ground map");
    let mode = CameraMode::Side;
    // Look at the solid edge of a small generated ground. The near cube is
    // outside the enclosure, still inside the section; the far cube is behind
    // its rock face. No assumed terrain height or reversed camera ordering.
    assert!(ground.solid([1, 1, 4]), "the occluder must be real ground");
    let centre = [1.0, 2.0, 4.0];
    let half_height = 6.0;
    let aspect = width as f32 / height as f32;
    let camera = TraceCamera::orthographic_slab(
        centre,
        mode.forward(),
        [0.0, 1.0, 0.0],
        half_height,
        aspect,
        SLAB_DEPTH,
    )
    .expect("section camera");
    let matrix = clip_from_world(mode, centre, half_height, aspect);
    let grade = Grade::clay();
    let mut tracer =
        BrickTracer::with_format(device.clone(), queue.clone(), width, height, FRAME_FORMAT);
    let mut bodies = LiveBodyRenderer::new(&device, FRAME_FORMAT, 8);
    let body_mesh = BodyMesh::single(VolumeRef::from_tag(9), &Volume::solid([3, 3, 3], 7));

    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("section depth test colour"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FRAME_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&Default::default());
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("section depth test depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth.create_view(&Default::default());

    let mut render = |body_z: Option<f32>, selected: bool, tracer: &mut BrickTracer| -> Vec<u8> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("section depth test"),
        });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("section depth test clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &colour_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        if let Some(z) = body_z {
            bodies
                .draw(
                    &device,
                    &queue,
                    &mut encoder,
                    &colour_view,
                    &depth_view,
                    matrix,
                    None,
                    &[LiveBody {
                        mesh: &body_mesh,
                        materials: &[],
                        origin: [0.0, 0.0, z],
                        scale: 1.0,
                        tint: [1.0; 3],
                        focused: false,
                        selected_part: selected.then_some(PartId(0)),
                    }],
                )
                .expect("body draw");
        }
        let input =
            BrickFrameInput::for_camera(&map, BrickRevision(ground.revision()), camera, &grade)
                .with_clip_from_world(matrix);
        tracer
            .encode_with_depth(&mut encoder, &colour_view, &depth_view, input)
            .expect("tracer depth join");
        queue.submit([encoder.finish()]);
        read_pixels(&device, &queue, &colour, width, height)
    };

    let terrain = render(None, false, &mut tracer);
    let near = render(Some(8.0), false, &mut tracer);
    let far = render(Some(0.0), false, &mut tracer);
    let selected_far = render(Some(0.0), true, &mut tracer);
    let near_difference = differing_pixels(&near, &terrain);
    let far_difference = differing_pixels(&far, &terrain);
    assert!(
        near_difference > 0,
        "the near voxel body must reach the frame"
    );
    assert!(
        far_difference < near_difference,
        "terrain must occlude the farther body"
    );
    assert!(
        far_difference * 10 < near_difference,
        "far body should be almost fully occluded"
    );
    assert!(
        differing_pixels(&selected_far, &far) <= far_difference,
        "selection tint may alter only the body's already visible fragments"
    );
}

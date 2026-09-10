use std::collections::HashSet;

use super::*;

const WIDTH: u32 = 32;
const HEIGHT: u32 = 24;
const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

fn source(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("RG3 Mesocosm tenant fixture"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut bytes = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let rgba = if x < WIDTH / 2 {
                [24, 72 + y as u8, 208, 255]
            } else {
                [224, 176, 24 + y as u8, 255]
            };
            bytes.extend_from_slice(&rgba);
        }
    }
    queue.write_texture(
        texture.as_image_copy(),
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(WIDTH * 4),
            rows_per_image: Some(HEIGHT),
        },
        texture.size(),
    );
    texture
}

fn legacy_presented(chrome: &Chrome, source: &wgpu::Texture) -> wgpu::Texture {
    let target = chrome.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("RG3 Mesocosm legacy presented frame"),
        size: source.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SURFACE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());
    let source_view = source.create_view(&Default::default());
    let mut encoder = chrome.device.create_command_encoder(&Default::default());
    {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("RG3 baseline clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
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
    }
    // A separate compositor models the old Section-owned full-frame draw.
    // Its rectangle cannot be rewritten by the chrome draw below.
    let legacy = Composite::new(&chrome.device, SURFACE_FORMAT);
    legacy.draw(
        &chrome.device,
        &chrome.queue,
        &mut encoder,
        &target_view,
        &source_view,
        (0.0, 0.0, WIDTH as f32, HEIGHT as f32),
        (WIDTH, HEIGHT),
    );
    legacy.draw(
        &chrome.device,
        &chrome.queue,
        &mut encoder,
        &target_view,
        &source_view,
        (WIDTH as f32 - 8.0, HEIGHT as f32 - 6.0, 8.0, 6.0),
        (WIDTH, HEIGHT),
    );
    chrome.queue.submit([encoder.finish()]);
    target
}

fn legacy_master(chrome: &Chrome, source: &wgpu::Texture) -> wgpu::Texture {
    let target = chrome.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("RG3 Mesocosm legacy linear master"),
        size: source.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FRAME_MASTER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());
    let source_view = source.create_view(&Default::default());
    let mut encoder = chrome.device.create_command_encoder(&Default::default());
    {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("RG3 linear master clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
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
    }
    Composite::new(&chrome.device, FRAME_MASTER_FORMAT).draw(
        &chrome.device,
        &chrome.queue,
        &mut encoder,
        &target_view,
        &source_view,
        (0.0, 0.0, WIDTH as f32, HEIGHT as f32),
        (WIDTH, HEIGHT),
    );
    chrome.queue.submit([encoder.finish()]);
    target
}

fn presented(chrome: &Chrome, master: &wgpu::Texture, overlay: &wgpu::Texture) -> wgpu::Texture {
    let master_view = master.create_view(&Default::default());
    let overlay_view = overlay.create_view(&Default::default());
    let mut overlay_encoder = chrome.device.create_command_encoder(&Default::default());
    chrome.draw(
        &mut overlay_encoder,
        &master_view,
        &overlay_view,
        (WIDTH as f32 - 8.0, HEIGHT as f32 - 6.0, 8.0, 6.0),
        (WIDTH, HEIGHT),
    );
    chrome.queue.submit([overlay_encoder.finish()]);

    let target = chrome.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("RG3 Mesocosm presented frame"),
        size: master.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SURFACE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());
    let mut encoder = chrome.device.create_command_encoder(&Default::default());
    {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("RG3 presented clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    chrome.draw_surface(&mut encoder, &target_view, &master_view, (WIDTH, HEIGHT));
    chrome.queue.submit([encoder.finish()]);
    target
}

#[test]
#[ignore = "physical RG3 Mesocosm opaque-tenant receipt"]
fn opaque_section_graph_byte_matches_direct_composite() {
    let handles = netrender::boot().expect("shared wgpu device");
    let adapter = handles.adapter.get_info();
    let device = handles.device.clone();
    let queue = handles.queue.clone();
    let chrome = Chrome::new(handles, SURFACE_FORMAT, 32).expect("Netrender on shared handles");
    let source = source(&device, &queue);
    let direct_master = legacy_master(&chrome, &source);
    let framed = chrome.frame_master(&source, (WIDTH, HEIGHT), 3);
    let master_allocations = chrome
        .net
        .vello_master_allocations()
        .expect("RG3 renderer has a vello master pool");
    let chrome_raster = Raster::new(&device, "RG3 shared chrome raster", WIDTH, HEIGHT);
    let mut chrome_scene = Scene::new(WIDTH, HEIGHT);
    chrome_scene.push_rect(2.0, 3.0, 13.0, 11.0, [0.15, 0.65, 0.35, 1.0]);
    chrome.raster(&chrome_raster, &chrome_scene);
    let chrome_raster_bytes =
        chrome
            .net
            .wgpu_device
            .read_rgba8_texture(&chrome_raster.texture, WIDTH, HEIGHT);
    let intervening_chrome_visible = chrome_raster_bytes
        .chunks_exact(4)
        .any(|pixel| pixel[3] != 0);
    assert!(
        intervening_chrome_visible,
        "shared renderer must rasterize a visible chrome scene between tenant frames"
    );
    let reused = chrome.frame_master(&source, (WIDTH, HEIGHT), 3);
    let master_allocations_after = chrome
        .net
        .vello_master_allocations()
        .expect("RG3 renderer has a vello master pool after rasterization");
    assert_eq!(
        master_allocations_after, master_allocations,
        "the tenant master should survive intervening chrome rasterization"
    );
    let direct_master_bytes =
        chrome
            .net
            .wgpu_device
            .read_rgba8_texture(&direct_master, WIDTH, HEIGHT);
    let graph_master_bytes =
        chrome
            .net
            .wgpu_device
            .read_rgba8_texture(&framed.texture, WIDTH, HEIGHT);
    let reused_master_bytes =
        chrome
            .net
            .wgpu_device
            .read_rgba8_texture(&reused.texture, WIDTH, HEIGHT);
    assert_eq!(graph_master_bytes, direct_master_bytes);
    let reused_master_byte_match = reused_master_bytes == direct_master_bytes;
    assert!(reused_master_byte_match);
    let legacy_presented = legacy_presented(&chrome, &source);
    let graph_presented = presented(&chrome, &framed.texture, &source);
    let legacy_presented_bytes =
        chrome
            .net
            .wgpu_device
            .read_rgba8_texture(&legacy_presented, WIDTH, HEIGHT);
    let graph_presented_bytes =
        chrome
            .net
            .wgpu_device
            .read_rgba8_texture(&graph_presented, WIDTH, HEIGHT);
    let presentation_max_channel_delta = graph_presented_bytes
        .iter()
        .zip(&legacy_presented_bytes)
        .map(|(graph, legacy)| graph.abs_diff(*legacy))
        .max()
        .unwrap_or_default();
    assert!(
        presentation_max_channel_delta <= 3,
        "presentation conversion drifted by {presentation_max_channel_delta} channel levels"
    );
    assert!(
        HashSet::<[u8; 4]>::from_iter(
            graph_presented_bytes
                .chunks_exact(4)
                .map(|pixel| pixel.try_into().expect("RGBA pixel"))
        )
        .len()
            > 8,
        "fixture must retain visible variation"
    );

    let receipt = &framed.receipt;
    assert_eq!(receipt.tenant_name, "mesocosm-section");
    assert_eq!(receipt.fallback_count, 3);
    assert_eq!(receipt.scene_op_boundary, 0);
    assert_eq!(receipt.caller_reported_physical_submission_count, Some(1));
    assert_eq!(receipt.logical_opaque_producer_boundaries, 1);
    assert_eq!(receipt.graph_encoder_batches, 1);
    assert_eq!(receipt.graph_submission_boundaries, 1);
    for needle in [
        "mesocosm-section",
        "opaque tenant composite",
        "netrender initialized master",
        "rasterizer=Classic execution_boundary=opaque_submission",
    ] {
        assert!(
            receipt.logical_plan_dump.contains(needle),
            "missing {needle}"
        );
    }
    let reused_receipt = &reused.receipt;
    assert_eq!(
        reused_receipt.caller_reported_physical_submission_count,
        Some(1)
    );
    assert_eq!(reused_receipt.logical_opaque_producer_boundaries, 1);
    assert_eq!(reused_receipt.graph_encoder_batches, 1);
    assert_eq!(reused_receipt.graph_submission_boundaries, 1);

    let path = crate::played::default_out_dir().join("rg3b_opaque_tenant.json");
    std::fs::create_dir_all(path.parent().expect("receipt parent"))
        .expect("create receipt directory");
    let json = serde_json::json!({
        "adapter": adapter.name,
        "backend": format!("{:?}", adapter.backend),
        "byte_matches_direct_composite": true,
        "intervening_chrome_visible": intervening_chrome_visible,
        "master_allocations_before": master_allocations,
        "master_allocations_after": master_allocations_after,
        "reused_master_byte_match": reused_master_byte_match,
        "presented_surface_max_channel_delta": presentation_max_channel_delta,
        "master_format": format!("{FRAME_MASTER_FORMAT:?}"),
        "presentation_format": format!("{SURFACE_FORMAT:?}"),
        "width": WIDTH,
        "height": HEIGHT,
        "tenant_name": receipt.tenant_name,
        "producer_path": receipt.producer_path,
        "fallback_count": receipt.fallback_count,
        "scene_op_boundary": receipt.scene_op_boundary,
        "caller_reported_physical_submission_count": receipt.caller_reported_physical_submission_count,
        "logical_opaque_producer_boundaries": receipt.logical_opaque_producer_boundaries,
        "graph_encoder_batches": receipt.graph_encoder_batches,
        "graph_submission_boundaries": receipt.graph_submission_boundaries,
        "logical_plan_dump": receipt.logical_plan_dump,
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&json).unwrap())
        .expect("write RG3 Mesocosm receipt");
    println!("RG3 Mesocosm receipt: {}", path.display());
}

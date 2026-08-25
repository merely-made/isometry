use isometry_core::{apply, Facing, MapDocument, SessionEvent, Token, TokenId};
use isometry_runtime::{
    IsometryBodyRenderConfig, IsometryBodyRenderError, IsometryBodyTenant, IsometryResidentBodies,
    IsometryRuntimeProfile,
};
use netrender::{
    create_netrender_instance, Compositor, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Scene, SourceAlpha, TenantNeeds,
};
use quint::resident::{ReadEpoch, ResidentClient};

const DIM: u32 = 96;

fn setup() -> Option<(netrender::WgpuHandles, ResidentClient)> {
    let handles = netrender::boot_shared(
        wgpu::Backends::all(),
        None,
        &TenantNeeds {
            greedy: true,
            label: Some("Isometry resident body render receipt"),
            ..Default::default()
        },
    )
    .ok()?;
    let backend = handles.adapter.get_info().backend;
    let client = ResidentClient::init(cubecl::wgpu::WgpuSetup {
        instance: handles.instance.clone(),
        adapter: handles.adapter.clone(),
        device: handles.device.clone(),
        queue: handles.queue.clone(),
        backend,
    });
    Some((handles, client))
}

fn token(id: u32, at: (i32, i32)) -> Token {
    Token {
        id: TokenId(id),
        at,
        facing: Facing::South,
        sprite: "resident render receipt".to_owned(),
        owner: None,
    }
}

fn board() -> MapDocument {
    let mut map = MapDocument::new("field", 8, 8);
    for token in [token(1, (0, 0)), token(2, (2, 0)), token(3, (4, 4))] {
        apply(&mut map, &SessionEvent::TokenPlaced(token)).unwrap();
    }
    map
}

fn render_config() -> IsometryBodyRenderConfig {
    IsometryBodyRenderConfig {
        target_size: [DIM, DIM],
        origin: [48.0, 16.0],
        basis: [[8.0, 4.0], [0.0, -8.0], [-8.0, 4.0]],
        marker_size: [6.0, 6.0],
        color: [0.0, 1.0, 0.0, 1.0],
    }
}

fn project(config: IsometryBodyRenderConfig, at: (i32, i32)) -> [u32; 2] {
    let x = at.0 as f32;
    let y = 0.5;
    let z = at.1 as f32;
    [
        (config.origin[0]
            + x * config.basis[0][0]
            + y * config.basis[1][0]
            + z * config.basis[2][0]) as u32,
        (config.origin[1]
            + x * config.basis[0][1]
            + y * config.basis[1][1]
            + z * config.basis[2][1]) as u32,
    ]
}

fn pixel(bytes: &[u8], at: [u32; 2]) -> [u8; 4] {
    let offset = ((at[1] * DIM + at[0]) * 4) as usize;
    bytes[offset..offset + 4].try_into().unwrap()
}

#[derive(Default)]
struct CaptureMaster {
    master: Option<wgpu::Texture>,
}

impl Compositor for CaptureMaster {
    fn declare_surface(&mut self, _key: netrender::SurfaceKey, _world_bounds: [f32; 4]) {}

    fn destroy_surface(&mut self, _key: netrender::SurfaceKey) {}

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.master = Some(frame.master.clone());
    }
}

#[track_caller]
fn assert_pixel_close(actual: [u8; 4], expected: [u8; 4]) {
    let error = actual
        .iter()
        .zip(expected)
        .map(|(actual, expected)| (*actual as i16 - expected as i16).unsigned_abs())
        .max()
        .unwrap();
    assert!(error <= 2, "actual {actual:?}, expected {expected:?}");
}

#[test]
fn stamped_resident_bodies_render_and_compose_on_the_host_device() {
    let Some((handles, client)) = setup() else {
        eprintln!("no wgpu adapter: skipping the Isometry render-tenant receipt");
        return;
    };
    let device = handles.device.clone();
    let queue = handles.queue.clone();
    let netrender = create_netrender_instance(
        handles,
        NetrenderOptions {
            tile_cache_size: Some(32),
            enable_vello: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut profile = IsometryRuntimeProfile::new(Default::default()).unwrap();
    let mut map = board();
    let authoritative = map.clone();
    let initial_frame = profile.sync_accepted_map("field", &map).unwrap();
    let mut resident = IsometryResidentBodies::new(client, 8, ReadEpoch::new(70)).unwrap();
    resident
        .apply_frame(&queue, &initial_frame, ReadEpoch::new(71))
        .unwrap();

    let config = render_config();
    let mut tenant = IsometryBodyTenant::new(&device, &queue, config, 8).unwrap();
    let initial_view = resident.positions().unwrap();
    let allocation = initial_view.allocation().clone();
    let initial_render = tenant.render_if_changed(initial_view).unwrap().unwrap();
    assert_eq!(initial_render.stamp, resident.stamp());
    assert!(!initial_render.rebound);
    assert_eq!(
        map, authoritative,
        "rendering changed accepted product facts"
    );

    let tenant_bytes = netrender
        .wgpu_device
        .read_rgba8_texture(tenant.target_texture(), DIM, DIM);
    for at in [(0, 0), (2, 0), (4, 4)] {
        assert_eq!(pixel(&tenant_bytes, project(config, at)), [0, 255, 0, 255]);
    }

    let render_frame = tenant.frame().unwrap();
    let composite = render_frame.netrender_composite(
        ExternalTexturePlacement::new([0.0, 0.0, DIM as f32, DIM as f32]),
        1,
    );
    assert_eq!(composite.scene_op_boundary, 1);
    assert_eq!(composite.placement.alpha, SourceAlpha::Premultiplied);
    let covered = project(config, (0, 0));
    let visible = project(config, (2, 0));
    let mut scene = Scene::new(DIM, DIM);
    scene.push_rect(0.0, 0.0, DIM as f32, DIM as f32, [1.0, 0.0, 0.0, 1.0]);
    scene.push_rect(
        covered[0] as f32 - 3.0,
        covered[1] as f32 - 3.0,
        covered[0] as f32 + 3.0,
        covered[1] as f32 + 3.0,
        [0.0, 0.0, 1.0, 1.0],
    );
    let mut compositor = CaptureMaster::default();
    netrender.render_with_compositor_and_external_textures(
        &scene,
        wgpu::TextureFormat::Rgba8Unorm,
        &mut compositor,
        netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
        &[composite],
    );
    let composite_target = compositor.master.unwrap();
    let composed = netrender
        .wgpu_device
        .read_rgba8_texture(&composite_target, DIM, DIM);
    assert_pixel_close(pixel(&composed, covered), [0, 0, 255, 255]);
    assert_pixel_close(pixel(&composed, visible), [0, 255, 0, 255]);
    assert_pixel_close(pixel(&composed, [2, 2]), [255, 0, 0, 255]);

    let stamp = resident.stamp();
    let still = profile.sync_accepted_map("field", &map).unwrap();
    assert!(still.is_silent());
    assert!(resident
        .apply_frame(&queue, &still, ReadEpoch::new(71))
        .unwrap()
        .is_none());
    assert!(tenant
        .render_if_changed(resident.positions().unwrap())
        .unwrap()
        .is_none());
    assert_eq!(tenant.frame().unwrap().stamp, stamp);

    let stale_view = resident.positions().unwrap();
    let old_positions = [(0, 0), (4, 4)];
    apply(
        &mut map,
        &SessionEvent::TokenMoved {
            id: TokenId(1),
            to: (1, 3),
        },
    )
    .unwrap();
    apply(
        &mut map,
        &SessionEvent::TokenMoved {
            id: TokenId(3),
            to: (5, 2),
        },
    )
    .unwrap();
    let moved_facts = map.clone();
    let moved = profile.sync_accepted_map("field", &map).unwrap();
    assert_eq!(moved.changed.len(), 2);
    resident
        .apply_frame(&queue, &moved, ReadEpoch::new(72))
        .unwrap();
    let moved_view = resident.positions().unwrap();
    assert_eq!(moved_view.allocation(), &allocation);
    let moved_render = tenant.render_if_changed(moved_view).unwrap().unwrap();
    assert!(!moved_render.rebound);
    assert_eq!(map, moved_facts, "rendering changed moved product facts");

    let moved_bytes = netrender
        .wgpu_device
        .read_rgba8_texture(tenant.target_texture(), DIM, DIM);
    for at in old_positions {
        assert_eq!(pixel(&moved_bytes, project(config, at)), [0, 0, 0, 0]);
    }
    for at in [(1, 3), (5, 2)] {
        assert_eq!(pixel(&moved_bytes, project(config, at)), [0, 255, 0, 255]);
    }

    let target_stamp = tenant.frame().unwrap().stamp;
    let refused = tenant.render_if_changed(stale_view).unwrap_err();
    assert!(matches!(
        refused,
        IsometryBodyRenderError::RegressingStamp { .. }
    ));
    assert_eq!(tenant.frame().unwrap().stamp, target_stamp);

    let old_body = profile.body_for(&isometry_runtime::TokenSourceId {
        map: isometry_runtime::MapSourceId::new("field"),
        token: TokenId(3),
    });
    let empty = MapDocument::new("field", 8, 8);
    let removed = profile.sync_accepted_map("field", &empty).unwrap();
    resident
        .apply_frame(&queue, &removed, ReadEpoch::new(73))
        .unwrap();
    tenant
        .render_if_changed(resident.positions().unwrap())
        .unwrap();
    let empty_bytes = netrender
        .wgpu_device
        .read_rgba8_texture(tenant.target_texture(), DIM, DIM);
    assert!(empty_bytes
        .chunks_exact(4)
        .all(|pixel| pixel == [0, 0, 0, 0]));

    let mut replacement = MapDocument::new("field", 8, 8);
    apply(
        &mut replacement,
        &SessionEvent::TokenPlaced(token(9, (1, 1))),
    )
    .unwrap();
    let replaced = profile.sync_accepted_map("field", &replacement).unwrap();
    resident
        .apply_frame(&queue, &replaced, ReadEpoch::new(74))
        .unwrap();
    tenant
        .render_if_changed(resident.positions().unwrap())
        .unwrap();
    let new_body = profile.body_for(&isometry_runtime::TokenSourceId {
        map: isometry_runtime::MapSourceId::new("field"),
        token: TokenId(9),
    });
    assert_eq!(new_body.unwrap().slot(), old_body.unwrap().slot());
    assert_ne!(
        new_body.unwrap().generation(),
        old_body.unwrap().generation()
    );
    let replacement_bytes =
        netrender
            .wgpu_device
            .read_rgba8_texture(tenant.target_texture(), DIM, DIM);
    assert_eq!(
        pixel(&replacement_bytes, project(config, (1, 1))),
        [0, 255, 0, 255]
    );

    let mut wrong_capacity = IsometryBodyTenant::new(&device, &queue, config, 2).unwrap();
    let wrong_shape = wrong_capacity
        .render_if_changed(resident.positions().unwrap())
        .unwrap_err();
    assert!(matches!(
        wrong_shape,
        IsometryBodyRenderError::UnexpectedShape { .. }
    ));
    assert!(wrong_capacity.frame().is_none());

    let storage_limit = device.limits().max_storage_buffer_binding_size as usize;
    let oversized_capacity = storage_limit / 16 + 1;
    let oversized = IsometryBodyTenant::new(&device, &queue, config, oversized_capacity)
        .err()
        .expect("oversized capacity is refused before creating GPU resources");
    assert!(matches!(
        oversized,
        IsometryBodyRenderError::CapacityExceedsDevice { .. }
    ));

    let mut oversized_target = config;
    oversized_target.target_size[0] = device.limits().max_texture_dimension_2d + 1;
    let oversized = IsometryBodyTenant::new(&device, &queue, oversized_target, 8)
        .err()
        .expect("oversized target is refused before texture creation");
    assert!(matches!(
        oversized,
        IsometryBodyRenderError::TargetExceedsDevice { .. }
    ));
}

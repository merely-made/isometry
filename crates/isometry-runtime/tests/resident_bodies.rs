#![cfg(feature = "resident-gpu")]

use conatus::BodyId;
use isometry_core::{apply, Facing, MapDocument, SessionEvent, Token, TokenId};
use isometry_runtime::{
    IsometryResidentBodies, IsometryResidentError, IsometryRuntimeProfile, MapSourceId,
    TokenSourceId,
};
use quint::resident::{RawKernelView, ReadEpoch, ResidentClient};

fn setup() -> Option<cubecl::wgpu::WgpuSetup> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .ok()?;
    let backend = adapter.get_info().backend;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Isometry resident body receipt"),
        ..Default::default()
    }))
    .ok()?;
    Some(cubecl::wgpu::WgpuSetup {
        instance,
        adapter,
        device,
        queue,
        backend,
    })
}

fn token(id: u32, at: (i32, i32)) -> Token {
    Token {
        id: TokenId(id),
        at,
        facing: Facing::South,
        sprite: "resident receipt".to_owned(),
        owner: None,
    }
}

fn board() -> MapDocument {
    let mut map = MapDocument::new("field", 8, 8);
    for token in [token(1, (0, 0)), token(2, (2, 2)), token(3, (4, 4))] {
        apply(&mut map, &SessionEvent::TokenPlaced(token)).unwrap();
    }
    map
}

fn source(token: u32) -> TokenSourceId {
    TokenSourceId {
        map: MapSourceId::new("field"),
        token: TokenId(token),
    }
}

fn readback(device: &wgpu::Device, queue: &wgpu::Queue, view: &RawKernelView) -> Vec<f32> {
    let lease = view.lease();
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Isometry resident body readback"),
        size: lease.byte_len(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Isometry resident body readback"),
    });
    encoder.copy_buffer_to_buffer(lease.buffer, lease.offset, &staging, 0, lease.byte_len());
    let submission = queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |_| ());
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        })
        .unwrap();
    let mapped = staging
        .slice(..)
        .get_mapped_range()
        .expect("resident body readback mapped");
    let values = bytemuck::cast_slice(&mapped).to_vec();
    drop(mapped);
    staging.unmap();
    values
}

fn body_slot(frame_body: BodyId) -> usize {
    frame_body.slot() as usize * 4
}

#[test]
fn accepted_frames_patch_stable_resident_slots_without_touching_the_map() {
    let Some(setup) = setup() else {
        eprintln!("no wgpu adapter: skipping the Isometry resident body receipt");
        return;
    };
    let device = setup.device.clone();
    let queue = setup.queue.clone();
    let client = ResidentClient::init(setup);
    let mut profile = IsometryRuntimeProfile::new(Default::default()).unwrap();
    let mut map = board();
    let authoritative = map.clone();
    let initial_frame = profile.sync_accepted_map("field", &map).unwrap();

    let mut resident = IsometryResidentBodies::new(client.clone(), 8, ReadEpoch::new(40)).unwrap();
    let initial = resident
        .apply_frame(&queue, &initial_frame, ReadEpoch::new(41))
        .unwrap()
        .unwrap();
    assert_eq!(initial.changed_slots, vec![0, 1, 2]);
    assert_eq!(map, authoritative, "resident publication changed the map");

    let before = resident.positions().unwrap();
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
            to: (6, 5),
        },
    )
    .unwrap();
    let accepted = map.clone();
    let moved = profile.sync_accepted_map("field", &map).unwrap();
    assert_eq!(moved.changed.len(), 2);
    let update = resident
        .apply_frame(&queue, &moved, ReadEpoch::new(42))
        .unwrap()
        .unwrap();
    assert_eq!(update.changed_slots, vec![0, 2]);
    assert_eq!(map, accepted, "resident publication changed accepted facts");

    let after = resident.positions().unwrap();
    assert_eq!(before.allocation(), after.allocation());
    assert_eq!(before.stamp().revision, initial_frame.revision);
    assert_eq!(after.stamp().revision, moved.revision);
    let values = readback(&device, &queue, &after);
    for changed in &moved.changed {
        let offset = body_slot(changed.body.id);
        let [x, y, z] = changed.body.transform.translation;
        assert_eq!(&values[offset..offset + 4], &[x, y, z, 1.0]);
    }

    let stamp = resident.stamp();
    let still = profile.sync_accepted_map("field", &map).unwrap();
    assert!(still.is_silent());
    assert!(resident
        .apply_frame(&queue, &still, ReadEpoch::new(42))
        .unwrap()
        .is_none());
    assert_eq!(resident.stamp(), stamp);

    let old_body = resident.body_for(&source(3)).unwrap();
    let empty = MapDocument::new("field", 8, 8);
    let removed = profile.sync_accepted_map("field", &empty).unwrap();
    resident
        .apply_frame(&queue, &removed, ReadEpoch::new(43))
        .unwrap();
    assert!(resident.body_for(&source(3)).is_none());

    let mut replacement = MapDocument::new("field", 8, 8);
    apply(
        &mut replacement,
        &SessionEvent::TokenPlaced(token(9, (7, 1))),
    )
    .unwrap();
    let replaced = profile.sync_accepted_map("field", &replacement).unwrap();
    resident
        .apply_frame(&queue, &replaced, ReadEpoch::new(44))
        .unwrap();
    let new_body = resident.body_for(&source(9)).unwrap();
    assert_eq!(new_body.slot(), old_body.slot());
    assert_ne!(new_body.generation(), old_body.generation());

    let mut too_small = IsometryResidentBodies::new(client, 2, ReadEpoch::new(50)).unwrap();
    let refused = too_small
        .apply_frame(&queue, &initial_frame, ReadEpoch::new(51))
        .unwrap_err();
    assert!(matches!(
        refused,
        IsometryResidentError::CapacityExceeded { capacity: 2, .. }
    ));
    assert_eq!(too_small.stamp().revision, 0);
    assert!(readback(&device, &queue, &too_small.positions().unwrap())
        .iter()
        .all(|value| *value == 0.0));
}

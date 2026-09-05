// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Opt-in physical receipt for host-owned validation attribution.

use paredros_room::frame_health::{FrameDecision, FrameHealth, PresentationPolicy};

#[test]
#[ignore = "opt-in physical validation-scope receipt"]
fn tenant_validation_scope_leaves_later_frame_usable() {
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let handles = paredros_room::gpu::boot(&instance, None);
    let mut health = FrameHealth::new(PresentationPolicy::AwaitedDiagnostic);

    assert_eq!(health.begin_frame(1), FrameDecision::Proceed);
    let scope = handles
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let source = handles.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("validation source"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let target = handles.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("validation target"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = handles
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("disposable invalid tenant encoder"),
        });
    encoder.copy_buffer_to_buffer(&source, 0, &target, 0, 8);
    drop(encoder.finish());
    let error = pollster::block_on(scope.pop()).expect("validation error was captured");
    assert!(matches!(&error, wgpu::Error::Validation { .. }));
    let detail = format!("{error:?}");
    assert_eq!(
        health.finish_validation(
            "paredros-room",
            "renderling::Stage::encode_into (opaque)",
            1,
            Some(detail.clone()),
        ),
        FrameDecision::Suppress
    );
    assert_eq!(health.validations().len(), 1);
    assert_eq!(health.validations()[0].tenant_name, "paredros-room");
    assert_eq!(
        health.validations()[0].producer_path,
        "renderling::Stage::encode_into (opaque)"
    );
    assert_eq!(health.validations()[0].frame, 1);
    assert!(!health.validations()[0].error.is_empty());

    assert_eq!(health.begin_frame(2), FrameDecision::Proceed);
    let valid_scope = handles
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let mut valid_encoder =
        handles
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("disposable valid tenant encoder"),
            });
    valid_encoder.copy_buffer_to_buffer(&source, 0, &target, 0, 4);
    handles.queue.submit([valid_encoder.finish()]);
    assert!(pollster::block_on(valid_scope.pop()).is_none());
    assert_eq!(
        health.finish_validation(
            "paredros-room",
            "renderling::Stage::encode_into (opaque)",
            2,
            None
        ),
        FrameDecision::Proceed
    );
}

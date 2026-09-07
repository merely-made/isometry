// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bringing the window and the device up.
//!
//! Split out of `app.rs` at the 600-line ceiling when PE1 added the third
//! chrome lane. One-shot construction — surface, adapter, device, the section's
//! pipeline, and the chrome the three lanes share — with nothing per-frame in
//! it; the loop next door is what the file is for.

use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::{Gpu, Host, Lanes};
use crate::chrome::Chrome;
use crate::section::{Framing, Section};

impl Host {
    /// Builds the window, the device, and everything drawn through it. Idempotent:
    /// winit may resume more than once, and a second window would be a second game.
    pub(super) fn boot(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("Mesocosm | H graft tissue | O express discovery")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("a window is available"),
        );

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("a surface for this window");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
            compatible_surface: Some(&surface),
        }))
        .expect("an adapter that can present to this surface");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("mesocosm host"),
            ..Default::default()
        }))
        .expect("a device");
        self.adapter = Some(adapter.get_info());

        let size = window.inner_size();
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: self.config.width,
            height: self.config.height,
            present_mode: caps.present_modes[0],
            // wgpu 30 made surface color space explicit; Auto keeps the
            // pre-30 platform-chosen behavior.
            color_space: wgpu::SurfaceColorSpace::Auto,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // The section binds the live Ground at genesis and refreshes from the
        // world's own dirty drain thereafter.
        let mut section = match Section::new(
            device.clone(),
            queue.clone(),
            self.config.width,
            self.config.height,
            format,
            self.runtime.world().ground(),
            Framing::new(self.config.slab_half_height, self.config.camera),
        ) {
            Ok(section) => section,
            Err(error) => {
                eprintln!("section: {error}");
                self.code = 1;
                event_loop.exit();
                return;
            },
        };

        section.configure_bodies(self.config.body_mode, self.config.body_budget);

        // The chrome shares the game's device rather than creating a second
        // one, which is the arrangement the workspace's wgpu pin exists for.
        // One Chrome carries every lane. Until the Mere/Genet paint pins align,
        // it contains two Netrender instances over these same physical handles:
        // the existing UI rasterizer and the current frame-graph owner.
        let chrome = Chrome::new(
            netrender::WgpuHandles {
                instance,
                adapter,
                device: device.clone(),
                queue: queue.clone(),
            },
            format,
            crate::hud::SIDE,
        )
        .map(|device| Lanes {
            hud: crate::hud::Hud::new(&device, self.runtime.world()),
            vitals: crate::vitals::VitalsChrome::new(&device),
            checkpoint: crate::succession::SuccessionChrome::new(&device),
            board: crate::review::BoardChrome::new(&device),
            grafting: crate::body_menu::BodyMenuChrome::new(&device),
            dev: crate::dev::DevChrome::new(&device),
            device,
        });

        self.gpu = Some(Gpu {
            device,
            queue,
            surface,
            config,
            section,
            chrome,
            last_tenant_receipt: None,
        });
        self.window = Some(window);
    }
}

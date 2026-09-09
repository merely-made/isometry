// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

use crate::gpu::{Gpu, INITIAL_SIZE, MIN_FRAMES, WINDOW_TITLE};

pub fn run() -> Result<(), String> {
    let event_loop = EventLoop::<InitEvent>::with_user_event()
        .build()
        .map_err(|error| error.to_string())?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let app = App::new(Config::read()?, event_loop.create_proxy());

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        event_loop
            .run_app(&mut app)
            .map_err(|error| error.to_string())
    }
}

struct Config {
    frames: u32,
    #[cfg(not(target_arch = "wasm32"))]
    capture: Option<std::path::PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    receipt: Option<std::path::PathBuf>,
}

impl Config {
    fn read() -> Result<Self, String> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(Self { frames: MIN_FRAMES })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut config = Self {
                frames: MIN_FRAMES,
                capture: None,
                receipt: None,
            };
            let mut args = std::env::args().skip(1);
            while let Some(argument) = args.next() {
                match argument.as_str() {
                    "--frames" => {
                        let value = args.next().ok_or("--frames needs a number")?;
                        config.frames = value.parse().map_err(|_| "invalid --frames")?;
                    },
                    "--capture" => {
                        config.capture = Some(args.next().ok_or("--capture needs a path")?.into());
                    },
                    "--receipt" => {
                        config.receipt = Some(args.next().ok_or("--receipt needs a path")?.into());
                    },
                    other => return Err(format!("unknown argument: {other}")),
                }
            }
            config.frames = config.frames.max(MIN_FRAMES);
            Ok(config)
        }
    }
}

enum InitEvent {
    Ready(Result<Gpu, String>),
}

struct App {
    config: Config,
    #[cfg(target_arch = "wasm32")]
    proxy: EventLoopProxy<InitEvent>,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    initializing: bool,
    frames: u32,
    done: bool,
}

impl App {
    fn new(config: Config, proxy: EventLoopProxy<InitEvent>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let _ = proxy;
        Self {
            config,
            #[cfg(target_arch = "wasm32")]
            proxy,
            window: None,
            gpu: None,
            initializing: false,
            frames: 0,
            done: false,
        }
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if self.done {
            return;
        }
        let Some(gpu) = &mut self.gpu else { return };
        let rendered = match gpu.draw(self.frames + 1) {
            Ok(Some(rendered)) => rendered,
            Ok(None) => return,
            Err(error) => {
                self.fail(event_loop, &error);
                return;
            },
        };
        self.frames += 1;
        if self.frames < self.config.frames.max(1) {
            return;
        }
        let (receipt, master) = rendered;
        #[cfg(target_arch = "wasm32")]
        let _ = &master;
        let json = match serde_json::to_string_pretty(&receipt) {
            Ok(json) => json,
            Err(error) => {
                self.fail(event_loop, &error.to_string());
                return;
            },
        };

        #[cfg(target_arch = "wasm32")]
        publish_receipt(&json);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Err(error) = gpu.write_native_receipts(
                self.config.receipt.as_deref(),
                self.config.capture.as_deref(),
                &json,
                &master,
            ) {
                self.fail(event_loop, &error);
                return;
            }
            println!("{json}");
        }

        self.done = true;
        event_loop.set_control_flow(ControlFlow::Wait);
        #[cfg(not(target_arch = "wasm32"))]
        event_loop.exit();
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, message: &str) {
        self.done = true;
        #[cfg(target_arch = "wasm32")]
        publish_error(message);
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("{message}");
        event_loop.exit();
    }
}

impl ApplicationHandler<InitEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() || self.initializing {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                INITIAL_SIZE[0],
                INITIAL_SIZE[1],
            ));
        #[cfg(target_arch = "wasm32")]
        let attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            attributes.with_append(true)
        };
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.fail(event_loop, &error.to_string());
                return;
            },
        };
        self.window = Some(window.clone());
        self.initializing = true;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let ready = pollster::block_on(Gpu::new(window));
            self.user_event(event_loop, InitEvent::Ready(ready));
        }
        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = proxy.send_event(InitEvent::Ready(Gpu::new(window).await));
            });
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: InitEvent) {
        match event {
            InitEvent::Ready(Ok(gpu)) => {
                self.gpu = Some(gpu);
                self.initializing = false;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            InitEvent::Ready(Err(error)) => self.fail(event_loop, &error),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            },
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            _ => {},
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.done
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn publish_receipt(json: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    if let Some(receipt) = document.get_element_by_id("receipt") {
        receipt.set_text_content(Some(json));
    }
    if let Some(body) = document.body() {
        let _ = body.set_attribute("data-status", "ready");
    }
}

#[cfg(target_arch = "wasm32")]
pub fn publish_error(message: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    if let Some(receipt) = document.get_element_by_id("receipt") {
        receipt.set_text_content(Some(message));
    }
    if let Some(body) = document.body() {
        let _ = body.set_attribute("data-status", "error");
    }
}

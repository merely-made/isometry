// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G2's headed native/WebGPU receipt: real Ground, DDA, SDF body, and
//! netrender composition in one frame.

#[path = "v1_frame/app.rs"]
mod app;
#[path = "g2_frame/gpu.rs"]
mod gpu;
#[path = "g2_frame/receipt.rs"]
mod receipt;
#[path = "g2_frame/scenario.rs"]
mod scenario;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    if let Err(error) = app::run() {
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("{error}");
        #[cfg(target_arch = "wasm32")]
        app::publish_error(&error.to_string());
    }
}

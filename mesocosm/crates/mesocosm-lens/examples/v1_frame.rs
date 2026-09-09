// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V1 same-device native/WebGPU presentation receipt.

#[path = "v1_frame/app.rs"]
mod app;
#[path = "v1_frame/gpu.rs"]
mod gpu;
#[path = "v1_frame/receipt.rs"]
mod receipt;

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

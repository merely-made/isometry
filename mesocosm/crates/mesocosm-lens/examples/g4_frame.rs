// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G4's headed native/browser receipt. The shared V1/G2 frame host presents
//! the seed-0 player and autonomous hunter crossing one generated burrow entry
//! and one place boundary in the same ordered run.

#[path = "v1_frame/app.rs"]
mod app;
#[path = "g4_frame/fixture.rs"]
mod burrow_scenario;
#[path = "g2_frame/gpu.rs"]
mod gpu;
#[path = "g4_frame/receipt.rs"]
mod receipt;
#[path = "g4_frame/scenario.rs"]
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

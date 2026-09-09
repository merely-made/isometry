// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! T1's headed receipt: terrarium pointer picking through Conatus.
//!
//! The whole sweep is judged on the CPU before the first frame (see
//! `t1_picking/scenario.rs`); the frames present the judged cursor stops
//! over the real traced terrarium and the receipt lands on the last one.

#[path = "v1_frame/app.rs"]
mod app;
#[path = "t1_picking/gpu.rs"]
mod gpu;
#[path = "t1_picking/receipt.rs"]
mod receipt;
#[path = "t1_picking/scenario.rs"]
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

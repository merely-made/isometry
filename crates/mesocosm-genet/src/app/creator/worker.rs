// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::{
    PartPalette,
    world::generation::{Error, Prepared, Request},
};
use std::sync::mpsc::{self, Receiver, Sender};

type ResultMessage = (u64, Result<Prepared, Error>);
pub(super) struct Worker {
    requests: Sender<(u64, Request)>,
    pub results: Receiver<ResultMessage>,
}

impl Worker {
    pub fn new(palette: PartPalette) -> Self {
        let (requests, incoming) = mpsc::channel::<(u64, Request)>();
        let (finished, results) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(mut job) = incoming.recv() {
                // Coalesce queued control changes. The UI also rejects any
                // result already in flight when the user changes the request.
                for newer in incoming.try_iter() {
                    job = newer;
                }
                let result = job.1.prepare(palette);
                if finished.send((job.0, result)).is_err() {
                    break;
                }
            }
        });
        Self { requests, results }
    }

    pub fn send(&self, serial: u64, request: Request) -> bool {
        self.requests.send((serial, request)).is_ok()
    }
}

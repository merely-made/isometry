// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the host writes on the way out.
//!
//! A run that left nothing behind cannot be judged, and a hash that drifted
//! looks exactly like one that did not until somebody says so. Every exit —
//! the window closing, Escape, a frame limit, a replay reaching the end of its
//! trace — comes through [`Host::finish`].

use winit::event_loop::ActiveEventLoop;

use super::Host;
use crate::played::{self, FrameGraphReceipt, PlayedReceipt, PlayedTrace};

impl Host {
    /// Writes what the run leaves behind, once, and stops the loop.
    pub(super) fn finish(&mut self, event_loop: &ActiveEventLoop) {
        if !self.finished {
            self.finished = true;
            self.write_capture();
            self.write_trace();
            self.write_receipt();
        }
        event_loop.exit();
    }

    /// The frame the player was last looking at, chrome included, at the path
    /// the run was told to write.
    fn write_capture(&self) {
        let Some(path) = self.config.capture.clone() else {
            return;
        };
        if let Err(error) = self.capture_to(&path) {
            eprintln!("capture: {error}");
        }
    }

    /// The same frame, at any path. **The one capture path** — a scenario's
    /// `capture <name>` verb comes through here too (DT4), so a screenshot a
    /// script asks for is byte-for-byte the one the receipt writes rather than
    /// a second read-back that could composite a different set of lanes.
    pub(crate) fn capture_to(&self, path: &std::path::Path) -> Result<(), String> {
        let Some(gpu) = &self.gpu else {
            return Err("there is no device to read a frame back from".into());
        };
        let frame = (gpu.config.width, gpu.config.height);
        let shot = if let Some(lanes) = &gpu.chrome {
            let master = lanes.device.frame_master(
                gpu.section.display_texture(),
                frame,
                gpu.section.body_stats().fallback_bodies as u64,
            );
            let master_view = master.texture.create_view(&Default::default());
            let mut encoder =
                lanes
                    .device
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("mesocosm captured chrome into master"),
                    });
            if !self.grafting.open {
                lanes
                    .hud
                    .composite(&lanes.device, &mut encoder, &master_view, frame);
                lanes
                    .vitals
                    .composite(&lanes.device, &mut encoder, &master_view, frame);
                if self.config.dev {
                    lanes
                        .dev
                        .composite(&lanes.device, &mut encoder, &master_view, frame);
                }
            }
            lanes
                .checkpoint
                .composite(&lanes.device, &mut encoder, &master_view, frame);
            lanes
                .board
                .composite(&lanes.device, &mut encoder, &master_view, frame);
            lanes
                .grafting
                .composite(&lanes.device, &mut encoder, &master_view, frame);
            lanes.device.queue().submit(Some(encoder.finish()));
            gpu.section.capture_from(&master.texture, |_, _, _| {})
        } else {
            gpu.section.capture(|_, _, _| {})
        };
        let Some((width, height, pixels)) = shot else {
            return Err("the section could not be read back".into());
        };
        played::write_png(path, width, height, &pixels)
    }

    /// A replay's trace is an input; only a played session writes one.
    fn write_trace(&self) {
        let (Some(path), None) = (&self.config.trace, &self.config.replay) else {
            return;
        };
        let recorded = PlayedTrace {
            start: self.config.effective_start().cloned(),
            trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
            scene: self.config.effective_scene(),
            body_layout: self.config.effective_body_layout(),
            seed: self.config.seed,
            organisms: self.runtime.receipt().organisms,
            steps: self.runtime.trace().len() as u64,
            state_hash: self.runtime.state_hash(),
            intents: self.runtime.trace().to_vec(),
            content: self.content.clone(),
        };
        if let Err(error) = played::write_json(path, &recorded) {
            eprintln!("trace: {error}");
        }
    }

    fn write_receipt(&mut self) {
        let receipt = self.receipt();
        println!(
            "{}{} {} steps over {} frames, hash {:016x}{}",
            // **The label goes first, where it cannot be read past** (DT3).
            // A run that ended an epoch, forced a birth, killed something or
            // placed matter is not an unaided playtest, and the line that
            // reports it should not be able to be skimmed as one.
            match played::assisted_label(receipt.dev_intents) {
                label if label.is_empty() => label,
                label => format!("{label} "),
            },
            receipt.mode,
            receipt.steps,
            receipt.frames,
            receipt.state_hash,
            match receipt.state_hash_matches {
                Some(true) => " (matches the recorded hash)".to_string(),
                Some(false) => format!(
                    " (MISMATCH: the trace recorded {:016x})",
                    receipt.expected_state_hash.unwrap_or_default()
                ),
                None => String::new(),
            }
        );
        // Which arm this capture is. One line, beside the hash, so a sheet of
        // three captures cannot be assembled out of order. (DC4)
        println!(
            "camera: {} section, slab half-height {}",
            receipt.camera, receipt.slab_half_height
        );
        // Loud, because the alternative this replaces was a body that was
        // simply not there.
        if receipt.body_capsules_dropped > 0 {
            println!(
                "body: {} of {} parts past the lens capsule budget, drawn truncated to its widest",
                receipt.body_capsules_dropped, receipt.body_parts
            );
        }
        // The only route to a nonzero exit: a replay that landed elsewhere.
        if receipt.state_hash_matches == Some(false) {
            self.code = 1;
        }
        let Some(path) = &self.config.receipt else {
            return;
        };
        if let Err(error) = played::write_json(path, &receipt) {
            eprintln!("receipt: {error}");
        }
    }

    /// `replay` for a run a trace drives, `played` for every other. One
    /// definition, because the receipt and a scenario's `assert snap mode` both
    /// ask.
    pub(crate) fn mode(&self) -> &'static str {
        if self.config.replay.is_some() {
            "replay"
        } else {
            "played"
        }
    }

    fn receipt(&self) -> PlayedReceipt {
        let run = self.runtime.receipt();
        // Only a replay that reached the end of its trace has a hash to be held
        // to. A capture run cut short by `--frames` stopped somewhere in the
        // middle of the recording on purpose, and comparing its world against
        // the recording's *final* hash would report a determinism failure for
        // having taken a photograph.
        let expected = self
            .config
            .replay
            .as_ref()
            .filter(|trace| self.cursor >= trace.intents.len())
            .map(|trace| trace.state_hash);
        let world = self.runtime.world();
        PlayedReceipt {
            trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
            scene: self.config.effective_scene().name(),
            habitat_bounds: self.habitat.as_ref().map(|h| [h.bounds.min, h.bounds.max]),
            terrarium_pitch: self.habitat.as_ref().map(|_| self.config.terrarium_pitch),
            burrow_occupied: self.habitat.as_ref().and_then(|h| {
                world
                    .position()
                    .map(|at| crate::section::terrarium_occupied(h, at))
            }),
            body_layout: if self.config.effective_start().is_some() {
                "generated"
            } else {
                self.config.effective_body_layout().name()
            },
            body_content: if self.content.is_some() {
                "generated-v1"
            } else {
                "fixtures"
            },
            mode: self.mode(),
            seed: run.seed,
            organisms: run.organisms,
            steps: run.steps,
            state_hash: run.state_hash,
            expected_state_hash: expected,
            state_hash_matches: expected.map(|hash| hash == run.state_hash),
            adapter: self
                .adapter
                .as_ref()
                .map_or_else(|| "none".into(), |info| info.name.clone()),
            backend: self
                .adapter
                .as_ref()
                .map_or_else(|| "none".into(), |info| format!("{:?}", info.backend)),
            frames: self.frames,
            trace_len: self.runtime.trace().len(),
            ground_revision: world.ground().revision(),
            body_parts: world.body().map(|body| body.len()).unwrap_or(0),
            body_capsules_dropped: self.body_capsules_dropped,
            section_roster: self
                .gpu
                .as_ref()
                .map_or(0, |gpu| gpu.section.last_roster_members()),
            roster_capsules_dropped: self
                .gpu
                .as_ref()
                .map_or(0, |gpu| gpu.section.last_roster_capsules_dropped()),
            slab_half_height: self
                .gpu
                .as_ref()
                .map_or(self.config.slab_half_height, |gpu| {
                    gpu.section.half_height()
                }),
            camera: self
                .gpu
                .as_ref()
                .map_or(self.config.camera, |gpu| gpu.section.mode())
                .name(),
            cutaway: self.config.cutaway.name(),
            terrain_style: self
                .config
                .terrain_style
                .resolved(self.habitat.is_some())
                .name(),
            bodies: if self.grafting.open {
                "voxels"
            } else {
                self.config.body_mode.name()
            },
            inspecting: self.inspection.open,
            graft_menu: self.grafting.open
                && self.grafting.operation == super::grafting::BodyOperation::Graft,
            expression_tissue_mg: self.expression_tissue(false),
            expression_candidate_tissue_mg: self.expression_tissue(true),
            expression_menu: self.grafting.open
                && self.grafting.operation == super::grafting::BodyOperation::Express,
            expression_preview: self.grafting.operation == super::grafting::BodyOperation::Express
                && self.grafting.preview.is_some(),
            expression_part: (self.grafting.operation == super::grafting::BodyOperation::Express)
                .then_some(self.grafting.root)
                .flatten()
                .map(|p| p.0),
            body_view: if self.grafting.open {
                "isolated"
            } else {
                "scene"
            },
            graft_preview: self.grafting.operation == super::grafting::BodyOperation::Graft
                && self.grafting.preview.is_some(),
            graft_root: (self.grafting.operation == super::grafting::BodyOperation::Graft)
                .then_some(self.grafting.root)
                .flatten()
                .map(|part| part.0),
            selected_part: self
                .inspection
                .selected
                .map(|selection| played::PartSelectionReceipt {
                    organism: selection.organism.0,
                    part: selection.part.0,
                    revision: selection.revision.0,
                }),
            body_budget: self.config.body_budget,
            body_projection: self
                .gpu
                .as_ref()
                .map(|gpu| gpu.section.body_stats())
                .unwrap_or_default(),
            frame_graph: self
                .gpu
                .as_ref()
                .and_then(|gpu| gpu.last_tenant_receipt.as_ref())
                .map(|receipt| FrameGraphReceipt {
                    tenant_name: receipt.tenant_name.clone(),
                    producer_path: receipt.producer_path.clone(),
                    fallback_count: receipt.fallback_count,
                    scene_op_boundary: receipt.scene_op_boundary,
                    caller_reported_physical_submission_count: receipt
                        .caller_reported_physical_submission_count,
                    logical_opaque_producer_boundaries: receipt.logical_opaque_producer_boundaries,
                    graph_encoder_batches: receipt.graph_encoder_batches,
                    graph_submission_boundaries: receipt.graph_submission_boundaries,
                    logical_plan_dump: receipt.logical_plan_dump.clone(),
                }),
            trace: self
                .config
                .trace
                .as_ref()
                .map(|path| path.display().to_string()),
            capture: self
                .config
                .capture
                .as_ref()
                .map(|path| path.display().to_string()),
            dev: self.config.dev,
            // Off the driver, which counts what the world accepted rather than
            // what a key asked for — so a replay of an assisted trace reports
            // the same number, and a refused dev intent reports none. (DT3)
            dev_intents: self.runtime.dev_intents(),
        }
    }
}

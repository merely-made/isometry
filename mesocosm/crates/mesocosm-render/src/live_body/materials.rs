// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Disposable process-tissue material projection for live body instances.

use std::collections::BTreeMap;

use mesocosm_core::{PartId, process::Process};

use super::LiveBody;

/// One process allocation summarized for one addressed part.
///
/// `fraction` is the process's living mosaic cells divided by that part's
/// living capacity. It is deliberately a summary: allocation cells are a
/// graph, not render voxels. The renderer therefore shows a voxel-aligned
/// density mark for the supplied amount, and never claims to know which mesh
/// voxel owns a particular cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartMaterial {
    pub part: PartId,
    pub process: Process,
    pub fraction: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PartAppearance {
    /// RGB organism tint, plus a restrained amber inspection accent.
    pub(super) tint: [f32; 4],
    /// Contract, intake, sense, fix fractions.
    pub(super) expression: [f32; 4],
    /// Secrete fraction, then reserved instance fields.
    pub(super) expression_tail: [f32; 4],
}

pub(super) fn valid_materials(materials: &[PartMaterial]) -> bool {
    let mut total_by_part = BTreeMap::<PartId, f32>::new();
    for material in materials {
        if !material.fraction.is_finite() || material.fraction < 0.0 {
            return false;
        }
        let total = total_by_part.entry(material.part).or_default();
        *total += material.fraction;
        if *total > 1.0 + f32::EPSILON {
            return false;
        }
    }
    true
}

pub(super) fn part_appearance(body: LiveBody<'_>, part: PartId) -> PartAppearance {
    let mut fractions = [0.0; 5];
    for material in body
        .materials
        .iter()
        .filter(|material| material.part == part)
    {
        // `valid_materials` checks the input at the draw boundary. Keeping
        // aggregation here makes this helper useful for direct appearance
        // tests without turning it into a second validation authority.
        fractions[process_index(material.process)] += material.fraction;
    }
    let tint = if body.focused {
        body.tint.map(|channel| channel * 1.08)
    } else {
        body.tint
    };
    PartAppearance {
        tint: [
            tint[0],
            tint[1],
            tint[2],
            if body.selected_part == Some(part) {
                1.0
            } else {
                0.0
            },
        ],
        expression: [fractions[0], fractions[1], fractions[2], fractions[3]],
        expression_tail: [fractions[4], 0.0, 0.0, 0.0],
    }
}

fn process_index(process: Process) -> usize {
    Process::ALL
        .iter()
        .position(|candidate| *candidate == process)
        .expect("Process::ALL lists every native process")
}

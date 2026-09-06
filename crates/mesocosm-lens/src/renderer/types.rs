// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use crate::{CritterPose, Flight, Grade, maps::BiomeMaps};

/// Caller-owned identity for one map revision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MapRevision(pub u64);

/// One changed rectangle in map texels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// How a new map revision differs from the one already resident.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MapChange {
    #[default]
    Full,
    Region(DirtyRect),
}

/// Everything needed to encode one frame.
#[derive(Clone, Copy)]
pub struct FrameInput<'a> {
    pub maps: &'a BiomeMaps,
    pub map_revision: MapRevision,
    pub map_change: MapChange,
    pub flight: &'a Flight,
    pub grade: &'a Grade,
    pub pose: Option<&'a CritterPose>,
}

impl<'a> FrameInput<'a> {
    pub fn new(
        maps: &'a BiomeMaps,
        map_revision: MapRevision,
        flight: &'a Flight,
        grade: &'a Grade,
    ) -> Self {
        Self {
            maps,
            map_revision,
            map_change: MapChange::Full,
            flight,
            grade,
            pose: None,
        }
    }

    pub fn changed(mut self, change: MapChange) -> Self {
        self.map_change = change;
        self
    }

    pub fn with_pose(mut self, pose: &'a CritterPose) -> Self {
        self.pose = Some(pose);
        self
    }
}

/// Per-call evidence. Counts describe CPU submissions and resource churn;
/// V1 adds backend GPU timestamps beside netrender's spans.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameDiagnostics {
    pub cpu_prepare_us: u64,
    pub map_upload_bytes: u64,
    pub uniform_upload_bytes: u64,
    pub resource_creations: u32,
    pub bind_group_rebuilds: u32,
    pub map_recreated: bool,
    pub target_recreated: bool,
    pub march_passes: u32,
    pub grade_passes: u32,
    pub readback_bytes: u64,
}

/// Pixel receipt produced by `Lens::capture`.
#[derive(Clone, Debug)]
pub struct Capture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub diagnostics: FrameDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LensError {
    EmptyMap,
    InvalidHeightLength { expected: usize, actual: usize },
    InvalidColorLength { expected: usize, actual: usize },
    PaletteTooLarge(usize),
    TooManyCapsules { actual: usize, maximum: usize },
    DirtyRegionOutsideMap(DirtyRect),
    CaptureFormat(wgpu::TextureFormat),
    DevicePoll(String),
    Readback(String),
}

impl fmt::Display for LensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMap => write!(f, "a lens map must have a non-zero side"),
            Self::InvalidHeightLength { expected, actual } => {
                write!(f, "height map has {actual} bytes; expected {expected}")
            },
            Self::InvalidColorLength { expected, actual } => {
                write!(f, "colour map has {actual} bytes; expected {expected}")
            },
            Self::PaletteTooLarge(actual) => {
                write!(
                    f,
                    "palette has {actual} entries; the lens accepts at most 256"
                )
            },
            Self::TooManyCapsules { actual, maximum } => {
                write!(
                    f,
                    "body has {actual} capsules; the baseline accepts at most {maximum}"
                )
            },
            Self::DirtyRegionOutsideMap(rect) => write!(
                f,
                "dirty region ({}, {}) {}x{} lies outside the map",
                rect.x, rect.y, rect.width, rect.height
            ),
            Self::CaptureFormat(format) => {
                write!(f, "capture requires Rgba8Unorm output, not {format:?}")
            },
            Self::DevicePoll(message) => write!(f, "device poll failed: {message}"),
            Self::Readback(message) => write!(f, "readback failed: {message}"),
        }
    }
}

impl std::error::Error for LensError {}

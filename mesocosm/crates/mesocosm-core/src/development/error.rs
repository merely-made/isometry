// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::body::AttachError;
use crate::plan::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevelopmentError {
    SomaLength {
        tagmata: usize,
        realised: usize,
    },
    EmptyAxis,
    InvalidAbsence {
        tagma: u8,
        segment: u8,
    },
    /// A layout either describes every stretch or none: absence is the exact
    /// legacy serial layout, and a partial layout would leave a body to guess.
    LayoutLength {
        tagmata: usize,
        layout: usize,
    },
    /// A branch must be rooted in an earlier stretch, so construction remains
    /// ordered and the part dependency graph cannot cycle.
    InvalidLayoutParent {
        tagma: usize,
        parent: usize,
    },
    /// A laid-out body needs its root stretch to exist: the root volume and
    /// every later continuation are defined by that realised first segment.
    LayoutEmptyRoot,
    /// A branch cannot anchor to a stretch this individual did not realise.
    LayoutEmptyParent {
        tagma: usize,
        parent: usize,
    },
    /// Chains either describe every stretch or none. This keeps an omitted
    /// table as the exact direct-appendage legacy program.
    AppendageChainLength {
        tagmata: usize,
        chains: usize,
    },
    /// A finite program may not create an unbounded part graph.
    AppendageChainTooLong {
        tagma: usize,
        steps: usize,
    },
    /// The last chain part is the old appendage, so changing it cannot quietly
    /// change a lineage's feeding anatomy or palette selector.
    AppendageChainTerminal {
        tagma: usize,
    },
    /// Structural links may be bulk or the endpoint's own role. A chain does
    /// not manufacture a new sensor or actuator vocabulary on the way out.
    AppendageChainIntermediate {
        tagma: usize,
        step: usize,
    },
    /// The first link has no prior tangent, so `distal` would otherwise be a
    /// silent, meaningless bit.
    AppendageChainRootDistal {
        tagma: usize,
    },
    WrongRole {
        expected: Role,
        actual: Role,
    },
    /// A `Limb` or `Sensor` shape whose build price exceeds the primitive
    /// palette's, which would move every TD-series rate without moving a
    /// single constant. See [`super::overpriced`].
    Overpriced {
        role: Role,
        half_extent: [i32; 3],
    },
    TooManyParts,
    InsufficientMass {
        mass_mg: u64,
        parts: u32,
    },
    Attach(AttachError),
}

impl From<AttachError> for DevelopmentError {
    fn from(value: AttachError) -> Self {
        Self::Attach(value)
    }
}

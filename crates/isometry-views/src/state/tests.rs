//! Tests for the runner state.
//!
//! Each module below covers the `state/` sibling it is named for, so a
//! module's tests sit beside the module someone is actually reading. They
//! still drive `UiState` as a whole: the seam is the subject under test, not
//! the surface each one touches.
//!
//! Split out of `state.rs` on 2026-07-24, and along the submodule boundaries
//! on 2026-09-04; unchanged both times.

use super::*;
use crate::demo::demo_map;

mod interaction;
mod character;
mod lanes;
mod play;
mod session;
mod surfaces;

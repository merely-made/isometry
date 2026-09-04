//! Integer pixel rounding: laying the board on whole device pixels.
//!
//! The locked isometric lens is a low internal resolution scaled up, so a
//! fractional device pixel anywhere in the tile geometry shows as a seam. The
//! host states the device scale and interface zoom it is drawing under, and
//! this recomputes the board's own scale from them.
//!
//! Split out of `state.rs` on 2026-09-04; unchanged.

use super::*;

/// The board's finest geometric unit in CSS pixels: the elevation step, which
/// is half a tile's height and a quarter of its width.
///
/// Rounding *this* onto a whole device pixel lands the tile height (twice it)
/// and the tile width (four times it) on whole device pixels as well, so an
/// elevation column stacks without a seam. Rounding the tile width instead
/// would leave the height on a half pixel whenever the width came out odd.
pub const BOARD_UNIT: f32 = 8.0;

impl UiState {
    /// State the device scale and interface zoom the board is being drawn
    /// under, and recompute the board's own scale from them.
    ///
    /// The host calls this, from a hook, on a zoom change and at boot: the
    /// window's scale factor and the effective zoom are the host's to know, and
    /// a view that asked for either would be reaching back across the boundary
    /// the migration drew.
    pub fn set_pixel_grid(&mut self, grid: (f32, f32)) {
        self.pixel_grid = grid;
        self.apply_pixel_grid();
    }

    /// Flip the integer-rounding setting and lay the board out under it.
    pub fn toggle_integer_pixel_rounding(&mut self) {
        self.integer_pixel_rounding = !self.integer_pixel_rounding;
        self.apply_pixel_grid();
        self.status = format!(
            "pixel grid: {}",
            if self.integer_pixel_rounding {
                "on"
            } else {
                "off"
            }
        );
    }

    /// Recompute [`Self::board_scale`] and the geometry it drives.
    fn apply_pixel_grid(&mut self) {
        self.board_scale = if self.integer_pixel_rounding {
            board_scale_for(self.pixel_grid)
        } else {
            1.0
        };
        let base = IsoGeometry::default();
        self.geo = IsoGeometry {
            tile_w: base.tile_w * self.board_scale,
            tile_h: base.tile_h * self.board_scale,
            elev_step: base.elev_step * self.board_scale,
        };
    }
}

/// The board scale that lands [`BOARD_UNIT`] on a whole number of device
/// pixels under a `(device scale, interface zoom)` pair.
///
/// `device = BOARD_UNIT * scale * zoom`, rounded to the nearest whole pixel
/// (never below one), divided back. A pair whose product is already whole —
/// every ordinary device scale at zoom 1 — gives exactly `1.0`, so the setting
/// costs nothing at all until a fractional zoom is in play.
fn board_scale_for((scale, zoom): (f32, f32)) -> f32 {
    let device = BOARD_UNIT * scale * zoom;
    if !device.is_finite() || device <= 0.0 {
        return 1.0;
    }
    device.round().max(1.0) / device
}

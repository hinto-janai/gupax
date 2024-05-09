// Free functions.

//---------------------------------------------------------------------------------------------------- Use
use crate::constants::*;

//----------------------------------------------------------------------------------------------------
#[cold]
#[inline(never)]
// Clamp the scaling resolution `f32` to a known good `f32`.
pub fn clamp_scale(scale: f32) -> f32 {
    // Make sure it is finite.
    if !scale.is_finite() {
        return APP_DEFAULT_SCALE;
    }

    // Clamp between valid range.
    scale.clamp(APP_MIN_SCALE, APP_MAX_SCALE)
}

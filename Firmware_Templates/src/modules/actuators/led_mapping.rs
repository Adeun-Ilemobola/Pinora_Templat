//! Pure LED brightness logic, kept free of ESP-IDF types so it can be tested on the
//! host (`tests/firmware-host`).

use crate::utilities::math::range_u32;

/// Highest accepted brightness, in percent.
pub const MAX_LEVEL: u32 = 100;

/// The brightness actually applied for a requested `SetState` value. Requests above
/// [`MAX_LEVEL`] are clamped (PIN-TPL-002, Phase 1B).
pub fn applied_level(requested: u32) -> u32 {
    requested.min(MAX_LEVEL)
}

/// LEDC duty for a level in `0..=MAX_LEVEL`.
pub fn level_to_duty(level: u32, max_duty: u32) -> u32 {
    range_u32(level, 0, MAX_LEVEL, 0, max_duty)
}

/// The level `Toggle` switches to: off goes to full, anything else goes to off.
pub fn toggled_level(current: u32) -> u32 {
    if current == 0 {
        MAX_LEVEL
    } else {
        0
    }
}

/// SysLog text sent when a request was clamped.
pub fn clamp_warning(requested: u32, applied: u32) -> String {
    format!("LED brightness {requested} is out of range 0-{MAX_LEVEL}; applied {applied}")
}

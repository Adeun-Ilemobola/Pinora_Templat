//! Tests for `Firmware_Templates/src/utilities/math.rs`.
//!
//! Phase 1B (PIN-TPL-002) made the integer mappings overflow-safe: they compute in
//! i128 and saturate. The `old_*` reference functions below are the pre-1B formulas
//! with checked arithmetic; wherever they did not overflow, the new results must be
//! identical.

use pinora_firmware_host_tests::utilities::math::*;

/// The LED mapping used by `LedModule::set_state` (13-bit LEDC, max duty 8191).
fn led_duty(state: u32) -> u32 {
    range_u32(state, 0, 100, 0, 8191)
}

fn old_range_u32(v: u32, in_min: u32, in_max: u32, out_min: u32, out_max: u32) -> Option<u32> {
    if in_max == in_min {
        return Some(out_min);
    }
    v.checked_sub(in_min)?
        .checked_mul(out_max.checked_sub(out_min)?)?
        .checked_div(in_max.checked_sub(in_min)?)?
        .checked_add(out_min)
}

fn old_range_i32(v: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> Option<i32> {
    if in_max == in_min {
        return Some(out_min);
    }
    v.checked_sub(in_min)?
        .checked_mul(out_max.checked_sub(out_min)?)?
        .checked_div(in_max.checked_sub(in_min)?)?
        .checked_add(out_min)
}

#[test]
fn range_u32_maps_the_led_percentage_range() {
    assert_eq!(led_duty(0), 0);
    assert_eq!(led_duty(1), 81);
    assert_eq!(led_duty(50), 4095);
    assert_eq!(led_duty(99), 8109);
    assert_eq!(led_duty(100), 8191);
}

#[test]
fn range_u32_extrapolates_above_the_range_without_clamping() {
    // The LED module clamps its input before mapping (Phase 1B); the helper itself
    // extrapolates, as before.
    assert_eq!(led_duty(101), 8272);
    assert_eq!(led_duty(524_352), 42_949_672);
}

#[test]
fn range_u32_no_longer_overflows() {
    // PIN-TPL-002 (fixed in 1B): previously a panic in debug and a wrap to 81 in release.
    assert_eq!(led_duty(524_353), 42_949_754);
    assert_eq!(led_duty(u32::MAX), u32::MAX); // saturates
}

#[test]
fn range_u32_saturates_at_zero_below_the_input_minimum() {
    // Previously an underflow panic in debug.
    assert_eq!(range_u32(5, 10, 20, 0, 100), 0);
    // A descending output range used to underflow too.
    assert_eq!(range_u32(5, 0, 10, 100, 0), 50);
}

#[test]
fn range_u32_with_empty_input_range_returns_output_min() {
    assert_eq!(range_u32(7, 3, 3, 11, 99), 11);
}

#[test]
fn range_u32_matches_the_old_formula_wherever_it_did_not_overflow() {
    let samples = [
        0u32,
        1,
        2,
        7,
        50,
        99,
        100,
        101,
        255,
        4095,
        65_535,
        524_352,
        1 << 20,
        u32::MAX,
    ];
    let mut compared = 0;
    for &v in &samples {
        for &(in_min, in_max) in &[(0u32, 100u32), (0, 180), (10, 20), (0, 4095), (3, 3)] {
            for &(out_min, out_max) in &[(0u32, 8191u32), (500, 2500), (0, 100), (7, 7)] {
                if let Some(old) = old_range_u32(v, in_min, in_max, out_min, out_max) {
                    assert_eq!(
                        range_u32(v, in_min, in_max, out_min, out_max),
                        old,
                        "v={v} in=({in_min},{in_max}) out=({out_min},{out_max})"
                    );
                    compared += 1;
                }
            }
        }
    }
    assert!(compared > 100, "too few comparable cases: {compared}");
}

#[test]
fn range_i32_maps_servo_angles_to_pulse_widths() {
    // Servo mapping shape: angle 0..180 -> 500..2500 us.
    assert_eq!(range_i32(0, 0, 180, 500, 2500), 500);
    assert_eq!(range_i32(90, 0, 180, 500, 2500), 1500);
    assert_eq!(range_i32(180, 0, 180, 500, 2500), 2500);
    // No clamping outside the input range.
    assert_eq!(range_i32(270, 0, 180, 500, 2500), 3500);
    assert_eq!(range_i32(-90, 0, 180, 500, 2500), -500);
    assert_eq!(range_i32(4, 1, 1, -3, 9), -3);
}

#[test]
fn range_i32_saturates_instead_of_overflowing() {
    assert_eq!(range_i32(i32::MAX, 0, 1, 0, i32::MAX), i32::MAX);
    assert_eq!(range_i32(i32::MIN, 0, 1, 0, i32::MAX), i32::MIN);
}

#[test]
fn range_i32_matches_the_old_formula_wherever_it_did_not_overflow() {
    let samples = [
        i32::MIN,
        -1_000_000,
        -180,
        -90,
        -1,
        0,
        1,
        45,
        90,
        180,
        270,
        1_000_000,
        i32::MAX,
    ];
    let mut compared = 0;
    for &v in &samples {
        for &(in_min, in_max) in &[(0i32, 180i32), (-90, 90), (180, 0), (5, 5)] {
            for &(out_min, out_max) in &[(500i32, 2500i32), (-30, 30), (2500, 500)] {
                if let Some(old) = old_range_i32(v, in_min, in_max, out_min, out_max) {
                    assert_eq!(
                        range_i32(v, in_min, in_max, out_min, out_max),
                        old,
                        "v={v} in=({in_min},{in_max}) out=({out_min},{out_max})"
                    );
                    compared += 1;
                }
            }
        }
    }
    assert!(compared > 50, "too few comparable cases: {compared}");
}

#[test]
fn range_i16_maps_and_saturates() {
    // Joystick shape: raw ADC 0..4095 -> -30..30 (unchanged results).
    assert_eq!(range_i16(0, 0, 4095, -30, 30), -30);
    assert_eq!(range_i16(2048, 0, 4095, -30, 30), 0);
    assert_eq!(range_i16(4095, 0, 4095, -30, 30), 30);
    assert_eq!(range_i16(i16::MAX, 0, 4095, -30, 30), 450);
    assert_eq!(range_i16(1, 2, 2, 5, 6), 5);
    // Previously truncated with `as i16` (wrapping); now saturates.
    assert_eq!(range_i16(i16::MAX, 0, 1, 0, i16::MAX), i16::MAX);
    assert_eq!(range_i16(i16::MIN, 0, 1, 0, i16::MAX), i16::MIN);
}

#[test]
fn range_f32_is_linear_and_unclamped() {
    assert_eq!(range_f32(0.5, 0.0, 1.0, 0.0, 100.0), 50.0);
    assert_eq!(range_f32(2.0, 0.0, 1.0, 0.0, 100.0), 200.0);
    assert_eq!(range_f32(1.0, 1.0, 1.0, 3.0, 9.0), 3.0);
}

#[test]
fn constrain_helpers_clamp() {
    assert_eq!(constrain_f32(5.0, 0.0, 1.0), 1.0);
    assert_eq!(constrain_f32(-5.0, 0.0, 1.0), 0.0);
    assert_eq!(constrain_u32(7, 1, 5), 5);
    assert_eq!(constrain_u32(0, 1, 5), 1);
}

#[test]
fn pulse_us_to_tick_assumes_a_60_hz_pca9685_and_clamps() {
    assert_eq!(pulse_us_to_tick(500), 122);
    assert_eq!(pulse_us_to_tick(1500), 368);
    assert_eq!(pulse_us_to_tick(2500), 614);
    assert_eq!(pulse_us_to_tick(-100), 0);
    assert_eq!(pulse_us_to_tick(20_000), 4095);
    // Previously overflowed above 524 287 µs; now clamps.
    assert_eq!(pulse_us_to_tick(i32::MAX), 4095);
    assert_eq!(pulse_us_to_tick(i32::MIN), 0);
}

const X_CENTRE: u16 = 2610;
const DEADZONE: u16 = 200;
fn axis(raw: u16) -> i16 {
    centred_axis(raw, X_CENTRE, DEADZONE, 4095, 30)
}

#[test]
fn a_joystick_axis_is_zero_at_its_centre_and_inside_the_dead_zone() {
    for raw in [X_CENTRE, X_CENTRE - DEADZONE, X_CENTRE + DEADZONE] {
        assert_eq!(axis(raw), 0, "raw {raw}");
    }
}

#[test]
fn a_joystick_axis_follows_the_side_of_its_centre() {
    // 2400 is below the centre: before the fix it reported +5.
    assert!(axis(2400) <= 0);
    assert!(axis(2811) >= 0);
    for raw in 0..=4095u16 {
        let value = axis(raw);
        if raw < X_CENTRE {
            assert!(value <= 0, "raw {raw} gave {value}");
        } else {
            assert!(value >= 0, "raw {raw} gave {value}");
        }
    }
}

#[test]
fn a_joystick_axis_reaches_both_ends_and_does_not_jump_at_the_dead_zone() {
    assert_eq!(axis(0), -30);
    assert_eq!(axis(4095), 30);
    assert!(axis(X_CENTRE - DEADZONE - 1) >= -1);
    assert!(axis(X_CENTRE + DEADZONE + 1) <= 1);
    let mut previous = axis(0);
    for raw in 1..=4095u16 {
        let value = axis(raw);
        assert!(value >= previous, "not monotonic at raw {raw}");
        previous = value;
    }
}

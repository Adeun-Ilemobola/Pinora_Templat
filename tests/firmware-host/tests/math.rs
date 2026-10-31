//! Characterization tests for `Firmware_Templates/src/utilities/math.rs`.
//!
//! These record the current behavior. Cases marked PIN-TPL-002 document a known
//! defect (no range check, overflow in the input type) and will change deliberately
//! when that finding is fixed in Phase 1B.

use pinora_firmware_host_tests::utilities::math::*;

/// The LED mapping used by `LedModule::set_state` (13-bit LEDC, max duty 8191).
fn led_duty(state: u32) -> u32 {
    range_u32(state, 0, 100, 0, 8191)
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
fn range_u32_does_not_clamp_inputs_above_the_range() {
    // PIN-TPL-002: values above 100 map past max duty (the HAL later clamps the duty).
    assert_eq!(led_duty(101), 8272);
    assert_eq!(led_duty(524_352), 42_949_672);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "overflow")]
fn range_u32_overflows_in_debug_builds() {
    // PIN-TPL-002: (value - in_min) * (out_max - out_min) is computed in u32.
    led_duty(524_353);
}

#[test]
#[cfg(not(debug_assertions))]
fn range_u32_wraps_in_release_builds() {
    // PIN-TPL-002: the same input silently wraps to ~1 % duty.
    assert_eq!(led_duty(524_353), 81);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "overflow")]
fn range_u32_underflows_below_the_input_minimum() {
    range_u32(5, 10, 20, 0, 100);
}

#[test]
fn range_u32_with_empty_input_range_returns_output_min() {
    assert_eq!(range_u32(7, 3, 3, 11, 99), 11);
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
fn range_i16_widens_to_i32_and_does_not_overflow() {
    // Joystick shape: raw ADC 0..4095 -> -30..30.
    assert_eq!(range_i16(0, 0, 4095, -30, 30), -30);
    assert_eq!(range_i16(2048, 0, 4095, -30, 30), 0);
    assert_eq!(range_i16(4095, 0, 4095, -30, 30), 30);
    assert_eq!(range_i16(i16::MAX, 0, 4095, -30, 30), 450);
    assert_eq!(range_i16(1, 2, 2, 5, 6), 5);
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
}

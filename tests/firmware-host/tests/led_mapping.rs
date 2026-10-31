//! Tests for the LED brightness logic in
//! `Firmware_Templates/src/modules/actuators/led_mapping.rs` (Phase 1B, PIN-TPL-002).

use pinora_firmware_host_tests::modules::actuators::led_mapping::*;

const MAX_DUTY: u32 = 8191; // 13-bit LEDC timer used by the board

#[test]
fn in_range_requests_are_applied_unchanged() {
    for level in 0..=MAX_LEVEL {
        assert_eq!(applied_level(level), level);
    }
}

#[test]
fn out_of_range_requests_are_clamped_to_100() {
    for requested in [101, 255, 524_352, 524_353, 600_000, u32::MAX] {
        assert_eq!(applied_level(requested), 100, "requested {requested}");
    }
}

#[test]
fn duty_is_monotonic_and_spans_the_full_range() {
    assert_eq!(level_to_duty(0, MAX_DUTY), 0);
    assert_eq!(level_to_duty(50, MAX_DUTY), 4095);
    assert_eq!(level_to_duty(100, MAX_DUTY), MAX_DUTY);
    for level in 1..=MAX_LEVEL {
        assert!(level_to_duty(level, MAX_DUTY) >= level_to_duty(level - 1, MAX_DUTY));
    }
}

#[test]
fn applied_duty_never_exceeds_max_for_any_request() {
    // The end-to-end property the old code violated (it relied on the HAL clamp, and
    // debug builds panicked above 524 352).
    for requested in [0, 1, 100, 101, 524_353, u32::MAX] {
        assert!(level_to_duty(applied_level(requested), MAX_DUTY) <= MAX_DUTY);
    }
}

#[test]
fn toggle_switches_between_off_and_full() {
    assert_eq!(toggled_level(0), 100);
    assert_eq!(toggled_level(1), 0);
    assert_eq!(toggled_level(100), 0);
}

#[test]
fn clamp_warning_names_requested_and_applied_values() {
    assert_eq!(
        clamp_warning(600_000, 100),
        "LED brightness 600000 is out of range 0-100; applied 100"
    );
}

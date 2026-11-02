//! `modules/sensors/nec.rs`: NEC frame decoding from RMT pulses (audit Phase 2C).

use pinora_firmware_host_tests::modules::sensors::nec::{decode, Pulse};

/// A valid NEC frame for `address`/`command`, as an IR receiver outputs it.
fn frame(address: u8, command: u8) -> Vec<Pulse> {
    let mut pulses = vec![(true, 20_000), (false, 9_000), (true, 4_500)];
    for byte in [address, !address, command, !command] {
        for bit in 0..8 {
            pulses.push((false, 560));
            pulses.push((true, if byte >> bit & 1 == 1 { 1_690 } else { 560 }));
        }
    }
    pulses.push((false, 560));
    pulses
}

#[test]
fn decodes_every_command_byte() {
    for command in 0..=u8::MAX {
        assert_eq!(decode(&frame(0x00, command)), Some(command));
    }
}

#[test]
fn tolerates_timing_spread_within_the_old_windows() {
    let mut pulses = frame(0x10, 0x45);
    for (high, micros) in pulses.iter_mut() {
        *micros = match (*high, *micros) {
            (false, 9_000) => 8_200,
            (true, 4_500) => 5_300,
            (false, 560) => 700,
            (true, 1_690) => 2_100,
            (true, 560) => 400,
            (_, other) => other,
        };
    }
    assert_eq!(decode(&pulses), Some(0x45));
}

#[test]
fn rejects_corrupted_frames() {
    // Inverse byte wrong.
    let mut bad = frame(0x00, 0x45);
    let last_bit_high = bad.len() - 2;
    bad[last_bit_high].1 = if bad[last_bit_high].1 == 560 {
        1_690
    } else {
        560
    };
    assert_eq!(decode(&bad), None);
    // Truncated.
    assert_eq!(decode(&frame(0x00, 0x45)[..40]), None);
    // No leader.
    assert_eq!(decode(&frame(0x00, 0x45)[3..]), None);
    // A bit with an out-of-range gap.
    let mut gap = frame(0x00, 0x45);
    gap[4].1 = 3_000;
    assert_eq!(decode(&gap), None);
    assert_eq!(decode(&[]), None);
}

#[test]
fn a_repeat_code_alone_is_not_a_command() {
    // NEC repeat: 9 ms low, 2.25 ms high, 560 µs low.
    assert_eq!(decode(&[(false, 9_000), (true, 2_250), (false, 560)]), None);
}

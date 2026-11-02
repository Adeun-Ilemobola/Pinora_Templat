//! The board's pin assignments as data (audit Phase 2E, PIN-TPL-018).
//!
//! Every GPIO that `main.rs` takes from `Peripherals` must be listed in [`PIN_MAP`];
//! a host test checks that, rejects duplicates, and requires a note for every
//! strapping pin. To wire a new module: add its pins here first.
//!
//! Pure data, so it is tested on the host.

/// One pin assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinUse {
    pub gpio: u8,
    pub owner: &'static str,
    pub note: &'static str,
}

/// Classic ESP32 boot strapping pins. Their level at reset selects boot mode, flash
/// voltage or log output, so external circuits must not force them.
pub const STRAPPING_PINS: [u8; 5] = [0, 2, 5, 12, 15];

/// The current image's pins.
pub const PIN_MAP: [PinUse; 5] = [
    PinUse {
        gpio: 12,
        owner: "led1 (LEDC channel 0)",
        note: "Strapping pin MTDI: held high at reset it selects 1.8 V flash on most modules. \
               Kept because the LEDs are wired here; the attached board booted reliably \
               through many resets in audit Phases 2B and 2C",
    },
    PinUse {
        gpio: 14,
        owner: "led2 (LEDC channel 1)",
        note: "",
    },
    PinUse {
        gpio: 27,
        owner: "led3 (LEDC channel 2)",
        note: "",
    },
    PinUse {
        gpio: 21,
        owner: "I²C0 SDA",
        note: "100 kHz; no consumer in this image",
    },
    PinUse {
        gpio: 22,
        owner: "I²C0 SCL",
        note: "",
    },
];

/// GPIOs listed more than once, sorted.
pub fn duplicate_pins(map: &[PinUse]) -> Vec<u8> {
    let mut gpios: Vec<u8> = map.iter().map(|pin| pin.gpio).collect();
    gpios.sort_unstable();
    let mut duplicates: Vec<u8> = gpios
        .windows(2)
        .filter(|w| w[0] == w[1])
        .map(|w| w[0])
        .collect();
    duplicates.dedup();
    duplicates
}

/// Strapping pins in use without an explanatory note.
pub fn unexplained_strapping_pins(map: &[PinUse]) -> Vec<u8> {
    map.iter()
        .filter(|pin| STRAPPING_PINS.contains(&pin.gpio) && pin.note.is_empty())
        .map(|pin| pin.gpio)
        .collect()
}

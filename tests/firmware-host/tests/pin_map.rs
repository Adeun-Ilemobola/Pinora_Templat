//! `board/pin_map.rs` against the code that takes the pins (audit Phase 2E,
//! PIN-TPL-018).

use pinora_firmware_host_tests::board::pin_map::*;

fn source(path: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../../Firmware_Templates/src/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// GPIO numbers taken from `Peripherals` as `p.pins.gpioN`.
fn taken_pins(text: &str) -> Vec<u8> {
    let mut pins: Vec<u8> = text
        .match_indices("p.pins.gpio")
        .filter_map(|(at, needle)| {
            let digits: String = text[at + needle.len()..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse().ok()
        })
        .collect();
    pins.sort_unstable();
    pins
}

#[test]
fn every_pin_main_takes_is_in_the_map_and_nothing_else() {
    let mut taken = taken_pins(&source("main.rs"));
    taken.dedup();
    let mut mapped: Vec<u8> = PIN_MAP.iter().map(|p| p.gpio).collect();
    mapped.sort_unstable();
    assert_eq!(taken, mapped, "update board/pin_map.rs when wiring changes");
}

#[test]
fn no_pin_is_assigned_twice() {
    assert_eq!(duplicate_pins(&PIN_MAP), Vec::<u8>::new());
    let doubled = [PIN_MAP[0], PIN_MAP[0]];
    assert_eq!(duplicate_pins(&doubled), vec![12]);
}

#[test]
fn strapping_pins_in_use_are_explained() {
    assert_eq!(unexplained_strapping_pins(&PIN_MAP), Vec::<u8>::new());
    let bare = [PinUse {
        gpio: 2,
        owner: "x",
        note: "",
    }];
    assert_eq!(unexplained_strapping_pins(&bare), vec![2]);
}

#[test]
fn the_board_docs_list_the_led_pins() {
    let docs = source("board/mod.rs");
    for pin in PIN_MAP.iter().filter(|p| p.owner.starts_with("led")) {
        let name = pin.owner.split(' ').next().unwrap();
        assert!(
            docs.contains(&format!("| `{name}` | {} |", pin.gpio)),
            "{name} on GPIO{} missing from the board docs",
            pin.gpio
        );
    }
}

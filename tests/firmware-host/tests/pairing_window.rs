//! Tests for `Firmware_Templates/src/core/transport/pairing_window.rs`
//! (Phase 1C, PIN-TPL-003): new Bluetooth pairings only within the window.

use std::time::Duration;

use pinora_firmware_host_tests::core::transport::pairing_window::*;

const fn s(secs: u64) -> Duration {
    Duration::from_secs(secs)
}

#[test]
fn the_default_window_is_120_seconds() {
    assert_eq!(PAIRING_WINDOW, s(120));
}

#[test]
fn pairing_is_rejected_before_the_service_starts() {
    let window = PairingWindow::new(PAIRING_WINDOW);
    assert_eq!(window.state(), WindowState::NotStarted);
    assert!(!window.accepts_pairing(s(0)));
}

#[test]
fn pairing_is_accepted_only_inside_the_window() {
    let mut window = PairingWindow::new(PAIRING_WINDOW);
    assert!(window.open(s(3)));
    assert!(window.accepts_pairing(s(3)));
    assert!(window.accepts_pairing(s(3) + PAIRING_WINDOW - Duration::from_millis(1)));
    assert!(!window.accepts_pairing(s(3) + PAIRING_WINDOW));
    assert!(!window.accepts_pairing(s(10_000)));
}

#[test]
fn expiry_is_reported_exactly_once_and_closes_the_window() {
    let mut window = PairingWindow::new(PAIRING_WINDOW);
    window.open(s(0));
    assert!(!window.poll_expired(s(119)));
    assert!(window.poll_expired(s(120)));
    assert_eq!(window.state(), WindowState::Closed);
    assert!(!window.poll_expired(s(121)), "reported only once");
    assert!(!window.accepts_pairing(s(1)), "closed stays closed");
}

#[test]
fn the_window_opens_only_once_per_boot() {
    let mut window = PairingWindow::new(PAIRING_WINDOW);
    assert!(window.open(s(0)));
    assert!(
        !window.open(s(50)),
        "a second start does not extend the window"
    );
    window.poll_expired(s(120));
    assert!(!window.open(s(200)), "cannot reopen after closing");
    assert!(!window.accepts_pairing(s(200)));
}

#[test]
fn polling_before_start_does_nothing() {
    let mut window = PairingWindow::new(PAIRING_WINDOW);
    assert!(!window.poll_expired(s(1_000)));
    assert_eq!(window.state(), WindowState::NotStarted);
}

#[test]
fn time_going_backwards_does_not_panic() {
    let mut window = PairingWindow::new(s(10));
    window.open(s(5));
    assert!(window.accepts_pairing(s(1)));
    assert!(!window.poll_expired(s(1)));
}

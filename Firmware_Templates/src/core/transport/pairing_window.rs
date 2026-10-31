//! Bluetooth pairing window (PIN-TPL-003, Phase 1C).
//!
//! New devices may pair only during [`PAIRING_WINDOW`] after the SPP service starts.
//! After that the board stops being discoverable and rejects new pairings. Peers
//! already bonded (bonds are stored in NVS) can still connect. The window reopens
//! only on the next boot.
//!
//! This limits *who can pair*. It is not application-level authentication, which is
//! planned for Phase 2B: a device paired during the window can still send commands.
//!
//! Pure (no ESP-IDF): callers pass the elapsed time, so this is tested on the host
//! (`tests/firmware-host`).

use std::time::Duration;

/// How long new devices may pair after the Bluetooth service starts.
pub const PAIRING_WINDOW: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// The service has not started yet; pairing is not accepted.
    NotStarted,
    /// Pairing is accepted until `opened_at + length`.
    Open { opened_at: Duration },
    /// The window has expired; pairing is rejected until reboot.
    Closed,
}

#[derive(Debug, Clone)]
pub struct PairingWindow {
    length: Duration,
    state: WindowState,
}

impl PairingWindow {
    pub const fn new(length: Duration) -> Self {
        Self {
            length,
            state: WindowState::NotStarted,
        }
    }

    pub fn state(&self) -> WindowState {
        self.state
    }

    /// Opens the window at `now`. It opens only once per boot; later calls are
    /// ignored and return `false`.
    pub fn open(&mut self, now: Duration) -> bool {
        if self.state != WindowState::NotStarted {
            return false;
        }
        self.state = WindowState::Open { opened_at: now };
        true
    }

    /// Whether a pairing request arriving at `now` may be accepted.
    pub fn accepts_pairing(&self, now: Duration) -> bool {
        match self.state {
            WindowState::Open { opened_at } => now.saturating_sub(opened_at) < self.length,
            WindowState::NotStarted | WindowState::Closed => false,
        }
    }

    /// Returns `true` exactly once: the first time it is called at or after expiry.
    /// The caller then makes the device non-discoverable.
    pub fn poll_expired(&mut self, now: Duration) -> bool {
        match self.state {
            WindowState::Open { opened_at } if now.saturating_sub(opened_at) >= self.length => {
                self.state = WindowState::Closed;
                true
            }
            _ => false,
        }
    }
}

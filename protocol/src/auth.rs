//! Device authentication (protocol v2, audit Phase 2B, PIN-TPL-003 / PIN-TPL-013).
//!
//! Wire types are always compiled. The cryptography and the two session state
//! machines ([`DeviceSession`] for the firmware, [`ClientSession`] for the desktop)
//! need the `auth` feature, so the base crate stays dependency-free.
//!
//! Handshake (mutual HMAC-SHA256 with a per-device 256-bit key `K`):
//! 1. desktop → `System(AuthBegin { client_nonce })`
//! 2. device → `Auth(Challenge { device_id, device_nonce, device_proof })`, or
//!    `Auth(Unprovisioned)` when the device has no key
//! 3. desktop verifies `device_proof`, then → `System(AuthResponse { proof })`
//! 4. device → `Auth(Result { authenticated, reason })`
//! 5. every later command is a [`SignedCommand`]: the exact command JSON as a string,
//!    a strictly increasing `seq`, and `mac = HMAC(S, seq ‖ command)` under the
//!    session key `S`.
//!
//! Every HMAC input is a domain label followed by length-prefixed parts, so no two
//! different inputs encode alike. Keys and nonces are lowercase hex on the wire.

use serde::{Deserialize, Serialize};

use crate::{IncomingCommand, ModuleCommand, SystemCommand};

pub const KEY_BYTES: usize = 32;
pub const NONCE_BYTES: usize = 16;
pub const MAC_BYTES: usize = 32;

/// Authentication messages from the device. Tauri consumes them; they never reach
/// the WebView.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMessage {
    /// Answer to `AuthBegin` from a device that has a key.
    Challenge {
        device_id: String,
        device_nonce: String,
        device_proof: String,
    },
    /// Answer to `AuthBegin` from a device without a key: commands are not
    /// authenticated.
    Unprovisioned { device_id: String },
    /// Outcome of `AuthResponse`, or of a malformed handshake step.
    Result { authenticated: bool, reason: String },
    /// The device key, sent **only** over the USB serial console in answer to
    /// `ProvisionKey`.
    Key { device_id: String, key: String },
    /// Answer to `ClearKey` (USB serial console only).
    KeyCleared { device_id: String },
}

/// An authenticated command line: `{"seq":n,"mac":"…","signed":"<IncomingCommand JSON>"}`.
/// The MAC covers the exact `signed` text, so no JSON canonicalization is needed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedCommand {
    pub seq: u32,
    pub mac: String,
    pub signed: String,
}

/// One command line as received by a device transport.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandLine {
    Plain(IncomingCommand),
    Signed(SignedCommand),
}

/// Parses a command line: a plain [`IncomingCommand`] or a [`SignedCommand`]. When
/// neither matches, the error is the one for the plain form.
pub fn parse_command_line(line: &[u8]) -> Result<CommandLine, serde_json::Error> {
    match serde_json::from_slice::<IncomingCommand>(line) {
        Ok(command) => Ok(CommandLine::Plain(command)),
        Err(plain_error) => serde_json::from_slice::<SignedCommand>(line)
            .map(CommandLine::Signed)
            .map_err(|_| plain_error),
    }
}

/// True for the system commands that only the USB serial console may carry.
pub fn is_provisioning_command(command: &IncomingCommand) -> bool {
    matches!(
        command.command,
        ModuleCommand::System(SystemCommand::ProvisionKey {} | SystemCommand::ClearKey {})
    )
}

/// Lowercase hex encoding.
pub fn to_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Decodes hex of exactly `N` bytes (either case). `None` on any other input.
pub fn from_hex<const N: usize>(text: &str) -> Option<[u8; N]> {
    let digits = text.as_bytes();
    if digits.len() != N * 2 {
        return None;
    }
    let value = |digit: u8| match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        _ => None,
    };
    let mut out = [0u8; N];
    for (index, pair) in digits.as_chunks::<2>().0.iter().enumerate() {
        out[index] = (value(pair[0])? << 4) | value(pair[1])?;
    }
    Some(out)
}

#[cfg(feature = "auth")]
mod session;

#[cfg(feature = "auth")]
pub use session::*;

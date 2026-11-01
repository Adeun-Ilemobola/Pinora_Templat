//! Serial (UART0 stdio) transport output: newline-delimited JSON on the console.
//!
//! Protocol output shares UART0 with ESP-IDF log lines; the desktop ignores lines
//! that are not JSON objects. This transport has no registration cache; since audit
//! Phase 2A the desktop requests discovery with `Rediscover` (PIN-TPL-012). Input is
//! read by [`super::console`], which runs in every image since Phase 2B.

use pinora_protocol::registration::ProtocolMessage;

/// Writes one protocol message as a JSON line on stdout.
pub fn send(data: &ProtocolMessage) -> Result<(), String> {
    let serialized = serde_json::to_string(data).map_err(|error| error.to_string())?;
    println!("{serialized}");
    Ok(())
}

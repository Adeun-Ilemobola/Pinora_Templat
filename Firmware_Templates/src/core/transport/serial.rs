//! Serial (UART0 stdio) transport: newline-delimited JSON over the console.
//!
//! Protocol output shares UART0 with ESP-IDF log lines; the desktop ignores lines
//! that are not JSON objects. This transport has no registration cache, so a
//! desktop that connects after boot sees no modules unless the board resets
//! (PIN-TPL-012).

use pinora_protocol::{registration::ProtocolMessage, IncomingCommand};
use std::io::{self, BufRead, ErrorKind};
use std::sync::mpsc;

/// Writes one protocol message as a JSON line on stdout.
pub fn send(data: &ProtocolMessage) -> Result<(), String> {
    let serialized = serde_json::to_string(data).map_err(|error| error.to_string())?;
    println!("{serialized}");
    Ok(())
}

/// Blocking stdin reader; runs on its own thread for the life of the firmware.
pub fn command_reader(command_sender: &mpsc::Sender<IncomingCommand>) {
    let stdin = io::stdin();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                // Not an error — just no data yet. Back off and retry.
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Err(err) => {
                log::error!("Real serial read error: {:?}", err);
                continue;
            }
        };

        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<IncomingCommand>(line) {
            Ok(command) => {
                log::info!("Parsed command: {:?}", command);

                let _ = command_sender.send(command);
            }

            Err(err) => {
                log::error!("Failed to parse command: {:?}", err);
                log::error!("Raw line was: {}", line);
            }
        }
    }
}

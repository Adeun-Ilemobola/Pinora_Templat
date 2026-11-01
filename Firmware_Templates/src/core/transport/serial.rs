//! Serial (UART0 stdio) transport: newline-delimited JSON over the console.
//!
//! Protocol output shares UART0 with ESP-IDF log lines; the desktop ignores lines
//! that are not JSON objects. This transport has no registration cache; since audit
//! Phase 2A the desktop requests discovery with `Rediscover` (PIN-TPL-012). Input is
//! framed by the shared [`LineFramer`] (Phase 2B), so a line can no longer grow
//! without bound.

use pinora_protocol::{
    framing::{Frame, LineFramer, MAX_LINE_BYTES},
    registration::ProtocolMessage,
    IncomingCommand,
};
use std::io::{self, ErrorKind, Read};
use std::sync::mpsc;
use std::time::Duration;

/// Writes one protocol message as a JSON line on stdout.
pub fn send(data: &ProtocolMessage) -> Result<(), String> {
    let serialized = serde_json::to_string(data).map_err(|error| error.to_string())?;
    println!("{serialized}");
    Ok(())
}

/// Blocking stdin reader; runs on its own thread for the life of the firmware.
pub fn command_reader(command_sender: &mpsc::Sender<IncomingCommand>) {
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    let mut buffer = [0u8; 256];

    loop {
        let count = match io::stdin().lock().read(&mut buffer) {
            Ok(0) => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Ok(count) => count,
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                // Not an error — just no data yet. Back off and retry.
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(err) => {
                log::error!("Real serial read error: {:?}", err);
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        };

        for frame in framer.push(&buffer[..count]) {
            let line = match frame {
                Frame::Line(line) => line,
                Frame::Overflow => {
                    log::error!("Serial command longer than {MAX_LINE_BYTES} bytes; discarded");
                    continue;
                }
            };
            match serde_json::from_slice::<IncomingCommand>(&line) {
                Ok(command) => {
                    log::info!("Parsed command: {:?}", command);

                    let _ = command_sender.send(command);
                }

                Err(err) => {
                    log::error!("Failed to parse command: {:?}", err);
                    log::error!("Raw line was: {}", String::from_utf8_lossy(&line));
                }
            }
        }
    }
}

//! The USB serial console reader. It runs in **every** image (audit Phase 2B,
//! PIN-TPL-003):
//! * `ProvisionKey` and `ClearKey` are handled here and answered on the console only.
//!   USB means physical access; the radio transports never accept them.
//! * In the Serial image every other command is forwarded to the main loop, as the
//!   Serial transport's reader did before 2B. Radio images ignore them.
//!
//! Lines are framed by the shared [`LineFramer`]. The key is written to the console
//! once, as an `Auth(Key)` line for the desktop, and never logged.

use std::io::{self, ErrorKind, Read};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use pinora_protocol::{
    framing::{Frame, LineFramer, MAX_LINE_BYTES},
    AuthMessage, IncomingCommand, ProtocolMessage,
};

use crate::core::transport::auth_gate::{route_console_line, ConsoleRoute};
use crate::core::transport::device_key::KeyStore;
use crate::core::transport::serial;

/// Starts the console reader. `forward` is `Some` in the Serial image.
pub fn spawn(
    keys: Arc<KeyStore>,
    forward: Option<mpsc::Sender<IncomingCommand>>,
) -> anyhow::Result<()> {
    thread::Builder::new()
        .name("usb-console".into())
        .stack_size(8192)
        .spawn(move || run(&keys, forward.as_ref()))?;
    Ok(())
}

fn run(keys: &KeyStore, forward: Option<&mpsc::Sender<IncomingCommand>>) {
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    let mut buffer = [0u8; 256];

    loop {
        let count = match io::stdin().lock().read(&mut buffer) {
            Ok(0) => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Ok(count) => count,
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                // Not an error — just no data yet. Back off and retry.
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(err) => {
                log::error!("Real serial read error: {:?}", err);
                thread::sleep(Duration::from_millis(10));
                continue;
            }
        };

        for frame in framer.push(&buffer[..count]) {
            let line = match frame {
                Frame::Line(line) => line,
                Frame::Overflow => {
                    log::error!("Console line longer than {MAX_LINE_BYTES} bytes; discarded");
                    continue;
                }
            };
            match route_console_line(&line, forward.is_some()) {
                Ok(ConsoleRoute::ProvisionKey) => reply(match keys.provision() {
                    Ok(key) => {
                        log::info!("Device key provided over USB");
                        AuthMessage::Key {
                            device_id: keys.device_id().to_string(),
                            key: key.to_hex(),
                        }
                    }
                    Err(error) => failure(format!("Key provisioning failed: {error}")),
                }),
                Ok(ConsoleRoute::ClearKey) => reply(match keys.clear() {
                    Ok(()) => {
                        log::warn!("Device key cleared over USB; radio commands are no longer authenticated");
                        AuthMessage::KeyCleared {
                            device_id: keys.device_id().to_string(),
                        }
                    }
                    Err(error) => failure(format!("Clearing the key failed: {error}")),
                }),
                Ok(ConsoleRoute::Forward(command)) => {
                    log::info!("Parsed command: {:?}", command);
                    if let Some(sender) = forward {
                        let _ = sender.send(command);
                    }
                }
                Ok(ConsoleRoute::Ignore(reason)) => log::warn!("Console line ignored: {reason}"),
                Err(err) => {
                    log::error!("Failed to parse command: {:?}", err);
                    log::error!("Raw line was: {}", String::from_utf8_lossy(&line));
                }
            }
        }
    }
}

fn failure(reason: String) -> AuthMessage {
    AuthMessage::Result {
        authenticated: false,
        reason,
    }
}

fn reply(message: AuthMessage) {
    if let Err(error) = serial::send(&ProtocolMessage::Auth(message)) {
        log::error!("Could not answer on the console: {error}");
    }
}

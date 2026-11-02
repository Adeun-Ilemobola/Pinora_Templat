//! COM-port transport (USB serial or a paired Bluetooth SPP virtual port).
//!
//! One port is open at a time. A reader thread owns a cloned handle. Lines are framed
//! by the shared [`LineFramer`] (bounded at 8 KiB since Phase 2B; unbounded before,
//! PIN-TPL-016) and classified by [`crate::line_classifier`]: protocol messages go
//! out as `espState`, other lines as `espConsole` (Phase 1C), and authentication
//! messages are handled here and never forwarded (Phase 2B).
//!
//! Authentication (Phase 2B, PIN-TPL-003): writes go through one [`ComLink`], which
//! holds the port and its [`AuthLink`]. A Bluetooth link starts the handshake as soon
//! as the reader runs and then signs every command; a USB link never authenticates
//! but is the only one that can pair a device.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use serialport::SerialPort;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::JoinHandle;

use pinora_protocol::auth::AuthStatus;
use pinora_protocol::framing::{Frame, LineFramer, MAX_LINE_BYTES};

use crate::auth_link::{random_nonce, AuthLink, LinkKind, Outcome, AUTH_EVENT};
use crate::device_keys::DeviceKeys;
use crate::events::{emit_serialized, EventSink};
use crate::line_classifier::{classify, ComLine};
use std::io::{Read, Write};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TransportError {
    ConnectionFailed {
        message: String,
        raw_error: Option<String>,
    },
}

/// Returned to the frontend by `start_port` on success (serialized as `"Connected"`;
/// failures return the error message since Phase 3A).
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) enum ConnectionType {
    Connected,
}

/// Which kind of COM port is open; only USB serial gets the DTR/RTS reset pulse.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) enum TransportType {
    Serial,
    Bluetooth,
}

impl TransportType {
    /// The event that reports this transport's connection loss (contract C-07).
    /// `serialConnection` was added in Phase 2A (PIN-TPL-012); before that a USB
    /// disconnect was not reported at all.
    pub(crate) fn connection_event(self) -> &'static str {
        match self {
            TransportType::Serial => "serialConnection",
            TransportType::Bluetooth => "bluetoothConnection",
        }
    }

    fn link_kind(self) -> LinkKind {
        match self {
            TransportType::Serial => LinkKind::UsbSerial,
            TransportType::Bluetooth => LinkKind::Bluetooth,
        }
    }
}

/// The write half of the open COM port and its authentication state.
pub(crate) struct ComLink {
    writer: Box<dyn SerialPort>,
    pub(crate) auth: AuthLink,
}

impl ComLink {
    /// Writes one complete line (it must end with `\n`).
    pub(crate) fn write_line(&mut self, line: &str) -> std::io::Result<()> {
        self.writer.write_all(line.as_bytes())?;
        self.writer.flush()
    }
}

/// Managed Tauri state for the COM-port connection.
#[derive(Default)]
pub(crate) struct AppState {
    reader_thread: Option<JoinHandle<()>>,
    reader_running: Arc<AtomicBool>,
    pub(crate) link: Option<Arc<Mutex<ComLink>>>,
    pub(crate) name: String,
    pub(crate) rate: u32,
    keys: Arc<Mutex<DeviceKeys>>,
}
impl AppState {
    pub(crate) fn new(keys: Arc<Mutex<DeviceKeys>>) -> Self {
        Self {
            reader_thread: None,
            reader_running: Arc::new(AtomicBool::new(false)),
            link: None,
            name: String::new(),
            rate: 0,
            keys,
        }
    }
    pub(crate) fn shutdown(&mut self) {
        let had_connection = self.link.is_some() || self.reader_thread.is_some();

        self.reader_running.store(false, Ordering::Relaxed);

        // The reader holds the other reference to the link; it is released when the
        // reader thread ends.
        self.link.take();

        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }

        if had_connection {
            log::info!("Shutting down connection");
        }
    }

    /// The current link's authentication status (`Idle` when no port is open).
    pub(crate) fn auth_status(&self) -> AuthStatus {
        self.link
            .as_ref()
            .and_then(|link| link.lock().ok().map(|link| link.auth.status()))
            .unwrap_or(AuthStatus::Idle)
    }

    /// Opens `self.name` at `self.rate` and starts the reader thread. Any previous
    /// connection is shut down first.
    pub fn connect(
        &mut self,
        app: Arc<AppHandle>,
        transport: TransportType,
    ) -> Result<(), TransportError> {
        self.shutdown();

        if self.name.is_empty() || self.rate == 0 {
            return Err(TransportError::ConnectionFailed {
                message: "Serial port name or rate is not set".to_string(),
                raw_error: None,
            });
        }
        let mut port = serialport::new(self.name.clone(), self.rate)
            .timeout(Duration::from_millis(10))
            .open()
            .map_err(|e| TransportError::ConnectionFailed {
                message: "Failed to open serial port".to_string(),
                raw_error: Some(e.to_string()),
            })?;
        if matches!(transport, TransportType::Serial) {
            // USB control-line sequencing can reset the board; SPP virtual ports skip it.
            port.write_data_terminal_ready(false).map_err(|e| {
                TransportError::ConnectionFailed {
                    message: "Failed to set DTR".to_string(),
                    raw_error: Some(e.to_string()),
                }
            })?;
            port.write_request_to_send(true)
                .map_err(|e| TransportError::ConnectionFailed {
                    message: "Failed to assert RTS".to_string(),
                    raw_error: Some(e.to_string()),
                })?;
            std::thread::sleep(Duration::from_millis(100));
            port.write_request_to_send(false)
                .map_err(|e| TransportError::ConnectionFailed {
                    message: "Failed to release RTS".to_string(),
                    raw_error: Some(e.to_string()),
                })?;
        }
        let mut reader_port = port
            .try_clone()
            .map_err(|e| TransportError::ConnectionFailed {
                message: "Failed to clone serial port".to_string(),
                raw_error: Some(e.to_string()),
            })?;

        let link = Arc::new(Mutex::new(ComLink {
            writer: port,
            auth: AuthLink::new(transport.link_kind()),
        }));
        let running = Arc::clone(&self.reader_running);
        let reader_link = Arc::clone(&link);
        let reader_keys = Arc::clone(&self.keys);
        let reader_app = Arc::clone(&app);

        running.store(true, Ordering::Relaxed);
        // The cloned handle owns reads; complete lines are classified and forwarded.
        let reader_thread = std::thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut framer = LineFramer::new(MAX_LINE_BYTES);

            while running.load(Ordering::Relaxed) {
                let count = match reader_port.read(&mut buffer) {
                    Ok(count) => count,
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                        ) =>
                    {
                        0
                    }
                    Err(error) => {
                        // A read error after a deliberate shutdown is not reported.
                        log::warn!("Port read failed: {error}");
                        if running.load(Ordering::Relaxed) {
                            let _ = reader_app.emit(transport.connection_event(), "Disconnected");
                        }
                        break;
                    }
                };
                for frame in framer.push(&buffer[..count]) {
                    match frame {
                        Frame::Line(line) => {
                            handle_line(&line, reader_app.as_ref(), &reader_link, &reader_keys)
                        }
                        Frame::Overflow => {
                            log::warn!("COM line longer than {MAX_LINE_BYTES} bytes; discarded")
                        }
                    }
                }
            }
        });

        self.link = Some(Arc::clone(&link));
        self.reader_thread = Some(reader_thread);

        // Start the handshake on Bluetooth once the reader is listening.
        let nonce = random_nonce().map_err(|error| TransportError::ConnectionFailed {
            message: "Could not generate a nonce".to_string(),
            raw_error: Some(error),
        })?;
        let mut guard = link.lock().map_err(|_| TransportError::ConnectionFailed {
            message: "Connection state poisoned".to_string(),
            raw_error: None,
        })?;
        if let Some(begin) = guard.auth.begin(nonce) {
            guard
                .write_line(&begin)
                .map_err(|e| TransportError::ConnectionFailed {
                    message: "Failed to start authentication".to_string(),
                    raw_error: Some(e.to_string()),
                })?;
            emit_serialized(app.as_ref(), AUTH_EVENT, &guard.auth.status());
        }
        drop(guard);

        Ok(())
    }
}

/// Forwards one COM line to the WebView, or handles it here if it is an `Auth`
/// message (handshake replies, pairing results). Lock order: link, then keys.
pub(crate) fn handle_line<S: EventSink>(
    line: &[u8],
    sink: &S,
    link: &Mutex<ComLink>,
    keys: &Mutex<DeviceKeys>,
) {
    let classified = classify(line);
    let ComLine::Auth(message) = classified else {
        if let (Some(event), Some(text)) = (classified.event_name(), classified.into_text()) {
            sink.emit_text(event, text);
        }
        return;
    };
    let Ok(mut link) = link.lock() else {
        return;
    };
    let outcomes = match keys.lock() {
        Ok(mut keys) => link.auth.on_auth_message(message, &mut keys),
        Err(_) => return,
    };
    for outcome in outcomes {
        match outcome {
            Outcome::Reply(reply) => {
                if let Err(error) = link.write_line(&reply) {
                    log::warn!("Could not answer the device: {error}");
                }
            }
            Outcome::Status(status) => emit_serialized(sink, AUTH_EVENT, &status),
            Outcome::Pairing(event) => emit_serialized(sink, AUTH_EVENT, &event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TransportType;

    #[test]
    fn each_transport_reports_its_own_disconnect_event() {
        assert_eq!(TransportType::Serial.connection_event(), "serialConnection");
        assert_eq!(
            TransportType::Bluetooth.connection_event(),
            "bluetoothConnection"
        );
    }
}

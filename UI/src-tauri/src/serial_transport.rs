//! COM-port transport (USB serial or a paired Bluetooth SPP virtual port).
//!
//! One port is open at a time. A reader thread owns a cloned handle and forwards
//! every complete newline-terminated line to the frontend as an `espState` event,
//! *without* validating it (PIN-TPL-006/007); the Wi-Fi transport validates instead.
//! The line buffer is unbounded (PIN-TPL-016).

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use serialport::SerialPort;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::JoinHandle;

use std::io::Read;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TransportError {
    ConnectionFailed {
        message: String,
        raw_error: Option<String>,
    },
}

pub(crate) struct ConnectionState {
    connection_type: ConnectionType,
    transport_type: Option<TransportType>,
    error: Option<String>,
}

/// Returned to the frontend by `start_port` (serialized as `"Connected"`/`"Error"`).
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) enum ConnectionType {
    Connected,
    Disconnected,
    Error,
}

/// Which kind of COM port is open; only USB serial gets the DTR/RTS reset pulse.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) enum TransportType {
    Serial,
    Bluetooth,
}

/// Managed Tauri state for the COM-port connection.
#[derive(Default)]
pub(crate) struct AppState {
    reader_thread: Option<JoinHandle<()>>,
    reader_running: Arc<AtomicBool>,
    pub(crate) port: Option<Box<dyn SerialPort>>,
    pub(crate) name: String,
    pub(crate) rate: u32,
}
impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            reader_thread: None,
            reader_running: Arc::new(AtomicBool::new(false)),
            port: None,
            name: String::new(),
            rate: 0,
        }
    }
    pub(crate) fn shutdown(&mut self) {
        let had_connection = self.port.is_some() || self.reader_thread.is_some();

        self.reader_running.store(false, Ordering::Relaxed);

        // Closing the writer releases the COM handle before joining the reader thread.
        self.port.take();

        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }

        if had_connection {
            log::info!("Shutting down connection");
        }
    }

    /// Opens `self.name` at `self.rate` and starts the reader thread. Any previous
    /// connection is shut down first.
    pub fn connect(
        &mut self,
        app: Arc<AppHandle>,
        transport: TransportType,
    ) -> Result<ConnectionState, TransportError> {
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

        let running = Arc::clone(&self.reader_running);

        running.store(true, Ordering::Relaxed);
        // The cloned handle owns reads; complete JSON lines are forwarded to React as events.
        let reader_thread = std::thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut line_buff: Vec<u8> = vec![];

            while running.load(Ordering::Relaxed) {
                let bytes_read = match reader_port.read(&mut buffer) {
                    Ok(count) if count > 0 => buffer[..count].to_vec(),
                    Ok(_) => Vec::new(),
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                        ) =>
                    {
                        Vec::new()
                    }
                    Err(error) => {
                        // Only Bluetooth reports the loss to the UI (PIN-TPL-012).
                        log::warn!("Port read failed: {error}");
                        if matches!(transport, TransportType::Bluetooth)
                            && running.load(Ordering::Relaxed)
                        {
                            let _ = app.emit("bluetoothConnection", "Disconnected");
                        }
                        break;
                    }
                };
                if bytes_read.is_empty() {
                    continue;
                }

                line_buff.extend_from_slice(&bytes_read);

                while let Some(index) = line_buff.iter().position(|byte| *byte == b'\n') {
                    let line: Vec<u8> = line_buff.drain(..=index).collect();
                    let line_str = String::from_utf8_lossy(&line);
                    if let Err(e) = app.emit("espState", line_str.to_string()) {
                        log::error!("Failed to emit espState event: {:?}", e);
                    }
                }
            }
        });

        self.port = Some(port);
        self.reader_thread = Some(reader_thread);
        Ok(ConnectionState {
            connection_type: ConnectionType::Connected,
            transport_type: Some(transport),
            error: None,
        })
    }
}

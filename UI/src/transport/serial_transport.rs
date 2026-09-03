use crate::{
    transport::transport_type::{ConnectionState, ConnectionType, TransportError, TransportType},
    type_box::EventCallback,
};
use serialport::SerialPort;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct SerialTransport {
    name: String,
    rate: u32,

    serial: Option<Box<dyn SerialPort>>,
    event_callback: EventCallback,
    reader_thread: Option<JoinHandle<()>>,
    reader_running: Arc<AtomicBool>,
}

impl SerialTransport {
    pub fn new(event_callback: EventCallback) -> Self {
        SerialTransport {
            name: "".to_string(),
            rate: 0,
            serial: None,
            event_callback,
            reader_running: Arc::new(AtomicBool::new(false)),
            reader_thread: None,
        }
    }

    fn shutdown(&mut self) {
        self.reader_running.store(false, Ordering::Relaxed);
        self.serial.take();

        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }
    }

    pub fn set_port(&mut self, name: String, rate: u32) {
        self.name = name;
        self.rate = rate;
    }

    pub fn get_available_ports() -> Result<Vec<String>, TransportError> {
        let ports = serialport::available_ports().map_err(|e| TransportError::Unknown {
            message: "Failed to enumerate serial ports".to_string(),
            raw_error: Some(e.to_string()),
        })?;

        Ok(ports.into_iter().map(|port| port.port_name).collect())
    }

    pub fn disconnect(&mut self) -> Result<(), TransportError> {
        self.shutdown();
        Ok(())
    }

    pub fn connect(&mut self) -> Result<ConnectionState, TransportError> {
        self.shutdown();

        if self.name.is_empty() || self.rate == 0 {
            return Err(TransportError::ConnectionFailed {
                message: "Serial port name or rate is not set".to_string(),
                raw_error: None,
            });
        }
        let port = serialport::new(self.name.clone(), self.rate)
            .timeout(Duration::from_millis(10))
            .open()
            .map_err(|e| TransportError::ConnectionFailed {
                message: "Failed to open serial port".to_string(),
                raw_error: Some(e.to_string()),
            })?;

        let mut reader_port = port
            .try_clone()
            .map_err(|e| TransportError::ConnectionFailed {
                message: "Failed to clone serial port".to_string(),
                raw_error: Some(e.to_string()),
            })?;

        let running = Arc::clone(&self.reader_running);
        let callback = Arc::clone(&self.event_callback);

        running.store(true, Ordering::Relaxed);
        let reader_thread = std::thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut line_buff: Vec<u8> = vec![];

            while running.load(Ordering::Relaxed) {
                let bytes_read = match reader_port.read(&mut buffer) {
                    Ok(count) if count > 0 => buffer[..count].to_vec(),
                    _ => Vec::new(),
                };
                if bytes_read.is_empty() {
                    continue;
                }

                line_buff.extend_from_slice(&bytes_read);

                while let Some(index) = line_buff.iter().position(|byte| *byte == b'\n') {
                    let line: Vec<u8> = line_buff.drain(..=index).collect();
                    callback(line);
                }
            }
        });

        self.serial = Some(port);
        self.reader_thread = Some(reader_thread);
        Ok(ConnectionState {
            connection_type: ConnectionType::Connected,
            transport_type: Some(TransportType::Serial),
            error: None,
        })
    }
}

impl Drop for SerialTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

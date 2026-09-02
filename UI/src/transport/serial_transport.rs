use crate::transport::transport_type::{
    ConnectionState, ConnectionType, TransportError, TransportType,
};
use serialport::SerialPort;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct SerialTransport {
    pub name: String,
    pub rate: u32,
    pub serial_buf: Vec<u8>,

    pub serial: Option<Box<dyn SerialPort>>,
    pub event_callback: Arc<Mutex<Box<dyn FnMut(Vec<u8>) + Send + 'static>>>,
    pub reader_thread: Option<JoinHandle<()>>,
    pub reader_running: Arc<AtomicBool>,
}

impl SerialTransport {
    pub fn new(event_callback: Box<dyn FnMut(Vec<u8>) + Send + 'static>) -> Self {
        SerialTransport {
            name: "".to_string(),
            rate: 0,
            serial_buf: Vec::new(),
            serial: None,
            event_callback: Arc::new(Mutex::new(event_callback)),
            reader_running: Arc::new(AtomicBool::new(true)),
            reader_thread: None,
        }
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_rate(&self) -> u32 {
        self.rate
    }

    pub fn stop_reader(&mut self) {
        self.reader_running.store(false, Ordering::Relaxed);

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
        // Implement the disconnection logic here
        self.stop_reader();
        self.serial
            .take()
            .ok_or_else(|| TransportError::ConnectionFailed {
                message: "Serial port is not open".to_string(),
                raw_error: None,
            })?;

        Ok(())
    }
    pub fn connect(&mut self) -> Result<ConnectionState, TransportError> {
        // Implement the connection logic here
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
                let buffer_ves = match reader_port.read(&mut buffer) {
                    Ok(count) if count > 0 => {
                        let data = buffer[..count].to_vec();
                        data
                    }
                    _ => Vec::new(),
                };
                if buffer_ves.is_empty() {
                    continue;
                }
                line_buff.extend(buffer_ves.clone());
                let is_new_line = match line_buff.last() {
                    Some(newline_byte) => *newline_byte == 10 || *newline_byte == 0x0A,
                    _ => false,
                };
                if is_new_line {
                    if let Ok(mut callback) = callback.lock() {
                        (callback)(line_buff.clone());
                    }
                    match String::from_utf8(line_buff.clone()) {
                        Ok(string) => println!("Success: {string}"),
                        Err(e) => println!("Invalid UTF-8 sequence: {e}"),
                    }
                    line_buff.clear();
                }

                buffer = [0; 1024];
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

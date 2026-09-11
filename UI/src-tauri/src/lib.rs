use log::Log;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tauri::{Manager, State};

use serialport::SerialPort;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::JoinHandle;

use std::io::Read;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TransportError {
    ConnectionFailed {
        message: String,
        raw_error: Option<String>,
    },
}

struct ConnectionState {
    connection_type: ConnectionType,
    transport_type: Option<TransportType>,
    error: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize)]
enum ConnectionType {
    Connected,
    Disconnected,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum TransportType {
    Serial,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Default)]
struct AppState {
    reader_thread: Option<JoinHandle<()>>,
    reader_running: Arc<AtomicBool>,
    port: Option<Box<dyn SerialPort>>,
    name: String,
    rate: u32,
}
impl AppState {
    fn new() -> Self {
        Self {
            reader_thread: None,
            reader_running: Arc::new(AtomicBool::new(false)),
            port: None,
            name: String::new(),
            rate: 0,
        }
    }
    fn shutdown(&mut self) {
        let had_connection = self.port.is_some() || self.reader_thread.is_some();

        self.reader_running.store(false, Ordering::Relaxed);

        self.port.take();

        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }

        if had_connection {
            log::info!("Shutting down connection");
        }
    }

    pub fn connect(&mut self, app: Arc<AppHandle>) -> Result<ConnectionState, TransportError> {
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
        port.write_data_terminal_ready(false)
            .map_err(|e| TransportError::ConnectionFailed {
                message: "Failed to set DTR".to_string(),
                raw_error: Some(e.to_string()),
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
        let mut reader_port = port
            .try_clone()
            .map_err(|e| TransportError::ConnectionFailed {
                message: "Failed to clone serial port".to_string(),
                raw_error: Some(e.to_string()),
            })?;

        let running = Arc::clone(&self.reader_running);

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
            transport_type: Some(TransportType::Serial),
            error: None,
        })
    }
}

#[tauri::command]
fn get_available_ports() -> Result<Vec<String>, String> {
    match serialport::available_ports() {
        Ok(ports) => Ok(ports.into_iter().map(|p| p.port_name).collect()),
        Err(e) => {
            log::error!("Failed to get available ports: {}", e);
            Err(format!("Failed to get available ports: {}", e))
        }
    }
}

#[tauri::command]
fn send_data(state: State<'_, Mutex<AppState>>, data: String) -> Result<(), String> {
    let mut state = state.lock().unwrap();
  
    if let Some(port) = &mut state.port {
        port.write_all(format!("{}\n", data).as_bytes()).map_err(|e| {
            log::error!("Failed to send data: {}", e);
            format!("Failed to send data: {}", e)
        })?;

        port.flush().map_err(|e| {
            log::error!("Failed to flush data: {}", e);
            format!("Failed to flush data: {}", e)
        })?;
    }
    Ok(())
}

// remember to call `.manage(MyState::default())`
#[tauri::command]
fn start_port(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    name: &str,
    rate: u32,
) -> Result<ConnectionType, ConnectionType> {
    let mut state = state.lock().unwrap();
    let appS = Arc::new(app);

    state.name = name.to_string();
    state.rate = rate;

    log::info!(
        "Starting connection to port: {} at rate: {}",
        state.name,
        state.rate
    );

    if let Err(e) = state.connect(appS) {
        log::error!("Failed to connect: {:?}", e);
        return Err(ConnectionType::Error);
    }

    log::info!(
        "Successfully connected to port: {} at rate: {}",
        state.name,
        state.rate
    );

    Ok(ConnectionType::Connected)
}

#[tauri::command]
fn stop_port(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.shutdown();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .clear_targets()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        record.level(),
                        message
                    ))
                })
                .build(),
        )
        .setup(|app| {
            app.manage(Mutex::new(AppState::new()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_port,
            get_available_ports,
            send_data,
            stop_port
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

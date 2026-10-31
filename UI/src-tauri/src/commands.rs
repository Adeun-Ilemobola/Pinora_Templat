//! Tauri IPC commands. Function names, argument names and return shapes are an IPC
//! contract with the frontend (audit contract C-06). Do not rename them.
//!
//! All commands are synchronous and run on Tauri's main thread (PIN-TPL-014).

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

use serialport::SerialPortType;

#[cfg(windows)]
use crate::bluetooth_ports::windows_bluetooth_ports;
use crate::serial_transport::{AppState, ConnectionType, TransportType};
use crate::wifi_transport::{WifiListenerStatus, WifiTransport};

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_available_ports() -> Result<Vec<String>, String> {
    match serialport::available_ports() {
        Ok(ports) => Ok(ports.into_iter().map(|p| p.port_name).collect()),
        Err(e) => {
            log::error!("Failed to get available ports: {}", e);
            Err(format!("Failed to get available ports: {}", e))
        }
    }
}

#[tauri::command]
pub fn get_available_bluetooth_ports() -> Result<Vec<String>, String> {
    let ports = serialport::available_ports()
        .map_err(|error| format!("Could not list paired Bluetooth COM ports: {error}"))?;
    #[cfg(windows)]
    // Windows serialport can label SPP virtual COM ports Unknown, so check PnP ownership.
    let windows_ports = windows_bluetooth_ports()?;

    Ok(ports
        .into_iter()
        .filter(|port| {
            matches!(port.port_type, SerialPortType::BluetoothPort) || {
                #[cfg(windows)]
                {
                    windows_ports
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&port.port_name))
                }
                #[cfg(not(windows))]
                {
                    false
                }
            }
        })
        .map(|port| port.port_name)
        .collect())
}

#[tauri::command]
pub fn start_bluetooth_port(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    name: String,
) -> Result<(), String> {
    // Only allow a port found by the same outgoing-SPP discovery used by the selector.
    let available = get_available_bluetooth_ports()?;
    if !available
        .iter()
        .any(|port| port.eq_ignore_ascii_case(&name))
    {
        return Err(format!("{name} is not a paired Bluetooth COM port"));
    }
    let mut state = state.lock().map_err(|error| error.to_string())?;
    state.name = name;
    state.rate = 115200; // Windows ignores the baud rate for SPP virtual COM ports.
    state
        .connect(Arc::new(app), TransportType::Bluetooth)
        .map_err(|error| format!("Bluetooth SPP connection failed: {error:?}"))?;
    Ok(())
}

/// Writes one command line. COM (USB/SPP) takes precedence and is not validated;
/// Wi-Fi validates the JSON as an `IncomingCommand` first.
#[tauri::command]
pub fn send_data(
    state: State<'_, Mutex<AppState>>,
    wifi: State<'_, Mutex<Option<WifiTransport>>>,
    data: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    // The active COM connection (USB or SPP) takes precedence over the Wi-Fi writer.
    if let Some(port) = &mut state.port {
        port.write_all(format!("{}\n", data).as_bytes())
            .map_err(|e| {
                log::error!("Failed to send data: {}", e);
                format!("Failed to send data: {}", e)
            })?;

        port.flush().map_err(|e| {
            log::error!("Failed to flush data: {}", e);
            format!("Failed to flush data: {}", e)
        })?;
        return Ok(());
    }
    drop(state);
    let wifi = wifi.lock().map_err(|error| error.to_string())?;
    wifi.as_ref()
        .ok_or_else(|| "No active transport".to_string())?
        .send(&data)
}

#[tauri::command]
pub fn start_wifi_listener(
    state: State<'_, Mutex<Option<WifiTransport>>>,
    app: AppHandle,
    port: u16,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    if state.is_some() {
        return Err("Wi-Fi listener is already running".into());
    }
    *state = Some(WifiTransport::start(app, port)?);
    Ok(())
}

#[tauri::command]
pub fn get_wifi_listener_status(
    state: State<'_, Mutex<Option<WifiTransport>>>,
) -> Result<WifiListenerStatus, String> {
    let state = state.lock().map_err(|error| error.to_string())?;
    Ok(state
        .as_ref()
        .map(WifiTransport::status)
        .unwrap_or_else(WifiListenerStatus::stopped))
}

#[tauri::command]
pub fn stop_wifi_listener(state: State<'_, Mutex<Option<WifiTransport>>>) -> Result<(), String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    if let Some(mut wifi) = state.take() {
        wifi.stop();
    }
    Ok(())
}

/// Opens a USB serial port. Note: every failure is reported only as
/// `ConnectionType::Error`; the detail goes to the log (PIN-TPL-023).
#[tauri::command]
pub fn start_port(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    name: &str,
    rate: u32,
) -> Result<ConnectionType, ConnectionType> {
    let mut state = state.lock().unwrap();
    let app_handle = Arc::new(app);

    state.name = name.to_string();
    state.rate = rate;

    log::info!(
        "Starting connection to port: {} at rate: {}",
        state.name,
        state.rate
    );

    if let Err(e) = state.connect(app_handle, TransportType::Serial) {
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
pub fn stop_port(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    state.shutdown();
    Ok(())
}

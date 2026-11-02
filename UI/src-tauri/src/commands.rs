//! Tauri IPC commands. Function names, argument names and return shapes are an IPC
//! contract with the frontend (audit contract C-06). Do not rename them.
//!
//! Since audit Phase 2D (PIN-TPL-014) the slow commands (port listing, the PowerShell
//! Bluetooth query, connect, disconnect) are `async` and run their work on a blocking
//! thread, so the window stays responsive. `send_data` and the other quick commands
//! stay synchronous, which also keeps commands in order. Names and arguments are
//! unchanged.
//!
//! Phase 2B (additive): `start_wifi_listener` takes an optional `bindAddress`;
//! `provision_device_key`, `clear_device_key`, `forget_device_key`,
//! `list_paired_devices` and `get_auth_status` are new. None returns a key.

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

use serialport::SerialPortType;

use crate::auth_link::AuthLink;
#[cfg(windows)]
use crate::bluetooth_ports::windows_bluetooth_ports;
use crate::device_keys::DeviceKeys;
use crate::serial_transport::{AppState, ConnectionType, TransportType};
use crate::wifi_transport::{WifiListenerStatus, WifiTransport};
use pinora_protocol::auth::AuthStatus;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Runs blocking work off the main thread.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| format!("Background task failed: {error}"))
}

#[tauri::command]
pub async fn get_available_ports() -> Result<Vec<String>, String> {
    blocking(list_ports).await?
}

fn list_ports() -> Result<Vec<String>, String> {
    match serialport::available_ports() {
        Ok(ports) => Ok(ports.into_iter().map(|p| p.port_name).collect()),
        Err(e) => {
            log::error!("Failed to get available ports: {}", e);
            Err(format!("Failed to get available ports: {}", e))
        }
    }
}

/// The port selector's Refresh: always queries Windows again.
#[tauri::command]
pub async fn get_available_bluetooth_ports() -> Result<Vec<String>, String> {
    blocking(|| list_bluetooth_ports(true)).await?
}

fn list_bluetooth_ports(#[allow(unused_variables)] refresh: bool) -> Result<Vec<String>, String> {
    let ports = serialport::available_ports()
        .map_err(|error| format!("Could not list paired Bluetooth COM ports: {error}"))?;
    #[cfg(windows)]
    // Windows serialport can label SPP virtual COM ports Unknown, so check PnP ownership.
    let windows_ports = windows_bluetooth_ports(refresh)?;

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
pub async fn start_bluetooth_port(app: AppHandle, name: String) -> Result<(), String> {
    blocking(move || start_bluetooth_port_blocking(app, name)).await?
}

fn start_bluetooth_port_blocking(app: AppHandle, name: String) -> Result<(), String> {
    // Only allow a port found by the same outgoing-SPP discovery used by the selector
    // (cached for a few seconds, so a connect right after a refresh is quick).
    let available = list_bluetooth_ports(false)?;
    if !available
        .iter()
        .any(|port| port.eq_ignore_ascii_case(&name))
    {
        return Err(format!("{name} is not a paired Bluetooth COM port"));
    }
    let state = app.state::<Mutex<AppState>>();
    let mut state = state.lock().map_err(|error| error.to_string())?;
    state.name = name;
    state.rate = 115200; // Windows ignores the baud rate for SPP virtual COM ports.
    state
        .connect(Arc::new(app.clone()), TransportType::Bluetooth)
        .map_err(|error| format!("Bluetooth SPP connection failed: {error:?}"))?;
    Ok(())
}

/// Writes one command line. COM (USB/SPP) takes precedence over Wi-Fi. Since Phase
/// 2B every transport validates the JSON as an `IncomingCommand`, refuses reserved
/// system commands, and signs it once the link is authenticated.
#[tauri::command]
pub fn send_data(
    state: State<'_, Mutex<AppState>>,
    wifi: State<'_, Mutex<Option<WifiTransport>>>,
    data: String,
) -> Result<(), String> {
    let state = state.lock().unwrap();

    // The active COM connection (USB or SPP) takes precedence over the Wi-Fi writer.
    if let Some(link) = &state.link {
        let mut link = link.lock().map_err(|error| error.to_string())?;
        let line = link.auth.outgoing(&data)?;
        link.write_line(&line).map_err(|e| {
            log::error!("Failed to send data: {}", e);
            format!("Failed to send data: {}", e)
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
    keys: State<'_, Arc<Mutex<DeviceKeys>>>,
    app: AppHandle,
    port: u16,
    bind_address: Option<String>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    if state.is_some() {
        return Err("Wi-Fi listener is already running".into());
    }
    *state = Some(WifiTransport::start(
        app,
        port,
        bind_address.as_deref(),
        Arc::clone(&keys),
    )?);
    Ok(())
}

/// Writes a pairing line built by `build` to the open USB serial link.
fn write_pairing_line(
    state: &State<'_, Mutex<AppState>>,
    build: impl FnOnce(&AuthLink) -> Result<String, String>,
) -> Result<(), String> {
    let state = state.lock().map_err(|error| error.to_string())?;
    let link = state
        .link
        .as_ref()
        .ok_or_else(|| "Connect to the device over USB serial to pair it".to_string())?;
    let mut link = link.lock().map_err(|error| error.to_string())?;
    let line = build(&link.auth)?;
    link.write_line(&line).map_err(|error| error.to_string())
}

/// Asks the device (over USB serial only) for its key; the result arrives as a
/// `deviceAuth` event (`Paired` or `PairingFailed`). The key itself stays in the backend.
#[tauri::command]
pub fn provision_device_key(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    write_pairing_line(&state, AuthLink::provision_line)
}

/// Deletes the device's key (over USB serial only); the result arrives as a
/// `deviceAuth` event (`KeyCleared`). The device then accepts unauthenticated commands.
#[tauri::command]
pub fn clear_device_key(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    write_pairing_line(&state, AuthLink::clear_line)
}

/// Forgets a device's key on this desktop only. Returns whether it was known.
#[tauri::command]
pub fn forget_device_key(
    keys: State<'_, Arc<Mutex<DeviceKeys>>>,
    device_id: String,
) -> Result<bool, String> {
    keys.lock()
        .map_err(|error| error.to_string())?
        .remove(&device_id)
}

/// The ids (not the keys) of the devices this desktop is paired with.
#[tauri::command]
pub fn list_paired_devices(keys: State<'_, Arc<Mutex<DeviceKeys>>>) -> Result<Vec<String>, String> {
    Ok(keys.lock().map_err(|error| error.to_string())?.device_ids())
}

/// The authentication status of the active link (COM first, then Wi-Fi).
#[tauri::command]
pub fn get_auth_status(
    state: State<'_, Mutex<AppState>>,
    wifi: State<'_, Mutex<Option<WifiTransport>>>,
) -> Result<AuthStatus, String> {
    let state = state.lock().map_err(|error| error.to_string())?;
    if state.link.is_some() {
        return Ok(state.auth_status());
    }
    drop(state);
    let wifi = wifi.lock().map_err(|error| error.to_string())?;
    Ok(wifi
        .as_ref()
        .map(WifiTransport::auth_status)
        .unwrap_or(AuthStatus::Idle))
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
pub async fn start_port(
    app: AppHandle,
    name: String,
    rate: u32,
) -> Result<ConnectionType, ConnectionType> {
    blocking(move || start_port_blocking(app, name, rate))
        .await
        .unwrap_or(Err(ConnectionType::Error))
}

fn start_port_blocking(
    app: AppHandle,
    name: String,
    rate: u32,
) -> Result<ConnectionType, ConnectionType> {
    let state = app.state::<Mutex<AppState>>();
    let mut state = state.lock().map_err(|_| ConnectionType::Error)?;
    let app_handle = Arc::new(app.clone());

    state.name = name;
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
pub async fn stop_port(app: AppHandle) -> Result<(), ()> {
    // Joins the reader thread, so it runs off the main thread too.
    blocking(move || {
        if let Ok(mut state) = app.state::<Mutex<AppState>>().lock() {
            state.shutdown();
        }
    })
    .await
    .map_err(|_| ())
}

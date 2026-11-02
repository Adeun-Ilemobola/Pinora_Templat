//! Pinora desktop backend: owns device connections and bridges them to the React
//! frontend.
//!
//! * `commands`: the IPC surface (`invoke` names are a contract, C-06).
//! * `serial_transport`: USB serial / Bluetooth SPP COM ports.
//! * `bluetooth_ports`: Windows discovery of outgoing SPP ports.
//! * `line_classifier`: splits COM lines into protocol messages and console text.
//! * `wifi_transport`: authenticated TCP listener for the Wi-Fi firmware transport.
//!
//! Events emitted to the frontend: `espState` (one validated protocol line),
//! `espConsole` (any other COM line, added in Phase 1C), and `wifiConnection`,
//! `bluetoothConnection` and `serialConnection` (added in Phase 2A; `"Disconnected"`
//! only) with `"Connected"` / `"Disconnected"`, contract C-07.

// Phase 2B Q5 (in progress): the auth modules are tested but not yet wired into
// the transports.
#[allow(dead_code)]
mod auth_link;
mod bluetooth_ports;
mod commands;
#[allow(dead_code)]
mod device_keys;
#[allow(dead_code)]
mod events;
mod line_classifier;
mod serial_transport;
mod wifi_transport;

use std::sync::Mutex;
use tauri::Manager;

use serial_transport::AppState;
use wifi_transport::WifiTransport;

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
            app.manage(Mutex::new(None::<WifiTransport>));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::start_port,
            commands::get_available_ports,
            commands::get_available_bluetooth_ports,
            commands::start_bluetooth_port,
            commands::send_data,
            commands::stop_port,
            commands::start_wifi_listener,
            commands::stop_wifi_listener,
            commands::get_wifi_listener_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

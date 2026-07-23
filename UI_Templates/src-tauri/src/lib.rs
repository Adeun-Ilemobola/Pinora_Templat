mod shared_types;
mod protocol;
mod  global_definition;


use crate::global_definition::{MAX_TIME_BETEEN, MAXBACTH, SerialParseError, SerialState};
use crate::shared_types::state::SerialRuntime;
use crate::{protocol::command::IncomingCommand, };
use crate::protocol::registration::ProtocolMessage;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/


use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Instant;
use std::{
    io::{BufRead, BufReader},
    thread,
    time::Duration,
};
use tauri::State;
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn list_serial_ports() -> Result<Vec<String>, String> {
    let ports = serialport::available_ports().map_err(|err| {
        log::error!("[list_serial_ports] Failed to list ports: {err}");
        err.to_string()
    })?;

    let names: Vec<String> = ports.iter().map(|p| p.port_name.clone()).collect();

    Ok(names)
}

#[tauri::command]
fn stop_runtime(state: State<SerialState>) -> Result<(), String> {
    let mut guard = state.runtime.lock().unwrap();
    if let Some(runtime) = guard.take() {
        log::info!("[stop_runtime] Stopping serial runtime");
        runtime.stop();
        log::info!("[stop_runtime] Serial runtime stopped");
    }
    Ok(())
}

fn flush(batch: &mut Vec<ProtocolMessage>, app: &AppHandle) {
    if batch.is_empty() {
        return;
    }

    if let Err(err) = app.emit("serial_batch", &batch) {
        log::error!("[serial-reader] Failed to emit serial_batch: {:?}", err);
    }
    batch.clear();
}

#[tauri::command]
fn start_serial_listener(
    app: AppHandle,
    port_name: String,
    baud_rate: Option<u32>,
    state: State<SerialState>,
) -> Result<(), String> {
    let mut guard = state.runtime.lock().unwrap();
    if let Some(runtime) = guard.take() {
        log::info!("[start_serial_listener] Stopping existing runtime before reopening");
        runtime.stop();
    };

    let baud_rate = baud_rate.unwrap_or(115_200);

    // log::info!(
    //     "[start_serial_listener] Opening port '{}' at {} baud",
    //     port_name,
    //     baud_rate
    // );

    let port = serialport::new(&port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| {
            log::error!(
                "[start_serial_listener] Failed to open '{}': {}",
                port_name,
                e
            );
            format!("Failed to open {port_name}: {e}")
        })?;

    log::info!(
        "[start_serial_listener] Port '{}' opened successfully",
        port_name
    );

    let reader_port = port.try_clone().map_err(|e| {
        log::error!("[start_serial_listener] Failed to clone port for reader: {e}");
        e.to_string()
    })?;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_thread = Arc::clone(&stop_flag);

    let port_name_thread = port_name.clone();
    let _handle = thread::spawn(move || {
        log::info!(
            "[serial-reader] Thread started for port '{}'",
            port_name_thread
        );

        let mut reader = BufReader::new(reader_port);
        let mut buf: Vec<u8> = Vec::new();
        // let mut line_count: u64 = 0;

        let mut batch: Vec<ProtocolMessage> = Vec::new();
        let mut first_stamp: Option<Instant> = None;
        loop {
            if stop_flag_thread.load(Ordering::Relaxed) {
                break;
            }
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => {
                    continue;
                }

                Err(err) if err.kind() == std::io::ErrorKind::TimedOut => {
                    if let Some(first) = first_stamp {
                        let elapsed = first.elapsed();

                        if !batch.is_empty() && elapsed.as_millis() >= MAX_TIME_BETEEN {
                            // log::info!(
                            //     "[serial-reader] Flushing batch on timeout — len={} elapsed={}ms",
                            //     batch.len(),
                            //     elapsed.as_millis()
                            // );

                            flush(&mut batch, &app);
                            first_stamp = None;
                        }
                    }
                    continue;
                }

                Err(err) => {
                    let _ = app.emit("serial_error", err.to_string());
                    break;
                }

                Ok(_bytes_read) => {
                    // line_count += 1;
                    let line = String::from_utf8_lossy(&buf);
                    let trimmed = line.trim();

                    // log::debug!(
                    //     "[serial-reader] Line #{} received ({} bytes raw): {:?}",
                    //     line_count,
                    //     bytes_read,
                    //     trimmed
                    // );

                    if trimmed.is_empty() {
                        // log::debug!("[serial-reader] Line #{} is empty, skipping", line_count);
                        // buf.clear();
                        continue;
                    }

                    match serde_json::from_str::<ProtocolMessage>(trimmed) {
                        Ok(event) => {
                            // log::info!(
                            //     "[serial-reader] Line #{}  data={:?}",
                            //     line_count, event
                            // );

                            if batch.is_empty() {
                                first_stamp = Some(Instant::now());
                                // log::debug!(
                                //     "[serial-reader] Starting new batch at line #{}",
                                //     line_count
                                // );
                            }

                            batch.push(event);

                            if let Some(first) = first_stamp {
                                let elapsed = first.elapsed();
                                let batch_full = batch.len() >= MAXBACTH;
                                let batch_timed_out = elapsed.as_millis() >= MAX_TIME_BETEEN;
                                if batch_full || batch_timed_out {
                                    // log::info!(
                                    //     "[serial-reader] Flushing batch — len={} elapsed={}ms reason={}",
                                    //     batch.len(),
                                    //     elapsed.as_millis(),
                                    //     if batch_full {
                                    //         "max batch size reached"
                                    //     } else {
                                    //         "max wait time reached"
                                    //     }
                                    // );
                                    flush(&mut batch, &app);

                                    first_stamp = None;
                                }
                            }

                            buf.clear();
                        }

                        Err(err) => {
                            let parse_error = SerialParseError {
                                raw: line.to_string(),
                                error: err.to_string(),
                            };
                            
                            let _ = app.emit("serial_error", parse_error);
                            buf.clear();
                        }
                    }
                }
            }
        }

        log::warn!(
            "[serial-reader] Thread exiting for port '{}'",
            port_name_thread
        );
    });

    *guard = Some(SerialRuntime::new(
        // port_name.clone(),
        // baud_rate,
        port,
        _handle,
        stop_flag,
    ));

    log::info!(
        "[start_serial_listener] Reader thread spawned for '{}'",
        &port_name
    );
    Ok(())
}

// fn log_payload(payload: &SerialPayload) {
//     match payload {
//         SerialPayload::Button(b) => {
//             log::info!("[payload] Type=Button  pressed={}", b.pressed);
//         }
//         SerialPayload::Led(l) => {
//             log::info!("[payload] Type=Led  state={}", l.state);
//         }
//         SerialPayload::Log(l) => {
//             log::info!(
//                 "[payload] Type=Log  message={:?}  rawjson={:?}",
//                 l.message,
//                 l.rawjson
//             );
//         }
//     }
// }

#[tauri::command]
fn send_serial_command(state: State<SerialState>, data: IncomingCommand) -> Result<(), String> {
    let started = std::time::Instant::now();
    log::info!(
        "[send_serial_command] Sending command —  id='{}' payload={:?}",
        data.id,
        data.command
    );

    let mut message = serde_json::to_string(&data).map_err(|err| {
        log::error!("[send_serial_command] Serialization failed: {err}");
        err.to_string()
    })?;
    message.push('\n');

    let mut guard = state.runtime.lock().unwrap();

    let runtime = guard.as_mut().ok_or_else(|| {
        log::error!("[send_serial_command] No serial port connected");
        "Serial port is not connected".to_string()
    })?;

    log::debug!("[send_serial_command] Serialized payload: {:?}", message);

    runtime.port.write_all(message.as_bytes()).map_err(|err| {
        log::error!("[send_serial_command] Write failed: {err}");
        err.to_string()
    })?;
    log::debug!("write_all took {:?}", started.elapsed());
    let flush_started = std::time::Instant::now();

    runtime.port.flush().map_err(|err| {
        log::error!("[send_serial_command] Flush failed: {err}");
        err.to_string()
    })?;

    log::debug!("flush took {:?}", flush_started.elapsed());
    Ok(())
}

#[tauri::command]
fn greet(name: &str) -> String {
    log::debug!("[greet] Called with name='{}'", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Debug)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .build(),
        )
        .manage(SerialState {
            // port: Mutex::new(None),
            runtime: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_serial_listener,
            stop_runtime,
            list_serial_ports,
            send_serial_command
        ])
        .setup(|_app| {
            log::info!("=== Tauri ESP App starting up ===");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

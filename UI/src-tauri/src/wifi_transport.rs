use pinora_protocol::{
    wifi_config::{DESKTOP_PORT, DEV_TOKEN},
    IncomingCommand, ProtocolMessage,
};
use serde::Serialize;
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::{AppHandle, Emitter};

const MAX_LINE_BYTES: usize = 262_144;

#[derive(Serialize)]
pub struct WifiListenerStatus {
    pub listening: bool,
    pub connected: bool,
    pub port: u16,
}

impl WifiListenerStatus {
    pub fn stopped() -> Self {
        Self {
            listening: false,
            connected: false,
            port: DESKTOP_PORT,
        }
    }
}

pub struct WifiTransport {
    running: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    port: u16,
    writer: Arc<Mutex<Option<TcpStream>>>,
    thread: Option<JoinHandle<()>>,
}

impl WifiTransport {
    pub fn start(app: AppHandle, port: u16) -> Result<Self, String> {
        if port == 0 {
            return Err("Listener port must be between 1 and 65535".into());
        }
        let listener = TcpListener::bind(("0.0.0.0", port)).map_err(|error| error.to_string())?;
        // The listener stays nonblocking so Stop can join its worker promptly.
        listener
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let running = Arc::new(AtomicBool::new(true));
        let connected = Arc::new(AtomicBool::new(false));
        let writer = Arc::new(Mutex::new(None));
        let worker_running = Arc::clone(&running);
        let worker_connected = Arc::clone(&connected);
        let worker_writer = Arc::clone(&writer);
        let thread = thread::Builder::new()
            .name("pinora-wifi-listener".into())
            .spawn(move || {
                listen_for_device(
                    listener,
                    app,
                    worker_running,
                    worker_connected,
                    worker_writer,
                )
            })
            .map_err(|error| error.to_string())?;

        log::info!("Wi-Fi listener started on port {port}");
        Ok(Self {
            running,
            connected,
            port,
            writer,
            thread: Some(thread),
        })
    }

    pub fn status(&self) -> WifiListenerStatus {
        WifiListenerStatus {
            listening: self.running.load(Ordering::Relaxed),
            connected: self.connected.load(Ordering::Relaxed),
            port: self.port,
        }
    }

    pub fn send(&self, data: &str) -> Result<(), String> {
        // Validate with the shared Rust protocol before writing to the authenticated device.
        let command: IncomingCommand = serde_json::from_str(data)
            .map_err(|error| format!("Invalid Pinora command: {error}"))?;
        let mut frame = serde_json::to_vec(&command).map_err(|error| error.to_string())?;
        frame.push(b'\n');

        let mut guard = self.writer.lock().map_err(|error| error.to_string())?;
        let stream = guard
            .as_mut()
            .ok_or_else(|| "No authenticated Wi-Fi device".to_string())?;
        if let Err(error) = stream.write_all(&frame) {
            if let Some(stream) = guard.take() {
                let _ = stream.shutdown(Shutdown::Both);
            }
            self.connected.store(false, Ordering::Relaxed);
            return Err(error.to_string());
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        // Closing the writer also wakes a blocked connection while the listener shuts down.
        self.running.store(false, Ordering::Relaxed);
        self.connected.store(false, Ordering::Relaxed);
        if let Ok(mut guard) = self.writer.lock() {
            if let Some(stream) = guard.take() {
                let _ = stream.shutdown(Shutdown::Both);
            }
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for WifiTransport {
    fn drop(&mut self) {
        self.stop();
    }
}

fn authenticate(stream: &mut TcpStream) -> io::Result<()> {
    // Reject unauthenticated peers before exposing their frames to the frontend.
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte)?;
        if byte[0] == b'\n' {
            break;
        }
        if line.len() >= 512 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "auth line too long",
            ));
        }
        line.push(byte[0]);
    }

    if line != format!("PINORA-AUTH 1 {DEV_TOKEN}").as_bytes() {
        let _ = stream.write_all(b"PINORA-AUTH REJECT\n");
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "authentication failed",
        ));
    }
    stream.write_all(b"PINORA-AUTH OK\n")?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    Ok(())
}

fn listen_for_device(
    listener: TcpListener,
    app: AppHandle,
    running: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    writer: Arc<Mutex<Option<TcpStream>>>,
) {
    // One authenticated ESP32 connection is read at a time; reconnects re-enter accept.
    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if let Err(error) = stream.set_nonblocking(false) {
                    log::warn!("Could not configure Wi-Fi stream for {peer}: {error}");
                    continue;
                }

                if let Err(error) = authenticate(&mut stream) {
                    log::warn!("Wi-Fi authentication failed for {peer}: {error}");
                    continue;
                }
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                let Ok(clone) = stream.try_clone() else {
                    log::warn!("Could not clone Wi-Fi stream");
                    continue;
                };
                if let Ok(mut guard) = writer.lock() {
                    *guard = Some(clone);
                }
                connected.store(true, Ordering::Relaxed);
                log::info!("ESP32 connected: {peer}");
                let _ = app.emit("wifiConnection", "Connected");
                read_messages(&mut stream, &app, &running);
                connected.store(false, Ordering::Relaxed);
                if let Ok(mut guard) = writer.lock() {
                    *guard = None;
                }
                log::info!("ESP32 disconnected");
                let _ = app.emit("wifiConnection", "Disconnected");
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => {
                log::warn!("Wi-Fi accept failed: {error}");
                thread::sleep(Duration::from_millis(250));
            }
        }
    }
}

fn read_messages(stream: &mut TcpStream, app: &AppHandle, running: &AtomicBool) {
    // TCP can split or merge JSON lines; validate each complete protocol message.
    let mut input = Vec::new();
    let mut buffer = [0u8; 1024];
    while running.load(Ordering::Relaxed) {
        match stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(count) => {
                input.extend_from_slice(&buffer[..count]);
                while let Some(end) = input.iter().position(|byte| *byte == b'\n') {
                    let line: Vec<u8> = input.drain(..=end).collect();
                    if line.len() > MAX_LINE_BYTES {
                        log::warn!("Wi-Fi message too long");
                        return;
                    }
                    if serde_json::from_slice::<ProtocolMessage>(&line).is_ok() {
                        if let Ok(text) = String::from_utf8(line) {
                            let _ = app.emit("espState", text);
                        }
                    } else {
                        log::warn!("Invalid Pinora message on Wi-Fi connection");
                    }
                }
                if input.len() > MAX_LINE_BYTES {
                    log::warn!("Wi-Fi message too long");
                    return;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => {
                log::warn!("Wi-Fi read failed: {error}");
                return;
            }
        }
    }
}

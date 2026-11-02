//! Wi-Fi listener: the ESP32 connects as a TCP client and authenticates with the
//! development token (contract C-03), then exchanges newline-delimited JSON.
//!
//! Since audit Phase 2B (PIN-TPL-013, PIN-TPL-003, PIN-TPL-016):
//! * One thread accepts; each peer authenticates on its own worker within an
//!   overall deadline ([`AUTH_DEADLINE`]), so a slow or silent peer can no longer
//!   hold the listener (before: 3 s per byte, up to ~25 minutes). At most
//!   [`MAX_PENDING_AUTH`] peers authenticate at once.
//! * The newest authenticated device replaces the previous connection, so a device
//!   that reconnects after a half-open TCP drop is not locked out.
//! * The bind address is configurable (default `0.0.0.0`).
//! * Lines use the shared [`LineFramer`] (8 KiB; 256 KiB before 2B).
//! * After the token, the backend runs the protocol v2 handshake ([`AuthLink`]) and
//!   signs every command; `Auth` messages never reach the WebView.

use pinora_protocol::{
    auth::AuthStatus,
    framing::{Frame, LineFramer, MAX_AUTH_LINE_BYTES, MAX_LINE_BYTES},
    wifi_config::{DESKTOP_PORT, DEV_TOKEN},
};
use serde::Serialize;
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tauri::AppHandle;

use crate::auth_link::{random_nonce, AuthLink, LinkKind, Outcome, AUTH_EVENT};
use crate::device_keys::DeviceKeys;
use crate::events::{emit_serialized, EventSink};
use crate::line_classifier::{classify, ComLine};

/// Total time a peer has to send a valid token line.
#[cfg(not(test))]
pub(crate) const AUTH_DEADLINE: Duration = Duration::from_secs(5);
#[cfg(test)]
pub(crate) const AUTH_DEADLINE: Duration = Duration::from_millis(400);
/// Peers allowed to authenticate at the same time; more are refused at once.
pub(crate) const MAX_PENDING_AUTH: usize = 4;
pub(crate) const DEFAULT_BIND: &str = "0.0.0.0";

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

struct Session {
    id: u64,
    writer: TcpStream,
    auth: AuthLink,
}

type Current = Arc<Mutex<Option<Session>>>;

pub struct WifiTransport {
    running: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    port: u16,
    current: Current,
    thread: Option<JoinHandle<()>>,
}

impl WifiTransport {
    pub fn start(
        app: AppHandle,
        port: u16,
        bind: Option<&str>,
        keys: Arc<Mutex<DeviceKeys>>,
    ) -> Result<Self, String> {
        if port == 0 {
            return Err("Listener port must be between 1 and 65535".into());
        }
        let bind = bind.unwrap_or(DEFAULT_BIND);
        let listener = TcpListener::bind((bind, port)).map_err(|error| error.to_string())?;
        let transport = Self::start_on(app, listener, keys)?;
        log::info!("Wi-Fi listener started on {bind}:{port}");
        Ok(transport)
    }

    pub(crate) fn start_on<S: EventSink>(
        sink: S,
        listener: TcpListener,
        keys: Arc<Mutex<DeviceKeys>>,
    ) -> Result<Self, String> {
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        // The listener stays nonblocking so Stop can join its accept thread promptly.
        listener
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let running = Arc::new(AtomicBool::new(true));
        let connected = Arc::new(AtomicBool::new(false));
        let current: Current = Arc::new(Mutex::new(None));
        let shared = Shared {
            sink,
            running: Arc::clone(&running),
            connected: Arc::clone(&connected),
            current: Arc::clone(&current),
            keys,
            pending: Arc::new(AtomicUsize::new(0)),
        };
        let thread = thread::Builder::new()
            .name("pinora-wifi-listener".into())
            .spawn(move || accept_loop(listener, shared))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            running,
            connected,
            port,
            current,
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

    pub(crate) fn auth_status(&self) -> AuthStatus {
        self.current
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().map(|session| session.auth.status()))
            .unwrap_or(AuthStatus::Idle)
    }

    /// Validates a WebView command and writes it, signed once the device is
    /// authenticated.
    pub fn send(&self, data: &str) -> Result<(), String> {
        let mut slot = self.current.lock().map_err(|error| error.to_string())?;
        let session = slot
            .as_mut()
            .ok_or_else(|| "No authenticated Wi-Fi device".to_string())?;
        let line = session.auth.outgoing(data)?;
        if let Err(error) = session.writer.write_all(line.as_bytes()) {
            if let Some(session) = slot.take() {
                let _ = session.writer.shutdown(Shutdown::Both);
            }
            self.connected.store(false, Ordering::Relaxed);
            return Err(error.to_string());
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        // Closing the current connection wakes its reader while the listener shuts down.
        self.running.store(false, Ordering::Relaxed);
        self.connected.store(false, Ordering::Relaxed);
        if let Ok(mut slot) = self.current.lock() {
            if let Some(session) = slot.take() {
                let _ = session.writer.shutdown(Shutdown::Both);
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

#[derive(Clone)]
struct Shared<S> {
    sink: S,
    running: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    current: Current,
    keys: Arc<Mutex<DeviceKeys>>,
    pending: Arc<AtomicUsize>,
}

fn accept_loop<S: EventSink>(listener: TcpListener, shared: Shared<S>) {
    let mut next_id = 0u64;
    while shared.running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, peer)) => {
                if shared.pending.load(Ordering::SeqCst) >= MAX_PENDING_AUTH {
                    log::warn!("Too many Wi-Fi peers authenticating; refusing {peer}");
                    continue;
                }
                shared.pending.fetch_add(1, Ordering::SeqCst);
                next_id += 1;
                let id = next_id;
                let worker = shared.clone();
                let spawned = thread::Builder::new()
                    .name("pinora-wifi-peer".into())
                    .spawn(move || serve_peer(stream, peer, id, worker));
                if let Err(error) = spawned {
                    log::warn!("Could not start a Wi-Fi worker: {error}");
                    shared.pending.fetch_sub(1, Ordering::SeqCst);
                }
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

fn serve_peer<S: EventSink>(
    mut stream: TcpStream,
    peer: std::net::SocketAddr,
    id: u64,
    shared: Shared<S>,
) {
    let authenticated = stream
        .set_nonblocking(false)
        .and_then(|()| authenticate(&mut stream, Instant::now() + AUTH_DEADLINE));
    shared.pending.fetch_sub(1, Ordering::SeqCst);
    if let Err(error) = authenticated {
        log::warn!("Wi-Fi authentication failed for {peer}: {error}");
        return;
    }
    if !shared.running.load(Ordering::Relaxed) {
        return;
    }
    let (Ok(writer), Ok(())) = (
        stream.try_clone(),
        stream.set_read_timeout(Some(Duration::from_millis(100))),
    ) else {
        log::warn!("Could not configure the Wi-Fi stream for {peer}");
        return;
    };

    let status = {
        let Ok(mut slot) = shared.current.lock() else {
            return;
        };
        if let Some(old) = slot.take() {
            log::info!("A newly authenticated device replaces the previous Wi-Fi connection");
            let _ = old.writer.shutdown(Shutdown::Both);
        }
        let mut session = Session {
            id,
            writer,
            auth: AuthLink::new(LinkKind::Wifi),
        };
        match random_nonce() {
            Ok(nonce) => {
                if let Some(begin) = session.auth.begin(nonce) {
                    if let Err(error) = session.writer.write_all(begin.as_bytes()) {
                        log::warn!("Could not start authentication with {peer}: {error}");
                        return;
                    }
                }
            }
            Err(error) => log::error!("Could not generate a nonce: {error}"),
        }
        let status = session.auth.status();
        *slot = Some(session);
        status
    };

    shared.connected.store(true, Ordering::Relaxed);
    log::info!("ESP32 connected: {peer}");
    shared
        .sink
        .emit_text("wifiConnection", "Connected".to_string());
    emit_serialized(&shared.sink, AUTH_EVENT, &status);

    read_messages(&mut stream, id, &shared);

    let still_current = match shared.current.lock() {
        Ok(mut slot) if slot.as_ref().map(|session| session.id) == Some(id) => {
            *slot = None;
            true
        }
        _ => false,
    };
    if still_current {
        shared.connected.store(false, Ordering::Relaxed);
        log::info!("ESP32 disconnected");
        shared
            .sink
            .emit_text("wifiConnection", "Disconnected".to_string());
    }
}

/// Reads the device's token line within `deadline` and answers OK or REJECT.
fn authenticate(stream: &mut TcpStream, deadline: Instant) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut framer = LineFramer::new(MAX_AUTH_LINE_BYTES);
    let mut buffer = [0u8; 128];
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "authentication deadline passed",
            ));
        }
        stream.set_read_timeout(Some(remaining))?;
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "peer closed during authentication",
            ));
        }
        // The device sends nothing after its token line until it reads OK.
        if let Some(frame) = framer.push(&buffer[..count]).into_iter().next() {
            return match frame {
                Frame::Line(line) if token_matches(&line) => stream.write_all(b"PINORA-AUTH OK\n"),
                Frame::Line(_) => {
                    let _ = stream.write_all(b"PINORA-AUTH REJECT\n");
                    Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "authentication failed",
                    ))
                }
                Frame::Overflow => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "auth line too long",
                )),
            };
        }
    }
}

fn token_matches(line: &[u8]) -> bool {
    let expected = format!("PINORA-AUTH 1 {DEV_TOKEN}");
    let expected = expected.as_bytes();
    // Constant time for equal lengths.
    line.len() == expected.len()
        && line
            .iter()
            .zip(expected)
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

fn read_messages<S: EventSink>(stream: &mut TcpStream, id: u64, shared: &Shared<S>) {
    // TCP can split or merge JSON lines; validate each complete protocol message.
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    let mut buffer = [0u8; 1024];
    while shared.running.load(Ordering::Relaxed) {
        match stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(count) => {
                for frame in framer.push(&buffer[..count]) {
                    let line = match frame {
                        Frame::Line(line) => line,
                        Frame::Overflow => {
                            log::warn!("Wi-Fi message too long");
                            return;
                        }
                    };
                    match classify(&line) {
                        ComLine::Protocol(text) => shared.sink.emit_text("espState", text),
                        ComLine::Auth(message) => handle_auth(message, id, shared),
                        ComLine::Console(_) => {
                            log::warn!("Invalid Pinora message on Wi-Fi connection")
                        }
                    }
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

/// Lock order: session slot, then keys.
fn handle_auth<S: EventSink>(message: pinora_protocol::AuthMessage, id: u64, shared: &Shared<S>) {
    let Ok(mut slot) = shared.current.lock() else {
        return;
    };
    // A replaced connection's late messages are ignored.
    let Some(session) = slot.as_mut().filter(|session| session.id == id) else {
        return;
    };
    let outcomes = match shared.keys.lock() {
        Ok(mut keys) => session.auth.on_auth_message(message, &mut keys),
        Err(_) => return,
    };
    for outcome in outcomes {
        match outcome {
            Outcome::Reply(reply) => {
                if let Err(error) = session.writer.write_all(reply.as_bytes()) {
                    log::warn!("Could not answer the device: {error}");
                }
            }
            Outcome::Status(status) => emit_serialized(&shared.sink, AUTH_EVENT, &status),
            Outcome::Pairing(event) => emit_serialized(&shared.sink, AUTH_EVENT, &event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinora_protocol::auth::{Decision, DeviceSession, Key, KEY_BYTES, NONCE_BYTES};
    use pinora_protocol::{parse_command_line, AuthMessage, CommandLine, ProtocolMessage};
    use std::io::{BufRead, BufReader};

    const DEVICE: &str = "a1b2c3d4e5f6";

    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<(String, String)>>>);

    impl EventSink for Recorder {
        fn emit_text(&self, event: &str, payload: String) {
            self.0.lock().unwrap().push((event.into(), payload));
        }
        fn emit_json(&self, event: &str, payload: serde_json::Value) {
            self.0
                .lock()
                .unwrap()
                .push((event.into(), payload.to_string()));
        }
    }

    impl Recorder {
        fn wait_for(&self, event: &str, contains: &str) {
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                if self
                    .0
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|(e, p)| e == event && p.contains(contains))
                {
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }
            panic!(
                "no {event} event containing {contains:?}: {:?}",
                self.0.lock().unwrap()
            );
        }
    }

    fn key() -> Key {
        Key::from_bytes([9; KEY_BYTES])
    }

    fn listener(keys: DeviceKeys) -> (WifiTransport, Recorder, u16) {
        let recorder = Recorder::default();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let transport =
            WifiTransport::start_on(recorder.clone(), listener, Arc::new(Mutex::new(keys)))
                .unwrap();
        let port = transport.port;
        (transport, recorder, port)
    }

    /// A fake device: connects and sends the token; returns the stream and a reader.
    fn device(port: u16, token_line: &str) -> (TcpStream, BufReader<TcpStream>) {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream.write_all(token_line.as_bytes()).unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        (stream, reader)
    }

    fn good_token() -> String {
        format!("PINORA-AUTH 1 {DEV_TOKEN}\n")
    }

    fn read_line(reader: &mut BufReader<TcpStream>) -> String {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        line
    }

    fn write_message(stream: &mut TcpStream, message: &ProtocolMessage) {
        let mut line = serde_json::to_vec(message).unwrap();
        line.push(b'\n');
        stream.write_all(&line).unwrap();
    }

    #[test]
    fn a_paired_device_completes_the_handshake_and_receives_signed_commands() {
        let mut keys = DeviceKeys::default();
        keys.insert(DEVICE, &key()).unwrap();
        let (transport, recorder, port) = listener(keys);
        let (mut stream, mut reader) = device(port, &good_token());
        assert_eq!(read_line(&mut reader), "PINORA-AUTH OK\n");
        recorder.wait_for("wifiConnection", "Connected");

        let mut session = DeviceSession::new(DEVICE);
        let answer = |line: String, session: &mut DeviceSession| -> Decision {
            session.handle(
                parse_command_line(line.trim_end().as_bytes()).unwrap(),
                Some(&key()),
                || [4; NONCE_BYTES],
            )
        };
        let Decision::Reply(challenge) = answer(read_line(&mut reader), &mut session) else {
            panic!("expected AuthBegin")
        };
        write_message(&mut stream, &ProtocolMessage::Auth(challenge));
        let Decision::Reply(result) = answer(read_line(&mut reader), &mut session) else {
            panic!("expected AuthResponse")
        };
        write_message(&mut stream, &ProtocolMessage::Auth(result));
        recorder.wait_for("deviceAuth", "Authenticated");

        transport
            .send(r#"{"id":"u","command":{"Led":{"Toggle":{}}}}"#)
            .unwrap();
        let line = read_line(&mut reader);
        let parsed = parse_command_line(line.trim_end().as_bytes()).unwrap();
        assert!(matches!(parsed, CommandLine::Signed(_)));
        assert!(matches!(
            session.handle(parsed, Some(&key()), || unreachable!()),
            Decision::Forward(_)
        ));
        // Protocol lines still reach the WebView; Auth lines never do.
        assert!(!recorder
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|(event, payload)| event == "espState" && payload.contains("Auth")));
    }

    #[test]
    fn a_trickling_peer_is_cut_off_at_the_overall_deadline() {
        let (_transport, _recorder, port) = listener(DeviceKeys::default());
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let started = Instant::now();
        // One byte every 50 ms never completes a line; each byte would have reset a
        // per-read timeout before 2B.
        for byte in b"PINORA-AUTH 1 slowly-slowly-slowly" {
            if stream.write_all(&[*byte]).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
            if started.elapsed() > AUTH_DEADLINE * 3 {
                break;
            }
        }
        let mut rest = Vec::new();
        let _ = stream.read_to_end(&mut rest);
        assert!(
            started.elapsed() < AUTH_DEADLINE * 4,
            "closed after {:?}",
            started.elapsed()
        );
        assert!(rest.is_empty(), "no OK for a trickling peer");
    }

    #[test]
    fn a_wrong_token_is_rejected() {
        let (transport, _recorder, port) = listener(DeviceKeys::default());
        let (_stream, mut reader) = device(port, "PINORA-AUTH 1 wrong\n");
        assert_eq!(read_line(&mut reader), "PINORA-AUTH REJECT\n");
        assert!(!transport.status().connected);
    }

    #[test]
    fn a_new_authenticated_device_replaces_the_old_connection() {
        let (transport, recorder, port) = listener(DeviceKeys::default());
        let (_first, mut first_reader) = device(port, &good_token());
        assert_eq!(read_line(&mut first_reader), "PINORA-AUTH OK\n");
        let _ = read_line(&mut first_reader); // AuthBegin
        recorder.wait_for("wifiConnection", "Connected");

        let (_second, mut second_reader) = device(port, &good_token());
        assert_eq!(read_line(&mut second_reader), "PINORA-AUTH OK\n");
        let _ = read_line(&mut second_reader); // AuthBegin

        // The first connection is closed.
        let mut rest = String::new();
        assert_eq!(first_reader.read_line(&mut rest).unwrap_or(0), 0);
        // Commands go to the second device (unauthenticated: sent plain).
        transport
            .send(r#"{"id":"u","command":{"Led":{"Toggle":{}}}}"#)
            .unwrap();
        assert!(read_line(&mut second_reader).contains("Toggle"));
        assert!(transport.status().connected);
    }

    #[test]
    fn a_device_without_a_key_is_reported_unprovisioned() {
        let (_transport, recorder, port) = listener(DeviceKeys::default());
        let (mut stream, mut reader) = device(port, &good_token());
        assert_eq!(read_line(&mut reader), "PINORA-AUTH OK\n");
        assert!(read_line(&mut reader).contains("AuthBegin"));
        write_message(
            &mut stream,
            &ProtocolMessage::Auth(AuthMessage::Unprovisioned {
                device_id: DEVICE.into(),
            }),
        );
        recorder.wait_for("deviceAuth", "Unprovisioned");
    }
}

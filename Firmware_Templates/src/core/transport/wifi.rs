//! Wi-Fi transport. The ESP32 joins the configured network and connects as a TCP
//! client to the desktop listener, authenticating with the shared development token
//! before exchanging newline-delimited JSON (contract C-03).
//!
//! Since audit Phase 2B (PIN-TPL-013, PIN-TPL-010):
//! * Bring-up runs in the background. `Wifi::new` only starts the driver, so the rest
//!   of the firmware boots even without an access point. A link thread connects with
//!   backoff (2 s doubling to 30 s) and reassociates after a drop.
//! * Each TCP connection has a reader thread and a writer loop, so outbound frames
//!   are no longer limited to one per read timeout (about 20 messages/s before 2B).
//! * Lines use the shared [`LineFramer`]; drops are counted in [`DropCounters`].
//! * Each connection has a [`DeviceSession`]: a device with a key is read-only until
//!   the desktop completes the HMAC handshake, then accepts only signed commands
//!   (PIN-TPL-003; see `auth_gate`). Session replies go to that connection only.
//!
//! The token and Wi-Fi credentials are still compiled in, and the token travels
//! unencrypted.

use std::{
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::core::transport::auth_gate::{process_line, Action};
use crate::core::transport::device_key::{random_bytes, KeyStore};
use crate::core::transport::discovery_cache::remember;
use crate::core::transport::drop_stats::{DropCounters, Stage};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use pinora_protocol::{
    auth::DeviceSession,
    framing::{encode_line, Frame, LineFramer, MAX_LINE_BYTES},
    wifi_config::{DESKTOP_IP, DESKTOP_PORT, DEV_TOKEN, WIFI_PASSWORD, WIFI_SSID},
    IncomingCommand, ProtocolMessage,
};

/// How often the link thread checks the association.
const LINK_POLL: Duration = Duration::from_secs(1);
const BACKOFF_MIN: Duration = Duration::from_secs(2);
const BACKOFF_MAX: Duration = Duration::from_secs(30);
/// Reader poll interval; bounds how long a closed connection takes to notice.
const READ_POLL: Duration = Duration::from_millis(200);

pub struct Wifi {
    outbound: mpsc::SyncSender<ProtocolMessage>,
    incoming: Mutex<Option<mpsc::Receiver<IncomingCommand>>>,
    drops: Arc<DropCounters>,
}

impl Wifi {
    pub fn new(
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
        modem: Modem<'static>,
        drops: Arc<DropCounters>,
        keys: Arc<KeyStore>,
    ) -> anyhow::Result<Self> {
        let address: SocketAddr = format!("{DESKTOP_IP}:{DESKTOP_PORT}").parse()?;

        let esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
        let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;
        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: WIFI_SSID
                .try_into()
                .map_err(|_| anyhow::anyhow!("Wi-Fi SSID is too long"))?,
            password: WIFI_PASSWORD
                .try_into()
                .map_err(|_| anyhow::anyhow!("Wi-Fi password is too long"))?,
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        }))?;
        wifi.start()?;

        // The link thread owns the driver for the life of the firmware.
        let link_up = Arc::new(AtomicBool::new(false));
        let link_flag = link_up.clone();
        thread::Builder::new()
            .name("wifi-link".into())
            .stack_size(4096)
            .spawn(move || run_link(wifi, link_flag))?;

        let (outbound, outbound_rx) = mpsc::sync_channel(128);
        let (incoming_tx, incoming) = mpsc::sync_channel(128);
        let client_drops = drops.clone();
        thread::Builder::new()
            .name("wifi-transport".into())
            .stack_size(8192)
            .spawn(move || {
                run_client(
                    address,
                    link_up,
                    outbound_rx,
                    incoming_tx,
                    client_drops,
                    keys,
                )
            })?;

        Ok(Self {
            outbound,
            incoming: Mutex::new(Some(incoming)),
            drops,
        })
    }

    pub fn event(&self, event: ProtocolMessage) -> Result<(), String> {
        // Discovery must survive queue pressure; live module events may be dropped.
        match event {
            ProtocolMessage::System(_) | ProtocolMessage::Registration(_) => self
                .outbound
                .send(event)
                .map_err(|error| format!("Wi-Fi outbound queue unavailable: {error}")),
            ProtocolMessage::ModuleEvent(_) | ProtocolMessage::Auth(_) => {
                match self.outbound.try_send(event) {
                    Ok(()) => Ok(()),
                    Err(mpsc::TrySendError::Full(_)) => {
                        self.drops.record(Stage::Transport);
                        Ok(())
                    }
                    Err(error) => Err(format!("Wi-Fi outbound queue unavailable: {error}")),
                }
            }
        }
    }

    pub fn take_incoming(&self) -> Option<mpsc::Receiver<IncomingCommand>> {
        self.incoming.lock().ok()?.take()
    }
}

/// Keeps the station associated: connects with backoff and reconnects after a drop.
fn run_link(mut wifi: BlockingWifi<EspWifi<'static>>, link_up: Arc<AtomicBool>) {
    let mut backoff = BACKOFF_MIN;
    loop {
        if wifi.is_connected().unwrap_or(false) {
            thread::sleep(LINK_POLL);
            continue;
        }
        if link_up.swap(false, Ordering::AcqRel) {
            log::warn!("Wi-Fi link lost");
        }
        log::info!("Wi-Fi connecting");
        match wifi.connect().and_then(|()| wifi.wait_netif_up()) {
            Ok(()) => {
                log::info!("Wi-Fi connected");
                link_up.store(true, Ordering::Release);
                backoff = BACKOFF_MIN;
            }
            Err(error) => {
                log::warn!(
                    "Wi-Fi connection failed: {error}; retrying in {} s",
                    backoff.as_secs()
                );
                let _ = wifi.disconnect();
                thread::sleep(backoff);
                backoff = (backoff * 2).min(BACKOFF_MAX);
            }
        }
    }
}

fn authenticate(stream: &mut TcpStream) -> io::Result<()> {
    // The token is a development handshake before normal JSON protocol frames begin.
    stream.write_all(b"PINORA-AUTH 1 ")?;
    stream.write_all(DEV_TOKEN.as_bytes())?;
    stream.write_all(b"\n")?;

    // The reply is `PINORA-AUTH OK` or `PINORA-AUTH REJECT`; anything longer is refused
    // at once, so a slow or hostile listener cannot hold the device for long.
    let mut framer = LineFramer::new(b"PINORA-AUTH REJECT".len());
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte)?;
        if let Some(frame) = framer.push(&byte).into_iter().next() {
            return match frame {
                Frame::Line(line) if line == b"PINORA-AUTH OK" => Ok(()),
                _ => Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "desktop authentication rejected",
                )),
            };
        }
    }
}

fn run_client(
    address: SocketAddr,
    link_up: Arc<AtomicBool>,
    outbound: mpsc::Receiver<ProtocolMessage>,
    incoming: mpsc::SyncSender<IncomingCommand>,
    drops: Arc<DropCounters>,
    keys: Arc<KeyStore>,
) {
    let mut system = None;
    let mut registrations = Vec::new();

    loop {
        if link_up.load(Ordering::Acquire) {
            log::info!("TCP connecting to {address}");
            match TcpStream::connect_timeout(&address, Duration::from_secs(5)) {
                Ok(stream) => {
                    serve_connection(
                        stream,
                        &outbound,
                        &incoming,
                        &mut system,
                        &mut registrations,
                        &drops,
                        &keys,
                    );
                    log::info!("Desktop disconnected; TCP reconnecting");
                }
                Err(error) => log::debug!("TCP connection failed: {error}"),
            }
        }

        // Save discovery messages so the next desktop connection receives them.
        for _ in 0..20 {
            while let Ok(message) = outbound.try_recv() {
                remember(&message, &mut system, &mut registrations);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn serve_connection(
    mut stream: TcpStream,
    outbound: &mpsc::Receiver<ProtocolMessage>,
    incoming: &mpsc::SyncSender<IncomingCommand>,
    system: &mut Option<ProtocolMessage>,
    registrations: &mut Vec<ProtocolMessage>,
    drops: &DropCounters,
    keys: &Arc<KeyStore>,
) {
    let setup = stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .and_then(|()| stream.set_write_timeout(Some(Duration::from_secs(2))))
        .and_then(|()| authenticate(&mut stream));
    if let Err(error) = setup {
        log::warn!("TCP authentication failed: {error}");
        return;
    }
    let reader_stream = match stream.try_clone() {
        Ok(clone) => clone,
        Err(error) => {
            log::warn!("Could not clone the TCP stream: {error}");
            return;
        }
    };
    log::info!("TCP connected");

    let closed = Arc::new(AtomicBool::new(false));
    let reader_closed = closed.clone();
    let reader_incoming = incoming.clone();
    let reader_keys = keys.clone();
    // Session replies (handshake, rejections) for this connection only.
    let (replies, reply_rx) = mpsc::channel::<ProtocolMessage>();
    let reader = match thread::Builder::new()
        .name("wifi-reader".into())
        .stack_size(8192)
        .spawn(move || {
            read_commands(
                reader_stream,
                reader_incoming,
                reader_closed,
                &reader_keys,
                &replies,
            )
        }) {
        Ok(handle) => handle,
        Err(error) => {
            log::warn!("Could not start the Wi-Fi reader: {error}");
            return;
        }
    };

    // Rebuild desktop discovery after a listener restart or TCP loss, then forward.
    let replay = system
        .iter()
        .chain(registrations.iter())
        .try_for_each(|message| write_frame(&mut stream, message, drops));
    match replay {
        Err(error) => log::warn!("TCP write failed: {error}"),
        Ok(()) => {
            'connection: while !closed.load(Ordering::Acquire) {
                while let Ok(reply) = reply_rx.try_recv() {
                    if let Err(error) = write_frame(&mut stream, &reply, drops) {
                        log::warn!("TCP write failed: {error}");
                        break 'connection;
                    }
                }
                match outbound.recv_timeout(Duration::from_millis(50)) {
                    Ok(message) => {
                        remember(&message, system, registrations);
                        if let Err(error) = write_frame(&mut stream, &message, drops) {
                            log::warn!("TCP write failed: {error}");
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        }
    }

    closed.store(true, Ordering::Release);
    let _ = stream.shutdown(Shutdown::Both);
    let _ = reader.join();
}

/// Reads and parses commands until the peer closes, a read fails, a line is too long
/// or the writer ends the connection.
fn read_commands(
    mut stream: TcpStream,
    incoming: mpsc::SyncSender<IncomingCommand>,
    closed: Arc<AtomicBool>,
    keys: &KeyStore,
    replies: &mpsc::Sender<ProtocolMessage>,
) {
    let mut session = DeviceSession::new(keys.device_id());
    let _ = stream.set_read_timeout(Some(READ_POLL));
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    let mut buffer = [0u8; 1024];
    'connection: while !closed.load(Ordering::Acquire) {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                // TCP reads may contain partial or multiple newline-delimited commands.
                for frame in framer.push(&buffer[..count]) {
                    match frame {
                        Frame::Line(line) => {
                            let key = keys.current();
                            match process_line(&mut session, &line, key.as_ref(), random_bytes) {
                                Ok(actions) => {
                                    for action in actions {
                                        match action {
                                            Action::Deliver(command) => {
                                                if incoming.send(command).is_err() {
                                                    break 'connection;
                                                }
                                            }
                                            Action::Send(message) => {
                                                let _ = replies.send(message);
                                            }
                                        }
                                    }
                                }
                                Err(error) => log::warn!("Invalid Wi-Fi command: {error}"),
                            }
                        }
                        Frame::Overflow => {
                            log::warn!("Wi-Fi command too long; disconnecting");
                            break 'connection;
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
                log::warn!("TCP read failed: {error}");
                break;
            }
        }
    }
    closed.store(true, Ordering::Release);
}

/// Writes one frame. A message over the line limit is dropped (and counted), not
/// fatal to the connection.
fn write_frame(
    stream: &mut TcpStream,
    message: &ProtocolMessage,
    drops: &DropCounters,
) -> io::Result<()> {
    match encode_line(message) {
        Ok(frame) => stream.write_all(&frame),
        Err(error) => {
            log::warn!("Wi-Fi frame dropped: {error}");
            drops.record(Stage::Link);
            Ok(())
        }
    }
}

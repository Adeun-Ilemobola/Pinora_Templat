use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use pinora_protocol::{
    wifi_config::{DESKTOP_IP, DESKTOP_PORT, DEV_TOKEN, WIFI_PASSWORD, WIFI_SSID},
    IncomingCommand, ProtocolMessage,
};

const MAX_COMMAND_BYTES: usize = 8192;

pub struct Wifi {
    _wifi: BlockingWifi<EspWifi<'static>>,
    outbound: mpsc::SyncSender<ProtocolMessage>,
    incoming: Mutex<Option<mpsc::Receiver<IncomingCommand>>>,
}

impl Wifi {
    pub fn new(
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
        modem: Modem<'static>,
    ) -> anyhow::Result<Self> {
        let address: SocketAddr = format!("{DESKTOP_IP}:{DESKTOP_PORT}").parse()?;

        // Keep the Wi-Fi driver alive while the dedicated TCP worker reconnects to the desktop.
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
        log::info!("Wi-Fi connecting");
        wifi.start()?;
        wifi.connect()?;
        wifi.wait_netif_up()?;
        log::info!("Wi-Fi connected");

        let (outbound, outbound_rx) = mpsc::sync_channel(128);
        let (incoming_tx, incoming) = mpsc::sync_channel(128);
        thread::Builder::new()
            .name("wifi-transport".into())
            .stack_size(8192)
            .spawn(move || run_client(address, outbound_rx, incoming_tx))?;

        Ok(Self {
            _wifi: wifi,
            outbound,
            incoming: Mutex::new(Some(incoming)),
        })
    }

    pub fn event(&self, event: ProtocolMessage) -> Result<(), String> {
        // Discovery must survive queue pressure; live module events may be dropped.
        match event {
            ProtocolMessage::System(_) | ProtocolMessage::Registration(_) => self
                .outbound
                .send(event)
                .map_err(|error| format!("Wi-Fi outbound queue unavailable: {error}")),
            ProtocolMessage::ModuleEvent(_) => self
                .outbound
                .try_send(event)
                .map_err(|error| format!("Wi-Fi outbound queue unavailable: {error}")),
        }
    }

    pub fn take_incoming(&self) -> Option<mpsc::Receiver<IncomingCommand>> {
        self.incoming.lock().ok()?.take()
    }
}

fn authenticate(stream: &mut TcpStream) -> io::Result<()> {
    // The token is a development handshake before normal JSON protocol frames begin.
    stream.write_all(b"PINORA-AUTH 1 ")?;
    stream.write_all(DEV_TOKEN.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut response = Vec::new();
    let mut byte = [0u8; 1];
    while response.len() <= b"PINORA-AUTH OK".len() {
        stream.read_exact(&mut byte)?;
        if byte[0] == b'\n' {
            if response == b"PINORA-AUTH OK" {
                return Ok(());
            }
            break;
        }
        response.push(byte[0]);
    }
    Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "desktop authentication rejected",
    ))
}

fn run_client(
    address: SocketAddr,
    outbound: mpsc::Receiver<ProtocolMessage>,
    incoming: mpsc::SyncSender<IncomingCommand>,
) {
    let mut system = None;
    let mut registrations = Vec::new();

    log::info!("TCP connecting to {address}");
    loop {
        match TcpStream::connect_timeout(&address, Duration::from_secs(5)) {
            Ok(mut stream) => {
                let setup = stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(2))))
                    .and_then(|_| authenticate(&mut stream))
                    .and_then(|_| stream.set_read_timeout(Some(Duration::from_millis(50))));
                if let Err(error) = setup {
                    log::warn!("TCP authentication failed: {error}");
                } else {
                    log::info!("TCP connected");
                    // Rebuild desktop discovery after listener restart or TCP loss.
                    let replay = system
                        .iter()
                        .chain(registrations.iter())
                        .try_for_each(|message| write_frame(&mut stream, message));
                    if replay.is_ok() {
                        serve_connection(
                            &mut stream,
                            &outbound,
                            &incoming,
                            &mut system,
                            &mut registrations,
                        );
                    }
                    log::info!("Desktop disconnected; TCP reconnecting");
                }
            }
            Err(error) => log::debug!("TCP connection failed: {error}"),
        }

        // Save discovery messages so a restarted listener receives them again.
        for _ in 0..20 {
            while let Ok(message) = outbound.try_recv() {
                remember(&message, &mut system, &mut registrations);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn serve_connection(
    stream: &mut TcpStream,
    outbound: &mpsc::Receiver<ProtocolMessage>,
    incoming: &mpsc::SyncSender<IncomingCommand>,
    system: &mut Option<ProtocolMessage>,
    registrations: &mut Vec<ProtocolMessage>,
) {
    let mut input = Vec::new();
    let mut buffer = [0u8; 1024];

    loop {
        if let Ok(message) = outbound.try_recv() {
            remember(&message, system, registrations);
            if let Err(error) = write_frame(stream, &message) {
                log::warn!("TCP write failed: {error}");
                return;
            }
        }

        match stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(count) => {
                // TCP reads may contain partial or multiple newline-delimited commands.
                input.extend_from_slice(&buffer[..count]);
                while let Some(end) = input.iter().position(|byte| *byte == b'\n') {
                    let line: Vec<u8> = input.drain(..=end).collect();
                    if line.len() > MAX_COMMAND_BYTES {
                        log::warn!("Wi-Fi command too long");
                        return;
                    }
                    if line.iter().all(u8::is_ascii_whitespace) {
                        continue;
                    }
                    match serde_json::from_slice::<IncomingCommand>(&line) {
                        Ok(command) => {
                            if incoming.send(command).is_err() {
                                return;
                            }
                        }
                        Err(error) => log::warn!("Invalid Wi-Fi command: {error}"),
                    }
                }
                if input.len() > MAX_COMMAND_BYTES {
                    log::warn!("Wi-Fi command too long");
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
                log::warn!("TCP read failed: {error}");
                return;
            }
        }
    }
}

fn remember(
    message: &ProtocolMessage,
    system: &mut Option<ProtocolMessage>,
    registrations: &mut Vec<ProtocolMessage>,
) {
    match message {
        ProtocolMessage::System(_) => *system = Some(message.clone()),
        ProtocolMessage::Registration(registration) => {
            if let Some(saved) = registrations.iter_mut().find(|saved| {
                matches!(saved, ProtocolMessage::Registration(item) if item.id == registration.id)
            }) {
                *saved = message.clone();
            } else {
                registrations.push(message.clone());
            }
        }
        ProtocolMessage::ModuleEvent(_) => {}
    }
}

fn write_frame(stream: &mut TcpStream, message: &ProtocolMessage) -> io::Result<()> {
    let mut frame = serde_json::to_vec(message)?;
    frame.push(b'\n');
    stream.write_all(&frame)
}

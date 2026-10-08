
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use esp_idf_svc::{
    bt::{
        gap::{DiscoveryMode, EspGap, GapEvent},
        spp::{self, EspSpp, SppConfig, SppEvent, Status},
        reduce_bt_memory, BtClassic, BtDriver, BtStatus,
    },
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    sys::{
        esp, esp_bt_gap_set_security_param, esp_bt_gap_ssp_confirm_reply,
        esp_bt_sp_param_t_ESP_BT_SP_IOCAP_MODE, ESP_BT_IO_CAP_NONE,
    },
};
use pinora_protocol::{
    bluetooth_config::{DEVICE_NAME, MAX_MESSAGE_BYTES, SERVICE_NAME, SPP_WRITE_CHUNK_BYTES},
    IncomingCommand, ProtocolMessage,
};

type ClassicDriver = BtDriver<'static, BtClassic>;
type ClassicGap = EspGap<'static, BtClassic, Arc<ClassicDriver>>;
type ClassicSpp = EspSpp<'static, BtClassic, Arc<ClassicDriver>>;

enum Work {
    Message(ProtocolMessage),
    Started,
    Connected(u32),
    Closed(u32),
    Data(u32, Vec<u8>),
    WriteDone(u32, bool, bool),
    Congestion(u32, bool),
}

pub struct Bluetooth {
    outbound: SyncSender<Work>,
    incoming: Mutex<Option<Receiver<IncomingCommand>>>,
    _gap: Arc<ClassicGap>,
    _spp: Arc<ClassicSpp>,
    _driver: Arc<ClassicDriver>,
}

impl Bluetooth {
    pub fn new(mut modem: Modem<'static>, nvs: EspDefaultNvsPartition) -> anyhow::Result<Self> {
        // Release BLE-only memory; keep NVS-backed state for Classic pairing.
        reduce_bt_memory(unsafe { modem.reborrow() })?;
        let driver = Arc::new(BtDriver::<BtClassic>::new(modem, Some(nvs))?);
        let gap = Arc::new(EspGap::new(driver.clone())?);
        gap.set_device_name(DEVICE_NAME)?;
        let config = SppConfig {
            mode: spp::Mode::Cb,
            enable_l2cap_ertm: true,
            tx_buffer_size: 0,
        };
        let spp = Arc::new(EspSpp::new(driver.clone(), &config)?);
        let (outbound, work_rx) = mpsc::sync_channel(128);
        let (incoming_tx, incoming) = mpsc::sync_channel(128);
        let overflow = Arc::new(AtomicBool::new(false));

        let gap_for_callback = gap.clone();
        gap.subscribe(move |event| match event {
            GapEvent::AuthenticationCompleted { status, .. } => {
                if status == BtStatus::Success {
                    log::info!("Bluetooth pairing authenticated");
                } else {
                    log::warn!("Bluetooth pairing failed: {status:?}");
                }
            }
            GapEvent::PairingPinRequest { bd_addr, .. } => {
                // No fixed legacy PIN is shipped. Windows should use SSP.
                if let Err(error) = gap_for_callback.reply_variable_pin(&bd_addr, None) {
                    log::warn!("Could not reject legacy PIN pairing: {error}");
                }
            }
            GapEvent::PairingUserConfirmationRequest { bd_addr, .. } => {
                // The development board has no input or display: SSP Just Works.
                if let Err(error) = esp!(unsafe {
                    esp_bt_gap_ssp_confirm_reply(&bd_addr as *const _ as *mut _, true)
                }) {
                    log::warn!("Bluetooth SSP confirmation failed: {error}");
                }
            }
            _ => {}
        })?;

        let work_tx = outbound.clone();
        let callback_overflow = overflow.clone();
        // ESP-IDF callbacks only copy data and queue work; parsing and writes run on the worker.
        spp.subscribe(move |event| {
            let work = match event {
                SppEvent::Start { status: Status::Success, .. } => Some(Work::Started),
                SppEvent::ServerOpen { status: Status::Success, handle, .. } => {
                    Some(Work::Connected(handle))
                }
                SppEvent::Close { handle, .. } => Some(Work::Closed(handle)),
                SppEvent::DataInd { status: Status::Success, handle, length, data } => {
                    if data.is_null() || length as usize > MAX_MESSAGE_BYTES {
                        callback_overflow.store(true, Ordering::Relaxed);
                        None
                    } else {
                        let bytes = unsafe { std::slice::from_raw_parts(data, length as usize) };
                        Some(Work::Data(handle, bytes.to_vec()))
                    }
                }
                SppEvent::Write { status, handle, cong, .. } => {
                    Some(Work::WriteDone(handle, status == Status::Success, cong))
                }
                SppEvent::Cong { handle, cong, .. } => Some(Work::Congestion(handle, cong)),
                _ => None,
            };
            if let Some(work) = work {
                if work_tx.try_send(work).is_err() {
                    callback_overflow.store(true, Ordering::Relaxed);
                }
            }
        })?;

        let iocap = ESP_BT_IO_CAP_NONE as u8;
        esp!(unsafe {
            esp_bt_gap_set_security_param(
                esp_bt_sp_param_t_ESP_BT_SP_IOCAP_MODE,
                &iocap as *const _ as *mut _,
                1,
            )
        })?;
        gap.request_variable_pin()?;
        // The server becomes discoverable after ESP-IDF reports SPP startup complete.
        spp.start_server(spp::Security::Encrypt, spp::Role::Slave, 0, SERVICE_NAME)?;

        let worker_gap = gap.clone();
        let worker_spp = spp.clone();
        thread::Builder::new()
            .name("bluetooth-transport".into())
            .stack_size(8192)
            .spawn(move || run_worker(worker_gap, worker_spp, work_rx, incoming_tx, overflow))?;

        Ok(Self {
            outbound,
            incoming: Mutex::new(Some(incoming)),
            _gap: gap,
            _spp: spp,
            _driver: driver,
        })
    }

    pub fn event(&self, message: ProtocolMessage) -> Result<(), String> {
        let work = Work::Message(message);
        // Preserve discovery messages for reconnect; runtime events may be dropped under load.
        match &work {
            Work::Message(ProtocolMessage::System(_) | ProtocolMessage::Registration(_)) => self
                .outbound
                .send(work)
                .map_err(|error| format!("Bluetooth queue unavailable: {error}")),
            _ => self
                .outbound
                .try_send(work)
                .map_err(|error| format!("Bluetooth queue unavailable: {error}")),
        }
    }

    pub fn take_incoming(&self) -> Option<Receiver<IncomingCommand>> {
        self.incoming.lock().ok()?.take()
    }
}

fn run_worker(
    gap: Arc<ClassicGap>,
    spp: Arc<ClassicSpp>,
    work_rx: Receiver<Work>,
    incoming: SyncSender<IncomingCommand>,
    overflow: Arc<AtomicBool>,
) {
    let mut connection = None;
    let mut write_pending = false;
    let mut congested = false;
    let mut input = Vec::new();
    let mut pending = VecDeque::new();
    let mut system = None;
    let mut registrations = Vec::new();

    loop {
        if overflow.swap(false, Ordering::Relaxed) {
            // A lost callback may contain frame bytes or write state, so resynchronize by reconnecting.
            input.clear();
            pending.clear();
            if let Some(handle) = connection.take() {
                let _ = spp.disconnect(handle);
            }
            log::warn!("Bluetooth callback queue overflow; disconnecting to resynchronize");
        }
        let work = match work_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(work) => work,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        };
        match work {
            Work::Started => {
                if let Err(error) = gap.set_scan_mode(true, DiscoveryMode::Discoverable) {
                    log::error!("Bluetooth advertising failed: {error}");
                } else {
                    log::info!("Pinora SPP service is discoverable");
                }
            }
            Work::Connected(handle) => {
                if connection.is_some() {
                    let _ = spp.disconnect(handle);
                    continue;
                }
                connection = Some(handle);
                write_pending = false;
                congested = false;
                input.clear();
                pending.clear();
                // Only system and registration messages are replayed; live state is not cached.
                for message in system.iter().chain(registrations.iter()) {
                    enqueue(message, &mut pending);
                }
                log::info!("Bluetooth SPP connected");
            }
            Work::Closed(handle) if connection == Some(handle) => {
                connection = None;
                write_pending = false;
                congested = false;
                input.clear();
                pending.clear();
                log::info!("Bluetooth SPP disconnected");
            }
            Work::Data(handle, bytes) if connection == Some(handle) => {
                // SPP callbacks can split or combine newline-delimited JSON commands.
                for byte in bytes {
                    if byte == b'\n' {
                        if !input.iter().all(u8::is_ascii_whitespace) {
                            match serde_json::from_slice::<IncomingCommand>(&input) {
                                Ok(command) => {
                                    if incoming.send(command).is_err() {
                                        return;
                                    }
                                }
                                Err(error) => log::warn!("Invalid Bluetooth command: {error}"),
                            }
                        }
                        input.clear();
                    } else if input.len() < MAX_MESSAGE_BYTES {
                        input.push(byte);
                    } else {
                        log::warn!("Bluetooth command too long; disconnecting");
                        input.clear();
                        connection = None;
                        pending.clear();
                        let _ = spp.disconnect(handle);
                        break;
                    }
                }
            }
            Work::WriteDone(handle, success, is_congested) if connection == Some(handle) => {
                if !success {
                    log::warn!("Bluetooth SPP write failed; disconnecting");
                    connection = None;
                    pending.clear();
                    let _ = spp.disconnect(handle);
                } else {
                    write_pending = false;
                    congested = is_congested;
                }
            }
            Work::Congestion(handle, is_congested) if connection == Some(handle) => {
                congested = is_congested;
            }
            Work::Message(message) => {
                remember(&message, &mut system, &mut registrations);
                if connection.is_some() {
                    enqueue(&message, &mut pending);
                }
            }
            _ => {}
        }

        if let Some(handle) = connection {
            // Keep one SPP write in flight until completion and congestion events allow the next.
            if !write_pending && !congested {
                if let Some(frame) = pending.front_mut() {
                    let length = frame.len().min(SPP_WRITE_CHUNK_BYTES);
                    let mut chunk = frame[..length].to_vec();
                    match spp.write(handle, &mut chunk) {
                        Ok(()) => {
                            write_pending = true; // Wait for WRITE_EVT and congestion clearance.
                            frame.drain(..length);
                            if frame.is_empty() {
                                pending.pop_front();
                            }
                        }
                        Err(error) => {
                            log::warn!("Bluetooth SPP write rejected: {error}");
                            connection = None;
                            pending.clear();
                            let _ = spp.disconnect(handle);
                        }
                    }
                }
            }
        }
    }
}

fn enqueue(message: &ProtocolMessage, pending: &mut VecDeque<Vec<u8>>) {
    if pending.len() >= 128 {
        log::warn!("Bluetooth outbound queue full; dropping message");
        return;
    }
    match serde_json::to_vec(message) {
        Ok(mut frame) if frame.len() < MAX_MESSAGE_BYTES => {
            frame.push(b'\n');
            pending.push_back(frame);
        }
        Ok(_) => log::warn!("Bluetooth protocol message exceeds size limit"),
        Err(error) => log::warn!("Bluetooth serialization failed: {error}"),
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

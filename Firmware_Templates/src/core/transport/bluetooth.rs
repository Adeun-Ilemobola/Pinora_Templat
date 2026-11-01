//! Bluetooth Classic SPP transport (Bluedroid). This is the transport selected by the
//! current image.
//!
//! ESP-IDF callbacks (GAP, SPP) only copy data and push a `Work` item onto a bounded
//! queue (128). A dedicated worker thread owns all state: the single connection
//! handle, the partial-line buffer, the outbound frame queue and the replay cache.
//! The worker enforces:
//! * one connection at a time (later `ServerOpen`s are disconnected);
//! * inbound lines framed by the shared [`LineFramer`] (`MAX_LINE_BYTES`); a longer
//!   line disconnects;
//! * one SPP write in flight; the next chunk (≤ `SPP_WRITE_CHUNK_BYTES`) waits for
//!   `WriteDone` and for congestion to clear;
//! * a callback-queue overflow drops the link to resynchronize framing;
//! * on connect, the cached system snapshot and registrations are replayed;
//! * runtime messages dropped at the work queue or the pending-frame queue are
//!   counted in [`DropCounters`] and reported periodically (Phase 2B, PIN-TPL-010).
//!
//! Security (PIN-TPL-003): SSP Just Works pairing is accepted only during the
//! pairing window after boot (Phase 1C). Since Phase 2B every connection also has
//! an application-level [`DeviceSession`]: a device with a key is read-only until
//! the peer completes the HMAC handshake, and then accepts only signed commands
//! (see `auth_gate`).

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crate::core::transport::auth_gate::{process_line, Action};
use crate::core::transport::device_key::{random_bytes, KeyStore};
use crate::core::transport::discovery_cache::remember;
use crate::core::transport::drop_stats::{DropCounters, Stage};
use crate::core::transport::pairing_window::{PairingWindow, PAIRING_WINDOW};
use esp_idf_svc::{
    bt::{
        gap::{DiscoveryMode, EspGap, GapEvent},
        reduce_bt_memory,
        spp::{self, EspSpp, SppConfig, SppEvent, Status},
        BtClassic, BtDriver, BtStatus,
    },
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    sys::{
        esp, esp_bt_gap_set_security_param, esp_bt_gap_ssp_confirm_reply,
        esp_bt_sp_param_t_ESP_BT_SP_IOCAP_MODE, ESP_BT_IO_CAP_NONE,
    },
};
use pinora_protocol::{
    auth::DeviceSession,
    bluetooth_config::{DEVICE_NAME, MAX_MESSAGE_BYTES, SERVICE_NAME, SPP_WRITE_CHUNK_BYTES},
    framing::{encode_line, EncodeError, Frame, LineFramer, MAX_LINE_BYTES},
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
    drops: Arc<DropCounters>,
    _gap: Arc<ClassicGap>,
    _spp: Arc<ClassicSpp>,
    _driver: Arc<ClassicDriver>,
}

impl Bluetooth {
    pub fn new(
        mut modem: Modem<'static>,
        nvs: EspDefaultNvsPartition,
        drops: Arc<DropCounters>,
        keys: Arc<KeyStore>,
    ) -> anyhow::Result<Self> {
        // Release BLE-only memory; keep NVS-backed state for Classic pairing.
        // SAFETY: the reborrowed modem is used only for this call, before BtDriver takes ownership.
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
        // Set by the worker while the pairing window is open; read by the GAP callback.
        let pairing_open = Arc::new(AtomicBool::new(false));

        let gap_for_callback = gap.clone();
        let callback_pairing_open = pairing_open.clone();
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
                // The development board has no input or display: SSP Just Works. New
                // pairings are accepted only while the pairing window is open (PIN-TPL-003).
                let accept = callback_pairing_open.load(Ordering::Acquire);
                if !accept {
                    log::warn!("Bluetooth pairing rejected: the pairing window is closed");
                }
                // (`EspGap::reply_ssp_confirm` is cfg-gated off in this build, so call ESP-IDF directly.)
                if let Err(error) = esp!(unsafe {
                    // SAFETY: `bd_addr` is a valid 6-byte address from the event; ESP-IDF only reads it.
                    esp_bt_gap_ssp_confirm_reply(&bd_addr as *const _ as *mut _, accept)
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
                SppEvent::Start {
                    status: Status::Success,
                    ..
                } => Some(Work::Started),
                SppEvent::ServerOpen {
                    status: Status::Success,
                    handle,
                    ..
                } => Some(Work::Connected(handle)),
                SppEvent::Close { handle, .. } => Some(Work::Closed(handle)),
                SppEvent::DataInd {
                    status: Status::Success,
                    handle,
                    length,
                    data,
                } => {
                    if data.is_null() || length as usize > MAX_MESSAGE_BYTES {
                        callback_overflow.store(true, Ordering::Relaxed);
                        None
                    } else {
                        // SAFETY: `data` is non-null (checked above) and valid for `length` bytes for the duration of this callback; it is copied before returning.
                        let bytes = unsafe { std::slice::from_raw_parts(data, length as usize) };
                        Some(Work::Data(handle, bytes.to_vec()))
                    }
                }
                SppEvent::Write {
                    status,
                    handle,
                    cong,
                    ..
                } => Some(Work::WriteDone(handle, status == Status::Success, cong)),
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
            // SAFETY: `iocap` is a live u8 and the length passed is 1; ESP-IDF only reads it.
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
        let worker_drops = drops.clone();
        thread::Builder::new()
            .name("bluetooth-transport".into())
            .stack_size(8192)
            .spawn(move || {
                run_worker(
                    worker_gap,
                    worker_spp,
                    work_rx,
                    incoming_tx,
                    overflow,
                    pairing_open,
                    worker_drops,
                    keys,
                )
            })?;

        Ok(Self {
            outbound,
            incoming: Mutex::new(Some(incoming)),
            drops,
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
            _ => match self.outbound.try_send(work) {
                Ok(()) => Ok(()),
                Err(mpsc::TrySendError::Full(_)) => {
                    self.drops.record(Stage::Transport);
                    Ok(())
                }
                Err(error) => Err(format!("Bluetooth queue unavailable: {error}")),
            },
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
    pairing_open: Arc<AtomicBool>,
    drops: Arc<DropCounters>,
    keys: Arc<KeyStore>,
) {
    let mut session = DeviceSession::new(keys.device_id());
    let mut connection = None;
    let mut write_pending = false;
    let mut congested = false;
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    let mut pending = VecDeque::new();
    let mut system = None;
    let mut registrations = Vec::new();
    let started = Instant::now();
    let mut pairing = PairingWindow::new(PAIRING_WINDOW);

    loop {
        if pairing.poll_expired(started.elapsed()) {
            // Window over: stop accepting new pairings and stop advertising. Bonded peers
            // can still connect because the device stays connectable.
            pairing_open.store(false, Ordering::Release);
            match gap.set_scan_mode(true, DiscoveryMode::NonDiscoverable) {
                Ok(()) => {
                    log::info!("Bluetooth pairing window closed; device is no longer discoverable")
                }
                Err(error) => log::error!("Bluetooth could not leave discoverable mode: {error}"),
            }
        }
        if overflow.swap(false, Ordering::Relaxed) {
            // A lost callback may contain frame bytes or write state, so resynchronize by reconnecting.
            framer.clear();
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
                } else if pairing.open(started.elapsed()) {
                    pairing_open.store(true, Ordering::Release);
                    log::info!(
                        "Pinora SPP service is discoverable; pairing accepted for {} s",
                        PAIRING_WINDOW.as_secs()
                    );
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
                framer.clear();
                session.reset();
                pending.clear();
                // Only system and registration messages are replayed; live state is not cached.
                for message in system.iter().chain(registrations.iter()) {
                    enqueue(message, &mut pending, &drops);
                }
                log::info!("Bluetooth SPP connected");
            }
            Work::Closed(handle) if connection == Some(handle) => {
                connection = None;
                write_pending = false;
                congested = false;
                framer.clear();
                pending.clear();
                session.reset();
                log::info!("Bluetooth SPP disconnected");
            }
            Work::Data(handle, bytes) if connection == Some(handle) => {
                // SPP callbacks can split or combine newline-delimited JSON commands.
                for frame in framer.push(&bytes) {
                    match frame {
                        Frame::Line(line) => {
                            let key = keys.current();
                            match process_line(&mut session, &line, key.as_ref(), random_bytes) {
                                Ok(actions) => {
                                    for action in actions {
                                        match action {
                                            Action::Deliver(command) => {
                                                if incoming.send(command).is_err() {
                                                    return;
                                                }
                                            }
                                            Action::Send(message) => {
                                                enqueue(&message, &mut pending, &drops)
                                            }
                                        }
                                    }
                                }
                                Err(error) => log::warn!("Invalid Bluetooth command: {error}"),
                            }
                        }
                        Frame::Overflow => {
                            log::warn!("Bluetooth command too long; disconnecting");
                            framer.clear();
                            session.reset();
                            connection = None;
                            pending.clear();
                            let _ = spp.disconnect(handle);
                            break;
                        }
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
                    enqueue(&message, &mut pending, &drops);
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

fn enqueue(message: &ProtocolMessage, pending: &mut VecDeque<Vec<u8>>, drops: &DropCounters) {
    if pending.len() >= 128 {
        drops.record(Stage::Link);
        return;
    }
    match encode_line(message) {
        Ok(frame) => pending.push_back(frame),
        Err(EncodeError::TooLong(length)) => {
            log::warn!("Bluetooth protocol message of {length} bytes exceeds the line limit");
            drops.record(Stage::Link);
        }
        Err(error) => log::warn!("Bluetooth serialization failed: {error}"),
    }
}

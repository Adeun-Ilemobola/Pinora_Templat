//! Selects and owns the image's transport, and runs the sender thread that moves
//! messages from the [`TransportEmitter`] queue to that transport.
//!
//! Queueing (PIN-TPL-010): emitter queue (128) → sender thread → transport queue
//! (128) → transport worker. Registration and system messages block at each stage;
//! runtime events are dropped when a stage is full. Since audit Phase 2B every drop
//! is counted ([`DropCounters`]) and the sender thread reports the totals as one
//! Medium SysLog (source `"log"`) every [`REPORT_INTERVAL`].

use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    hal::modem::Modem,
    nvs::{EspNvsPartition, NvsDefault},
};

use pinora_protocol::{
    registration::ProtocolMessage, EventPackage, IncomingCommand, LogPriority, ModuleEvent,
    SysLogEvent,
};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use std::{
    sync::{mpsc::Receiver, Arc},
    thread,
};

use crate::core::transport::drop_stats::{DropCounters, REPORT_INTERVAL};
use crate::core::transport::serial;
use crate::core::transport::wifi::Wifi;
use crate::core::transport::{bluetooth::Bluetooth, emitter::TransportEmitter};

/// Which transport this firmware image uses. Chosen in source; no runtime switching.
#[derive(Debug, Clone, Copy)]
pub enum TransportType {
    Wifi,
    Bluetooth,
    Serial,
}

enum Core {
    Wifi(Wifi),
    Bluetooth(Bluetooth),
}
pub struct TransportCore {
    pub transport_type: Arc<TransportType>,
    core: Arc<Mutex<Option<Core>>>,
    pub emitter: TransportEmitter,
}

impl TransportCore {
    pub fn new(
        transport_type: TransportType,
        sys_loop: EspEventLoop<System>,
        nvs: EspNvsPartition<NvsDefault>,
        modem: Modem<'static>,
    ) -> anyhow::Result<Self> {
        let (emitter, receiver) = TransportEmitter::new();
        let drops = emitter.drop_counters();

        // Firmware selects one ingress/egress transport at startup; protocol messages stay shared.
        let core = match transport_type {
            TransportType::Wifi => Some(Core::Wifi(Wifi::new(sys_loop, nvs, modem, drops)?)),
            TransportType::Bluetooth => Some(Core::Bluetooth(Bluetooth::new(modem, nvs, drops)?)),
            _ => None,
        };
        let mut transport_core = TransportCore {
            transport_type: Arc::new(transport_type),
            core: Arc::new(Mutex::new(core)),
            emitter,
        };
        transport_core.build_sender(transport_type, receiver);
        Ok(transport_core)
    }

    fn build_sender(&mut self, transport: TransportType, tr: Receiver<ProtocolMessage>) {
        let core: Arc<Mutex<Option<Core>>> = Arc::clone(&self.core);
        let receiver = tr;
        let drops = self.emitter.drop_counters();

        thread::spawn(move || {
            let mut last_report = Instant::now();
            loop {
                match receiver.recv_timeout(Duration::from_millis(500)) {
                    Ok(event) => {
                        if let Err(error) = Self::send(event, transport, Arc::clone(&core)) {
                            log::error!("Failed to emit event: {error}");
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
                if last_report.elapsed() >= REPORT_INTERVAL {
                    last_report = Instant::now();
                    Self::report_drops(&drops, transport, &core);
                }
            }
        });
    }

    /// Sends one SysLog summarizing the drops since the last report, if there were any.
    fn report_drops(
        drops: &DropCounters,
        transport: TransportType,
        core: &Arc<Mutex<Option<Core>>>,
    ) {
        let Some(report) = drops.take() else {
            return;
        };
        let text = report.describe();
        log::warn!("{text}");
        let message = ProtocolMessage::ModuleEvent(EventPackage {
            id: "log".to_string(),
            event: ModuleEvent::SysLog(SysLogEvent {
                text,
                raw_err: None,
                priority: LogPriority::Medium,
            }),
        });
        if let Err(error) = Self::send(message, transport, Arc::clone(core)) {
            log::error!("Failed to report dropped telemetry: {error}");
        }
    }

    fn send(
        data: ProtocolMessage,
        mode: TransportType,
        core: Arc<Mutex<Option<Core>>>,
    ) -> Result<(), String> {
        match mode {
            TransportType::Wifi => {
                let guard = core.lock().map_err(|error| error.to_string())?;
                if let Some(Core::Wifi(wifi)) = &*guard {
                    wifi.event(data)?;
                }
            }
            TransportType::Bluetooth => {
                let guard = core.lock().map_err(|error| error.to_string())?;
                if let Some(Core::Bluetooth(bluetooth)) = &*guard {
                    bluetooth.event(data)?;
                }
            }
            TransportType::Serial => {
                serial::send(&data)?;
            }
        }

        Ok(())
    }

    /// The receiver of commands from the selected transport. Call it once: a second
    /// call is an error (before Phase 2B it returned a receiver that never yielded,
    /// PIN-TPL-016).
    pub fn handle_incoming(&self) -> anyhow::Result<Receiver<IncomingCommand>> {
        let guard = self
            .core
            .lock()
            .map_err(|_| anyhow::anyhow!("transport lock poisoned"))?;
        match (*self.transport_type, &*guard) {
            (TransportType::Wifi, Some(Core::Wifi(wifi))) => wifi.take_incoming(),
            (TransportType::Bluetooth, Some(Core::Bluetooth(bluetooth))) => {
                bluetooth.take_incoming()
            }
            (TransportType::Serial, _) => {
                let (command_sender, command_receiver) = mpsc::channel::<IncomingCommand>();
                std::thread::spawn(move || serial::command_reader(&command_sender));
                Some(command_receiver)
            }
            _ => None,
        }
        .ok_or_else(|| anyhow::anyhow!("the transport's command receiver was already taken"))
    }
}

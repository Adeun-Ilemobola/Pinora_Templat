//! Selects and owns the image's transport, and runs the sender thread that moves
//! messages from the [`TransportEmitter`] queue to that transport.
//!
//! Queueing (PIN-TPL-010): emitter queue (128) → sender thread → transport queue
//! (128) → transport worker. Registration and system messages block at each stage;
//! runtime events are dropped when a stage is full.

use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    hal::modem::Modem,
    nvs::{EspNvsPartition, NvsDefault},
};

use pinora_protocol::{registration::ProtocolMessage, IncomingCommand};
use std::sync::{mpsc, Mutex};
use std::{
    sync::{mpsc::Receiver, Arc},
    thread,
};

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

        // Firmware selects one ingress/egress transport at startup; protocol messages stay shared.
        let core = match transport_type {
            TransportType::Wifi => Some(Core::Wifi(Wifi::new(sys_loop, nvs, modem)?)),
            TransportType::Bluetooth => Some(Core::Bluetooth(Bluetooth::new(modem, nvs)?)),
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

        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                if let Err(error) = Self::send(event, transport, Arc::clone(&core)) {
                    log::error!("Failed to emit event: {error}");
                }
            }
        });
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

    pub fn handle_incoming(&self) -> Receiver<IncomingCommand> {
        if matches!(*self.transport_type, TransportType::Wifi) {
            if let Ok(guard) = self.core.lock() {
                if let Some(Core::Wifi(wifi)) = &*guard {
                    if let Some(receiver) = wifi.take_incoming() {
                        return receiver;
                    }
                }
            }
        }
        if matches!(*self.transport_type, TransportType::Bluetooth) {
            if let Ok(guard) = self.core.lock() {
                if let Some(Core::Bluetooth(bluetooth)) = &*guard {
                    if let Some(receiver) = bluetooth.take_incoming() {
                        return receiver;
                    }
                }
            }
        }

        let (command_sender, command_receiver) = mpsc::channel::<IncomingCommand>();
        if matches!(*self.transport_type, TransportType::Serial) {
            std::thread::spawn(move || serial::command_reader(&command_sender));
        }
        command_receiver
    }
}

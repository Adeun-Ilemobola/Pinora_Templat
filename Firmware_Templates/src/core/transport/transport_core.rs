use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    hal::modem::Modem,
    nvs::{EspNvsPartition, NvsDefault},
};
use pinora_protocol::{registration::ProtocolMessage, IncomingCommand};
use std::io::{BufRead, ErrorKind};
use std::{
    io,
    sync::{mpsc, Mutex},
};
use std::{
    sync::{mpsc::Receiver, Arc},
    thread,
};

use crate::core::transport::wifi::Wifi;
use crate::core::transport::{bluetooth::Bluetooth, transport_emiter::TransportEmiter};

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
    pub emitter: TransportEmiter,
}

impl TransportCore {
    pub fn new(
        transport_type: TransportType,
        sys_loop: EspEventLoop<System>,
        nvs: EspNvsPartition<NvsDefault>,
        modem: Modem<'static>,
    ) -> anyhow::Result<Self> {
        let (emitter, receiver) = TransportEmiter::new();

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
                let serialized = serde_json::to_string(&data).map_err(|error| error.to_string())?;
                println!("{serialized}");
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
            std::thread::spawn(move || Self::serial_command_reader(&command_sender));
        }
        command_receiver
    }

    fn serial_command_reader(command_sender: &mpsc::Sender<IncomingCommand>) {
        let stdin = io::stdin();

        for line_result in stdin.lock().lines() {
            let line = match line_result {
                Ok(line) => line,
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    // Not an error — just no data yet. Back off and retry.
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(err) => {
                    log::error!("Real serial read error: {:?}", err);
                    continue;
                }
            };

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<IncomingCommand>(line) {
                Ok(command) => {
                    log::info!("Parsed command: {:?}", command);

                    let _ = command_sender.send(command);
                }

                Err(err) => {
                    log::error!("Failed to parse command: {:?}", err);
                    log::error!("Raw line was: {}", line);
                }
            }
        }
    }
}

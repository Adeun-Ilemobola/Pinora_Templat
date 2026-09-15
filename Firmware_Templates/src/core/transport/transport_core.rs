use pinora_protocol::{
    registration::{ProtocolMessage, SystemInfo},
    IncomingCommand,
};
use std::io;
use std::io::{BufRead, ErrorKind};
use std::{
    fmt,
    sync::{
        mpsc::{self, Sender, SyncSender, TrySendError , Receiver},
        Arc,
    },
    thread,
};

use crate::core::transport::bluetooth::Bluetooth;
use crate::core::transport::wifi::Wifi;

#[derive(Debug, Clone, Copy)]
pub enum TransportType {
    Wifi,
    Bluetooth,
    Serial,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitterError {
    Disconnected,
}

impl fmt::Display for EmitterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("event emitter is disconnected")
    }
}
impl std::error::Error for EmitterError {}

#[derive(Debug, Clone)]
enum Core {
    Wifi(Wifi),
    Bluetooth(Bluetooth),
}
#[derive(Debug, Clone)]
pub struct TransportCore {
    pub transport_type: Arc<TransportType>,
    core: Arc<Option<Core>>,
    pub sender: Option<SyncSender<ProtocolMessage>>,
}

impl TransportCore {
    pub fn new(transport_type: TransportType) -> Self {
        let core = match transport_type {
            TransportType::Wifi => Some(Core::Wifi(Wifi::new())),
            TransportType::Bluetooth => Some(Core::Bluetooth(Bluetooth::new())),
            _ => None,
        };
        let mut transport_core = TransportCore {
            transport_type: Arc::new(transport_type),
            core: Arc::new(core),
            sender: None,
        };
        transport_core.sender = Some(transport_core.build_sender(transport_type));
        transport_core
    }
    fn build_sender(&mut self, transport: TransportType) -> SyncSender<ProtocolMessage> {
        let core: Arc<Option<Core>> = Arc::clone(&self.core);
        let (sender, receiver) = mpsc::sync_channel::<ProtocolMessage>(128);

        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                if let Err(error) = Self::send(event, transport, Arc::clone(&core)) {
                    log::error!("Failed to emit event: {error}");
                }
            }
        });

        sender
    }

    fn send(
        data: ProtocolMessage,
        mode: TransportType,
        core: Arc<Option<Core>>,
    ) -> Result<(), String> {
        if let Some(_) = &*core {
            match mode {
                TransportType::Wifi => {
                    if let Some(Core::Wifi(wifi)) = &*core {
                        // Handle Wifi-specific sending logic here
                        wifi.event(data.clone());
                    }
                }
                TransportType::Bluetooth => {
                    if let Some(Core::Bluetooth(bluetooth)) = &*core {
                        // Handle Bluetooth-specific sending logic here
                        bluetooth.event(data.clone());
                    }
                }
                _ => {
                    // Handle other transport types or unsupported cases here
                }
            }
        }

        let serialized = serde_json::to_string(&data).map_err(|error| error.to_string())?;
        println!("{serialized}");

        Ok(())
    }
    pub fn emit_reliable(&self, message: ProtocolMessage) -> Result<(), EmitterError> {
        if let Some(sender) = &self.sender {
            return sender.send(message).map_err(|_| EmitterError::Disconnected);
        }
        Err(EmitterError::Disconnected)
    }

    pub fn try_emit(&self, message: ProtocolMessage) {
        if let Some(sender) = &self.sender {
            match sender.try_send(message) {
                Ok(()) => {}

                Err(TrySendError::Full(_)) => {
                    log::warn!("Event queue is full; dropping runtime message");
                }

                Err(TrySendError::Disconnected(_)) => {
                    log::error!("Event emitter is disconnected");
                }
            }
            return;
        }
        // match self.sender.try_send(message) {
        //     Ok(()) => {}

        //     Err(TrySendError::Full(_)) => {
        //         log::warn!("Event queue is full; dropping runtime message");
        //     }

        //     Err(TrySendError::Disconnected(_)) => {
        //         log::error!("Event emitter is disconnected");
        //     }
        // }
    }
    pub fn system_info(&self, data: SystemInfo) -> Result<(), EmitterError> {
        self.emit_reliable(ProtocolMessage::System(data))
    }

    pub fn any(&self, data: ProtocolMessage) {
        self.try_emit(data);
    }

    pub fn handle_incoming(&self) -> Receiver<IncomingCommand> {
        // Implement handling of incoming protocol messages here
        let core: Arc<Option<Core>> = Arc::clone(&self.core);
        let transport_type: Arc<TransportType> = Arc::clone(&self.transport_type);

        let (command_sender, command_receiver) = mpsc::channel::<IncomingCommand>();
        std::thread::spawn(move || {
            match &*transport_type {
                TransportType::Bluetooth => {
                    if let Some(Core::Bluetooth(bluetooth)) = &*core {
                        // Handle Bluetooth-specific incoming logic here
                    }
                }
                TransportType::Wifi => {
                    if let Some(Core::Wifi(wifi)) = &*core {
                        // Handle Wifi-specific incoming logic here
                    }
                }
                TransportType::Serial => {
                   
                        // Handle Serial-specific incoming logic here
                        Self::serial_command_reader(command_sender);
                    
                }
                _ => {
                    // Handle other transport types or unsupported cases here
                }
            }
            
        });
        command_receiver
    }

    fn serial_command_reader(command_sender: mpsc::Sender<IncomingCommand>) {
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

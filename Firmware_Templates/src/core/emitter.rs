use pinora_protocol::registration::{ProtocolMessage, SystemInfo};
use std::{
    fmt,
    sync::mpsc::{self, SyncSender, TrySendError},
    thread,
};

#[derive(Debug, Clone, Copy)]
pub enum TransportType {
    Wifi,
    Bluetooth,
    Serialized,
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
pub struct Emitter {
    pub  sender: SyncSender<ProtocolMessage>,
}

impl Emitter {
    pub fn new(transport_type: Option<TransportType>) -> Emitter{
        let transport = transport_type.unwrap_or(TransportType::Serialized);

        Emitter {
            sender: Self::build_sender(transport),
        }
    }

    pub fn emit_reliable(&self, message: ProtocolMessage) -> Result<(), EmitterError> {
        self.sender
            .send(message)
            .map_err(|_| EmitterError::Disconnected)
    }

    pub fn try_emit(&self, message: ProtocolMessage) {
        match self.sender.try_send(message) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full; dropping runtime message");
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }
    }

    fn build_sender(transport: TransportType) -> SyncSender<ProtocolMessage> {
        let (sender, receiver) = mpsc::sync_channel::<ProtocolMessage>(128);

        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                if let Err(error) = Self::send(event, transport) {
                    log::error!("Failed to emit event: {error}");
                }
            }
        });

        sender
    }

     fn send(data: ProtocolMessage, mode: TransportType) -> Result<(), String> {
        match mode {
            TransportType::Bluetooth => {}
            TransportType::Serialized => {
                let serialized =
                serde_json::to_string(&data).map_err(|error| error.to_string())?;
                println!("{serialized}");
                
            }
            TransportType::Wifi => {}
        }

        Ok(())
    }
    pub fn system_info(&self, data: SystemInfo) -> Result<(), EmitterError> {
        self.emit_reliable(ProtocolMessage::System(data))
    }

    pub fn any(&self, data: ProtocolMessage) {
        self.try_emit(data);
    }
}

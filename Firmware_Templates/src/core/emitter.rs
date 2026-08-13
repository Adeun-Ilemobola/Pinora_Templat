use crate::protocol::registration::{ProtocolMessage, SystemInfo};
use std::{
    sync::mpsc::{self, SyncSender, TrySendError},
    thread,
};

#[derive(Debug, Clone, Copy)]
pub enum TransportType {
    Wifi,
    Bluetooth,
    Serialized,
}

#[derive(Debug, Clone)]
pub struct Emitter {
    pub  sender: SyncSender<ProtocolMessage>,
    transport: TransportType,
}

impl Emitter {
    pub fn new(transport_type: Option<TransportType>) -> Emitter{
        let transport = transport_type.unwrap_or(TransportType::Serialized);

        let new_emitter = Emitter {
            sender: Self::build_sender(transport),
            transport,
        };
        new_emitter
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
    pub fn system_info(&self ,data: SystemInfo){
         match  self.sender.try_send(ProtocolMessage::System(data)) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full for ");
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }

    }
    pub fn  any(&self , data:ProtocolMessage){

         match  self.sender.try_send(data) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full for ");
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }

    }
}

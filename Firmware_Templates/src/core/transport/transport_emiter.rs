

use std::{
    fmt,
    sync::{
        mpsc::{ SyncSender, TrySendError , Receiver , sync_channel},
    },
};

use pinora_protocol::{ProtocolMessage, SystemInfo};




#[derive(Debug)]
pub enum EmitterError {
    Disconnected,
}
impl std::error::Error for EmitterError {}

impl fmt::Display for EmitterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("event emitter is disconnected")
    }
}
#[derive(Debug, Clone)]
pub struct TransportEmiter {
    tx: SyncSender<ProtocolMessage>,

}

impl TransportEmiter {
    pub fn new() -> (Self , Receiver<ProtocolMessage>) {
        let (tx, rx) = sync_channel::<ProtocolMessage>(128);
        (Self { tx }, rx)
    }

    pub fn system_info(&self, data: SystemInfo) -> Result<(), EmitterError> {
        self.emit_reliable(ProtocolMessage::System(data))
    }

    pub fn any(&self, data: ProtocolMessage) {
        self.try_emit(data);
    }
    pub fn emit_reliable(&self, message: ProtocolMessage) -> Result<(), EmitterError> {
        self.tx.send(message).map_err(|_| EmitterError::Disconnected)?;
        Ok(())
    }
    pub fn try_emit(&self, message: ProtocolMessage) {
        match self.tx.try_send(message) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full; dropping runtime message");
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }
    }

}
   


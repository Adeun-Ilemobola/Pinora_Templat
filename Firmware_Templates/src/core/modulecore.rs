use std::sync::mpsc::{SyncSender, TrySendError};

use uuid::Uuid;

use crate::protocol::{command::ModuleCommand, global_definitions::ModuleType, module_event::ModuleEvent, registration::{ProtocolMessage, Registration}};

#[derive(Debug, Clone)]
pub struct ModuleCore {
    pub id: String,
    pub module_type: ModuleType,
    pub manuel_id: String,
    event_sender: SyncSender<ProtocolMessage>,

}

impl ModuleCore {
    pub fn new(module_type: ModuleType, manuel_id: &str , sender:SyncSender<ProtocolMessage>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            module_type: module_type,
            manuel_id: manuel_id.to_string(),
            event_sender:sender
        }
    }

}

pub trait Module {
    fn core(&self) -> &ModuleCore;

    fn id(&self) -> &str {
        &self.core().id
    }

    fn get_module_type(&self) -> &ModuleType {
        &self.core().module_type
    }

    fn emit(&self, event: ModuleEvent) {
        match self.core().event_sender.try_send(ProtocolMessage::ModuleEvent(event)) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full for module {}", self.id().to_string());
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }
    }

    fn Registration(&self, registration: Registration) {
        match self.core().event_sender.try_send(ProtocolMessage::Registration(registration)) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full for module {}", self.id().to_string());
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()>;
}

pub mod emit {
    use crate::protocol::{
        registration::{ProtocolMessage, Registration, SystemInfo},
    };
    use std::{
        sync::mpsc::{self, SyncSender},
        thread,
    };
    pub fn start_event_emitter() -> SyncSender<ProtocolMessage> {
        let (sender, receiver) = mpsc::sync_channel::<ProtocolMessage>(128);

        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                if let Err(error) = emit_event(event) {
                    log::error!("Failed to emit event: {error}");
                }
            }
        });

        sender
    }

    fn emit_event(event: ProtocolMessage) -> Result<(), String> {
        let serialized = serde_json::to_string(&event).map_err(|error| error.to_string())?;

        // Send serialized data through USB, UART, WebSocket, etc.
        println!("{serialized}");

        Ok(())  
    }

    pub fn registration(data: Registration) {
        // serialize and send registration
        match emit_event(ProtocolMessage::Registration(data)) {
            Ok(_)=>{}
            Err(err)=>{
                println!("Failed to serialize JSON: {}", err)
            }
            
        };
    }

    pub fn event(data: ProtocolMessage) {
        // serialize and send event
        match emit_event(data) {
            Ok(_)=>{}
            Err(err)=>{
                println!("Failed to serialize JSON: {}", err)
            }
            
        };
    }
    pub fn system_info(data: SystemInfo) {
        // serialize and send system info
        match emit_event(ProtocolMessage::System(data)) {
            Ok(_)=>{}
            Err(err)=>{
                println!("Failed to serialize JSON: {}", err)
            }
            
        };
    }

    // pub fn error(data: ErrorEvent) {
    //     // serialize and send error
    // }
}

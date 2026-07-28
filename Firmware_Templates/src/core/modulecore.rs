use uuid::Uuid;

use crate::protocol::{command::ModuleCommand, global_definitions::ModuleType};

#[derive(Debug, Clone)]
pub struct ModuleCore {
    pub id: String,
    pub module_type: ModuleType,
    pub manuel_id: String,
}

impl ModuleCore {
    pub fn new(module_type: ModuleType, manuel_id: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            module_type: module_type,
            manuel_id: manuel_id.to_string(),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_module_type(&self) -> &ModuleType {
        &self.module_type
    }
}

pub trait Module {
    fn core(&self) -> &ModuleCore;
    fn id(&self) -> &String;
    fn get_module_type(&self) -> &ModuleType;
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()>;
}

pub mod emit {
    use crate::protocol::{
        module_event::ModuleEvent,
        registration::{ProtocolMessage, Registration, SystemInfo},
    };
    use std::{
        sync::mpsc::{self, SyncSender},
        thread,
    };
    pub fn start_event_emitter() -> SyncSender<ModuleEvent> {
        let (sender, receiver) = mpsc::sync_channel::<ModuleEvent>(128);

        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                if let Err(error) = emit_event(event) {
                    log::error!("Failed to emit event: {error}");
                }
            }
        });

        sender
    }

    fn emit_event(event: ModuleEvent) -> Result<(), String> {
        let message = ProtocolMessage::ModuleEvent(event);
        let serialized = serde_json::to_string(&message).map_err(|error| error.to_string())?;

        // Send serialized data through USB, UART, WebSocket, etc.
        println!("{serialized}");

        Ok(())
    }

    pub fn registration(data: Registration) {
        // serialize and send registration
        serde_json::to_string(&ProtocolMessage::Registration(data))
            .map(|s| println!("{}", s))
            .unwrap_or_else(|e| println!("Failed to serialize JSON: {}", e));
    }

    pub fn event(data: ModuleEvent) {
        // serialize and send event
        match emit_event(data) {
            Ok(_)=>{}
            Err(err)=>{
                println!("Failed to serialize JSON: {}", err)
            }
            
        };
    }
    pub fn system_info(data: SystemInfo) {
        serde_json::to_string(&ProtocolMessage::System(data))
            .map(|s| println!("{}", s))
            .unwrap_or_else(|e| println!("Failed to serialize JSON: {}", e));
    }

    // pub fn error(data: ErrorEvent) {
    //     // serialize and send error
    // }
}

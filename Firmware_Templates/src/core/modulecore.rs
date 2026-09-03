use std::sync::mpsc::{TrySendError};

use uuid::Uuid;

use crate::{core::emitter::Emitter};
use pinora_protocol::{command::ModuleCommand, global_definitions::ModuleType, module_event::{ModuleEvent, SysLogEvent}, registration::{ProtocolMessage, Registration}};
#[derive(Debug, Clone)]
pub struct ModuleCore {
    pub id: String,
    pub module_type: ModuleType,
    pub manuel_id: String,
    emitter: Emitter

}

impl ModuleCore {
    pub fn new(module_type: ModuleType, manuel_id: &str , emitter:Emitter) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            module_type: module_type,
            manuel_id: manuel_id.to_string(),
            emitter
        }
    }

}

pub trait Module {
    fn core(&self) -> &ModuleCore;

    fn id(&self) -> &str {
        &self.core().id
    }
    fn  tick(&mut self)->Result<() , ()>{
        Ok(())
    }

    fn get_module_type(&self) -> &ModuleType {
        &self.core().module_type
    }

    fn emit(&self, event: ModuleEvent) {
        match self.core().emitter.sender.try_send(ProtocolMessage::ModuleEvent(event)) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full for module {}", self.id().to_string());
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }
    }

    fn registration(&self, registration: Registration) {
        match self.core().emitter.sender.try_send(ProtocolMessage::Registration(registration)) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => {
                log::warn!("Event queue is full for module {}", self.id().to_string());
            }

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }
    }


    fn log(&self, data: SysLogEvent) {
        match self.core().emitter.sender.try_send(
            ProtocolMessage::ModuleEvent(ModuleEvent::SysLog(data))
        ) {
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

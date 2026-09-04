use uuid::Uuid;

use crate::core::emitter::{Emitter, EmitterError};
use pinora_protocol::{command::ModuleCommand, global_definitions::ModuleType, module_event::{ModuleEvent, SysLogEvent}, registration::{ProtocolMessage, Registration}};
#[derive(Debug, Clone)]
pub struct ModuleCore {
    pub id: String,
    pub module_type: ModuleType,
    pub manuel_id: String,
    pub parent_id: String,
    emitter: Emitter

}

impl ModuleCore {
    pub fn new(
        module_type: ModuleType,
        manuel_id: &str,
        parent_id: Option<String>,
        emitter: Emitter,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            module_type: module_type,
            manuel_id: manuel_id.to_string(),
            parent_id: parent_id.unwrap_or_default(),
            emitter
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleError {
    OperationFailed,
}

impl From<()> for ModuleError {
    fn from(_: ()) -> Self {
        Self::OperationFailed
    }
}

pub trait Module {
    fn core(&self) -> &ModuleCore;

    fn id(&self) -> &str {
        &self.core().id
    }
    fn tick(&mut self) -> Result<(), ModuleError>;

    fn register(&self) -> Result<(), EmitterError> {
        self.emit_registration()
    }

    fn emit_registration(&self) -> Result<(), EmitterError> {
        let registration = Registration {
            id: self.id().to_string(),
            module_type: self.get_module_type().clone(),
            lool_up_id: self.core().manuel_id.clone(),
            parent_id: self.core().parent_id.clone(),
        };

        self
            .core()
            .emitter
            .emit_reliable(ProtocolMessage::Registration(registration))
    }

    fn get_module_type(&self) -> &ModuleType {
        &self.core().module_type
    }

    fn emit(&self, event: ModuleEvent) {
        self.core()
            .emitter
            .try_emit(ProtocolMessage::ModuleEvent(event));
    }

    fn log(&self, data: SysLogEvent) {
        self.core()
            .emitter
            .try_emit(ProtocolMessage::ModuleEvent(ModuleEvent::SysLog(data)));
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()>;
}

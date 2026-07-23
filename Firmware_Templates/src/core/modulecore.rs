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
    use crate::protocol::{module_event::ModuleEvent, registration::{ProtocolMessage, Registration}};

    pub fn registration(data: Registration) {

        

        // serialize and send registration
        serde_json::to_string(&ProtocolMessage::Registration(data))
            .map(|s| println!("{}", s))
            .unwrap_or_else(|e| println!("Failed to serialize JSON: {}", e));

    }

    pub fn event(data: ModuleEvent) {
        // serialize and send event
         serde_json::to_string(&ProtocolMessage::ModuleEvent(data))
            .map(|s| println!("{}", s))
            .unwrap_or_else(|e| println!("Failed to serialize JSON: {}", e));

    }

    // pub fn error(data: ErrorEvent) {
    //     // serialize and send error
    // }
}

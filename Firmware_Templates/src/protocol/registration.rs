use serde::{Deserialize, Serialize};

use crate::protocol::{global_definitions::ModuleType, module_event::ModuleEvent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registration {
    pub id: String,
    pub module_type: ModuleType,
    pub lool_up_id: String,
    pub parent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "type", content = "payload")]
pub enum ProtocolMessage {
    Registration(Registration),
    ModuleEvent(ModuleEvent),
    System(SystemInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    pub esp_idf_version: String,
    pub total_heap: String,
    pub current_free_heap: String,
    pub lowest_free_heap: String,
    pub largest_allocation: String,
    pub maximum_app_slot: String,
    pub flash: String,
}

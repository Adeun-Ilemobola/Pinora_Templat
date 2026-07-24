use serde::{Deserialize, Serialize};

use crate::protocol::{global_definitions::ModuleType, module_event::ModuleEvent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registration {
    pub id: String,
    pub module_type: ModuleType,
    pub lool_up_id: String,
    pub parent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum ProtocolMessage {
    Registration(Registration),
    ModuleEvent(ModuleEvent),
}

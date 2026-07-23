use serde::{Deserialize, Serialize};

use crate::protocol::{global_definition_protocol::ModuleType, module_event::ModuleEvent};

#[derive(Debug, Serialize , Deserialize ,Clone )]
pub struct Registration {
    pub id: String,
    pub module_type: ModuleType,
    pub lool_up_id:String,
    pub parent_id:String

}

#[derive(Debug, Deserialize , Serialize ,Clone )]
#[serde(tag = "type", content = "payload")]
pub enum ProtocolMessage {
    Registration(Registration),
    ModuleEvent(ModuleEvent),
    // Command(Command),
}

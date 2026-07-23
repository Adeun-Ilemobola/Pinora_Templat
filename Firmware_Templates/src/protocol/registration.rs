use serde::{ Serialize , Deserialize};

use crate::protocol::{global_definitions::ModuleType, module_event::ModuleEvent};


// global_definitions
#[derive(Debug, Serialize  , Deserialize,Clone )]
pub struct Registration {
    pub id: String,
    pub module_type: ModuleType,
    pub lool_up_id:String,
    pub parent_id:String
}




#[derive(Debug, Serialize ,Clone )]
#[serde(tag = "type", content = "payload")]
pub enum ProtocolMessage {
    Registration(Registration),
    ModuleEvent(ModuleEvent),
    // Command(Command),
}
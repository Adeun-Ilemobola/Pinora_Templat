use serde::{Deserialize, Serialize};

use crate::{global_definitions::ModuleType, module_event::EventPackage};

/// Wire protocol version (audit Phase 2A). Version 1 adds `SystemInfo::protocol_version`
/// and `ModuleCommand::System`; everything else is unchanged from the unversioned
/// format, which reads as version 0.
pub const PROTOCOL_VERSION: u32 = 1;

/// Reserved command id for [`crate::SystemCommand`]s (contract C-05).
pub const SYSTEM_COMMAND_ID: &str = "SSI";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registration {
    pub id: String,
    pub module_type: ModuleType,
    /// Misspelling kept on the wire (C-01); `lookup_id` is accepted on input.
    #[serde(alias = "lookup_id")]
    pub lool_up_id: String,
    pub parent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtocolMessage {
    Registration(Registration),
    ModuleEvent(EventPackage),
    System(SystemInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    /// Protocol version of the sender. Absent in pre-v1 firmware, which reads as 0.
    #[serde(default)]
    pub protocol_version: u32,
    pub esp_idf_version: String,
    pub total_heap: String,
    pub current_free_heap: String,
    pub lowest_free_heap: String,
    pub largest_allocation: String,
    pub maximum_app_slot: String,
    pub flash: String,
}

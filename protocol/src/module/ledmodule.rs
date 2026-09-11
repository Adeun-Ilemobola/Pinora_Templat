use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub enum LedEvent {
    Brightness { level: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LedCommandPayload {
    SetState { state: u32 },
    Toggle {},
}

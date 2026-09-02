use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "command")]
pub enum RfidCommand {
    WriteMode,
    ReadMode,
    WritePayload { data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum RfidEvent {
    GetCard {
        id: String,
        card_uid: String,
        card_data: String,
    },
    GetMode {
        id: String,
        mode: MddeRfid,
    },
    GetWriteState {
        id: String,
        state: WriteState,
        info: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MddeRfid {
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WriteState {
    Good,
    Bad,
}

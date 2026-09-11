use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RfidCommand {
    WriteMode,
    ReadMode,
    WritePayload { data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RfidEvent {
    GetCard {
        card_uid: String,
        card_data: String,
    },
    GetMode {
        mode: MddeRfid,
    },
    GetWriteState {
        state: WriteState,
        info: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub enum MddeRfid {
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WriteState {
    Good,
    Bad,
}

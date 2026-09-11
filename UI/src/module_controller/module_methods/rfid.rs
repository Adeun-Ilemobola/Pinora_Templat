use pinora_protocol::{MddeRfid as ProtocolMddeRfid, RfidEvent, WriteState as ProtocolWriteState};

use crate::{MddeRfid, RfidState, WriteState};

impl From<ProtocolMddeRfid> for MddeRfid {
    fn from(mode: ProtocolMddeRfid) -> Self {
        match mode {
            ProtocolMddeRfid::Read => Self::Read,
            ProtocolMddeRfid::Write => Self::Write,
        }
    }
}

impl From<MddeRfid> for ProtocolMddeRfid {
    fn from(mode: MddeRfid) -> Self {
        match mode {
            MddeRfid::Read => Self::Read,
            MddeRfid::Write => Self::Write,
        }
    }
}

impl From<ProtocolWriteState> for WriteState {
    fn from(state: ProtocolWriteState) -> Self {
        match state {
            ProtocolWriteState::Good => Self::Good,
            ProtocolWriteState::Bad => Self::Bad,
        }
    }
}

impl From<WriteState> for ProtocolWriteState {
    fn from(state: WriteState) -> Self {
        match state {
            WriteState::Good => Self::Good,
            WriteState::Bad => Self::Bad,
        }
    }
}

impl RfidState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, id: &str, event: RfidEvent) {
        self.id = id.into();
        match event {
            RfidEvent::GetCard {
                card_uid,
                card_data,
            } => {
                self.has_card_uid = true;
                self.card_uid = card_uid.into();
                self.has_card_data = true;
                self.card_data = card_data.into();
            }
            RfidEvent::GetMode { mode } => {
                self.has_mode = true;
                self.mode = mode.into();
            }
            RfidEvent::GetWriteState { state, info } => {
                self.has_write_state = true;
                self.write_state = state.into();
                self.has_info = true;
                self.info = info.into();
            }
        }
    }
}

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

    pub fn update(&mut self, event: RfidEvent) {
        match event {
            RfidEvent::GetCard {
                id,
                card_uid,
                card_data,
            } => {
                self.id = id.into();
                self.has_card_uid = true;
                self.card_uid = card_uid.into();
                self.has_card_data = true;
                self.card_data = card_data.into();
            }
            RfidEvent::GetMode { id, mode } => {
                self.id = id.into();
                self.has_mode = true;
                self.mode = mode.into();
            }
            RfidEvent::GetWriteState { id, state, info } => {
                self.id = id.into();
                self.has_write_state = true;
                self.write_state = state.into();
                self.has_info = true;
                self.info = info.into();
            }
        }
    }
}

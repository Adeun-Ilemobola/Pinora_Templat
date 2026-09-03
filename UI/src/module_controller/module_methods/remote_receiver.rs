use crate::{RemoteButton, RemoteReceiverState};
use pinora_protocol::{RemoteButton as ProtocolRemoteButton, RemoteButtonEvent};

impl From<ProtocolRemoteButton> for RemoteButton {
    fn from(button: ProtocolRemoteButton) -> Self {
        match button {
            ProtocolRemoteButton::None => Self::None,
            ProtocolRemoteButton::Power => Self::Power,
            ProtocolRemoteButton::VolumeUp => Self::VolumeUp,
            ProtocolRemoteButton::FunctionStop => Self::FunctionStop,
            ProtocolRemoteButton::Previous => Self::Previous,
            ProtocolRemoteButton::PlayPause => Self::PlayPause,
            ProtocolRemoteButton::Next => Self::Next,
            ProtocolRemoteButton::Down => Self::Down,
            ProtocolRemoteButton::VolumeDown => Self::VolumeDown,
            ProtocolRemoteButton::Up => Self::Up,
            ProtocolRemoteButton::Zero => Self::Zero,
            ProtocolRemoteButton::Equalizer => Self::Equalizer,
            ProtocolRemoteButton::StopRepeat => Self::StopRepeat,
            ProtocolRemoteButton::One => Self::One,
            ProtocolRemoteButton::Two => Self::Two,
            ProtocolRemoteButton::Three => Self::Three,
            ProtocolRemoteButton::Four => Self::Four,
            ProtocolRemoteButton::Five => Self::Five,
            ProtocolRemoteButton::Six => Self::Six,
            ProtocolRemoteButton::Seven => Self::Seven,
            ProtocolRemoteButton::Eight => Self::Eight,
            ProtocolRemoteButton::Nine => Self::Nine,
        }
    }
}

impl From<RemoteButton> for ProtocolRemoteButton {
    fn from(button: RemoteButton) -> Self {
        match button {
            RemoteButton::None => Self::None,
            RemoteButton::Power => Self::Power,
            RemoteButton::VolumeUp => Self::VolumeUp,
            RemoteButton::FunctionStop => Self::FunctionStop,
            RemoteButton::Previous => Self::Previous,
            RemoteButton::PlayPause => Self::PlayPause,
            RemoteButton::Next => Self::Next,
            RemoteButton::Down => Self::Down,
            RemoteButton::VolumeDown => Self::VolumeDown,
            RemoteButton::Up => Self::Up,
            RemoteButton::Zero => Self::Zero,
            RemoteButton::Equalizer => Self::Equalizer,
            RemoteButton::StopRepeat => Self::StopRepeat,
            RemoteButton::One => Self::One,
            RemoteButton::Two => Self::Two,
            RemoteButton::Three => Self::Three,
            RemoteButton::Four => Self::Four,
            RemoteButton::Five => Self::Five,
            RemoteButton::Six => Self::Six,
            RemoteButton::Seven => Self::Seven,
            RemoteButton::Eight => Self::Eight,
            RemoteButton::Nine => Self::Nine,
        }
    }
}

impl RemoteReceiverState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: RemoteButtonEvent) {
        match event {
            RemoteButtonEvent::Click { id, key } => {
                self.id = id.into();
                self.key = key.into();
            }
        }
    }
}

use serde::{Deserialize, Serialize};




#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub enum RemoteButton {
    None,
    Power,
    VolumeUp,
    FunctionStop,

    Previous,
    PlayPause,
    Next,

    Down,
    VolumeDown,
    Up,

    Zero,
    Equalizer,
    StopRepeat,

    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl RemoteButton {
    pub fn from_command(command: u8) -> Self {
        match command {
            0x45 => (Self::Power),
            0x46 => Self::VolumeUp,
            0x47 => Self::FunctionStop,

            0x44 => Self::Previous,
            0x40 => Self::PlayPause,
            0x43 => Self::Next,

            0x07 => Self::Down,
            0x15 => Self::VolumeDown,
            0x09 => Self::Up,

            0x19 => Self::Equalizer,
            0x0D => Self::StopRepeat,

            // Num
            0x16 => Self::Zero,
            0x0C => Self::One,
            0x18 => Self::Two,
            0x5E => Self::Three,
            0x08 => Self::Four,
            0x1C => Self::Five,
            0x5A => Self::Six,
            0x42 => Self::Seven,
            0x52 => Self::Eight,
            0x4A => Self::Nine,

            _ => Self::None,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub enum RemoteButtonEvent {
    Click{id:String , key:RemoteButton}
}
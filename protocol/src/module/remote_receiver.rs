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
            69 => Self::Power,
            70 => Self::VolumeUp,
            71 => Self::FunctionStop,

            68 => Self::Previous,
            64 => Self::PlayPause,
            67 => Self::Next,

            7 => Self::Down,
            21 => Self::VolumeDown,
            9 => Self::Up,

            25 => Self::Equalizer,
            13 => Self::StopRepeat,

            // Num
            22 => Self::Zero,
            12 => Self::One,
            24 => Self::Two,
            94 => Self::Three,
            8 => Self::Four,
            28 => Self::Five,
            90 => Self::Six,
            66 => Self::Seven,
            82 => Self::Eight,
            74 => Self::Nine,

            _ => Self::None,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub enum RemoteButtonEvent {
    Click{id:String , key:RemoteButton}
}
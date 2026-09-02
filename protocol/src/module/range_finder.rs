use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RangefinderDistanceMode {
    Short,
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "command")]
pub enum RangefinderCommandPayload {
    StartRanging,
    StopRanging,
    SetTimingBudget { milliseconds: u16 },
    SetDistanceMode { mode: RangefinderDistanceMode },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum RangefinderEvent {
    Range {
        id: String,
        millimeters: u16,
    },
    RangingState {
        id: String,
        is_ranging: bool,
    },
    TimingBudget {
        id: String,
        milliseconds: u16,
    },
    DistanceMode {
        id: String,
        mode: RangefinderDistanceMode,
    },
    InvalidMeasurement {
        id: String,
        status: String,
    },
}

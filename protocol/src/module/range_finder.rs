use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RangefinderDistanceMode {
    Short,
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RangefinderCommandPayload {
    StartRanging,
    StopRanging,
    SetTimingBudget { milliseconds: u16 },
    SetDistanceMode { mode: RangefinderDistanceMode },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RangefinderEvent {
    Range {
        millimeters: u16,
    },
    RangingState {
        is_ranging: bool,
    },
    TimingBudget {
        milliseconds: u16,
    },
    DistanceMode {
        mode: RangefinderDistanceMode,
    },
    InvalidMeasurement {
        status: String,
    },
}

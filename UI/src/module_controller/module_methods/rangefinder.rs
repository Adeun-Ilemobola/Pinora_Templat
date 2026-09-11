use pinora_protocol::{
    RangefinderDistanceMode as ProtocolRangefinderDistanceMode, RangefinderEvent,
};

use crate::{RangefinderDistanceMode, RangefinderState};

impl From<ProtocolRangefinderDistanceMode> for RangefinderDistanceMode {
    fn from(mode: ProtocolRangefinderDistanceMode) -> Self {
        match mode {
            ProtocolRangefinderDistanceMode::Short => Self::Short,
            ProtocolRangefinderDistanceMode::Long => Self::Long,
        }
    }
}

impl From<RangefinderDistanceMode> for ProtocolRangefinderDistanceMode {
    fn from(mode: RangefinderDistanceMode) -> Self {
        match mode {
            RangefinderDistanceMode::Short => Self::Short,
            RangefinderDistanceMode::Long => Self::Long,
        }
    }
}

impl RangefinderState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, id: &str, event: RangefinderEvent) {
        self.id = id.into();
        match event {
            RangefinderEvent::Range { millimeters } => {
                self.has_millimeters = true;
                self.millimeters = i32::from(millimeters);
            }
            RangefinderEvent::RangingState { is_ranging } => {
                self.has_is_ranging = true;
                self.is_ranging = is_ranging;
            }
            RangefinderEvent::TimingBudget { milliseconds } => {
                self.has_timing_budget_milliseconds = true;
                self.timing_budget_milliseconds = i32::from(milliseconds);
            }
            RangefinderEvent::DistanceMode { mode } => {
                self.has_distance_mode = true;
                self.distance_mode = mode.into();
            }
            RangefinderEvent::InvalidMeasurement { status } => {
                self.has_invalid_measurement_status = true;
                self.invalid_measurement_status = status.into();
            }
        }
    }
}

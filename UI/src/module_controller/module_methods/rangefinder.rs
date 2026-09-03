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

    pub fn update(&mut self, event: RangefinderEvent) {
        match event {
            RangefinderEvent::Range { id, millimeters } => {
                self.id = id.into();
                self.has_millimeters = true;
                self.millimeters = i32::from(millimeters);
            }
            RangefinderEvent::RangingState { id, is_ranging } => {
                self.id = id.into();
                self.has_is_ranging = true;
                self.is_ranging = is_ranging;
            }
            RangefinderEvent::TimingBudget { id, milliseconds } => {
                self.id = id.into();
                self.has_timing_budget_milliseconds = true;
                self.timing_budget_milliseconds = i32::from(milliseconds);
            }
            RangefinderEvent::DistanceMode { id, mode } => {
                self.id = id.into();
                self.has_distance_mode = true;
                self.distance_mode = mode.into();
            }
            RangefinderEvent::InvalidMeasurement { id, status } => {
                self.id = id.into();
                self.has_invalid_measurement_status = true;
                self.invalid_measurement_status = status.into();
            }
        }
    }
}

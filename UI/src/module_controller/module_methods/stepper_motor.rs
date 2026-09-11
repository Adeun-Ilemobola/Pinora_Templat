use pinora_protocol::{
    PivotPoint as ProtocolPivotPoint, StepperMotorEvent,
    StepperStateType as ProtocolStepperStateType,
};

use crate::{PivotPoint, StepperMotorState, StepperStateType};

impl From<ProtocolPivotPoint> for PivotPoint {
    fn from(point: ProtocolPivotPoint) -> Self {
        match point {
            ProtocolPivotPoint::Min => Self::Min,
            ProtocolPivotPoint::Max => Self::Max,
        }
    }
}

impl From<PivotPoint> for ProtocolPivotPoint {
    fn from(point: PivotPoint) -> Self {
        match point {
            PivotPoint::Min => Self::Min,
            PivotPoint::Max => Self::Max,
        }
    }
}

impl From<ProtocolStepperStateType> for StepperStateType {
    fn from(state: ProtocolStepperStateType) -> Self {
        match state {
            ProtocolStepperStateType::Idle => Self::Idle,
            ProtocolStepperStateType::Moving => Self::Moving,
            ProtocolStepperStateType::Homing => Self::Homing,
            ProtocolStepperStateType::Pivot => Self::Pivot,
        }
    }
}

impl From<StepperStateType> for ProtocolStepperStateType {
    fn from(state: StepperStateType) -> Self {
        match state {
            StepperStateType::Idle => Self::Idle,
            StepperStateType::Moving => Self::Moving,
            StepperStateType::Homing => Self::Homing,
            StepperStateType::Pivot => Self::Pivot,
        }
    }
}

impl StepperMotorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, id: &str, event: StepperMotorEvent) {
        self.id = id.into();
        match event {
            StepperMotorEvent::GetAngle { angle, step } => {
                self.has_angle = true;
                self.angle = angle;
                self.has_step = true;
                self.step = step;
            }
            StepperMotorEvent::GetPivotMin { pivot_min } => {
                self.has_pivot_min = true;
                self.pivot_min = pivot_min;
            }
            StepperMotorEvent::GetPivotMax { pivot_max } => {
                self.has_pivot_max = true;
                self.pivot_max = pivot_max;
            }
            StepperMotorEvent::GetMode { mode } => {
                self.has_mode = true;
                self.mode = mode.into();
            }
            StepperMotorEvent::GetOrigin { origin } => {
                match origin {
                    Some(origin) => {
                        self.has_origin = true;
                        self.origin = origin;
                    }
                    None => {
                        self.has_origin = false;
                        self.origin = 0.0;
                    }
                }
            }
            StepperMotorEvent::GetPivotPoint { pivot_point } => {
                self.has_pivot_point = true;
                self.pivot_point = pivot_point.into();
            }
        }
    }
}

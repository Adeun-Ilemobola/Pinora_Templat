use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize , PartialEq)]
pub enum PivotPoint {
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StepperStateType {
    Idle,
    Moving,
    Homing,
    Pivot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepperMotorCommandPayload {
    SetPivotMin { pivot_min: f32 },
    SetPivotMax { pivot_max: f32 },
    MoveToOrigin,
    MoveToAngle { angle: f32 },
    MoveToPivotMin,
    MoveToPivotMax,
    SetMode { mode: StepperStateType },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepperMotorEvent {
    GetAngle { angle: f32, step: f32 },
    GetPivotMin { pivot_min: f32 },
    GetPivotMax { pivot_max: f32 },
    GetMode { mode: StepperStateType },
    GetOrigin { origin: Option<f32> },
    GetPivotPoint { pivot_point: PivotPoint },
}

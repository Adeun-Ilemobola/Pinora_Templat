use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[serde(tag = "command")]
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
#[serde(tag = "event_type")]
pub enum StepperMotorEvent {
    GetAngle { id: String, angle: f32, step: f32 },
    GetPivotMin { id: String, pivot_min: f32 },
    GetPivotMax { id: String, pivot_max: f32 },
    GetMode { id: String, mode: StepperStateType },
    GetOrigin { id: String, origin: Option<f32> },
    GetPivotPoint { id: String, pivot_point: PivotPoint },
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServoCapability {
    pub max_angle: i32,
    pub min_angle: i32,
    pub offset: i32,
    pub min_pivot: i32,
    pub max_pivot: i32,
    pub pulse_min: i32,
    pub pulse_max: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServoEvent {
    GetAngle { angle: i32 },
    GetMinPivot { min_pivot: i32 },
    GetMaxPivot { max_pivot: i32 },
    GetOffset { angle: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServoCommandPayload {
    SetAngle { angle: i32 },
    SetMinPivot { min_pivot: i32 },
    SetMaxPivot { max_pivot: i32 },
}

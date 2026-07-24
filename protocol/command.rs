use serde::{Deserialize, Serialize};

use crate::protocol::global_definitions::Point;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncomingCommand {
    pub id: String,
    #[serde(flatten)]
    pub command: ModuleCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "module_type", content = "payload")]
pub enum ModuleCommand {
    Led(LedCommandPayload),
    ClusterLeds(ClusterCommandPayload),
    Servo(ServoCommandPayload),
    Lidar(LidarCommandPayload),
    Rangefinder(RangefinderCommandPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command")]
pub enum LedCommandPayload {
    SetState { state: u32 },
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command")]
pub enum ServoCommandPayload {
    SetAngle { angle: i32 },
    SetMinPivot { min_pivot: i32 },
    SetMaxPivot { max_pivot: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command")]
pub enum ClusterCommandPayload {
    ToggleAll,
    SetAll { state: u32 },
    Toggle { id: String, state: u32 },
    SetState { id: String, state: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command")]
pub enum LidarCommandPayload {
    Roi { min: Point, max: Point },
    StartScan,
    StopScan,
    Test,
    SetStep { step: u32 },
    ChangeMotorAngle { id: String, step: i32 },
    MovePos { p: Point },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RangefinderDistanceMode {
    Short,
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command")]
pub enum RangefinderCommandPayload {
    StartRanging,
    StopRanging,
    SetTimingBudget { milliseconds: u16 },
    SetDistanceMode { mode: RangefinderDistanceMode },
}

use serde::{Deserialize, Serialize};

use crate::protocol::global_definitions::{Point, StepperState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub struct IncomingCommand {
    pub id: String,
    #[serde(flatten)]
    pub command: ModuleCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "module_type", content = "payload")]
pub enum ModuleCommand {
    Led(LedCommandPayload),
    ClusterLeds(ClusterCommandPayload),
    Servo(ServoCommandPayload),
    Lidar(LidarCommandPayload),
    Rangefinder(RangefinderCommandPayload),
    StepperMotor(StepperMotorCommandPayload),
    Rfid(RfidCommand)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "command")]
pub enum LedCommandPayload {
    SetState { state: u32 },
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "command")]
pub enum ServoCommandPayload {
    SetAngle { angle: i32 },
    SetMinPivot { min_pivot: i32 },
    SetMaxPivot { max_pivot: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "command")]
pub enum ClusterCommandPayload {
    ToggleAll,
    SetAll { state: u32 },
    Toggle { id: String, state: u32 },
    SetState { id: String, state: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub enum RangefinderDistanceMode {
    Short,
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "command")]
pub enum RangefinderCommandPayload {
    StartRanging,
    StopRanging,
    SetTimingBudget { milliseconds: u16 },
    SetDistanceMode { mode: RangefinderDistanceMode },
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq,)]
#[serde(tag = "command")]
pub enum StepperMotorCommandPayload {
    SetPivotMin { pivot_min: f32 },
    SetPivotMax { pivot_max: f32 },
    MoveToOrigin,
    MoveToAngle { angle: f32 },
    MoveToPivotMin,
    MoveToPivotMax,
    SetMode { mode: StepperState },
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq,)]
#[serde(tag = "command")]
pub enum RfidCommand {
    Scan,
    ReadUid,

    ReadBlock {
        block: u8,
    },

    WriteBlock {
        block: u8,
        data: [u8; 16],
    },

    ReadPayload,
    WritePayload {
        data: Vec<u8>,
    },
}
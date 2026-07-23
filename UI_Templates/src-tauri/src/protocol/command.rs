use serde::{Deserialize ,Serialize};

use crate::protocol::global_definition_protocol::Point;

#[derive(Debug, Deserialize  , Serialize , Clone)]
pub struct IncomingCommand {
    pub id: String,
    #[serde(flatten)]
    pub command: ModuleCommand,
}

#[derive(Debug, Deserialize  , Serialize , Clone)]
#[serde(tag = "module_type", content = "payload", )]
pub enum ModuleCommand {
    Led(LedCommandPayload),
    ClusterLeds(ClusterCommandPayload),
    Servo(ServoCommandPayload),
    Lidar(LidarCommandPayload),
    Rangefinder(RangefinderCommandPayload)
}

#[derive(Debug, Deserialize  , Serialize , Clone)]
#[serde(tag = "command", )]
pub enum LedCommandPayload {
    SetState { state: u32 },
    Toggle,
}

//Servo
#[derive(Debug, Deserialize  , Serialize , Clone)]
#[serde(tag = "command", )]
pub enum ServoCommandPayload {
    SetAngle { angle: i32 },
    SetMinPivot { min_pivot: i32 },
    SetMaxPivot { max_pivot: i32 },
}

//cluster Leds
#[derive(Debug, Deserialize  , Serialize , Clone)]
#[serde(tag = "command", )]
pub enum ClusterCommandPayload {
    ToggleAll,
    SetAll { state: u32 },

    Toggle { id: String, state: u32 },
    SetState { id: String, state: u32 },
}


// ---- Lidar

#[derive(Debug, Deserialize  , Serialize , Clone)]
#[serde(tag = "command", )]
pub enum  LidarCommandPayload  {
    Roi{min:Point , max:Point},
    StartScan,
    StopScan,
    Test,
    SetStep{ step:u32},
    ChangeMotorAngle {
        id: String,
        step:i32
    },
    MovePos{
        p:Point
    }    
}
// global_definition_protocol



// ---- RangeFinder
#[derive(Debug, Deserialize  , Serialize , Clone)]
pub enum RangefinderDistanceMode {
    Short,
    Long,
}

#[derive(Debug, Deserialize  , Serialize , Clone)]
#[serde(tag = "command", )]

pub enum RangefinderCommandPayload {
    StartRanging,
    StopRanging,

    SetTimingBudget {
        milliseconds: u16,
    },


    SetDistanceMode {
        mode: RangefinderDistanceMode,
    },
}
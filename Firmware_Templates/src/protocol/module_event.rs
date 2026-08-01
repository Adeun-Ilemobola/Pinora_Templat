use serde::{Deserialize, Serialize};

use crate::protocol::{
    command::RangefinderDistanceMode,
    global_definitions::{PivotPoint, Point, RangPoint, StepperState},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "module_type", content = "event")]
pub enum ModuleEvent {
    Led(LedEvent),
    Servo(ServoEvent),
    Lidar(LidarEvent),
    Button(ButtonEvent),
    SysLog(SysLogEvent),
    Rangefinder(RangefinderEvent),
    StepperMotor(StepperMotorEvent)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum LogPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub struct SysLogEvent {
    pub text: String,
    pub raw_err: Option<String>,
    pub priority: LogPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "event_type")]
pub enum LedEvent {
    Brightness { id: String, level: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "event_type")]
pub enum ServoEvent {
    GetAngle { id: String, angle: i32 },
    GetMinPivot { id: String, min_pivot: i32 },
    GetMaxPivot { id: String, max_pivot: i32 },
    GetOffset { id: String, angle: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub enum ScanState {
    Idol,
    Scanning,
    StopScan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq,)]
#[serde(tag = "event_type")]
pub enum LidarEvent {
    Roi {
        id: String,
        min: Point,
        max: Point,
    },
    PointMap {
        id: String,
        max_chunk: i32,
        curr_chunk: i32,
        map: Vec<RangPoint>,
    },
    Target {
        id: String,
        point: Point,
    },
    ScanState {
        id: String,
        state: ScanState,
        scan_time:f32
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "event_type")]
pub enum ButtonEvent {
    Ckick { id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "event_type")]
pub enum RangefinderEvent {
    Range {
        id: String,
        millimeters: u16,
    },
    RangingState {
        id: String,
        is_ranging: bool,
    },
    TimingBudget {
        id: String,
        milliseconds: u16,
    },
    DistanceMode {
        id: String,
        mode: RangefinderDistanceMode,
    },
    InvalidMeasurement {
        id: String,
        status: String,
    },
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "event_type")]
pub  enum  StepperMotorEvent {
     GetAngle { id: String, angle: f32 , step: f32},
     GetPivotMin { id: String, pivot_min: f32 },
     GetPivotMax { id: String, pivot_max: f32 },
     GetMode { id: String, mode: StepperState },
     GetOrigin { id: String, origin: Option<f32> },
     GetPivotPoint { id: String, pivot_point: PivotPoint },

}

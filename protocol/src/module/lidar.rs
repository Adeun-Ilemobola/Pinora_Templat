use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub struct RangPoint {
    pub x: i32,
    pub y: i32,
    pub distant: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanState {
    Idol,
    Scanning,
    StopScan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        scan_time: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        min: Point,
        max: Point,
    },
    PointMap {
        max_chunk: i32,
        curr_chunk: i32,
        map: Vec<RangPoint>,
    },
    Target {
        point: Point,
    },
    ScanState {
        state: ScanState,
        scan_time: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LidarCommandPayload {
    Roi { min: Point, max: Point },
    StartScan {},
    StopScan {},
    Test {},
    SetStep { step: u32 },
    ChangeMotorAngle { id: String, step: i32 },
    MovePos { p: Point },
}

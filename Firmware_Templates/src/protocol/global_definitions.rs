use serde::{Deserialize, Serialize};

use crate::core::hardware::OutputPinCore;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleType {
    Servo,
    Led,
    Imu,
    LedCluster,
    Button,
    Lidar,
    Rangefinder,
    SysLog,
    JoyStick,
    StepperMotor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "event_type")]
pub struct RangPoint {
    pub x: i32,
    pub y: i32,
    pub distant: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServoCapability {
    pub max_angle: i32,
    pub min_angle: i32,
    pub offset: i32,
    pub min_pivot: i32,
    pub max_pivot: i32,
    pub pulse_min: i32,
    pub pulse_max: i32,
}

pub struct StepperPins<'d> {
    pub in1: OutputPinCore<'d>,
    pub in2: OutputPinCore<'d>,
    pub in3: OutputPinCore<'d>,
    pub in4: OutputPinCore<'d>,
}

#[derive(Debug, Clone, Copy, PartialEq)]

pub enum StepperState {
    Idle,
    Moving,
    Homing,
    Pivot,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PivotPoint {
    Min,
    Max,
}

#[derive(Debug, Clone, Copy)]
pub struct PivotLimits {
    pub min: f32,
    pub max: f32,
}

impl PivotLimits {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn value(&self, point: PivotPoint) -> f32 {
        match point {
            PivotPoint::Min => self.min,
            PivotPoint::Max => self.max,
        }
    }

    pub fn opposite(&self, point: PivotPoint) -> PivotPoint {
        match point {
            PivotPoint::Min => PivotPoint::Max,
            PivotPoint::Max => PivotPoint::Min,
        }
    }
    pub fn update_max(&mut self , n:f32){ self.max = n}
    pub fn update_min(&mut self , n:f32){ self.min = n}
}
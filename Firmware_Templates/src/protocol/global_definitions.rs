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
    StepperMotor
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


pub  enum StepperState {
    Idle,
    Moving,
    Homing,
}

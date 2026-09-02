use serde::{Deserialize, Serialize};

use crate::{RemoteButtonEvent, module::{buttonmodule::ButtonEvent, imu::imu_type::ImuEvent, ledmodule::LedEvent, rfid::RfidEvent, stepper::StepperMotorEvent}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq )]
pub enum ModuleEvent {
    Led(LedEvent),
    //Servo(ServoEvent),
    //Lidar(LidarEvent),
    Button(ButtonEvent),
    SysLog(SysLogEvent),
    //Rangefinder(RangefinderEvent),
    StepperMotor(StepperMotorEvent),
    Imu(ImuEvent),
    Rfid(RfidEvent),
    RemoteReceiver(RemoteButtonEvent)
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
















use serde::{Deserialize, Serialize};

use crate::{LidarCommandPayload, RangefinderCommandPayload, ServoCommandPayload, module::{ledmodule::LedCommandPayload, rfid::RfidCommand, stepper::StepperMotorCommandPayload}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub struct IncomingCommand {
    pub id: String,
    pub command: ModuleCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub enum ModuleCommand {
    Led(LedCommandPayload),
    Servo(ServoCommandPayload),
    Lidar(LidarCommandPayload),
    Rangefinder(RangefinderCommandPayload),
    StepperMotor(StepperMotorCommandPayload),
    Rfid(RfidCommand)
}













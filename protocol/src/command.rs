use serde::{Deserialize, Serialize};

use crate::{module::{ledmodule::LedCommandPayload, rfid::RfidCommand, stepper::{StepperMotorCommandPayload}}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub struct IncomingCommand {
    pub id: String,
    #[serde(flatten)]
    pub command: ModuleCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub enum ModuleCommand {
    Led(LedCommandPayload),
    //Servo(ServoCommandPayload),
    //Lidar(LidarCommandPayload),
    // Rangefinder(RangefinderCommandPayload),
    StepperMotor(StepperMotorCommandPayload),
    Rfid(RfidCommand)
}













use serde::{Deserialize, Serialize};

use crate::{
    LidarCommandPayload, RangefinderCommandPayload, ServoCommandPayload,
    module::{
        ledmodule::LedCommandPayload, rfid::RfidCommand, stepper::StepperMotorCommandPayload,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IncomingCommand {
    pub id: String,
    pub command: ModuleCommand,
}

/// Commands addressed to the firmware itself rather than a module (protocol v1).
/// Sent with the reserved id [`crate::SYSTEM_COMMAND_ID`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemCommand {
    /// Re-send the system snapshot, every registration and every module's current
    /// state. The desktop sends it after each (re)connect.
    Rediscover {},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleCommand {
    Led(LedCommandPayload),
    Servo(ServoCommandPayload),
    Lidar(LidarCommandPayload),
    Rangefinder(RangefinderCommandPayload),
    StepperMotor(StepperMotorCommandPayload),
    Rfid(RfidCommand),
    /// Protocol v1. Older firmware rejects it as an invalid command.
    System(SystemCommand),
}

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
    /// Protocol v2: start the authentication handshake (sent by Tauri). See
    /// [`crate::auth`].
    AuthBegin { client_nonce: String },
    /// Protocol v2: the desktop's proof (sent by Tauri).
    AuthResponse { proof: String },
    /// Protocol v2: create the device key if missing and return it. Accepted **only**
    /// by the USB serial console.
    ProvisionKey {},
    /// Protocol v2: delete the device key (the device becomes unprovisioned). USB
    /// serial console only.
    ClearKey {},
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

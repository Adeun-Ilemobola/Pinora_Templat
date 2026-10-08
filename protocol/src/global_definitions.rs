use serde::{Deserialize, Serialize};

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
    Rfid,
    RemoteReceiver
}

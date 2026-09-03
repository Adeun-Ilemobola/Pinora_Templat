use pinora_protocol::{Point, RangPoint, ScanState};

pub use crate::{
    ButtonState, ImuState, JoyStickState, LedClusterState, LedState, RangefinderState,
    RemoteReceiverState, RfidState, ServoState, StepperMotorState, SysLogState,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LidarState {
    pub id: String,
    pub roi_min: Option<Point>,
    pub roi_max: Option<Point>,
    pub max_chunk: Option<i32>,
    pub current_chunk: Option<i32>,
    pub map: Option<Vec<RangPoint>>,
    pub target: Option<Point>,
    pub scan_state: Option<ScanState>,
    pub scan_time: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleState {
    Servo(ServoState),
    Led(LedState),
    Lidar(LidarState),
    Rangefinder(RangefinderState),
    Button(ButtonState),
    SysLog(SysLogState),
    RemoteReceiver(RemoteReceiverState),
    StepperMotor(StepperMotorState),
    Imu(ImuState),
    Rfid(RfidState),
    JoyStick(JoyStickState),
    LedCluster(LedClusterState),
}

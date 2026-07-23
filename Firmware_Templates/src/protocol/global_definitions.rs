use serde::{ Serialize , Deserialize};


#[derive(Debug, Serialize , Deserialize ,Clone )]
pub enum ModuleType {
    Servo,
    Led,
    Imu,
    LedCluster,
    Button,
    Lidar,
    Rangefinder,
     SysLog
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "event_type")]
pub struct RangPoint {
    pub x: i32,
    pub y: i32,
    pub distant: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}


#[derive(Debug, Serialize ,Clone )]
pub struct   ServoCapability{
    pub max_angle: i32,
    pub min_angle: i32,
    pub offset: i32,
    pub min_pivot: i32,
    pub max_pivot: i32,
    pub pulse_min:i32,
    pub pulse_max:i32,

}

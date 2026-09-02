use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct RawAxes {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct Axes {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Axes {
    pub fn add(self, axes: &Axes) -> Axes {
        Axes {
            x: self.x + axes.x,
            y: self.y + axes.y,
            z: self.z + axes.z,
        }
    }

    pub fn divide(self, amount: u16) -> Axes {
        Axes {
            x: self.x / amount as f32,
            y: self.y / amount as f32,
            z: self.z / amount as f32,
        }
    }
}

impl RawAxes {
    pub fn scale(self, sensitivity: f32) -> Axes {
        Axes {
            x: self.x as f32 / sensitivity,
            y: self.y as f32 / sensitivity,
            z: self.z as f32 / sensitivity,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MpuDeviceMode {
    Collecting,
    Idle,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum ImuEvent {
    Gyro {
        id: String,
        raw_axes: RawAxes,
        axes: Axes,
    },
    Accel {
        id: String,
        raw_axes: RawAxes,
        axes: Axes,
    },
    Mode {
        mode: MpuDeviceMode,
    },
}

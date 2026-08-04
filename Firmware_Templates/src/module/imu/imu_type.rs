use core::fmt;
use crate::core::{hardware::SharedI2cDevice, modulecore::ModuleCore};
use serde::{Serialize , Deserialize};

pub const  GYRO_XOUT_H: u8 = 0x43;
pub  const ACCEL_XOUT_H: u8 = 0x3B;


pub const ACCEL_SENSITIVITY: f32 = 16_384.0;
pub const GYRO_SENSITIVITY: f32 = 131.0;
#[derive(Debug , Serialize , Deserialize ,PartialEq, Eq , Clone, Copy)]
pub struct RawAxes {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug , Serialize ,Deserialize ,PartialEq, Clone, Copy)]

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
            z: self.z  + axes.z,
        }
    }

    pub fn divide(self, amount: u16) -> Axes {
        Axes {
            x: self.x / amount as f32,
            y: self.y / amount as f32,
            z: self.z  / amount as f32,
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

#[derive(Debug)]
pub enum ImuModel {
    Mpu6500,
    Mpu9250,
    Mpu9255,
}

#[derive(Debug, Serialize)]
pub enum ImuError<E> {
    // Your variants
    UnknownDevice(u8),
    I2cError(E),
    InvalidData,
}
impl<E> From<E> for ImuError<E> {
    fn from(error: E) -> Self {
        ImuError::I2cError(error)
    }
}

pub struct Mpu<'d> {
    pub device_address: u8,
    pub model: ImuModel,
    pub i2c: SharedI2cDevice<'d>,
}

impl<'d> fmt::Debug for Mpu<'d> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mpu")
            .field("model", &self.model)
            .field("device_address", &self.device_address)
            .field("i2c", &"<I2C device>")
            .finish()
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub  enum  MpuDeviceMode {
    Collecting,
    Idle,
    Off
    
}

pub struct MpuDevice<'d> {
    pub mpu: Mpu<'d>,
    pub core: ModuleCore,
    pub  mode: MpuDeviceMode,
    
    pub gyro_raw :RawAxes,
    pub accel_raw:RawAxes,


    pub gyro :Axes,
    pub accel:Axes,
    pub bias_gyro: Axes,
    pub bias_accel: Axes,
    pub bias_collection_accel:Vec<Axes>,
     pub bias_collection_gyro:Vec<Axes>,


    pub point_count:u16,
    pub point_count_max:u16,

}

#[derive(Debug , Serialize )]
pub enum MpuDeviceErr {
    InitI2c {
        info: Option<String>,
        i2c_err: String,
    },
}

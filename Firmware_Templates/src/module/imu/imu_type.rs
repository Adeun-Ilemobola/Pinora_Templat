use core::fmt;
use crate::core::{hardware::SharedI2cDevice, modulecore::ModuleCore};
use serde::Serialize;

pub use pinora_protocol::module::imu::imu_type::{Axes, ImuEvent, MpuDeviceMode, RawAxes};

pub const  GYRO_XOUT_H: u8 = 0x43;
pub  const ACCEL_XOUT_H: u8 = 0x3B;


pub const ACCEL_SENSITIVITY: f32 = 16_384.0;
pub const GYRO_SENSITIVITY: f32 = 131.0;
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

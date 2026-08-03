use core::fmt;
use crate::core::{hardware::SharedI2cDevice, modulecore::ModuleCore};
use esp_idf_svc::hal::i2c::I2cError;
use serde::Serialize;

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
}

#[derive(Debug , Serialize )]
pub enum MpuDeviceErr {
    InitI2c {
        info: Option<String>,
        i2c_err: String,
    },
}

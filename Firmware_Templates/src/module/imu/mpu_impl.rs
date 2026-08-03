use std::sync::mpsc::SyncSender;

use crate::{
    core::{hardware::SharedI2cDevice, modulecore::{Module, ModuleCore}},
    module::imu::imu_type::{ImuError, ImuModel, Mpu, MpuDevice, MpuDeviceErr}, protocol::{command::ModuleCommand, global_definitions::ModuleType, registration::ProtocolMessage},
};
use embedded_hal::i2c::I2c;
use esp_idf_svc::hal::i2c::I2cError;

impl<'d> Mpu<'d> {
    pub fn identify(
        mut i2c_bus: SharedI2cDevice<'d>,
        device_address: u8,
    ) -> Result<Mpu<'d>, ImuError<I2cError>> {
        let mut buffer = [0; 1];
        i2c_bus.write_read(device_address, &[0x75], &mut buffer)?;

        match buffer[0] {
            0x70 => Ok(Self {
                i2c: i2c_bus,
                model: ImuModel::Mpu6500,
                device_address: device_address,
            }),

            0x71 => Ok(Self {
                i2c: i2c_bus,
                model: ImuModel::Mpu9250,
                device_address: device_address,
            }),

            0x73 => Ok(Self {
                i2c: i2c_bus,
                model: ImuModel::Mpu9255,
                device_address: device_address,
            }),
            _ => Err(ImuError::UnknownDevice(buffer[0])),
        }
    }
}

impl<'d> MpuDevice<'d> {
    pub fn new(
        i2c: SharedI2cDevice<'d>,
        device_address: u8,
        sender:SyncSender<ProtocolMessage>,
        core_id:&str,
        parent_id:Option<String>
    ) -> Result<MpuDevice<'d>, MpuDeviceErr> {
        let new_mpi = Mpu::identify(i2c, device_address).map_err(|err| {
          let err_data =  MpuDeviceErr::InitI2c {
                info: Some(String::from("Failed during MPU identification")),
                i2c_err: format!("{:?}", err),
            };
            println!("{:?}" , err_data);
            
            err_data
        })?;
        println!("MpuDevice : {:?}" ,new_mpi);
        let imu = MpuDevice{
            mpu: new_mpi ,
            core: ModuleCore::new(ModuleType::Imu, core_id , sender.clone()),
        };
        if parent_id.is_some(){

        }

        Ok(imu)
    }
}
impl<'d> Module for MpuDevice<'d> {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            
            
            _ => {
                // Ignore commands intended for other module types.
            }
        }
        Ok(())
    }
}

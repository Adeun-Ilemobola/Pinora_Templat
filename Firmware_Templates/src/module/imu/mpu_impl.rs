
use crate::{
    core::{
        emitter::Emitter, hardware::SharedI2cDevice, modulecore::{Module, ModuleCore, ModuleError}
    },
    module::imu::imu_type::{
        ACCEL_SENSITIVITY, ACCEL_XOUT_H, Axes, GYRO_SENSITIVITY, GYRO_XOUT_H, ImuError, ImuEvent, ImuModel, Mpu, MpuDevice, MpuDeviceErr, MpuDeviceMode, RawAxes
    },
};
use embedded_hal::i2c::I2c;
use esp_idf_svc::hal::i2c::I2cError;
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::{LogPriority, ModuleEvent, SysLogEvent},
    registration::ProtocolMessage,
};

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
        sender: Emitter,
        core_id: &str,
        parent_id: Option<String>,
    ) -> Result<MpuDevice<'d>, MpuDeviceErr> {
        let new_mpi = Mpu::identify(i2c, device_address).map_err(|err| {
            let err_data = MpuDeviceErr::InitI2c {
                info: Some(String::from("Failed during MPU identification")),
                i2c_err: format!("{:?}", err),
            };
           
            
            sender.any(ProtocolMessage::ModuleEvent(ModuleEvent::SysLog(SysLogEvent{
                priority:LogPriority::Critical,
                text:"Accel error".to_string(),
                raw_err:Some(format!("{:?}" ,err_data))
            })));

            err_data
        })?;
        
        let  imu = MpuDevice {
            mpu: new_mpi,
            core: ModuleCore::new(ModuleType::Imu, core_id, parent_id, sender.clone()),
            mode: MpuDeviceMode::Collecting,
            point_count: 1,
            point_count_max: 200,

            accel_raw: RawAxes { x: 0, z: 0, y: 0 },
            accel: Axes {
                x: 0.0,
                z: 0.0,
                y: 0.0,
            },
            bias_accel: Axes {
                x: 0.0,
                z: 0.0,
                y: 0.0,
            },
            bias_collection_accel: vec![],

            gyro: Axes {
                x: 0.0,
                z: 0.0,
                y: 0.0,
            },
            gyro_raw: RawAxes { x: 0, z: 0, y: 0 },
            bias_gyro: Axes {
                x: 0.0,
                z: 0.0,
                y: 0.0,
            },
            bias_collection_gyro: vec![],
        };
        imu.emit(ModuleEvent::Imu(ImuEvent::Mode { mode: imu.mode.clone() }));

        Ok(imu)
    }
    pub fn raw_gyro(&mut self) -> Result<RawAxes, ImuError<I2cError>> {
        let mut bytes = [0_u8; 6];

        self.mpu
            .i2c
            .write_read(self.mpu.device_address, &[GYRO_XOUT_H], &mut bytes)?;

        let x = i16::from_be_bytes([bytes[0], bytes[1]]);
        let y = i16::from_be_bytes([bytes[2], bytes[3]]);
        let z = i16::from_be_bytes([bytes[4], bytes[5]]);

        Ok(RawAxes { x, y, z })
    }
    pub fn raw_accel(&mut self) -> Result<RawAxes, ImuError<I2cError>> {
        let mut bytes = [0_u8; 6];

        self.mpu
            .i2c
            .write_read(self.mpu.device_address, &[ACCEL_XOUT_H], &mut bytes)?;

        let x = i16::from_be_bytes([bytes[0], bytes[1]]);
        let y = i16::from_be_bytes([bytes[2], bytes[3]]);
        let z = i16::from_be_bytes([bytes[4], bytes[5]]);

        Ok(RawAxes { x, y, z })
    }
    fn tick_inner(&mut self) -> Result<(), ()> {
        let new_gyro = self.raw_gyro().map_err(|err| {
            self.emit(ModuleEvent::SysLog(SysLogEvent{
                priority:LogPriority::Critical,
                text:"Gyro error".to_string(),
                raw_err:Some(format!("{:?}" ,err))
            }));
            
        })?;
        let new_accel = self.raw_accel().map_err(|err| {
            self.emit(ModuleEvent::SysLog(SysLogEvent{
                priority:LogPriority::Critical,
                text:"Accel error".to_string(),
                raw_err:Some(format!("{:?}" ,err))
            }));
           
        })?;

        match self.mode {
            MpuDeviceMode::Collecting => {
                self.bias_collection_accel
                    .push(new_accel.scale(ACCEL_SENSITIVITY));
                self.bias_collection_gyro
                    .push(new_gyro.scale(GYRO_SENSITIVITY));

                if self.point_count >= self.point_count_max {
                    for axes in self.bias_collection_accel.iter() {
                        self.bias_accel = self.bias_accel.add(axes);
                    }
                    for axes in self.bias_collection_gyro.iter() {
                        self.bias_gyro = self.bias_gyro.add(axes);
                    }

                    self.bias_accel = self.bias_accel.divide(self.point_count_max.clone());
                    self.bias_gyro = self.bias_gyro.divide(self.point_count_max.clone());

                    self.mode = MpuDeviceMode::Idle;
                    self.point_count = 0;
                    self.bias_collection_gyro.clear();
                    self.bias_collection_accel.clear();
                    self.emit(ModuleEvent::Imu(ImuEvent::Mode { mode: self.mode.clone() }));
                    

                    return Ok(());
                }
               
                self.point_count += 1;
            }
            MpuDeviceMode::Idle => {
                self.update_gyro(new_gyro);
                self.update_accel(new_accel);
            }
            MpuDeviceMode::Off => {}
        }

        Ok(())
    }
    fn update_gyro(&mut self, new_gyro: RawAxes) {
        if new_gyro != self.gyro_raw {
            self.gyro_raw = new_gyro;
            let cover = self.gyro_raw.scale(GYRO_SENSITIVITY);
            self.gyro.x = cover.x - self.bias_gyro.x;
            self.gyro.z = cover.z - self.bias_gyro.z;
            self.gyro.y = cover.y - self.bias_gyro.y;
             self.emit(ModuleEvent::Imu(ImuEvent::Gyro { id: self.id().to_string(), raw_axes: self.gyro_raw, axes: self.gyro }));
            
        }
    }

    fn update_accel(&mut self, new_accel: RawAxes) {
        if new_accel != self.accel_raw {
            self.accel_raw = new_accel;
            let cover = self.accel_raw.scale(ACCEL_SENSITIVITY);
            self.accel.x = cover.x - self.bias_accel.x;
            self.accel.z = cover.z - self.bias_accel.z;
            self.accel.y = cover.y - self.bias_accel.y;
             self.emit(ModuleEvent::Imu(ImuEvent::Accel { id: self.id().to_string(), raw_axes: self.accel_raw, axes: self.accel }));
          
        }
    }
}
impl<'d> Module for MpuDevice<'d> {
    fn tick(&mut self) -> Result<(), ModuleError> {
        self.tick_inner().map_err(ModuleError::from)
    }

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

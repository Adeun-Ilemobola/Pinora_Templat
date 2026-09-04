use crate::core::emitter::Emitter;
use crate::core::hardware::RangefinderI2c;
use crate::core::modulecore::{Module, ModuleCore, ModuleError};
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::{LogPriority, ModuleEvent, SysLogEvent},
};
use vl53l1x_uld::{DistanceMode, IOVoltage, RangeStatus, DEFAULT_ADDRESS, VL53L1X};

pub use pinora_protocol::modules::range_finder::{
    RangefinderCommandPayload, RangefinderDistanceMode, RangefinderEvent,
};
pub struct Rangefinder<'d> {
    pub core: ModuleCore,
    pub sensor: VL53L1X<RangefinderI2c<'d>>,

    pub range_mm: u16,
    pub is_ranging: bool,
    pub timing_budget_ms: u16,
    pub inter_measurement_ms: u16,
    pub distance_mode: DistanceMode,
}

impl<'d> Rangefinder<'d> {
    pub fn new(
        rangefinder_i2c: RangefinderI2c<'d>,
        manual_id: String,
        cluster_id: Option<String>,
        sender: Emitter,
    ) -> anyhow::Result<Rangefinder<'d>> {
        let mut sensor = VL53L1X::new(rangefinder_i2c, DEFAULT_ADDRESS);

        let sensor_id = sensor
            .get_sensor_id()
            .map_err(|error| anyhow::anyhow!("Failed to read VL53L1X sensor ID: {error:?}"))?;

        if sensor_id != 0xEACC {
            anyhow::bail!("Unexpected VL53L1X sensor ID: 0x{sensor_id:04X}");
        }

        sensor
            .init(IOVoltage::Volt2_8)
            .map_err(|error| anyhow::anyhow!("VL53L1X initialization failed: {error:?}"))?;

        let rangefinder = Self {
            core: ModuleCore::new(ModuleType::Rangefinder, &manual_id, cluster_id, sender),
            sensor,
            range_mm: 0,
            is_ranging: false,
            timing_budget_ms: 50,
            inter_measurement_ms: 60,
            distance_mode: DistanceMode::Long,
        };

        Ok(rangefinder)
    }

    

    pub fn start_ranging(&mut self) -> anyhow::Result<()> {
        if self.is_ranging {
            return Ok(());
        }

        self.sensor
            .start_ranging()
            .map_err(|error| anyhow::anyhow!("Failed to start VL53L1X ranging: {error:?}"))?;

        self.is_ranging = true;

        self.emit(ModuleEvent::Rangefinder(RangefinderEvent::RangingState {
            id: self.id().to_string(),
            is_ranging: true,
        }));

        Ok(())
    }

    fn stop_ranging(&mut self) -> anyhow::Result<()> {
        if !self.is_ranging {
            return Ok(());
        }

        self.sensor
            .stop_ranging()
            .map_err(|error| anyhow::anyhow!("Failed to stop VL53L1X ranging: {error:?}"))?;

        self.is_ranging = false;

        self.emit(ModuleEvent::Rangefinder(RangefinderEvent::RangingState {
            id: self.id().to_string(),
            is_ranging: false,
        }));

        Ok(())
    }

    pub fn get_range(&mut self) -> Result<Option<u16>, ModuleError> {
        let ready = match self.sensor.is_data_ready() {
            Ok(ready) => ready,
            Err(error) => {
                self.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: "VL53L1X data-ready check failed".to_string(),
                    raw_err: Some(format!("{error:?}")),
                    priority: LogPriority::Critical,
                }));

                return Err(ModuleError::OperationFailed);
            }
        };

        if !ready {
            return Ok(None);
        }

        let status = match self.sensor.get_range_status() {
            Ok(status) => status,
            Err(error) => {
                self.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: "VL53L1X range-status read failed".to_string(),
                    raw_err: Some(format!("{error:?}")),
                    priority: LogPriority::Critical,
                }));
                return Err(ModuleError::OperationFailed);
            }
        };

        let distance = match self.sensor.get_distance() {
            Ok(distance) => distance,
            Err(error) => {
                self.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: "VL53L1X distance read failed".to_string(),
                    raw_err: Some(format!("{error:?}")),
                    priority: LogPriority::Critical,
                }));

                return Err(ModuleError::OperationFailed);
            }
        };

        if let Err(error) = self.sensor.clear_interrupt() {
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: "VL53L1X interrupt clear failed".to_string(),
                raw_err: Some(format!("{error:?}")),
                priority: LogPriority::Critical,
            }));

            return Err(ModuleError::OperationFailed);
        }

        if status != RangeStatus::Valid {
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: "VL53L1X RangeStatus is Valid failed".to_string(),
                raw_err: None,
                priority: LogPriority::Critical,
            }));
            return Err(ModuleError::OperationFailed);
        }
        self.range_mm = distance;

        Ok(Some(self.range_mm))
    }
}

impl<'d> Module for Rangefinder<'d> {
    fn tick(&mut self) -> Result<(), ModuleError> {
        if !self.is_ranging {
            return Ok(());
        }
        match self.get_range() {
            Ok(Some(rang)) => {
                self.emit(ModuleEvent::Rangefinder(RangefinderEvent::Range {
                    id: self.id().to_string(),
                    millimeters: rang,
                }));
                
            }
            Ok(None) => {
                return Ok(());
            }
            Err(error) => {
                return Err(error);
            }
        }

        Ok(())

    }
    fn core(&self) -> &ModuleCore {
        &self.core
    }

    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        let ModuleCommand::Rangefinder(command) = command else {
            return Ok(());
        };

        match command {
            RangefinderCommandPayload::StartRanging => {
                self.start_ranging()?;
            }

            RangefinderCommandPayload::StopRanging => {
                self.stop_ranging()?;
            }

            RangefinderCommandPayload::SetTimingBudget { milliseconds } => {
                self.sensor
                    .set_timing_budget_ms(*milliseconds)
                    .map_err(|error| anyhow::anyhow!("Failed to set timing budget: {error:?}"))?;

                self.timing_budget_ms = *milliseconds;

                self.emit(ModuleEvent::Rangefinder(RangefinderEvent::TimingBudget {
                    id: self.id().to_string(),
                    milliseconds: *milliseconds,
                }));
            }

            RangefinderCommandPayload::SetDistanceMode { mode } => {
                let sensor_mode = match mode {
                    RangefinderDistanceMode::Short => DistanceMode::Short,
                    RangefinderDistanceMode::Long => DistanceMode::Long,
                };

                self.sensor
                    .set_distance_mode(sensor_mode)
                    .map_err(|error| anyhow::anyhow!("Failed to set distance mode: {error:?}"))?;

                self.distance_mode = sensor_mode;

                self.emit(ModuleEvent::Rangefinder(RangefinderEvent::DistanceMode {
                    id: self.id().to_string(),
                    mode: mode.to_owned(),
                }));
            }
        }

        Ok(())
    }
}

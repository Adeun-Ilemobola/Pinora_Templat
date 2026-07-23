use crate::core::hardware::{
     RangefinderI2c,
};
use crate::core::modulecore::{emit, Module, ModuleCore};
use crate::protocol::command::{
    ModuleCommand, RangefinderCommandPayload, RangefinderDistanceMode,
};
use crate::protocol::global_definitions::ModuleType;
use crate::protocol::module_event::{
    ModuleEvent, RangefinderEvent,
};
use crate::protocol::registration::{
     Registration,
};

use vl53l1x_uld::{
    DistanceMode,
    IOVoltage,
    RangeStatus,
    VL53L1X,
    DEFAULT_ADDRESS,
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
    ) -> anyhow::Result<Rangefinder<'d>>
   {
    
        let mut sensor =
            VL53L1X::new(rangefinder_i2c, DEFAULT_ADDRESS);

        let sensor_id = sensor
            .get_sensor_id()
            .map_err(|error| {
                anyhow::anyhow!(
                    "Failed to read VL53L1X sensor ID: {error:?}"
                )
            })?;

        if sensor_id != 0xEACC {
            anyhow::bail!(
                "Unexpected VL53L1X sensor ID: 0x{sensor_id:04X}"
            );
        }

        sensor
            .init(IOVoltage::Volt2_8)
            .map_err(|error| {
                anyhow::anyhow!(
                    "VL53L1X initialization failed: {error:?}"
                )
            })?;

        let rangefinder = Self {
            core: ModuleCore::new(
                ModuleType::Rangefinder,
                &manual_id,
            ),
            sensor,
            range_mm: 0,
            is_ranging: false,
            timing_budget_ms: 50,
            inter_measurement_ms: 60,
            distance_mode: DistanceMode::Long,
        };

        emit::registration(Registration {
            id: rangefinder.id().clone(),
            module_type: ModuleType::Rangefinder,
            lool_up_id: manual_id,
            parent_id: cluster_id.unwrap_or_default(),
        });

        Ok(rangefinder)
    }

    pub fn tick(&mut self) {
        if !self.is_ranging {
            return;
        }

        let ready = match self.sensor.is_data_ready() {
            Ok(ready) => ready,
            Err(error) => {
                crate::utilities::logger::SysLog::error(
                    "VL53L1X data-ready check failed".to_string(),
                    Some(format!("{error:?}")),
                );
                return;
            }
        };

        if !ready {
            return;
        }

        let status = match self.sensor.get_range_status() {
            Ok(status) => status,
            Err(error) => {
                crate::utilities::logger::SysLog::error(
                    "VL53L1X range-status read failed".to_string(),
                    Some(format!("{error:?}")),
                );
                return;
            }
        };

        let distance = match self.sensor.get_distance() {
            Ok(distance) => distance,
            Err(error) => {
                crate::utilities::logger::SysLog::error(
                    "VL53L1X distance read failed".to_string(),
                    Some(format!("{error:?}")),
                );
                return;
            }
        };

        if let Err(error) = self.sensor.clear_interrupt() {
            crate::utilities::logger::SysLog::error(
                "VL53L1X interrupt clear failed".to_string(),
                Some(format!("{error:?}")),
            );
            return;
        }

        if status != RangeStatus::Valid {
            emit::event(ModuleEvent::Rangefinder(
                RangefinderEvent::InvalidMeasurement {
                    id: self.id().clone(),
                    status: format!("{status:?}"),
                },
            ));

            return;
        }

        self.range_mm = distance;

        emit::event(ModuleEvent::Rangefinder(
            RangefinderEvent::Range {
                id: self.id().clone(),
                millimeters: distance,
            },
        ));
    }

    pub fn start_ranging(&mut self) -> anyhow::Result<()> {
        if self.is_ranging {
            return Ok(());
        }

        self.sensor
            .start_ranging()
            .map_err(|error| {
                anyhow::anyhow!(
                    "Failed to start VL53L1X ranging: {error:?}"
                )
            })?;

        self.is_ranging = true;

        emit::event(ModuleEvent::Rangefinder(
            RangefinderEvent::RangingState {
                id: self.id().clone(),
                is_ranging: true,
            },
        ));

        Ok(())
    }

    fn stop_ranging(&mut self) -> anyhow::Result<()> {
        if !self.is_ranging {
            return Ok(());
        }

        self.sensor
            .stop_ranging()
            .map_err(|error| {
                anyhow::anyhow!(
                    "Failed to stop VL53L1X ranging: {error:?}"
                )
            })?;

        self.is_ranging = false;

        emit::event(ModuleEvent::Rangefinder(
            RangefinderEvent::RangingState {
                id: self.id().clone(),
                is_ranging: false,
            },
        ));

        Ok(())
    }
}

impl<'d> Module for Rangefinder<'d> {
    fn id(&self) -> &String {
        &self.core.id
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }

    fn get_module_type(&self) -> &ModuleType {
        &self.core.module_type
    }

    fn handle_command(
        &mut self,
        command: &ModuleCommand,
    ) -> anyhow::Result<()> {
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

            RangefinderCommandPayload::SetTimingBudget {
                milliseconds,
            } => {
                self.sensor
                    .set_timing_budget_ms(*milliseconds)
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Failed to set timing budget: {error:?}"
                        )
                    })?;

                self.timing_budget_ms = *milliseconds;

                emit::event(ModuleEvent::Rangefinder(
                    RangefinderEvent::TimingBudget {
                        id: self.id().clone(),
                        milliseconds: *milliseconds,
                    },
                ));
            }

           

            RangefinderCommandPayload::SetDistanceMode {
                mode,
            } => {
                let sensor_mode = match mode {
                    RangefinderDistanceMode::Short => {
                        DistanceMode::Short
                    }
                    RangefinderDistanceMode::Long => {
                        DistanceMode::Long
                    }
                };

                self.sensor
                    .set_distance_mode(sensor_mode)
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Failed to set distance mode: {error:?}"
                        )
                    })?;

                self.distance_mode = sensor_mode;

                emit::event(ModuleEvent::Rangefinder(
                    RangefinderEvent::DistanceMode {
                        id: self.id().clone(),
                        mode: mode.clone(),
                    },
                ));
            }
        }

        Ok(())
    }
}
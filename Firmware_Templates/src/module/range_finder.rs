use std::sync::mpsc::SyncSender;

use crate::core::emitter::Emitter;
use crate::core::hardware::RangefinderI2c;
use crate::core::modulecore::{Module, ModuleCore};
use crate::protocol::command::{ModuleCommand};
use crate::protocol::global_definitions::ModuleType;
use crate::protocol::module_event::{LogPriority, ModuleEvent, RangefinderEvent, SysLogEvent};
use crate::protocol::registration::{ Registration};

use serde::{Deserialize, Serialize};
use vl53l1x_uld::{DistanceMode, IOVoltage, RangeStatus, DEFAULT_ADDRESS, VL53L1X};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub enum RangefinderDistanceMode {
    Short,
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "command")]
pub enum RangefinderCommandPayload {
    StartRanging,
    StopRanging,
    SetTimingBudget { milliseconds: u16 },
    SetDistanceMode { mode: RangefinderDistanceMode },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "event_type")]
pub enum RangefinderEvent {
    Range {
        id: String,
        millimeters: u16,
    },
    RangingState {
        id: String,
        is_ranging: bool,
    },
    TimingBudget {
        id: String,
        milliseconds: u16,
    },
    DistanceMode {
        id: String,
        mode: RangefinderDistanceMode,
    },
    InvalidMeasurement {
        id: String,
        status: String,
    },
}
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
            core: ModuleCore::new(ModuleType::Rangefinder, &manual_id, sender),
            sensor,
            range_mm: 0,
            is_ranging: false,
            timing_budget_ms: 50,
            inter_measurement_ms: 60,
            distance_mode: DistanceMode::Long,
        };

        rangefinder.registration(Registration {
            id: rangefinder.id().to_string(),
            module_type: ModuleType::Rangefinder,
            lool_up_id: manual_id,
            parent_id: cluster_id.unwrap_or_default(),
        });

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

    pub fn get_range(&mut self) -> Option<u16> {
        let ready = match self.sensor.is_data_ready() {
            Ok(ready) => ready,
            Err(error) => {
                self.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: "VL53L1X data-ready check failed".to_string(),
                    raw_err: Some(format!("{error:?}")),
                    priority: LogPriority::Critical,
                }));

                return None;
            }
        };

        if !ready {
            return None;
        }

        let status = match self.sensor.get_range_status() {
            Ok(status) => status,
            Err(error) => {
                self.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: "VL53L1X range-status read failed".to_string(),
                    raw_err: Some(format!("{error:?}")),
                    priority: LogPriority::Critical,
                }));
                return None;
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

                return None;
            }
        };

        if let Err(error) = self.sensor.clear_interrupt() {
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: "VL53L1X interrupt clear failed".to_string(),
                raw_err: Some(format!("{error:?}")),
                priority: LogPriority::Critical,
            }));

            return None;
        }

        if status != RangeStatus::Valid {
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: "VL53L1X RangeStatus is Valid failed".to_string(),
                raw_err: None,
                priority: LogPriority::Critical,
            }));
            return None;
        }
        self.range_mm = distance;

        Some(self.range_mm)
    }
}

impl<'d> Module for Rangefinder<'d> {
    fn  tick(&mut self)->Result<() , ()> {
        if !self.is_ranging {
            return  Err(());
        }
        match self.get_range() {
            Some(rang) => {
                self.emit(ModuleEvent::Rangefinder(RangefinderEvent::Range {
                    id: self.id().to_string(),
                    millimeters: rang,
                }));
                
            }
            None => {
                return  Err(());
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

//! Experimental two-axis scanning rangefinder: two servos (pan/tilt) and a VL53L1X,
//! owned as children. It scans a serpentine raster over the ROI and emits `PointMap`
//! chunks of up to 100 points.
//!
//! Safety (PIN-TPL-004/-005, Phase 1D):
//! * Nothing moves during construction or registration. `first_emit` only reports the
//!   ROI; the corner sweep is the explicit [`Lidar::run_self_test`].
//! * The ROI must lie within the servo range and the step must be 1–180; invalid
//!   values are rejected, and the dispatcher reports the error as a `SysLog`.
//! * Each next scan position is worked out by the pure [`super::lidar_scan`] planner
//!   from the position read back from the servos, so a scan follows a clamped or moved
//!   servo. The point buffer is flushed every 100 points.
//! * A distance is recorded only if its measurement began after the servos arrived.
//! * A scan always ends: a position the sensor cannot measure is skipped (no point, an
//!   `InvalidMeasurement` event), and a failed servo move or a sensor that stops
//!   answering ends the scan with a report.
//!
//! Children are routable directly by their ids (Phase 2A, PIN-TPL-017) and through
//! `ChangeMotorAngle`. Not instantiated in the current image.

use crate::core::hardware::{now_us, RangefinderI2c};
use crate::core::transport::emitter::{EmitterError, TransportEmitter};
use crate::modules::composite::lidar_scan::{
    chunk_count, reading_usable_from_us, travel_degrees, validate_roi, validate_step, ScanAdvance,
    ScanPlan, POINTS_PER_CHUNK, READING_TIMEOUT_US,
};
use crate::utilities::rate_limit::{with_suppressed, Admission, RateLimiter};
use pinora_protocol::module::lidar::{LidarEvent, RangPoint, ScanState};
use pinora_protocol::module::servomodule::ServoCapability;

use crate::core::{
    hardware::SharedPwm,
    module::{Module, ModuleCore, ModuleError},
};
use crate::modules::actuators::servo::ServoModule;
use crate::modules::sensors::rangefinder::Rangefinder;
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::{LogPriority, ModuleEvent, SysLogEvent},
};
use pinora_protocol::{LidarCommandPayload, Point};
use pwm_pca9685::Channel;

pub struct Lidar<'d> {
    core: ModuleCore,

    servo_x: ServoModule<'d>,
    servo_y: ServoModule<'d>,
    rangefinder: Rangefinder<'d>,

    /// ROI end corner (the scan finishes here).
    min_point: Point,
    /// ROI start corner (the scan begins here).
    max_point: Point,

    step: u32,
    curr_scan_mode: ScanState,
    scan: Option<ScanPlan>,

    scan_time: std::time::Instant,
    /// How long the last finished scan took, for replay.
    last_scan_secs: f32,
    /// Measurements finishing before this time began while the servos were moving.
    reading_usable_from_us: u64,
    /// When the sensor last answered during this scan.
    last_reading_us: u64,
    point_map: Vec<RangPoint>,

    current_chunk: u32,
    total_chunks: u32,
    /// At most one "NEXT ROW/COLUMN" SysLog per second (Phase 2B, PIN-TPL-010).
    step_log_limit: RateLimiter,
}

impl<'d> Lidar<'d> {
    pub fn new(
        pwm: SharedPwm<'d>,
        lookup_name: String,
        rangefinder_i2c: RangefinderI2c<'d>,
        sender: TransportEmitter,
    ) -> anyhow::Result<Lidar<'d>> {
        let mc = ModuleCore::new(ModuleType::Lidar, &lookup_name, None, sender.clone());
        let config = ServoCapability {
            max_angle: 180,
            min_angle: 0,
            offset: 90,
            max_pivot: 90,
            min_pivot: -90,
            pulse_max: 2500,
            pulse_min: 500,
        };

        let servo_x = ServoModule::new(
            pwm.clone(),
            "servo_x".to_string(),
            Channel::C1,
            config.clone(),
            Some(mc.id.clone()),
            sender.clone(),
        )?;

        let servo_y = ServoModule::new(
            pwm.clone(),
            "servo_y".to_string(),
            Channel::C0,
            config.clone(),
            Some(mc.id.clone()),
            sender.clone(),
        )?;

        let rangefinder = Rangefinder::new(
            rangefinder_i2c,
            "rangefinder".to_string(),
            Some(mc.id.clone()),
            sender.clone(),
        )?;

        let new_lidar = Lidar {
            core: mc,
            servo_x,
            servo_y,
            min_point: Point { x: 90, y: -90 },
            max_point: Point { x: -90, y: 90 },
            step: 1,
            curr_scan_mode: ScanState::Idol,
            scan: None,
            scan_time: std::time::Instant::now(),
            last_scan_secs: 0.0,
            reading_usable_from_us: 0,
            last_reading_us: 0,
            rangefinder,
            point_map: vec![],
            current_chunk: 1,
            total_chunks: 0,
            step_log_limit: RateLimiter::per_second(1),
        };

        Ok(new_lidar)
    }

    fn flush_point_map(&mut self) {
        if self.point_map.is_empty() {
            return;
        }

        let map = std::mem::take(&mut self.point_map);

        self.emit(ModuleEvent::Lidar(LidarEvent::PointMap {
            curr_chunk: self.current_chunk as i32,
            max_chunk: self.total_chunks as i32,
            map,
        }));

        self.current_chunk += 1;
    }

    /// Moves both servos. On an error the position read back stays where the servos
    /// really are. Later readings count only once the servos have arrived.
    pub fn move_to_point(&mut self, target: &Point) -> anyhow::Result<()> {
        let from = self.servo_position();
        let result = self
            .servo_x
            .set_angle_silent(target.x)
            .and_then(|()| self.servo_y.set_angle_silent(target.y));
        self.note_moved_from(&from);
        result
    }

    fn note_moved_from(&mut self, from: &Point) {
        let travel = travel_degrees(from, &self.servo_position());
        self.reading_usable_from_us = reading_usable_from_us(
            now_us() as u64,
            travel,
            self.rangefinder.timing_budget_ms(),
        );
    }

    /// Where the servos are, as far as can be known: hobby servos have no feedback, so
    /// this is the last angle each was commanded to, after its limits.
    fn servo_position(&self) -> Point {
        Point {
            x: self.servo_x.pivot_angle(),
            y: self.servo_y.pivot_angle(),
        }
    }
    pub fn sync_all(&mut self) {
        self.servo_x.sync();
        self.servo_y.sync();
    }

    pub fn get_id(&self) -> String {
        self.id().to_string()
    }

    /// Sweeps both servos to the ROI corners and back to centre. **Moves hardware.**
    /// Never called automatically (it ran from `first_emit` before Phase 1D).
    pub fn run_self_test(&mut self) -> anyhow::Result<()> {
        self.move_to_point(&self.max_point.clone())?;
        self.move_to_point(&self.min_point.clone())?;
        self.move_to_point(&Point { x: 0, y: 0 })?;
        self.sync_all();
        Ok(())
    }

    /// Ends the scan: sends the points still buffered, then the final state and the
    /// servos' position.
    fn end_scan(&mut self, state: ScanState) {
        self.flush_point_map();
        self.scan = None;
        self.curr_scan_mode = state;
        self.last_scan_secs = self.scan_time.elapsed().as_secs_f32();

        self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
            state: self.curr_scan_mode.clone(),
            scan_time: self.last_scan_secs,
        }));
        self.sync_all();
    }

    fn finish_scan(&mut self) {
        self.end_scan(ScanState::Idol);
    }

    /// Ends a scan that cannot continue, and says why.
    fn abort_scan(&mut self, text: String) {
        self.emit(ModuleEvent::SysLog(SysLogEvent {
            text,
            raw_err: None,
            priority: LogPriority::High,
        }));
        self.end_scan(ScanState::StopScan);
    }
}

impl<'d> Module for Lidar<'d> {
    fn register(&self) -> Result<(), EmitterError> {
        self.emit_registration()?;
        self.servo_x.register()?;
        self.servo_y.register()?;
        self.rangefinder.register()?;
        Ok(())
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    /// Exposes the two servos and the rangefinder so commands addressed to them are
    /// routed (Phase 2A, PIN-TPL-017). A running scan continues from wherever a servo
    /// was moved to, because `tick` reads the position from the servos.
    fn child_mut(&mut self, id: &str) -> Option<&mut dyn Module> {
        if self.servo_x.id() == id {
            Some(&mut self.servo_x)
        } else if self.servo_y.id() == id {
            Some(&mut self.servo_y)
        } else if self.rangefinder.id() == id {
            Some(&mut self.rangefinder)
        } else {
            None
        }
    }

    /// Replays the ROI, the scan state and the children's state. Does not move anything.
    fn first_emit(&mut self) -> Result<(), EmitterError> {
        self.emit(ModuleEvent::Lidar(LidarEvent::Roi {
            min: self.min_point.clone(),
            max: self.max_point.clone(),
        }));
        let scan_time = if self.curr_scan_mode == ScanState::Scanning {
            self.scan_time.elapsed().as_secs_f32()
        } else {
            self.last_scan_secs
        };
        self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
            state: self.curr_scan_mode.clone(),
            scan_time,
        }));
        self.servo_x.first_emit()?;
        self.servo_y.first_emit()?;
        self.rangefinder.first_emit()?;
        Ok(())
    }
    fn tick(&mut self) -> Result<(), ModuleError> {
        if self.curr_scan_mode != ScanState::Scanning {
            return Ok(());
        }

        let now = now_us() as u64;
        let reading = match self.rangefinder.update_range() {
            Ok(()) => Some(self.rangefinder.range()),
            // Nothing measurable here: no point, and the scan moves on.
            Err(ModuleError::InvalidReading) => None,
            Err(error) => {
                if now.saturating_sub(self.last_reading_us) > READING_TIMEOUT_US {
                    self.abort_scan(format!(
                        "LiDAR scan ended: no reading from the rangefinder ({error:?})"
                    ));
                    return Ok(());
                }
                return match error {
                    ModuleError::SensorNotReady => Ok(()),
                    other => Err(other),
                };
            }
        };
        self.last_reading_us = now;

        // This measurement began before the servos arrived, so it is not for this position.
        if now < self.reading_usable_from_us {
            return Ok(());
        }

        let position = self.servo_position();
        if let Some(distant) = reading {
            self.point_map.push(RangPoint {
                x: position.x,
                y: position.y,
                distant,
            });
        }
        // Bounded buffer: flush every full chunk, not only at row ends.
        if self.point_map.len() >= POINTS_PER_CHUNK {
            self.flush_point_map();
        }

        let Some(scan) = self.scan.as_mut() else {
            self.finish_scan();
            return Ok(());
        };

        let (next, text) = match scan.advance(&position) {
            ScanAdvance::Done => {
                self.finish_scan();
                return Ok(());
            }
            ScanAdvance::NextRow(next) => {
                let text = format!("NEXT ROW: pos=({}, {})", next.x, next.y);
                (next, text)
            }
            ScanAdvance::NextColumn(next) => {
                let text = format!("NEXT COLUMN: pos=({}, {})", next.x, next.y);
                (next, text)
            }
        };
        if let Admission::Admit { suppressed } = self.step_log_limit.admit(now_us() as u64) {
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: with_suppressed(&text, suppressed),
                raw_err: None,
                priority: LogPriority::Low,
            }));
        }

        if let Err(error) = self.move_to_point(&next) {
            self.abort_scan(format!("LiDAR scan ended: servo move failed: {error}"));
        }

        Ok(())
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Lidar(lidar_command) => match lidar_command {
                LidarCommandPayload::ChangeMotorAngle { id, step } => {
                    let from = self.servo_position();
                    let result = if self.servo_x.id() == id {
                        self.servo_x.set_angle(*step)
                    } else if self.servo_y.id() == id {
                        self.servo_y.set_angle(*step)
                    } else {
                        Ok(())
                    };
                    self.note_moved_from(&from);
                    result?;
                }
                LidarCommandPayload::Roi { min, max } => {
                    // The plan in progress keeps its own corners; do not report others.
                    anyhow::ensure!(self.scan.is_none(), "LiDAR ROI cannot change during a scan");
                    validate_roi(min, max)?;
                    self.min_point = min.clone();
                    self.max_point = max.clone();

                    self.emit(ModuleEvent::Lidar(LidarEvent::Roi {
                        min: self.min_point.clone(),
                        max: self.max_point.clone(),
                    }));
                }
                LidarCommandPayload::SetStep { step } => {
                    anyhow::ensure!(self.scan.is_none(), "LiDAR step cannot change during a scan");
                    validate_step(*step)?;
                    self.step = *step;
                }
                LidarCommandPayload::StartScan {} => {
                    let plan = ScanPlan::new(&self.max_point, &self.min_point, self.step)?;
                    self.rangefinder.start_ranging()?;
                    self.move_to_point(&plan.start())?;
                    self.scan_time = std::time::Instant::now();
                    self.last_reading_us = now_us() as u64;
                    self.point_map.clear();
                    self.current_chunk = 1;
                    self.total_chunks = chunk_count(plan.total_points());

                    self.scan = Some(plan);
                    self.curr_scan_mode = ScanState::Scanning;
                    self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
                        state: self.curr_scan_mode.clone(),
                        scan_time: 0.0,
                    }));
                }

                LidarCommandPayload::StopScan {} => {
                    self.end_scan(ScanState::StopScan);
                }
                LidarCommandPayload::Test {} => {}
                LidarCommandPayload::MovePos { p } => {
                    let result = self.move_to_point(p);
                    self.sync_all();
                    result?;
                }
            },
            _ => {}
        }
        Ok(())
    }
}

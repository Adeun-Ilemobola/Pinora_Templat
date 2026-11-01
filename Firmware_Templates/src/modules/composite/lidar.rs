//! Experimental two-axis scanning rangefinder: two servos (pan/tilt) and a VL53L1X,
//! owned as children. It scans a serpentine raster over the ROI and emits `PointMap`
//! chunks of up to 100 points.
//!
//! Safety (PIN-TPL-004/-005, Phase 1D):
//! * Nothing moves during construction or registration. `first_emit` only reports the
//!   ROI; the corner sweep is the explicit [`Lidar::run_self_test`].
//! * The ROI must lie within the servo range and the step must be 1–180; invalid
//!   values are rejected, and the dispatcher reports the error as a `SysLog`.
//! * Scan positions come from the pure [`super::lidar_scan`] planner, so every scan
//!   terminates, and the point buffer is flushed every 100 points.
//!
//! Children are routable only through `ChangeMotorAngle` (PIN-TPL-017). Not
//! instantiated in the current image.

use crate::core::hardware::{now_us, RangefinderI2c, TimerState};
use crate::core::transport::emitter::{EmitterError, TransportEmitter};
use crate::modules::composite::lidar_scan::{
    chunk_count, validate_roi, validate_step, ScanAdvance, ScanPlan, POINTS_PER_CHUNK,
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

    curr_point_bottom: Point,
    step: u32,
    curr_scan_mode: ScanState,
    scan: Option<ScanPlan>,

    scan_time: std::time::Instant,
    step_timer: TimerState,
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

        let mut new_lidar = Lidar {
            core: mc,
            servo_x,
            servo_y,
            min_point: Point { x: 90, y: -90 },
            max_point: Point { x: -90, y: 90 },
            curr_point_bottom: Point { x: 0, y: 0 },
            step: 1,
            curr_scan_mode: ScanState::Idol,
            scan: None,
            scan_time: std::time::Instant::now(),
            step_timer: TimerState::from_ms(30.0),
            rangefinder,
            point_map: vec![],
            current_chunk: 1,
            total_chunks: 0,
            step_log_limit: RateLimiter::per_second(1),
        };

        match new_lidar.rangefinder.start_ranging() {
            Ok(_) => {}
            Err(err) => {
                new_lidar.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: format!("start_ranging in lidar has fail : {:?}", err),
                    raw_err: None,
                    priority: LogPriority::High,
                }));
            }
        }

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

    pub fn move_to_point(&mut self) {
        let _ = self
            .servo_x
            .set_angle_silent(self.curr_point_bottom.x.clone());
        let _ = self
            .servo_y
            .set_angle_silent(self.curr_point_bottom.y.clone());
        self.curr_point_bottom = Point {
            x: self.servo_x.pivot_angle(),
            y: self.servo_y.pivot_angle(),
        };
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
    pub fn run_self_test(&mut self) {
        self.curr_point_bottom = self.max_point.clone();
        self.move_to_point();
        self.curr_point_bottom = self.min_point.clone();
        self.move_to_point();
        self.curr_point_bottom = Point { x: 0, y: 0 };
        self.move_to_point();
        self.curr_point_bottom = Point { x: 0, y: 0 };
        self.move_to_point();
    }

    fn finish_scan(&mut self) {
        self.flush_point_map();
        self.scan = None;
        self.curr_scan_mode = ScanState::Idol;

        self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
            state: self.curr_scan_mode.clone(),
            scan_time: self.scan_time.elapsed().as_secs_f32(),
        }));
        self.sync_all();
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
    /// routed (Phase 2A, PIN-TPL-017). Note: moving a servo directly does not update
    /// the scan position until the next scan or `ChangeMotorAngle`.
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

    /// Reports the configured ROI. Does not move the servos.
    fn first_emit(&mut self) -> Result<(), EmitterError> {
        self.emit(ModuleEvent::Lidar(LidarEvent::Roi {
            min: self.min_point.clone(),
            max: self.max_point.clone(),
        }));

        Ok(())
    }
    fn tick(&mut self) -> Result<(), ModuleError> {
        if self.curr_scan_mode != ScanState::Scanning {
            return Ok(());
        }

        let range_bottom = match self.rangefinder.update_range() {
            Ok(()) => self.rangefinder.range(),

            Err(ModuleError::SensorNotReady) => {
                return Ok(());
            }

            Err(error) => {
                self.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: format!("Rangefinder error: {:?}", error),
                    priority: LogPriority::High,
                    raw_err: Some(format!("{:?}", error)),
                }));

                return Err(error);
            }
        };

        // Record the position where the servos have been resting.
        self.point_map.push(RangPoint {
            x: self.curr_point_bottom.x,
            y: self.curr_point_bottom.y,
            distant: range_bottom,
        });
        // Bounded buffer: flush every full chunk, not only at row ends.
        if self.point_map.len() >= POINTS_PER_CHUNK {
            self.flush_point_map();
        }

        let Some(scan) = self.scan.as_mut() else {
            self.finish_scan();
            return Ok(());
        };

        let (next, text) = match scan.advance() {
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

        self.curr_point_bottom = next;
        self.move_to_point();
        self.step_timer.reset();

        Ok(())
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Lidar(lidar_command) => match lidar_command {
                LidarCommandPayload::ChangeMotorAngle { id, step } => {
                    if self.servo_x.id() == id {
                        let _ = self.servo_x.set_angle(*step);
                        self.curr_point_bottom.x = self.servo_x.pivot_angle();
                    } else if self.servo_y.id() == id {
                        let _ = self.servo_y.set_angle(*step);
                        self.curr_point_bottom.y = self.servo_y.pivot_angle();
                    }
                }
                LidarCommandPayload::Roi { min, max } => {
                    validate_roi(min, max)?;
                    self.min_point = min.clone();
                    self.max_point = max.clone();

                    self.emit(ModuleEvent::Lidar(LidarEvent::Roi {
                        min: self.min_point.clone(),
                        max: self.max_point.clone(),
                    }));
                }
                LidarCommandPayload::SetStep { step } => {
                    validate_step(*step)?;
                    self.step = *step;
                }
                LidarCommandPayload::StartScan {} => {
                    let plan = ScanPlan::new(&self.max_point, &self.min_point, self.step)?;
                    self.scan_time = std::time::Instant::now();
                    self.point_map.clear();
                    self.current_chunk = 1;
                    self.total_chunks = chunk_count(plan.total_points());

                    self.curr_point_bottom = plan.position();
                    self.scan = Some(plan);
                    self.step_timer.reset();
                    self.move_to_point();
                    self.curr_scan_mode = ScanState::Scanning;
                    self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
                        state: self.curr_scan_mode.clone(),
                        scan_time: 0.0,
                    }));
                }

                LidarCommandPayload::StopScan {} => {
                    self.scan = None;
                    self.curr_scan_mode = ScanState::StopScan;
                    self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
                        state: self.curr_scan_mode.clone(),
                        scan_time: self.scan_time.elapsed().as_secs_f32(),
                    }));
                }
                LidarCommandPayload::Test {} => {}
                LidarCommandPayload::MovePos { p } => {
                    self.curr_point_bottom = p.clone();
                    self.move_to_point();
                }
            },
            _ => {}
        }
        Ok(())
    }
}

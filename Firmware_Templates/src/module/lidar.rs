use crate::core::hardware::{I2cDriver, RangefinderI2c, TimerState};
use crate::core::transport::transport_emiter::{EmitterError, TransportEmiter};
use pinora_protocol::module::lidar::{LidarEvent, RangPoint, ScanState};
use pinora_protocol::module::servomodule::ServoCapability;

use crate::core::{
    hardware::SharedPwm,
    modulecore::{Module, ModuleCore, ModuleError},
};
use crate::module::range_finder::Rangefinder;
use crate::module::servomodule::ServoModule;
use embedded_hal_compat::ReverseCompat;
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::{LogPriority, ModuleEvent, SysLogEvent},
    registration::ProtocolMessage,
};
use pinora_protocol::{LidarCommandPayload, Point};
use pwm_pca9685::Channel;
const POINTS_PER_CHUNK: usize = 100;

pub struct Lidar<'d> {
    core: ModuleCore,

    servo_x: ServoModule<'d>,
    servo_y: ServoModule<'d>,
    rangefinder: Rangefinder<'d>,

    min_point: Point,
    max_point: Point,

    limit_point: Point,
    curr_point_bottom: Point,
    step: u32,
    step_y: u32,
    curr_scan_mode: ScanState,
    x_d: i32,

    scan_time: std::time::Instant,
    step_timer: TimerState,
    point_map: Vec<RangPoint>,

    current_chunk: u32,
    total_chunks: u32,
}
// const CHUNK_SIZE: usize = 128;

impl<'d> Lidar<'d> {
    pub fn new(
        pwm: SharedPwm<'d>,
        manuel_id: String,
        rangefinder_i2c: RangefinderI2c<'d>,
        sender: TransportEmiter,
    ) -> anyhow::Result<Lidar<'d>> {
        let mc = ModuleCore::new(ModuleType::Lidar, &manuel_id, None, sender.clone());
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
            step_y: 2,
            curr_scan_mode: ScanState::Idol,
            x_d: 1,
            limit_point: Point { x: -90, y: 90 },
            scan_time: std::time::Instant::now(),
            step_timer: TimerState::from_ms(30.0),
            rangefinder,
            point_map: vec![],
            current_chunk: 1,
            total_chunks: 0,
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
        // const X_DIRECTION: i32 = -1;
        // const Y_DIRECTION: i32 = 1;

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

    pub fn initialize(&mut self) {
        self.curr_point_bottom = self.max_point.clone();
        self.move_to_point();
        self.curr_point_bottom = self.min_point.clone();
        self.move_to_point();
        self.curr_point_bottom = Point { x: 0, y: 0 };
        self.move_to_point();
        self.curr_point_bottom = Point { x: 0, y: 0 };
        self.move_to_point();
        self.emit(ModuleEvent::Lidar(LidarEvent::Roi {
            min: self.min_point.clone(),
            max: self.max_point.clone(),
        }));
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
    fn first_emit(&mut self) -> Result<(), EmitterError> {
        self.initialize();

        Ok(())
    }
    fn tick(&mut self) -> Result<(), ModuleError> {
        //self.rangefinder.tick()?;
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

        //  let range_top = match self.rangefinder_top.get_range() {
        //     Some(r) => r,
        //     None => {
        //         return Ok(());
        //     }
        // };

        // if !self.step_timer.ready() {
        //     return Ok(());
        // }

        // Record the position where the servo has already been resting.

        // if self.curr_point_bottom.y > self.max_point.y {
        //     self.point_map.push(RangPoint {
        //         x: self.curr_point_bottom.x,
        //         y: self.curr_point_bottom.y + self.step_y as i32,
        //         distant: self.rangefinder_top.range_mm,
        //     });
        // }

        self.point_map.push(RangPoint {
            x: self.curr_point_bottom.x,
            y: self.curr_point_bottom.y,
            distant: range_bottom,
        });

        let row_finished = self.curr_point_bottom.x == self.limit_point.x;
        let scan_finished = row_finished && self.curr_point_bottom.y == self.limit_point.y;

        if scan_finished {
            self.flush_point_map();
            self.curr_scan_mode = ScanState::Idol;

            self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
                state: self.curr_scan_mode.clone(),
                scan_time: self.scan_time.elapsed().as_secs_f32(),
            }));
            self.sync_all();

            return Ok(());
        }

        if row_finished {
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: format!(
                    "ROW FINISHED: pos=({}, {}), limit=({}, {}), direction={}",
                    self.curr_point_bottom.x,
                    self.curr_point_bottom.y,
                    self.limit_point.x,
                    self.limit_point.y,
                    self.x_d
                ),
                raw_err: None,
                priority: LogPriority::Low,
            }));
            if self.point_map.len() >= POINTS_PER_CHUNK {
                self.flush_point_map();
            }

            self.x_d *= -1;

            self.limit_point.x = if self.limit_point.x == self.max_point.x {
                self.min_point.x
            } else {
                self.max_point.x
            };

            self.curr_point_bottom.y -= self.step as i32;
            // self.curr_point_bottom.y -= self.step_y as i32;
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: format!(
                    "NEXT ROW: pos=({}, {}), limit_x={}, direction={}",
                    self.curr_point_bottom.x,
                    self.curr_point_bottom.y,
                    self.limit_point.x,
                    self.x_d
                ),
                raw_err: None,
                priority: LogPriority::Low,
            }));
        } else {
            self.curr_point_bottom.x += self.step as i32 * self.x_d;
            self.emit(ModuleEvent::SysLog(SysLogEvent {
                text: format!(
                    "NEXT COLUMN: pos=({}, {}), limit_x={}, direction={}",
                    self.curr_point_bottom.x,
                    self.curr_point_bottom.y,
                    self.limit_point.x,
                    self.x_d
                ),
                raw_err: None,
                priority: LogPriority::Low,
            }));
        }

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
                    self.min_point = min.clone();
                    self.max_point = max.clone();
                    let total_points = (max.x.abs() as u32 * 2) * (min.y.abs() as u32 * 2);

                    self.total_chunks = total_points.div_ceil(POINTS_PER_CHUNK as u32);

                    self.emit(ModuleEvent::Lidar(LidarEvent::Roi {
                        min: self.min_point.clone(),
                        max: self.max_point.clone(),
                    }));
                }
                LidarCommandPayload::SetStep { step } => {
                    self.step = *step;
                }
                LidarCommandPayload::StartScan {} => {
                    self.scan_time = std::time::Instant::now();
                    self.point_map.clear();
                    self.current_chunk = 1;

                    self.curr_point_bottom = self.max_point.clone();
                    self.limit_point = self.min_point.clone();
                    self.step_timer.reset();
                    self.x_d = if self.limit_point.x > self.curr_point_bottom.x {
                        1
                    } else {
                        -1
                    };
                    self.move_to_point();
                    self.curr_scan_mode = ScanState::Scanning;
                    self.emit(ModuleEvent::Lidar(LidarEvent::ScanState {
                        state: self.curr_scan_mode.clone(),
                        scan_time: 0.0,
                    }));
                }

                LidarCommandPayload::StopScan {} => {
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

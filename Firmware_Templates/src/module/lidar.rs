use crate::core::hardware::I2cDriver;
use crate::core::modulecore::emit;
use crate::module::range_finder::Rangefinder;
use crate::module::servomodule::ServoModule;
use crate::protocol::command::{LidarCommandPayload, ModuleCommand};
use crate::protocol::global_definitions::{ModuleType, Point, RangPoint, ServoCapability};
use crate::protocol::module_event::{
    LidarEvent, LogPriority, ModuleEvent, ScanState, SysLogEvent,
};
use crate::protocol::registration::{ Registration};
use crate::utilities::logger::SysLog;
use crate::{
    core::{
        hardware::SharedPwm,
        modulecore::{Module, ModuleCore},
    },
};
use embedded_hal_bus::i2c::RcDevice;
use embedded_hal_compat::ReverseCompat;
use pwm_pca9685::Channel;

pub struct Lidar<'d> {
    core: ModuleCore,

    servo_x: ServoModule<'d>,
    servo_y: ServoModule<'d>,
    rangefinder: Rangefinder<'d>,

    min_point: Point,
    max_point: Point,

    limit_point: Point,
    curr_point: Point,
    step: u32,
    curr_scan_mode: ScanState,
    x_d: i32,
    scan_time: Option<std::time::Instant>,
    point_map:Vec<RangPoint>
}

impl<'d> Lidar<'d> {
    pub fn new(
        pwm: SharedPwm<'d>,
        manuel_id: String,
        rangefinder_i2c: RcDevice<I2cDriver<'d>>,
    ) -> anyhow::Result<Lidar<'d>> {
        let mc = ModuleCore::new(ModuleType::Lidar, &manuel_id);
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
            Channel::C0,
            config.clone(),
            Some(mc.get_id().to_string()),
        )?;

        let servo_y = ServoModule::new(
            pwm.clone(),
            "servo_y".to_string(),
            Channel::C1,
            config.clone(),
            Some(mc.get_id().to_string()),
        )?;

        let mut rangefinder = Rangefinder::new(
            rangefinder_i2c.reverse(),
            "rangefinder".to_string(),
            Some(mc.get_id().to_string()),
        )?;
        match rangefinder.start_ranging() {
            Ok(_) => {}
            Err(err) => {
                emit::event(ModuleEvent::SysLog(SysLogEvent {
                    text: format!("start_ranging in lidar has fail : {:?}",err),
                    raw_err: None,
                    priority: LogPriority::High,
                }));
            }
        }

        let mut new_lidar = Lidar {
            core: mc,
            servo_x,
            servo_y,
            min_point: Point { x: 90, y: -90 },
            max_point: Point { x: -90, y: 90 },
            curr_point: Point { x: 0, y: 0 },
            step: 1,
            curr_scan_mode: ScanState::Idol,
            x_d: 1,
            limit_point: Point { x: -90, y: 90 },
            scan_time: None,
            rangefinder,
            point_map: vec![]
        };
        emit::registration(Registration {
            id: new_lidar.id().clone(),
            lool_up_id: manuel_id.clone(),
            module_type: ModuleType::Lidar,
            parent_id: String::new(),
        });

        new_lidar.curr_point = new_lidar.max_point.clone();
        new_lidar.move_to_point();
        new_lidar.curr_point = new_lidar.min_point.clone();
        new_lidar.move_to_point();
        new_lidar.curr_point = Point { x: 0, y: 0 };
        new_lidar.move_to_point();
        emit::event(ModuleEvent::Lidar(LidarEvent::Roi {
            id: new_lidar.id().clone(),
            min: new_lidar.min_point.clone(),
            max: new_lidar.max_point.clone(),
        }));
        SysLog::info("-----Lidar start----".to_string(), None);

        Ok(new_lidar)
    }

    pub fn tick(&mut self) {
        self.rangefinder.tick();
        if self.curr_scan_mode == ScanState::Scanning {
            let mut next_point = Point {
                x: self.curr_point.x + (self.step as i32 * self.x_d),
                y: self.curr_point.y,
            };
           
            if (self.curr_point.x) == self.limit_point.x {
                if self.limit_point.x == self.max_point.x {
                    self.limit_point.x = self.min_point.x
                } else {
                    self.limit_point.x = self.max_point.x;
                }
                self.x_d *= -1;
                if self.curr_point.y == self.limit_point.y {
                    self.curr_scan_mode = ScanState::Idol;
                    emit::event(ModuleEvent::Lidar(LidarEvent::ScanState {
                        id: self.id().clone(),
                        state: self.curr_scan_mode.clone(),
                    }));
                    if let Some(start) = self.scan_time {
                        // log::debug!("write_all took {:?}", start.elapsed());
                        emit::event(ModuleEvent::SysLog(SysLogEvent {
                            text: format!("Full Scan time {:?}", start.elapsed()),
                            raw_err: None,
                            priority: LogPriority::High,
                        }));
                    }
                    return;
                }
                next_point.y -= 1;
            }

             self.point_map.push(RangPoint{
                x:next_point.x.clone(),
                y:next_point.y.clone(),
                distant:self.rangefinder.range_mm.clone()
             });
            self.curr_point = next_point;

            self.move_to_point();
        }
    }

    pub fn move_to_point(&mut self) {
        // const X_DIRECTION: i32 = -1;
        // const Y_DIRECTION: i32 = 1;

        let _ = self.servo_x.set_angle(self.curr_point.x.clone());
        let _ = self.servo_y.set_angle(self.curr_point.y.clone());
        self.curr_point = Point {
            x: self.servo_x.pivot_angle(),
            y: self.servo_y.pivot_angle(),
        };
    }

    pub fn get_id(&self) -> String {
        self.id().clone()
    }
}

impl<'d> Module for Lidar<'d> {
    fn id(&self) -> &String {
        &self.core.id
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn get_module_type(&self) -> &ModuleType {
        &self.core.module_type
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Lidar(lidar_command) => match lidar_command {
                LidarCommandPayload::ChangeMotorAngle { id, step } => {
                    SysLog::info(
                        format!(
                            "LiDAR received ChangeMotorAngle: servo_id={}, step={}",
                            id, step
                        ),
                        None,
                    );

                    if self.servo_x.id() == id {
                        let _ = self.servo_x.set_angle(*step);
                        self.curr_point.x = self.servo_x.pivot_angle();
                    } else if self.servo_y.id() == id {
                        let _ = self.servo_y.set_angle(*step);
                        self.curr_point.y = self.servo_y.pivot_angle();
                    }
                }
                LidarCommandPayload::Roi { min, max } => {
                    SysLog::info(
                        format!("LiDAR received ROI: min={:?}, max={:?}", min, max),
                        None,
                    );
                    self.min_point = min.clone();
                    self.max_point = max.clone();
                    emit::event(ModuleEvent::Lidar(LidarEvent::Roi {
                        id: self.id().clone(),
                        min: self.min_point.clone(),
                        max: self.max_point.clone(),
                    }));
                    

                }
                LidarCommandPayload::SetStep { step } => {
                    SysLog::info(format!("LiDAR received SetStep: step={}", step), None);
                    self.scan_time = None;
                    self.step = *step;
                }
                LidarCommandPayload::StartScan => {
                    self.scan_time = Some(std::time::Instant::now());
                    SysLog::info("LiDAR received StartScan".to_string(), None);

                    self.curr_point = self.max_point.clone();
                    self.limit_point = self.min_point.clone();
                    self.x_d = -1;
                    self.move_to_point();
                    self.curr_scan_mode = ScanState::Scanning;
                    emit::event(ModuleEvent::Lidar(LidarEvent::ScanState {
                        id: self.id().clone(),
                        state: self.curr_scan_mode.clone(),
                    }));
                }

                LidarCommandPayload::StopScan => {
                    SysLog::info("LiDAR received StopScan".to_string(), None);
                    self.curr_scan_mode = ScanState::StopScan;
                    emit::event(ModuleEvent::Lidar(LidarEvent::ScanState {
                        id: self.id().clone(),
                        state: self.curr_scan_mode.clone(),
                    }));
                }
                LidarCommandPayload::Test => {
                    SysLog::info("LiDAR received Test".to_string(), None);
                }
                LidarCommandPayload::MovePos { p } => {
                    SysLog::info(format!("LiDAR received MovePos: point={:?}", p), None);
                    self.curr_point = p.clone();
                    self.move_to_point();
                }
            },
            _ => {
                SysLog::error(
                    format!(
                        "LiDAR module {} received an incompatible command",
                        self.id()
                    ),
                    Some(format!("Received command: {:?}", command)),
                );
            }
        }
        Ok(())
    }
}

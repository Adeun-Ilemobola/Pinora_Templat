use std::sync::mpsc::SyncSender;

use crate::core::hardware::{ SharedPwm};
use crate::core::modulecore::{Module, ModuleCore, emit};
use crate::protocol::command::{ModuleCommand , ServoCommandPayload};
use crate::protocol::global_definitions::{ModuleType, ServoCapability};
use crate::protocol::module_event::{ModuleEvent, ServoEvent};
use crate::protocol::registration::{ Registration};
use crate::utilities::math::{pulse_us_to_tick, range_i32};

use anyhow::Ok;
use pwm_pca9685::Channel;

pub struct ServoModule<'d> {
    core: ModuleCore,
    pwm: SharedPwm<'d>,
    config: ServoCapability,
    channel: Channel,

    offset: i32,
    angle: i32,
    min_pivot: i32,
    max_pivot: i32,
}

impl<'d> ServoModule<'d> {
    pub fn new(
        pwm: SharedPwm<'d>,
        manuel_id: String,
        channel: Channel,
        config: ServoCapability,
        cluster_id: Option<String>,
        sender:SyncSender<ModuleEvent>
    ) -> anyhow::Result<ServoModule<'d>> {
        let mut s = ServoModule {
            core: ModuleCore::new(ModuleType::Servo, &manuel_id , sender),
            pwm,
            config: config.clone(),
            offset: config.offset,
            angle: config.min_pivot,
            max_pivot: config.max_pivot,
            min_pivot: config.min_pivot,
            channel: channel.clone(),
        };
         emit::registration(Registration{
        id:s.id().to_string(),
         module_type:ModuleType::Servo,
         lool_up_id:manuel_id.clone(),
          parent_id: cluster_id.clone().unwrap_or_default()
      });

        s.set_offset(s.offset)?;
        s.set_angle(0)?;
       

        let testrang: [i32; 4] = [35, 10, -10, -35];
        for f in testrang {
            s.set_angle(f)?;
          
        }
         s.set_angle(0)?;

        Ok(s)
    }
    pub fn set_offset(&mut self, a: i32) -> anyhow::Result<()> {
        self.offset = a.clamp(self.config.min_angle, self.config.max_angle);
        let pulse = range_i32(
            self.offset,
            self.config.min_angle,
            self.config.max_angle,
            self.config.pulse_min,
            self.config.pulse_max,
        );
        self.pwm
            .borrow_mut()
            .set_channel_on_off(self.channel, 0, pulse_us_to_tick(pulse))
            .unwrap();

        self.emit(ModuleEvent::Servo(ServoEvent::GetOffset { 
            id:self.id().to_string(),
            angle: self.offset.clone() 
        }));

        Ok(())
    }
    pub fn set_angle(&mut self, a: i32) -> anyhow::Result<()> {
        //  -22  ,  25
        let pivotrang = a.clamp(self.min_pivot, self.max_pivot);

        let raw_rang =
            (self.offset + pivotrang).clamp(self.config.min_angle, self.config.max_angle);

         self.angle = raw_rang;
         

        let pulse = range_i32(
            self.angle.clone(),
            self.config.min_angle,
            self.config.max_angle,
            self.config.pulse_min,
            self.config.pulse_max,
        );
        self.pwm
            .borrow_mut()
            .set_channel_on_off(self.channel, 0, pulse_us_to_tick(pulse))
            .map_err(|error| {
                anyhow::anyhow!(
                    " Servo Module pwm id: [{:?}]  initialization failed: {error:?}"
                    ,self.core.manuel_id.clone()
                )
            })?;

        self.emit(ModuleEvent::Servo(ServoEvent::GetAngle { id:self.id().to_string(), angle: pivotrang.clone() } ));
   
        Ok(())
    }

    pub fn pivot_angle(&self) -> i32 {
        self.angle - self.offset
    }
    pub fn angle (&self) -> i32{
        self.angle
    }

    pub fn set_min_pivot(&mut self, min_pivot: i32) {
        self.min_pivot = min_pivot.min(self.max_pivot);
        self.emit(ModuleEvent::Servo(ServoEvent::GetMinPivot { id:self.id().to_string(), min_pivot }));

       
    }

    pub fn set_max_pivot(&mut self, max_pivot: i32) {
        self.max_pivot = max_pivot.max(self.min_pivot);
        self.emit(ModuleEvent::Servo(ServoEvent::GetMaxPivot { id:self.id().to_string(), max_pivot }));
    }

   
}

impl<'d> Module for ServoModule<'d> {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Servo(servo_command) => match servo_command {
                ServoCommandPayload::SetAngle { angle } => self.set_angle(*angle)?,
                ServoCommandPayload::SetMinPivot { min_pivot } => {
                    self.set_min_pivot(*min_pivot)
                }
                ServoCommandPayload::SetMaxPivot { max_pivot } => {
                    self.set_max_pivot(*max_pivot)
                }
            },
            _ => {
                // handle anything else
            }
        }
        Ok(())
    }

   
}

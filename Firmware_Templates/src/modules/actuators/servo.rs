//! Hobby servo on a PCA9685 channel. Angles are pivots around `offset`, clamped to
//! `[min_pivot, max_pivot]` and then to `[min_angle, max_angle]`.
//!
//! Safety (PIN-TPL-004, Phase 1D): construction writes no PWM, so the servo stays
//! unpowered until its first command. The old construction-time sweep is the explicit
//! [`ServoModule::run_self_test`]. PWM write errors are returned, not panicked on.
//! Not instantiated in the current image.

use crate::core::hardware::SharedPwm;
use crate::core::module::{Module, ModuleCore, ModuleError};
use crate::core::transport::emitter::{EmitterError, TransportEmitter};
use crate::utilities::math::{pulse_us_to_tick, range_i32};

use pinora_protocol::{
    command::ModuleCommand, global_definitions::ModuleType, module_event::ModuleEvent,
};
use pwm_pca9685::Channel;

pub use pinora_protocol::module::servomodule::{ServoCapability, ServoCommandPayload, ServoEvent};

pub struct ServoModule<'d> {
    core: ModuleCore,
    pwm: SharedPwm<'d>,
    config: ServoCapability,
    channel: Channel,

    offset: i32,
    angle: i32,
    pivot: i32,
    min_pivot: i32,
    max_pivot: i32,
}

impl<'d> ServoModule<'d> {
    pub fn new(
        pwm: SharedPwm<'d>,
        lookup_name: String,
        channel: Channel,
        config: ServoCapability,
        cluster_id: Option<String>,
        sender: TransportEmitter,
    ) -> anyhow::Result<ServoModule<'d>> {
        // No PWM is written here: the servo does not move until it is commanded.
        let offset = config.offset.clamp(config.min_angle, config.max_angle);
        let s = ServoModule {
            core: ModuleCore::new(ModuleType::Servo, &lookup_name, cluster_id, sender),
            pwm,
            config: config.clone(),
            offset,
            angle: offset,
            max_pivot: config.max_pivot,
            min_pivot: config.min_pivot,
            channel: channel.clone(),
            pivot: 0,
        };

        Ok(s)
    }

    /// Drives the servo to its offset and through a short sweep (pivots 35°, 10°,
    /// -10°, -35°), then back to centre. **Moves hardware.** Never called
    /// automatically (it ran inside `new` before Phase 1D).
    pub fn run_self_test(&mut self) -> anyhow::Result<()> {
        self.set_offset(self.offset)?;
        self.set_angle(0)?;

        let self_test_angles: [i32; 4] = [35, 10, -10, -35];
        for f in self_test_angles {
            self.set_angle(f)?;
        }
        self.set_angle(0)
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
            .map_err(|error| {
                anyhow::anyhow!(
                    "Servo {:?} PWM write failed: {error:?}",
                    self.core.lookup_name.clone()
                )
            })?;

        self.emit(ModuleEvent::Servo(ServoEvent::GetOffset {
            angle: self.offset.clone(),
        }));

        Ok(())
    }
    pub fn set_angle(&mut self, a: i32) -> anyhow::Result<()> {
        //  -22  ,  25
        let pivotrang = a.clamp(self.min_pivot, self.max_pivot);
        self.pivot = pivotrang;

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
                    " Servo Module pwm id: [{:?}]  initialization failed: {error:?}",
                    self.core.lookup_name.clone()
                )
            })?;

        self.emit(ModuleEvent::Servo(ServoEvent::GetAngle {
            angle: pivotrang.clone(),
        }));

        Ok(())
    }
    pub fn set_angle_silent(&mut self, a: i32) -> anyhow::Result<()> {
        let pivotrang = a.clamp(self.min_pivot, self.max_pivot);
        self.pivot = pivotrang;

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
                    " Servo Module pwm id: [{:?}]  initialization failed: {error:?}",
                    self.core.lookup_name.clone()
                )
            })?;

        Ok(())
    }

    pub fn sync(&mut self) {
        self.emit(ModuleEvent::Servo(ServoEvent::GetAngle {
            angle: self.pivot.clone(),
        }));
    }
    pub fn pivot_angle(&self) -> i32 {
        self.angle - self.offset
    }
    pub fn angle(&self) -> i32 {
        self.angle
    }

    pub fn set_min_pivot(&mut self, min_pivot: i32) {
        self.min_pivot = min_pivot.min(self.max_pivot);
        // Report the stored (clamped) limit, not the request (PIN-TPL-025).
        self.emit(ModuleEvent::Servo(ServoEvent::GetMinPivot {
            min_pivot: self.min_pivot,
        }));
    }

    pub fn set_max_pivot(&mut self, max_pivot: i32) {
        self.max_pivot = max_pivot.max(self.min_pivot);
        // Report the stored (clamped) limit, not the request (PIN-TPL-025).
        self.emit(ModuleEvent::Servo(ServoEvent::GetMaxPivot {
            max_pivot: self.max_pivot,
        }));
    }
}

impl<'d> Module for ServoModule<'d> {
    fn tick(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Servo(servo_command) => match servo_command {
                ServoCommandPayload::SetAngle { angle } => self.set_angle(*angle)?,
                ServoCommandPayload::SetMinPivot { min_pivot } => self.set_min_pivot(*min_pivot),
                ServoCommandPayload::SetMaxPivot { max_pivot } => self.set_max_pivot(*max_pivot),
            },
            _ => {
                // handle anything else
            }
        }
        Ok(())
    }
}

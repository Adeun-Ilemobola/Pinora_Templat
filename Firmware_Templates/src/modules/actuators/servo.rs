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
use crate::modules::actuators::servo_position::ServoPosition;
use crate::utilities::math::pulse_us_to_tick;

use pinora_protocol::{
    command::ModuleCommand, global_definitions::ModuleType, module_event::ModuleEvent,
};
use pwm_pca9685::Channel;

pub use pinora_protocol::module::servomodule::{ServoCapability, ServoCommandPayload, ServoEvent};

pub struct ServoModule<'d> {
    core: ModuleCore,
    pwm: SharedPwm<'d>,
    channel: Channel,
    position: ServoPosition,
}

/// Writes one pulse to a PCA9685 channel.
fn write_pulse(
    pwm: &SharedPwm<'_>,
    channel: Channel,
    name: &str,
    pulse: i32,
) -> anyhow::Result<()> {
    pwm.borrow_mut()
        .set_channel_on_off(channel, 0, pulse_us_to_tick(pulse))
        .map_err(|error| anyhow::anyhow!("Servo {name:?} PWM write failed: {error:?}"))
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
        let s = ServoModule {
            core: ModuleCore::new(ModuleType::Servo, &lookup_name, cluster_id, sender),
            pwm,
            channel,
            position: ServoPosition::new(&config),
        };

        Ok(s)
    }

    /// Drives the servo to its offset and through a short sweep (pivots 35°, 10°,
    /// -10°, -35°), then back to centre. **Moves hardware.** Never called
    /// automatically (it ran inside `new` before Phase 1D).
    pub fn run_self_test(&mut self) -> anyhow::Result<()> {
        self.set_offset(self.position.offset())?;
        self.set_angle(0)?;

        let self_test_angles: [i32; 4] = [35, 10, -10, -35];
        for f in self_test_angles {
            self.set_angle(f)?;
        }
        self.set_angle(0)
    }

    pub fn set_offset(&mut self, a: i32) -> anyhow::Result<()> {
        let (pwm, channel, name) = (&self.pwm, self.channel, &self.core.lookup_name);
        self.position
            .move_to_new_offset(a, |pulse| write_pulse(pwm, channel, name, pulse))?;
        self.emit(ModuleEvent::Servo(ServoEvent::GetOffset {
            angle: self.position.offset(),
        }));
        self.sync();
        Ok(())
    }

    /// Moves the servo and reports where it ended up.
    pub fn set_angle(&mut self, a: i32) -> anyhow::Result<()> {
        self.set_angle_silent(a)?;
        self.sync();
        Ok(())
    }

    /// Moves the servo without reporting it (a scan moves too often to report each step).
    pub fn set_angle_silent(&mut self, a: i32) -> anyhow::Result<()> {
        let (pwm, channel, name) = (&self.pwm, self.channel, &self.core.lookup_name);
        self.position
            .move_to(a, |pulse| write_pulse(pwm, channel, name, pulse))
    }

    /// Reports the current position; nothing is reported before the first move, because
    /// the real position is unknown until then.
    pub fn sync(&self) {
        if self.position.commanded() {
            self.emit(ModuleEvent::Servo(ServoEvent::GetAngle {
                angle: self.position.pivot(),
            }));
        }
    }

    pub fn pivot_angle(&self) -> i32 {
        self.position.pivot()
    }

    pub fn angle(&self) -> i32 {
        self.position.angle()
    }

    /// Stores the limit, and moves the servo back inside if it now sits outside.
    pub fn set_min_pivot(&mut self, min_pivot: i32) -> anyhow::Result<()> {
        let min_pivot = self.position.set_min_pivot(min_pivot);
        // Report the stored (clamped) limit, not the request (PIN-TPL-025).
        self.emit(ModuleEvent::Servo(ServoEvent::GetMinPivot { min_pivot }));
        self.return_inside_limits()
    }

    /// Stores the limit, and moves the servo back inside if it now sits outside.
    pub fn set_max_pivot(&mut self, max_pivot: i32) -> anyhow::Result<()> {
        let max_pivot = self.position.set_max_pivot(max_pivot);
        // Report the stored (clamped) limit, not the request (PIN-TPL-025).
        self.emit(ModuleEvent::Servo(ServoEvent::GetMaxPivot { max_pivot }));
        self.return_inside_limits()
    }

    fn return_inside_limits(&mut self) -> anyhow::Result<()> {
        if self.position.outside_limits() {
            self.set_angle(self.position.pivot())?;
        }
        Ok(())
    }
}

impl<'d> Module for ServoModule<'d> {
    fn tick(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }

    /// Replays the offset, limits and position (at boot and on rediscovery).
    fn first_emit(&mut self) -> Result<(), EmitterError> {
        let (min_pivot, max_pivot) = self.position.pivot_limits();
        self.emit(ModuleEvent::Servo(ServoEvent::GetOffset {
            angle: self.position.offset(),
        }));
        self.emit(ModuleEvent::Servo(ServoEvent::GetMinPivot { min_pivot }));
        self.emit(ModuleEvent::Servo(ServoEvent::GetMaxPivot { max_pivot }));
        self.sync();
        Ok(())
    }

    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Servo(servo_command) => match servo_command {
                ServoCommandPayload::SetAngle { angle } => self.set_angle(*angle)?,
                ServoCommandPayload::SetMinPivot { min_pivot } => self.set_min_pivot(*min_pivot)?,
                ServoCommandPayload::SetMaxPivot { max_pivot } => self.set_max_pivot(*max_pivot)?,
            },
            _ => {
                // handle anything else
            }
        }
        Ok(())
    }
}

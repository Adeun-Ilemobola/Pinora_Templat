//! PWM LED on one LEDC channel. `SetState { state }` maps 0–100 % onto the 13-bit
//! duty range; `Toggle` switches between 0 and 100.
//!
//! Requests above 100 are clamped to 100 and reported with a `SysLog` warning.
//! `Brightness` always reports the level actually applied (PIN-TPL-002, Phase 1B).
//! The arithmetic lives in [`super::led_mapping`] so it can be tested on the host.

use crate::core::hardware::{ledc, LedTimer, OutputPin};
use crate::core::module::{Module, ModuleCore, ModuleError};
use crate::core::transport::emitter::{EmitterError, TransportEmitter};
use crate::modules::actuators::led_mapping::{
    applied_level, clamp_warning, level_to_duty, toggled_level,
};
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::{LogPriority, ModuleEvent, SysLogEvent},
};

pub use pinora_protocol::module::ledmodule::{LedCommandPayload, LedEvent};
pub struct LedModule<'d> {
    core: ModuleCore,
    state: u32,
    pwm: ledc::LedcDriver<'d>,
}
impl<'d> LedModule<'d> {
    pub fn new<T, C>(
        pin: T,
        channel: C,
        lookup_name: String,
        timer: &LedTimer<'d>,
        cluster_id: Option<String>,
        sender: TransportEmitter,
    ) -> anyhow::Result<LedModule<'d>>
    where
        T: OutputPin + 'd,
        C: ledc::LedcChannel<SpeedMode = ledc::LowSpeed> + 'd,
    {
        let pwm = ledc::LedcDriver::new(channel, timer, pin)?;

        let ledmodule = LedModule {
            core: ModuleCore::new(ModuleType::Led, &lookup_name, cluster_id, sender),
            state: 0,
            pwm,
        };

        Ok(ledmodule)
    }

    /// Applies a brightness request. Out-of-range requests are clamped and reported.
    pub fn set_state(&mut self, state: u32) -> anyhow::Result<()> {
        let level = applied_level(state);
        if level != state {
            self.log(SysLogEvent {
                text: clamp_warning(state, level),
                raw_err: None,
                priority: LogPriority::Medium,
            });
        }
        self.pwm
            .set_duty(level_to_duty(level, self.pwm.get_max_duty()))?;
        self.state = level;
        self.emit(ModuleEvent::Led(LedEvent::Brightness { level }));

        Ok(())
    }
    pub fn get_state(&self) -> anyhow::Result<&u32> {
        Ok(&self.state)
    }

    pub fn toggle(&mut self) -> anyhow::Result<()> {
        self.set_state(toggled_level(self.state))
    }
}

impl<'d> Module for LedModule<'d> {
    /// Replays the current brightness (at boot and on rediscovery; Phase 2A).
    fn first_emit(&mut self) -> Result<(), EmitterError> {
        self.emit(ModuleEvent::Led(LedEvent::Brightness { level: self.state }));
        Ok(())
    }

    fn tick(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Led(led_command) => match led_command {
                LedCommandPayload::SetState { state } => self.set_state(*state)?,
                LedCommandPayload::Toggle {} => self.toggle()?,
            },
            _ => {
                // handle anything else
            }
        }
        Ok(())
    }
}

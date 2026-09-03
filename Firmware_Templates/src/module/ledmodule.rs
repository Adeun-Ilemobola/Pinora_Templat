
use crate::core::emitter::Emitter;
use crate::core::hardware::{ledc, LedTimer, OutputPin};
use crate::core::modulecore::{Module, ModuleCore};
use crate::utilities::math::range_u32;
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::ModuleEvent,
    registration::Registration,
};

pub use pinora_protocol::module::ledmodule::{LedCommandPayload, LedEvent};
pub struct Ledmodule<'d> {
    core: ModuleCore,
    state: u32,
    pwm: ledc::LedcDriver<'d>,
}
impl<'d> Ledmodule<'d> {
    pub fn new<T, C>(
        pin: T,
        channel: C,
        manuel_id: String,
        timer: &LedTimer<'d>,
        cluster_id: Option<String>,
        sender:Emitter
    ) -> anyhow::Result<Ledmodule<'d>>
    where
        T: OutputPin + 'd,
        C: ledc::LedcChannel<SpeedMode = ledc::LowSpeed> + 'd,
    {
        let pwm = ledc::LedcDriver::new(channel, timer, pin)?;

        let  ledmodule = Ledmodule {
            core: ModuleCore::new(ModuleType::Led, &manuel_id , sender),
            state: 0,
            pwm,
        };
      
      ledmodule.registration(Registration{
        id:ledmodule.id().to_string(),
         module_type:ModuleType::Led,
         lool_up_id:manuel_id.clone(),
         parent_id: cluster_id.clone().unwrap_or_default()

      });


        Ok(ledmodule)
    }

    pub fn set_state(&mut self, state: u32) -> anyhow::Result<()> {
        let p = range_u32(state, 0, 100, 0, self.pwm.get_max_duty());
        self.pwm.set_duty(p)?;
        self.state = state;
        self.emit(ModuleEvent::Led(LedEvent::Brightness { id:self.id().to_string(), level: state }));
    

        Ok(())
    }
    pub fn get_state(&self) -> anyhow::Result<&u32> {
        Ok(&self.state)
    }

    pub fn toggle(&mut self) -> anyhow::Result<()> {
        if self.state == 0 {
            self.set_state(100)?;
        } else {
            self.set_state(0)?;
        }

        Ok(())
    }
}

impl<'d> Module for Ledmodule<'d> {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Led(led_command) => match led_command {
                LedCommandPayload::SetState { state } => self.set_state(*state)?,
                LedCommandPayload::Toggle => self.toggle()?,
            },
            _ => {
                // handle anything else
            }
        }
        Ok(())
    }
}

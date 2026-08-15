use std::time::{Duration, Instant};

use esp_idf_svc::hal::gpio::Level;

use crate::{
    core::{
        emitter::Emitter,
        hardware::{InputPinCore, TimerState},
        modulecore::{Module, ModuleCore},
    },
    protocol::{command::ModuleCommand, global_definitions::ModuleType},
};

pub struct RemoteReceiver<'d> {
    core: ModuleCore,
    state: Level,
    step_timer: TimerState,
    pin_driver: InputPinCore<'d>,
    test_time: Instant,
    buffer:Vec<(Level, Duration)>
}

impl<'d> RemoteReceiver<'d> {
    pub fn new(
        pin: InputPinCore<'d>,
        core_id: String,
        sender: Emitter,
    ) -> Result<RemoteReceiver<'d>, ()> {
        let r = RemoteReceiver {
            core: ModuleCore::new(ModuleType::RemoteReceiver, &core_id, sender),
            pin_driver: pin,
            state: Level::Low,
            test_time: Instant::now(),
            buffer:vec![],
            step_timer: TimerState::from_ms(230.6)
        };
        Ok(r)
    }
}

impl<'d> Module for RemoteReceiver<'d> {
    fn tick(&mut self) -> Result<(), ()> {
        let data_now = self.pin_driver.now().unwrap();
        let previous_state = self.state;
        if data_now != self.state {
            let elapsed = self.test_time.elapsed();
            self.test_time = Instant::now();
            self.buffer.push((previous_state, elapsed));
            self.state = data_now;
        }
        if self.step_timer.ready() && !self.buffer.is_empty(){
            println!("-----------------Next-------------------");
            for (index , bit) in self.buffer.iter().enumerate()  {
                println!("[{}] | {:?} ----- {:?}" , index, bit.0 , bit.1)    
            }
            self.buffer.clear();
            self.step_timer.reset();
        }

        Ok(())
    }
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn get_module_type(&self) -> &ModuleType {
        &self.core.module_type
    }
    fn handle_command(&mut self, _command: &ModuleCommand) -> anyhow::Result<()> {
        Ok(())
    }
}


use crate::core::emitter::Emitter;
use crate::core::hardware::{InputPin, InputPinCore, Pull};
use crate::core::modulecore::{Module, ModuleCore};
use esp_idf_svc::hal::gpio::Level;
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::ModuleEvent,
    registration::Registration,
};

pub use pinora_protocol::module::buttonmodule::ButtonEvent;

static BUTTON_MODULE_MAX_TIME: u64 = 30; // Maximum time in milliseconds to consider a button press valid
pub struct Buttonmodule<'d> {
    core: ModuleCore,
    state: Level,      // debounced/committed level
    prev_state: Level, // previous committed level, for edge detection
    pin_driver: InputPinCore<'d>,
    last_state: Level,
    last_change_time: std::time::Instant,
}

impl<'d> Buttonmodule<'d> {
    pub fn new<T>(pin: T , lool_up_id:String , sender:Emitter) -> anyhow::Result<Buttonmodule<'d>>
    where
        T: InputPin + 'd,
    {
        let  buttonmodule = Buttonmodule {
            core: ModuleCore::new(ModuleType::Button, &lool_up_id , sender),
            state: Level::High,
            pin_driver: InputPinCore::new(pin, Pull::Up)?,
            last_state: Level::High,
            last_change_time: std::time::Instant::now(),
            prev_state: Level::High,
        };

       buttonmodule.registration(Registration{
        id:buttonmodule.id().to_string(),
        lool_up_id :lool_up_id.clone(),
        module_type:ModuleType::Button,
        parent_id:String::new()
       });
       

        Ok(buttonmodule)
    }
    pub fn update_state(&mut self) -> anyhow::Result<()> {
        let current_state = self.pin_driver.now()?;
        println!("GPIO32: {:?}", current_state);
        let now = std::time::Instant::now();

        if current_state != self.last_state {
            self.last_state = current_state;
            self.last_change_time = now;
        }
        if now.duration_since(self.last_change_time).as_millis() >= BUTTON_MODULE_MAX_TIME as u128 {
            self.state = self.last_state;
        }
        Ok(())
    }
    pub fn poll(&mut self) -> anyhow::Result<bool> {
        self.update_state()?;

        if self.state != self.prev_state {
            let pressed = self.state == Level::Low;
            self.prev_state = self.state;
            self.emit(ModuleEvent::Button(ButtonEvent::Ckick{ id:self.id().to_string()}));
            return Ok(pressed);
        }
        Ok(false)
    }
    
}
impl<'d> Module for Buttonmodule<'d> {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn get_module_type(&self) -> &ModuleType {
        &self.core.module_type
    }
    fn handle_command(
        &mut self,
        _command: &ModuleCommand,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    // fn serialize(&self) -> anyhow::Result<()> {
    //     serde_json::to_string(&self.get_event()?)
    //         .map(|s| println!("{}", s))
    //         .unwrap_or_else(|e| println!("Failed to serialize JSON: {}", e));

    //     Ok(())
    // }
}

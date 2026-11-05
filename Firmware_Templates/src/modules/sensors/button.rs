use crate::core::hardware::{InputPin, InputPinCore, Pull};
use crate::core::module::{Module, ModuleCore, ModuleError};
use crate::core::transport::emitter::TransportEmitter;
use esp_idf_svc::hal::gpio::Level;
use pinora_protocol::{
    command::ModuleCommand, global_definitions::ModuleType, module_event::ModuleEvent,
};

pub use pinora_protocol::module::buttonmodule::ButtonEvent;

static BUTTON_MODULE_MAX_TIME: u64 = 30; // Maximum time in milliseconds to consider a button press valid
pub struct ButtonModule<'d> {
    core: ModuleCore,
    state: Level,      // debounced/committed level
    prev_state: Level, // previous committed level, for edge detection
    pin_driver: InputPinCore<'d>,
    last_state: Level,
    last_change_time: std::time::Instant,
}

impl<'d> ButtonModule<'d> {
    pub fn new<T>(
        pin: T,
        lookup_name: String,
        sender: TransportEmitter,
    ) -> anyhow::Result<ButtonModule<'d>>
    where
        T: InputPin + 'd,
    {
        let pin_driver = InputPinCore::new(pin, Pull::Up)?;
        // Start from the real level, so a button held at boot is not a click.
        let level = pin_driver.now()?;
        let buttonmodule = ButtonModule {
            core: ModuleCore::new(ModuleType::Button, &lookup_name, None, sender),
            state: level,
            pin_driver,
            last_state: level,
            last_change_time: std::time::Instant::now(),
            prev_state: level,
        };

        Ok(buttonmodule)
    }
    pub fn update_state(&mut self) -> anyhow::Result<()> {
        let current_state = self.pin_driver.now()?;
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
            // One click per press: releasing the button is not another click.
            if pressed {
                self.emit(ModuleEvent::Button(ButtonEvent::Ckick {}));
            }
            return Ok(pressed);
        }
        Ok(false)
    }
}
impl<'d> Module for ButtonModule<'d> {
    fn tick(&mut self) -> Result<(), ModuleError> {
        self.poll()
            .map(|_| ())
            .map_err(|_| ModuleError::OperationFailed)
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, _command: &ModuleCommand) -> anyhow::Result<()> {
        Ok(())
    }

    // fn serialize(&self) -> anyhow::Result<()> {
    //     serde_json::to_string(&self.get_event()?)
    //         .map(|s| println!("{}", s))
    //         .unwrap_or_else(|e| println!("Failed to serialize JSON: {}", e));

    //     Ok(())
    // }
}

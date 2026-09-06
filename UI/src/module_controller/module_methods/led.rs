use pinora_protocol::LedEvent;

use crate::{LedState, Unsigned32};

impl LedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: LedEvent) {
        match event {
            LedEvent::Brightness { id, level } => {
                self.id = id.into();
                self.brightness = level.into();
            }
        }
    }

    pub fn set_brightness(&mut self, level: Unsigned32) {
        self.brightness = level.into();
    }
}

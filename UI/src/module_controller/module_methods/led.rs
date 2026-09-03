use pinora_protocol::LedEvent;

use crate::LedState;

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
}

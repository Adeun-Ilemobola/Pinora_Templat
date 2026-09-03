use pinora_protocol::ButtonEvent;

use crate::ButtonState;

impl ButtonState {
    pub fn new() -> Self {
        Self {
            id: String::new().into(),
        }
    }

    pub fn update(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::Ckick { id } => self.id = id.into(),
        }
    }
}

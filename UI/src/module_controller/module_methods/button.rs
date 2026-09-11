use pinora_protocol::ButtonEvent;

use crate::ButtonState;

impl ButtonState {
    pub fn new() -> Self {
        Self {
            id: String::new().into(),
        }
    }

    pub fn update(&mut self, id: &str, event: ButtonEvent) {
        self.id = id.into();
        match event {
            ButtonEvent::Ckick {} => {},
        }
    }
}

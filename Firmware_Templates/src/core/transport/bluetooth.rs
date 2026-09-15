
use pinora_protocol::registration::ProtocolMessage;

#[derive(Debug, Clone)]
pub struct Bluetooth {
}

impl Bluetooth {
    pub fn new() -> Self {
        Bluetooth {}
    }

    pub fn event(&self, event: ProtocolMessage) {
        // Handle Bluetooth event
    }
}
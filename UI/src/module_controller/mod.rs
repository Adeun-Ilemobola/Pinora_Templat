use pinora_protocol::ProtocolMessage;

pub struct ModuleController {}

impl ModuleController {
    pub fn new() -> Self {
        ModuleController {}
    }

    pub fn incoming_event(&mut self, data: Vec<u8>) {
        let Ok(message) = serde_json::from_slice::<ProtocolMessage>(&data) else {
             println!("Ignoring non-protocol data: {}", String::from_utf8_lossy(&data));
            return;
        };

        match message {
            ProtocolMessage::ModuleEvent(state) => {
                println!("state Change")
            }

            ProtocolMessage::Registration(register) => {
                println!("register Item")
            }

            ProtocolMessage::System(sys) => {
                println!("System Info")
            }
            _ => {}
        }
    }
}

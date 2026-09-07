use std::sync::Arc;
use pinora_protocol::{IncomingCommand, ModuleCommand};
pub type EventCallback = Arc<dyn Fn(Vec<u8>) + Send + Sync + 'static>;

pub type CommandsEventCallback = Arc<dyn Fn(IncomingCommand) + Send + Sync + 'static>;

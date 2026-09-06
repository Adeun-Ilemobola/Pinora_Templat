use std::sync::Arc;

pub type EventCallback = Arc<dyn Fn(Vec<u8>) + Send + Sync + 'static>;

pub type CommandsEventCallback = Arc<dyn Fn(String) + Send + Sync + 'static>;

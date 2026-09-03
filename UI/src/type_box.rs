use std::sync::Arc;

pub type EventCallback = Arc<dyn Fn(Vec<u8>) + Send + Sync + 'static>;

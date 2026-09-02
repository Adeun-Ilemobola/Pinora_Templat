use std::sync::{Arc, Mutex};

pub type EventCallback = Arc<Mutex<Box<dyn FnMut(Vec<u8>) + Send + 'static>>>;
pub type Callback<T> =
    Arc<Mutex<Box<dyn FnMut(T) + Send + 'static>>>;
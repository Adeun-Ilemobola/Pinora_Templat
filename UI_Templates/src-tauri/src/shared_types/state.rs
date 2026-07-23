use serialport::SerialPort;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::JoinHandle;
pub struct SerialRuntime {
    
    // pub port_name: Option<String>,
    // pub baud_rate: u32,
    pub port: Box<dyn SerialPort>,
    pub worker: Option<JoinHandle<()>>,
    pub stop_flag: Arc<AtomicBool>,
}

impl SerialRuntime {
    pub fn new(
        // port_name: String,
        // baud_rate: u32,
        port: Box<dyn SerialPort>,
        worker: JoinHandle<()>,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            // port_name: Some(port_name),
            // baud_rate,
            port,
            worker: Some(worker),
            stop_flag,
        }
    }

    pub fn stop(self) {
        let SerialRuntime { port, worker, stop_flag, .. } = self;
        stop_flag.store(true, Ordering::Relaxed);
        drop(port);
        if let Some(handle) = worker {
            let _ = handle.join();
        }
    }
}




// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod transport;
pub mod ui_bridge;
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::transport::transport_type::{BaudRate, TransportType};
use crate::transport::{
    serial_transport::SerialTransport, transport_gate::Transport, transport_type::to_slint_model,
};
use slint::{SharedString, VecModel};
use std::rc::Rc;
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
   
    let shared = Arc::new(Mutex::new(Transport::new()));
    let transport_gateway: Arc<Mutex<Transport>> = Arc::clone(&shared);


    ui_bridge::transport_form::bind(&ui, Arc::clone(&transport_gateway));
    ui.run()?;

    Ok(())
}

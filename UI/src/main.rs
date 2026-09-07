// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod module_controller;
pub mod transport;
pub mod type_box;
pub mod ui_bridge;
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::module_controller::ModuleController;
use crate::transport::transport_gate::Transport;
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let transport_gateway = Arc::new(Mutex::new(Transport::new()));
      let transport = Arc::clone(&transport_gateway);
    let controller = Arc::new(Mutex::new(
        ModuleController::new(
            &ui,
            Arc::new(move |command| {
               let mut transport = transport.lock().unwrap();
               let _ = transport.send_command(command.clone()).map_err(|e| println!("Failed to send command: {:?}", e));
               
              
            }),
        )
    ));
    let event_callback: type_box::EventCallback = Arc::new(move |data| {
        let mut controller = controller.lock().unwrap();

        controller.incoming_event(data);
    });

    ui_bridge::transport_form::bind(&ui, Arc::clone(&transport_gateway), event_callback);
    ui.run()?;

    Ok(())
}

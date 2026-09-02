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

    let shared = Arc::new(Mutex::new(Transport::new()));
    let transport_gateway: Arc<Mutex<Transport>> = Arc::clone(&shared);

    let module_controller_core = Arc::new(Mutex::new(ModuleController::new(
        Arc::clone(&transport_gateway),
        &ui,
    )));
    let mc_gate: Arc<Mutex<ModuleController>> = Arc::clone(&module_controller_core);

    ui_bridge::transport_form::bind(
        &ui,
        Arc::clone(&transport_gateway),
        Box::new(move |data| {
            let mut controller = mc_gate.lock().unwrap();

            controller.incoming_event(data);
        }),
    );
    ui.run()?;

    Ok(())
}

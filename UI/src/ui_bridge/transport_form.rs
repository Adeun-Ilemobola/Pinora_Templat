use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

use crate::{
    AppWindow, ConnectionTypeS,
    transport::{
        serial_transport::SerialTransport,
        transport_gate::Transport,
        transport_type::{BaudRate, TransportType, to_slint_model},
    },
    type_box::EventCallback,
};

pub fn disconnect(ui: slint::Weak<AppWindow>, transport: Arc<Mutex<Transport>>) {
    let ui_core = ui.unwrap();
    let mut current_transport = transport.lock().unwrap();
    match current_transport.disconnect() {
        Ok(_) => {
            ui_core.set_Connection_mode(ConnectionTypeS::Disconnected);
        }
        Err(r) => {
            println!("{:?}", r);
            ui_core.set_Connection_mode(ConnectionTypeS::Error);
        }
    }

    println!(" try to disconnect")
}

pub fn connection(
    ui: slint::Weak<AppWindow>,
    transport: Arc<Mutex<Transport>>,
    event_callback: EventCallback,
) {
    let ui_core = ui.unwrap();
    let mut bri = transport.lock().unwrap();
    let selected_transport = ui_core.get_selected_transport();
    let transport_type = TransportType::to_self(selected_transport);
    let port = ui_core.get_port().to_string();
    let baudrate = ui_core.get_baudrate();
    let wifi_ssid = ui_core.get_wifi_ssid().to_string();
    let wifi_password = ui_core.get_wifi_password().to_string();
    let bluetooth_name = ui_core.get_bluetooth_name().to_string();

    println!("Selected transport type: {:?}", transport_type);

    match transport_type {
        TransportType::Serial => {
            // Handle serial transport logic
            let new_baud_rate = BaudRate::from_str(baudrate.as_str());
            println!("Port: {}", port);
            println!("Baudrate: {:?}", new_baud_rate);

            let statse =
                bri.set_serial_transport(port, new_baud_rate.as_u32(), Arc::clone(&event_callback));
            ui_core.set_Connection_mode(statse.to_slint());
        }
        TransportType::Wifi => {
            // Handle WiFi transport logic
            println!("WiFi SSID: {}", wifi_ssid);
            println!("WiFi Password: {}", wifi_password);
        }
        TransportType::Bluetooth => {
            // Handle Bluetooth transport logic
            println!("Bluetooth Name: {}", bluetooth_name);
        }
        TransportType::None => {
            // Handle None transport logic
        }
    }
}

pub fn bind(
    ui: &AppWindow,
    gateway: Arc<Mutex<Transport>>,
    event_callback: Box<dyn FnMut(Vec<u8>) + Send + 'static>,
) {
    let serialports = SerialTransport::get_available_ports().unwrap();
    let initial_port = serialports.first().cloned().unwrap_or_default();

    let event_callback = EventCallback::new(Mutex::new(event_callback));

    let model_ports = to_slint_model(serialports);

    let transport_types = TransportType::to_slint_model();

    ui.set_combo_transport_model(transport_types.into());

    ui.set_selected_transport(TransportType::Serial.format());

    ui.set_ports(model_ports);
    ui.set_port(initial_port.into());

    ui.set_baudrates(BaudRate::to_slint_model());
    ui.set_baudrate(BaudRate::B9600.as_str().into());

    // Weak UI handles for each callback
    let disconnect_ui = ui.as_weak();
    let connection_ui = ui.as_weak();

    // Shared Transport handles for each callback
    let disconnect_gateway = Arc::clone(&gateway);
    let connection_gateway = Arc::clone(&gateway);

    ui.on_request_Make_Disconnect(move || {
        disconnect(disconnect_ui.clone(), Arc::clone(&disconnect_gateway))
    });

    ui.on_request_Make_Connection(move || {
        connection(
            connection_ui.clone(),
            Arc::clone(&connection_gateway),
            Arc::clone(&event_callback),
        )
    });
}

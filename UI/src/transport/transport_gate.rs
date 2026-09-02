use crate::{
    transport::{
        bluetooth_transport::BluetoothTransport,
        serial_transport::SerialTransport,
        transport_type::{ConnectionState, ConnectionType, TransportError, TransportType},
        wifi_transport::WifiTransport,
    },
    type_box::EventCallback,
};

enum TransportCore {
    Serial(SerialTransport),
    Wifi(WifiTransport),
    Bluetooth(BluetoothTransport),
}

pub struct Transport {
    core: Option<TransportCore>,
}

impl Transport {
    pub fn new() -> Self {
        Transport { core: None }
    }

    pub fn set_serial_transport(
        &mut self,
        name: String,
        rate: u32,
        event_callback: EventCallback,
    ) -> ConnectionType {
        let serial_transport = SerialTransport::new(event_callback);
        self.core = Some(TransportCore::Serial(serial_transport));
        match self.core {
            Some(TransportCore::Serial(ref mut serial_transport)) => {
                serial_transport.set_port(name, rate);

                match self.connect() {
                    Ok(connection_state) => {
                        if connection_state.connection_type == ConnectionType::Connected {
                            println!("Serial transport connected successfully.");
                            return ConnectionType::Connected;
                        } else {
                            println!(
                                "Serial transport connection state: {:?}",
                                connection_state.connection_type
                            );
                            return ConnectionType::Error;
                        }
                    }
                    Err(e) => {
                        println!("Failed to connect serial transport: {:?}", e);
                        return ConnectionType::Error;
                    }
                }
            }
            _ => {
                return ConnectionType::Error;
            }
        }
    }

    pub fn set_wifi_transport(&mut self, transport: WifiTransport) {
        self.core = Some(TransportCore::Wifi(transport));
    }

    pub fn set_bluetooth_transport(&mut self, transport: BluetoothTransport) {
        self.core = Some(TransportCore::Bluetooth(transport));
    }

    pub fn connect(&mut self) -> Result<ConnectionState, TransportError> {
        match &mut self.core {
            Some(TransportCore::Serial(serial_transport)) => serial_transport.connect(),
            Some(TransportCore::Wifi(wifi_transport)) => wifi_transport.connect(),
            Some(TransportCore::Bluetooth(bluetooth_transport)) => bluetooth_transport.connect(),
            None => Err(TransportError::ConnectionFailed {
                message: "No transport set".to_string(),
                raw_error: None,
            }),
        }
    }

    pub fn disconnect(&mut self) -> Result<(), TransportError> {
        match &mut self.core {
            Some(TransportCore::Serial(serial_transport)) => serial_transport.disconnect(),
            Some(TransportCore::Wifi(wifi_transport)) => wifi_transport.disconnect(),
            Some(TransportCore::Bluetooth(bluetooth_transport)) => bluetooth_transport.disconnect(),
            None => Err(TransportError::ConnectionFailed {
                message: "No transport set".to_string(),
                raw_error: None,
            }),
        }
    }
    pub fn get_type(&self) -> TransportType {
        match &self.core {
            Some(TransportCore::Serial(_)) => TransportType::Serial,
            Some(TransportCore::Wifi(_)) => TransportType::Wifi,
            Some(TransportCore::Bluetooth(_)) => TransportType::Bluetooth,
            None => TransportType::None,
        }
    }
}

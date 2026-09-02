use crate::transport::transport_type::{
    ConnectionState, ConnectionType, TransportError, TransportType,
};

pub struct WifiTransport {
    ssid: String,
    password: String,
}

impl WifiTransport {
    pub fn new(ssid: String, password: String, _event_callback: Box<dyn FnMut()>) -> Self {
        WifiTransport { ssid, password }
    }

    pub fn get_ssid(&self) -> &str {
        &self.ssid
    }

    pub fn get_password(&self) -> &str {
        &self.password
    }

    pub fn disconnect(&self) -> Result<(), TransportError> {
        // Implement the disconnection logic here
        Ok(())
    }
    pub fn connect(&self) -> Result<ConnectionState, TransportError> {
        // Implement the connection logic here
        Ok(ConnectionState {
            connection_type: ConnectionType::Disconnected,
            transport_type: Some(TransportType::Wifi),
            error: None,
        })
    }
}

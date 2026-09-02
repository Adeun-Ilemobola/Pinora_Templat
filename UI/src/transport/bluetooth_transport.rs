use crate::transport::transport_type::{ConnectionState, ConnectionType, TransportError, TransportType};



pub struct BluetoothTransport {
    device_name: String,
    device_address: String,
}

impl BluetoothTransport {
    pub fn new(device_name: String, device_address: String , eventCallback: Box<dyn FnMut() + Send + 'static>) -> Self {
        BluetoothTransport { device_name, device_address }
    }

    pub fn get_device_name(&self) -> &str {
        &self.device_name
    }

    pub fn get_device_address(&self) -> &str {
        &self.device_address
    }
 pub fn disconnect(&self) -> Result<(), TransportError> {
        // Implement the disconnection logic here
        Ok(())
    }
     pub fn connect(&self) -> Result<ConnectionState, TransportError> {
        // Implement the connection logic here
        Ok(ConnectionState{
            connection_type: ConnectionType::Disconnected,
            transport_type: Some(TransportType::Bluetooth),
            error: None,
        })
    }
}
//! Development Bluetooth Classic SPP settings shared by firmware and desktop.

pub const DEVICE_NAME: &str = "Pinora Dev ESP32";
pub const SERVICE_NAME: &str = "Pinora SPP";
pub const MAX_MESSAGE_BYTES: usize = 8192;
pub const SPP_WRITE_CHUNK_BYTES: usize = 512;

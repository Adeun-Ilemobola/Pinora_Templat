//! Device ↔ desktop transport layer.
//!
//! One transport is selected per firmware image (`TransportType` in `main.rs`).
//! Outbound messages flow `TransportEmitter` → sender thread (`manager`) → the
//! selected transport. Inbound commands arrive on an `mpsc` channel that the main
//! loop drains.

pub mod auth_gate;
pub mod bluetooth;
pub mod console;
pub mod device_key;
pub mod discovery_cache;
pub mod drop_stats;
pub mod emitter;
pub mod manager;
pub mod pairing_window;
pub mod serial;
pub mod wifi;
//pub mod zigbee;

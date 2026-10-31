//! Device ↔ desktop transport layer.
//!
//! One transport is selected per firmware image (`TransportType` in `main.rs`).
//! Outbound messages flow `TransportEmitter` → sender thread (`manager`) → the
//! selected transport. Inbound commands arrive on an `mpsc` channel that the main
//! loop drains.

pub mod bluetooth;
pub mod discovery_cache;
pub mod emitter;
pub mod manager;
pub mod serial;
pub mod wifi;
//pub mod zigbee;

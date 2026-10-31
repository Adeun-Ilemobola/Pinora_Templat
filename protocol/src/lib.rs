//! Pinora wire protocol, shared by the firmware and the desktop backend.
//!
//! Every message is one JSON document per line. Firmware → desktop sends
//! [`ProtocolMessage`]; desktop → firmware sends [`IncomingCommand`]
//! (`{"id": "<module uuid>", "command": {"<Family>": {...}}}`).
//!
//! The JSON shape is a compatibility contract (audit contract C-01), locked by
//! `tests/wire_format.rs`. These quirks must not be "fixed" without a protocol
//! version: enums are externally tagged except `ImuEvent` (internally tagged by
//! `event_type`); unit variants such as `RangefinderCommandPayload::StartRanging`
//! serialize as bare strings; the misspelled names `lool_up_id`, `Ckick`, `Idol` and
//! `distant` are on the wire; `Registration::parent_id` is `""` for top-level modules.
//!
//! `wifi_config` is a local, git-ignored file (copy `wifi_config.example.rs`).

pub mod bluetooth_config;
pub mod command;
pub mod global_definitions;
pub mod module;
pub mod module_event;
pub mod registration;
pub mod wifi_config;

pub use command::*;
pub use global_definitions::*;
pub use module::*;
pub use module_event::*;
pub use registration::*;

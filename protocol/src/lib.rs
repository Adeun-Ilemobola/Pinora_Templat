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
//! `wifi_config` comes from the local, git-ignored `src/wifi_config.rs`, or from the
//! placeholders in `wifi_config.example.rs` when that file is absent (`build.rs`,
//! audit Phase 2E).
//!
//! Protocol v2 (audit Phase 2B): [`framing`] is the one line framer and limit table
//! for every transport; [`auth`] holds the authentication wire types and, with the
//! `auth` feature, the HMAC session state machines.

pub mod auth;
pub mod bluetooth_config;
pub mod command;
pub mod framing;
pub mod global_definitions;
pub mod module;
pub mod module_event;
pub mod registration;
/// Local Wi-Fi settings (see `build.rs`).
pub mod wifi_config {
    include!(concat!(env!("OUT_DIR"), "/wifi_config.rs"));
}

pub use auth::{AuthMessage, CommandLine, SignedCommand, parse_command_line};
pub use command::*;
pub use global_definitions::*;
pub use module::*;
pub use module_event::*;
pub use registration::*;

//! What a transport does with one received command line (audit Phase 2B,
//! PIN-TPL-003).
//!
//! * Radio transports (Bluetooth, Wi-Fi) pass every line through
//!   [`process_line`]: the per-connection [`DeviceSession`] from the protocol crate
//!   decides, and this module turns the decision into actions (deliver to the main
//!   loop, or send a message back to that peer only).
//! * The USB serial console routes its lines with [`route_console_line`]: key
//!   provisioning is accepted only there, because USB means physical access.
//!
//! Pure (no ESP-IDF), so it is tested on the host together with the desktop side of
//! the handshake.

use pinora_protocol::auth::{is_provisioning_command, Decision, DeviceSession, Key, Nonce};
use pinora_protocol::{
    parse_command_line, CommandLine, EventPackage, IncomingCommand, LogPriority, ModuleCommand,
    ModuleEvent, ProtocolMessage, SysLogEvent, SystemCommand,
};

/// Sent once per connection to a device without a key.
pub const UNPROVISIONED_WARNING: &str = "This device has no key, so commands are not authenticated. Pair it over USB to require authentication.";

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Hand the command to the main loop.
    Deliver(IncomingCommand),
    /// Send this message to the connected peer only (never cached or replayed).
    Send(ProtocolMessage),
}

/// Parses one line and applies the session's decision.
pub fn process_line(
    session: &mut DeviceSession,
    line: &[u8],
    key: Option<&Key>,
    fresh_nonce: impl FnOnce() -> Nonce,
) -> Result<Vec<Action>, serde_json::Error> {
    let parsed = parse_command_line(line)?;
    Ok(match session.handle(parsed, key, fresh_nonce) {
        Decision::Forward(command) => vec![Action::Deliver(command)],
        Decision::ForwardUnprovisioned(command) => vec![
            Action::Send(peer_log(UNPROVISIONED_WARNING, LogPriority::Medium)),
            Action::Deliver(command),
        ],
        Decision::Reply(message) => vec![Action::Send(ProtocolMessage::Auth(message))],
        Decision::Reject(reason) => vec![Action::Send(peer_log(&reason, LogPriority::High))],
    })
}

/// A SysLog from the reserved source `"log"` (contract C-05).
pub fn peer_log(text: &str, priority: LogPriority) -> ProtocolMessage {
    ProtocolMessage::ModuleEvent(EventPackage {
        id: "log".to_string(),
        event: ModuleEvent::SysLog(SysLogEvent {
            text: text.to_string(),
            raw_err: None,
            priority,
        }),
    })
}

/// How the USB serial console handles one line.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleRoute {
    ProvisionKey,
    ClearKey,
    /// Serial image only: the console is also the command transport.
    Forward(IncomingCommand),
    Ignore(&'static str),
}

/// Routes a console line. `forwards` is true in the Serial image, where USB is the
/// command transport and is trusted without a handshake (physical access).
pub fn route_console_line(line: &[u8], forwards: bool) -> Result<ConsoleRoute, serde_json::Error> {
    Ok(match parse_command_line(line)? {
        CommandLine::Plain(command) if is_provisioning_command(&command) => match command.command {
            ModuleCommand::System(SystemCommand::ProvisionKey {}) => ConsoleRoute::ProvisionKey,
            _ => ConsoleRoute::ClearKey,
        },
        CommandLine::Plain(command) if forwards => ConsoleRoute::Forward(command),
        CommandLine::Plain(_) => {
            ConsoleRoute::Ignore("this image takes commands over its radio transport, not USB")
        }
        CommandLine::Signed(_) => ConsoleRoute::Ignore("signed commands are not used over USB"),
    })
}

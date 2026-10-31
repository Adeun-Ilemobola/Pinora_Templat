//! Reconnect replay cache shared by the Bluetooth and Wi-Fi transports.
//!
//! The latest `System` snapshot and one `Registration` per module UUID are kept so a
//! desktop that (re)connects can rediscover the modules. Runtime module events are
//! deliberately not cached, so current values are not replayed (PIN-TPL-012).

use pinora_protocol::ProtocolMessage;

/// Records discovery messages and ignores module events. A registration whose module
/// id is already stored replaces it in place, so first-seen order is kept.
pub fn remember(
    message: &ProtocolMessage,
    system: &mut Option<ProtocolMessage>,
    registrations: &mut Vec<ProtocolMessage>,
) {
    match message {
        ProtocolMessage::System(_) => *system = Some(message.clone()),
        ProtocolMessage::Registration(registration) => {
            if let Some(saved) = registrations.iter_mut().find(|saved| {
                matches!(saved, ProtocolMessage::Registration(item) if item.id == registration.id)
            }) {
                *saved = message.clone();
            } else {
                registrations.push(message.clone());
            }
        }
        ProtocolMessage::ModuleEvent(_) => {}
    }
}

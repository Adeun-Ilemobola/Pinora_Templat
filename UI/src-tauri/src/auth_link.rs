//! Authentication of one desktop ↔ device link (audit Phase 2B, PIN-TPL-003).
//!
//! The handshake runs here, in the backend, so device keys never reach the WebView:
//! * Bluetooth and Wi-Fi links start a handshake ([`AuthLink::begin`]) and, once
//!   authenticated, sign every command from the WebView ([`AuthLink::outgoing`]).
//! * A USB serial link never authenticates (the device trusts USB); it is the only
//!   link that may pair (`ProvisionKey`) or unpair (`ClearKey`), and the only link
//!   whose `Auth(Key)` replies are stored.
//!
//! Pure: no I/O, so it is unit-tested.

use pinora_protocol::auth::{AuthStatus, ClientSession, NONCE_BYTES};
use pinora_protocol::{
    AuthMessage, IncomingCommand, ModuleCommand, SystemCommand, SYSTEM_COMMAND_ID,
};
use serde::Serialize;

use crate::device_keys::DeviceKeys;

/// A fresh handshake nonce from the operating system's RNG.
pub(crate) fn random_nonce() -> Result<[u8; NONCE_BYTES], String> {
    let mut nonce = [0u8; NONCE_BYTES];
    getrandom::getrandom(&mut nonce).map_err(|error| error.to_string())?;
    Ok(nonce)
}

/// Tauri event carrying [`AuthStatus`] and [`PairingEvent`] payloads (contract C-07,
/// added in Phase 2B).
pub(crate) const AUTH_EVENT: &str = "deviceAuth";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkKind {
    UsbSerial,
    Bluetooth,
    Wifi,
}

/// Outcome of a pairing request over USB. Serialized like [`AuthStatus`] (`state`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state")]
pub(crate) enum PairingEvent {
    Paired { device_id: String },
    KeyCleared { device_id: String },
    PairingFailed { reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Outcome {
    /// Write this line (with its `\n`) to the device.
    Reply(String),
    /// Emit as `deviceAuth`.
    Status(AuthStatus),
    /// Emit as `deviceAuth`.
    Pairing(PairingEvent),
}

#[derive(Debug)]
pub(crate) struct AuthLink {
    kind: LinkKind,
    session: ClientSession,
}

fn line_of<T: Serialize>(value: &T) -> String {
    let mut line = serde_json::to_string(value).expect("protocol types serialize");
    line.push('\n');
    line
}

fn system(command: SystemCommand) -> IncomingCommand {
    IncomingCommand {
        id: SYSTEM_COMMAND_ID.to_string(),
        command: ModuleCommand::System(command),
    }
}

impl AuthLink {
    pub(crate) fn new(kind: LinkKind) -> Self {
        Self {
            kind,
            session: ClientSession::new(),
        }
    }

    pub(crate) fn status(&self) -> AuthStatus {
        self.session.status()
    }

    /// The `AuthBegin` line for a radio link; `None` for USB serial.
    pub(crate) fn begin(&mut self, nonce: [u8; NONCE_BYTES]) -> Option<String> {
        (self.kind != LinkKind::UsbSerial).then(|| line_of(&self.session.begin(nonce)))
    }

    /// The pairing request lines; USB serial links only.
    pub(crate) fn provision_line(&self) -> Result<String, String> {
        self.require_usb()?;
        Ok(line_of(&system(SystemCommand::ProvisionKey {})))
    }

    pub(crate) fn clear_line(&self) -> Result<String, String> {
        self.require_usb()?;
        Ok(line_of(&system(SystemCommand::ClearKey {})))
    }

    fn require_usb(&self) -> Result<(), String> {
        if self.kind == LinkKind::UsbSerial {
            Ok(())
        } else {
            Err("Pairing is possible only over a USB serial connection".into())
        }
    }

    /// Handles one `Auth` message from the device.
    pub(crate) fn on_auth_message(
        &mut self,
        message: AuthMessage,
        keys: &mut DeviceKeys,
    ) -> Vec<Outcome> {
        match (&message, self.kind) {
            (AuthMessage::Key { device_id, key }, LinkKind::UsbSerial) => {
                let stored = pinora_protocol::auth::Key::from_hex(key)
                    .ok_or_else(|| "the device sent a malformed key".to_string())
                    .and_then(|key| keys.insert(device_id, &key));
                vec![Outcome::Pairing(match stored {
                    Ok(()) => PairingEvent::Paired {
                        device_id: device_id.clone(),
                    },
                    Err(reason) => PairingEvent::PairingFailed { reason },
                })]
            }
            (AuthMessage::KeyCleared { device_id }, LinkKind::UsbSerial) => {
                vec![Outcome::Pairing(match keys.remove(device_id) {
                    Ok(_) => PairingEvent::KeyCleared {
                        device_id: device_id.clone(),
                    },
                    Err(reason) => PairingEvent::PairingFailed { reason },
                })]
            }
            (AuthMessage::Result { reason, .. }, LinkKind::UsbSerial) => {
                vec![Outcome::Pairing(PairingEvent::PairingFailed {
                    reason: reason.clone(),
                })]
            }
            (AuthMessage::Key { .. } | AuthMessage::KeyCleared { .. }, _) => {
                // A key must never travel over a radio link; never store one from there.
                log::warn!("Ignoring a key message received over a radio link");
                Vec::new()
            }
            _ => {
                let before = self.session.status();
                let reply = self
                    .session
                    .on_message(&message, |device_id| keys.get(device_id));
                let after = self.session.status();
                let mut outcomes: Vec<Outcome> = reply
                    .map(|command| Outcome::Reply(line_of(&command)))
                    .into_iter()
                    .collect();
                if after != before {
                    outcomes.push(Outcome::Status(after));
                }
                outcomes
            }
        }
    }

    /// Validates a command from the WebView and returns the line to send: signed when
    /// the link is authenticated, plain otherwise. The WebView may not send
    /// authentication or pairing commands; only `Rediscover` among system commands.
    pub(crate) fn outgoing(&mut self, data: &str) -> Result<String, String> {
        let command: IncomingCommand = serde_json::from_str(data)
            .map_err(|error| format!("Invalid Pinora command: {error}"))?;
        if let ModuleCommand::System(system_command) = &command.command {
            if !matches!(system_command, SystemCommand::Rediscover {}) {
                return Err("This system command is reserved for the desktop backend".into());
            }
        }
        Ok(match self.session.sign(&command) {
            Some(signed) => line_of(&signed),
            None => line_of(&command),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinora_protocol::auth::{Decision, DeviceSession, Key, KEY_BYTES};
    use pinora_protocol::{parse_command_line, CommandLine};

    const DEVICE: &str = "a1b2c3d4e5f6";

    fn key() -> Key {
        Key::from_bytes([3; KEY_BYTES])
    }

    fn device_reply(device: &mut DeviceSession, line: &str, key: Option<&Key>) -> AuthMessage {
        let parsed = parse_command_line(line.trim_end().as_bytes()).unwrap();
        match device.handle(parsed, key, || [8; NONCE_BYTES]) {
            Decision::Reply(message) => message,
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn a_bluetooth_link_authenticates_and_then_signs_webview_commands() {
        let mut keys = DeviceKeys::default();
        keys.insert(DEVICE, &key()).unwrap();
        let mut link = AuthLink::new(LinkKind::Bluetooth);
        let mut device = DeviceSession::new(DEVICE);

        let begin = link.begin([1; NONCE_BYTES]).unwrap();
        let challenge = device_reply(&mut device, &begin, Some(&key()));
        let outcomes = link.on_auth_message(challenge, &mut keys);
        let Some(Outcome::Reply(response)) = outcomes.first().cloned() else {
            panic!("expected a reply, got {outcomes:?}")
        };
        let result = device_reply(&mut device, &response, Some(&key()));
        assert_eq!(
            link.on_auth_message(result, &mut keys),
            vec![Outcome::Status(AuthStatus::Authenticated {
                device_id: DEVICE.into()
            })]
        );

        let line = link
            .outgoing(r#"{"id":"u","command":{"Led":{"Toggle":{}}}}"#)
            .unwrap();
        let parsed = parse_command_line(line.trim_end().as_bytes()).unwrap();
        assert!(matches!(parsed, CommandLine::Signed(_)));
        assert!(matches!(
            device.handle(parsed, Some(&key()), || unreachable!()),
            Decision::Forward(_)
        ));
    }

    #[test]
    fn an_unknown_device_is_reported_as_no_key() {
        let mut keys = DeviceKeys::default();
        let mut link = AuthLink::new(LinkKind::Wifi);
        let mut device = DeviceSession::new(DEVICE);
        let begin = link.begin([1; NONCE_BYTES]).unwrap();
        let challenge = device_reply(&mut device, &begin, Some(&key()));
        assert_eq!(
            link.on_auth_message(challenge, &mut keys),
            vec![Outcome::Status(AuthStatus::NoKey {
                device_id: DEVICE.into()
            })]
        );
        // Commands still go out plain (the device will refuse them: read-only).
        let line = link
            .outgoing(r#"{"id":"u","command":{"Led":{"Toggle":{}}}}"#)
            .unwrap();
        assert!(matches!(
            parse_command_line(line.trim_end().as_bytes()).unwrap(),
            CommandLine::Plain(_)
        ));
    }

    #[test]
    fn usb_links_pair_and_unpair_but_never_handshake() {
        let mut keys = DeviceKeys::default();
        let mut link = AuthLink::new(LinkKind::UsbSerial);
        assert!(link.begin([1; NONCE_BYTES]).is_none());
        assert!(link.provision_line().unwrap().contains("ProvisionKey"));
        let outcomes = link.on_auth_message(
            AuthMessage::Key {
                device_id: DEVICE.into(),
                key: key().to_hex(),
            },
            &mut keys,
        );
        assert_eq!(
            outcomes,
            vec![Outcome::Pairing(PairingEvent::Paired {
                device_id: DEVICE.into()
            })]
        );
        assert_eq!(keys.get(DEVICE), Some(key()));
        link.on_auth_message(
            AuthMessage::KeyCleared {
                device_id: DEVICE.into(),
            },
            &mut keys,
        );
        assert_eq!(keys.get(DEVICE), None);
    }

    #[test]
    fn keys_arriving_over_a_radio_link_are_never_stored() {
        let mut keys = DeviceKeys::default();
        let mut link = AuthLink::new(LinkKind::Bluetooth);
        assert!(link.provision_line().is_err());
        let outcomes = link.on_auth_message(
            AuthMessage::Key {
                device_id: DEVICE.into(),
                key: key().to_hex(),
            },
            &mut keys,
        );
        assert!(outcomes.is_empty());
        assert!(keys.get(DEVICE).is_none());
    }

    #[test]
    fn the_webview_may_only_send_rediscover_among_system_commands() {
        let mut link = AuthLink::new(LinkKind::Bluetooth);
        assert!(link
            .outgoing(r#"{"id":"SSI","command":{"System":{"Rediscover":{}}}}"#)
            .is_ok());
        for reserved in ["ProvisionKey", "ClearKey"] {
            let data = format!(r#"{{"id":"SSI","command":{{"System":{{"{reserved}":{{}}}}}}}}"#);
            assert!(link.outgoing(&data).is_err(), "{reserved}");
        }
        assert!(link
            .outgoing(r#"{"id":"SSI","command":{"System":{"AuthResponse":{"proof":"00"}}}}"#)
            .is_err());
        assert!(link.outgoing("not json").is_err());
    }
}

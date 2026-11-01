//! HMAC primitives and the device and desktop session state machines. Pure: no I/O,
//! no clock, randomness is passed in. Feature `auth`.

use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

use super::{
    AuthMessage, CommandLine, KEY_BYTES, MAC_BYTES, NONCE_BYTES, SignedCommand, from_hex,
    is_provisioning_command, to_hex,
};
use crate::{IncomingCommand, ModuleCommand, SYSTEM_COMMAND_ID, SystemCommand};

type HmacSha256 = Hmac<Sha256>;

const DEVICE_LABEL: &[u8] = b"pinora-auth-v2/device";
const DESKTOP_LABEL: &[u8] = b"pinora-auth-v2/desktop";
const SESSION_LABEL: &[u8] = b"pinora-auth-v2/session";
const COMMAND_LABEL: &[u8] = b"pinora-auth-v2/command";

pub type Nonce = [u8; NONCE_BYTES];

/// A device key. `Debug` does not print it.
#[derive(Clone, PartialEq, Eq)]
pub struct Key([u8; KEY_BYTES]);

impl Key {
    pub fn from_bytes(bytes: [u8; KEY_BYTES]) -> Self {
        Self(bytes)
    }
    pub fn from_hex(text: &str) -> Option<Self> {
        from_hex::<KEY_BYTES>(text).map(Self)
    }
    pub fn to_hex(&self) -> String {
        to_hex(&self.0)
    }
    pub fn as_bytes(&self) -> &[u8; KEY_BYTES] {
        &self.0
    }
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Key(<redacted>)")
    }
}

/// Plain HMAC-SHA256 (exposed for test vectors).
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; MAC_BYTES] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

fn labelled(key: &[u8], label: &[u8], parts: &[&[u8]]) -> HmacSha256 {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    for part in std::iter::once(&label).chain(parts.iter()) {
        mac.update(&(part.len() as u32).to_be_bytes());
        mac.update(part);
    }
    mac
}

fn compute(key: &[u8], label: &[u8], parts: &[&[u8]]) -> [u8; MAC_BYTES] {
    labelled(key, label, parts).finalize().into_bytes().into()
}

/// Constant-time check of a hex MAC.
fn verify(key: &[u8], label: &[u8], parts: &[&[u8]], expected_hex: &str) -> bool {
    match from_hex::<MAC_BYTES>(expected_hex) {
        Some(expected) => labelled(key, label, parts).verify_slice(&expected).is_ok(),
        None => false,
    }
}

/// Proof that the device knows `K` (checked by the desktop).
pub fn device_proof(key: &Key, device_id: &str, client: &Nonce, device: &Nonce) -> String {
    to_hex(&compute(
        key.as_bytes(),
        DEVICE_LABEL,
        &[device_id.as_bytes(), client, device],
    ))
}

/// Proof that the desktop knows `K` (checked by the device).
pub fn desktop_proof(key: &Key, device_id: &str, client: &Nonce, device: &Nonce) -> String {
    to_hex(&compute(
        key.as_bytes(),
        DESKTOP_LABEL,
        &[device_id.as_bytes(), client, device],
    ))
}

/// Key for the per-command MACs of one session.
pub fn session_key(key: &Key, device_id: &str, client: &Nonce, device: &Nonce) -> [u8; MAC_BYTES] {
    compute(
        key.as_bytes(),
        SESSION_LABEL,
        &[device_id.as_bytes(), client, device],
    )
}

/// MAC of one signed command.
pub fn command_mac(session: &[u8; MAC_BYTES], seq: u32, signed: &str) -> String {
    to_hex(&compute(
        session,
        COMMAND_LABEL,
        &[&seq.to_be_bytes(), signed.as_bytes()],
    ))
}

// ---------------------------------------------------------------------------------
// Device side

/// What the device transport does with one received command line.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Deliver to the main loop.
    Forward(IncomingCommand),
    /// Deliver, and warn once: the device has no key, so commands are not
    /// authenticated.
    ForwardUnprovisioned(IncomingCommand),
    /// Send this message back to the peer only.
    Reply(AuthMessage),
    /// Drop it; tell the peer why (as a SysLog).
    Reject(String),
}

#[derive(Debug, Clone)]
enum DeviceState {
    Idle,
    Challenged {
        key: Key,
        client: Nonce,
        device: Nonce,
    },
    Authenticated {
        key: Key,
        session: [u8; MAC_BYTES],
        last_seq: u32,
    },
}

/// Per-connection authentication state on the device (Bluetooth and Wi-Fi).
///
/// Always allowed: `AuthBegin`, `AuthResponse`, `Rediscover`. With a key, everything
/// else must arrive as a valid [`SignedCommand`] in an authenticated session
/// (read-only otherwise). Without a key, plain commands pass, with one warning per
/// session. Provisioning commands are always rejected here: they are only accepted
/// by the USB serial console.
#[derive(Debug, Clone)]
pub struct DeviceSession {
    device_id: String,
    state: DeviceState,
    warned_unprovisioned: bool,
}

impl DeviceSession {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            state: DeviceState::Idle,
            warned_unprovisioned: false,
        }
    }

    /// Call when the connection closes or a new one opens.
    pub fn reset(&mut self) {
        self.state = DeviceState::Idle;
        self.warned_unprovisioned = false;
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, DeviceState::Authenticated { .. })
    }

    /// Decides what to do with one command line. `key` is the device's current key;
    /// `fresh_nonce` is called only when a new challenge is issued.
    pub fn handle(
        &mut self,
        line: CommandLine,
        key: Option<&Key>,
        fresh_nonce: impl FnOnce() -> Nonce,
    ) -> Decision {
        // A key change (provisioned or cleared over USB) ends the session.
        let session_key_matches = match (&self.state, key) {
            (DeviceState::Idle, _) => true,
            (DeviceState::Challenged { key: used, .. }, Some(current))
            | (DeviceState::Authenticated { key: used, .. }, Some(current)) => used == current,
            _ => false,
        };
        if !session_key_matches {
            self.state = DeviceState::Idle;
        }

        match line {
            CommandLine::Plain(command) => self.handle_plain(command, key, fresh_nonce),
            CommandLine::Signed(signed) => self.handle_signed(signed, key),
        }
    }

    fn handle_plain(
        &mut self,
        command: IncomingCommand,
        key: Option<&Key>,
        fresh_nonce: impl FnOnce() -> Nonce,
    ) -> Decision {
        if is_provisioning_command(&command) {
            return Decision::Reject("Key provisioning is accepted only over USB serial".into());
        }
        if command.id == SYSTEM_COMMAND_ID {
            match &command.command {
                ModuleCommand::System(SystemCommand::AuthBegin { client_nonce }) => {
                    return self.begin(client_nonce, key, fresh_nonce);
                }
                ModuleCommand::System(SystemCommand::AuthResponse { proof }) => {
                    return self.respond(proof, key);
                }
                ModuleCommand::System(SystemCommand::Rediscover {}) => {
                    return Decision::Forward(command);
                }
                _ => {}
            }
        }
        match key {
            None if !self.warned_unprovisioned => {
                self.warned_unprovisioned = true;
                Decision::ForwardUnprovisioned(command)
            }
            None => Decision::Forward(command),
            Some(_) if self.is_authenticated() => Decision::Reject(
                "Command rejected: authenticated sessions must sign commands".into(),
            ),
            Some(_) => Decision::Reject(
                "Command rejected: not authenticated (pair this desktop over USB)".into(),
            ),
        }
    }

    fn begin(
        &mut self,
        client_nonce: &str,
        key: Option<&Key>,
        fresh_nonce: impl FnOnce() -> Nonce,
    ) -> Decision {
        self.state = DeviceState::Idle;
        let Some(client) = from_hex::<NONCE_BYTES>(client_nonce) else {
            return Decision::Reply(failure("malformed client nonce"));
        };
        let Some(key) = key else {
            return Decision::Reply(AuthMessage::Unprovisioned {
                device_id: self.device_id.clone(),
            });
        };
        let device = fresh_nonce();
        let proof = device_proof(key, &self.device_id, &client, &device);
        self.state = DeviceState::Challenged {
            key: key.clone(),
            client,
            device,
        };
        Decision::Reply(AuthMessage::Challenge {
            device_id: self.device_id.clone(),
            device_nonce: to_hex(&device),
            device_proof: proof,
        })
    }

    fn respond(&mut self, proof: &str, key: Option<&Key>) -> Decision {
        let state = std::mem::replace(&mut self.state, DeviceState::Idle);
        let (
            Some(_),
            DeviceState::Challenged {
                key,
                client,
                device,
            },
        ) = (key, state)
        else {
            return Decision::Reply(failure("no challenge is pending"));
        };
        let parts: [&[u8]; 3] = [self.device_id.as_bytes(), &client, &device];
        if !verify(key.as_bytes(), DESKTOP_LABEL, &parts, proof) {
            return Decision::Reply(failure("authentication failed"));
        }
        self.state = DeviceState::Authenticated {
            session: session_key(&key, &self.device_id, &client, &device),
            key,
            last_seq: 0,
        };
        Decision::Reply(AuthMessage::Result {
            authenticated: true,
            reason: String::new(),
        })
    }

    fn handle_signed(&mut self, signed: SignedCommand, key: Option<&Key>) -> Decision {
        if key.is_none() {
            return Decision::Reject("Signed command rejected: this device has no key".into());
        }
        let DeviceState::Authenticated {
            session, last_seq, ..
        } = &mut self.state
        else {
            return Decision::Reject("Signed command rejected: not authenticated".into());
        };
        let parts: [&[u8]; 2] = [&signed.seq.to_be_bytes(), signed.signed.as_bytes()];
        if !verify(session, COMMAND_LABEL, &parts, &signed.mac) {
            return Decision::Reject("Signed command rejected: bad MAC".into());
        }
        if signed.seq <= *last_seq {
            return Decision::Reject("Signed command rejected: replayed or out of order".into());
        }
        *last_seq = signed.seq;
        match serde_json::from_str::<IncomingCommand>(&signed.signed) {
            Ok(command) if is_provisioning_command(&command) => {
                Decision::Reject("Key provisioning is accepted only over USB serial".into())
            }
            Ok(command) => Decision::Forward(command),
            Err(error) => Decision::Reject(format!("Signed command rejected: {error}")),
        }
    }
}

fn failure(reason: &str) -> AuthMessage {
    AuthMessage::Result {
        authenticated: false,
        reason: reason.into(),
    }
}

// ---------------------------------------------------------------------------------
// Desktop side

/// Authentication state of the desktop's current connection, as shown to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state")]
pub enum AuthStatus {
    /// No handshake started (USB serial, or not connected).
    Idle,
    /// `AuthBegin` sent, waiting for the device.
    Pending,
    Authenticated {
        device_id: String,
    },
    /// The device has no key; commands are not authenticated.
    Unprovisioned {
        device_id: String,
    },
    /// The device has a key, but this desktop does not know it: read-only.
    NoKey {
        device_id: String,
    },
    /// The device's proof did not verify: it is not the paired device (or the stored
    /// key is stale). Nothing is sent to it.
    DeviceRejected {
        device_id: String,
    },
    /// The device refused this desktop's proof, or the handshake was malformed.
    Failed {
        reason: String,
    },
}

#[derive(Debug, Clone)]
enum ClientState {
    Idle,
    Begun {
        client: Nonce,
    },
    Responded {
        device_id: String,
        session: [u8; MAC_BYTES],
    },
    Authenticated {
        device_id: String,
        session: [u8; MAC_BYTES],
        next_seq: u32,
    },
    Done(AuthStatus),
}

/// The desktop half of the handshake. Feed it the device's [`AuthMessage`]s; send
/// what it returns; sign commands with [`ClientSession::sign`].
#[derive(Debug, Clone)]
pub struct ClientSession {
    state: ClientState,
}

impl Default for ClientSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientSession {
    pub fn new() -> Self {
        Self {
            state: ClientState::Idle,
        }
    }

    pub fn reset(&mut self) {
        self.state = ClientState::Idle;
    }

    /// Starts a handshake; send the returned command to the device.
    pub fn begin(&mut self, client: Nonce) -> IncomingCommand {
        self.state = ClientState::Begun { client };
        system_command(SystemCommand::AuthBegin {
            client_nonce: to_hex(&client),
        })
    }

    pub fn status(&self) -> AuthStatus {
        match &self.state {
            ClientState::Idle => AuthStatus::Idle,
            ClientState::Begun { .. } | ClientState::Responded { .. } => AuthStatus::Pending,
            ClientState::Authenticated { device_id, .. } => AuthStatus::Authenticated {
                device_id: device_id.clone(),
            },
            ClientState::Done(status) => status.clone(),
        }
    }

    /// Handles one message from the device. Returns a reply to send, if any; check
    /// [`ClientSession::status`] afterwards.
    pub fn on_message(
        &mut self,
        message: &AuthMessage,
        key_for: impl FnOnce(&str) -> Option<Key>,
    ) -> Option<IncomingCommand> {
        let state = std::mem::replace(&mut self.state, ClientState::Idle);
        match (message, state) {
            (
                AuthMessage::Challenge {
                    device_id,
                    device_nonce,
                    device_proof: proof,
                },
                ClientState::Begun { client },
            ) => {
                let Some(device) = from_hex::<NONCE_BYTES>(device_nonce) else {
                    self.state = ClientState::Done(AuthStatus::Failed {
                        reason: "malformed device nonce".into(),
                    });
                    return None;
                };
                let Some(key) = key_for(device_id) else {
                    self.state = ClientState::Done(AuthStatus::NoKey {
                        device_id: device_id.clone(),
                    });
                    return None;
                };
                let parts: [&[u8]; 3] = [device_id.as_bytes(), &client, &device];
                if !verify(key.as_bytes(), DEVICE_LABEL, &parts, proof) {
                    self.state = ClientState::Done(AuthStatus::DeviceRejected {
                        device_id: device_id.clone(),
                    });
                    return None;
                }
                self.state = ClientState::Responded {
                    device_id: device_id.clone(),
                    session: session_key(&key, device_id, &client, &device),
                };
                Some(system_command(SystemCommand::AuthResponse {
                    proof: desktop_proof(&key, device_id, &client, &device),
                }))
            }
            (AuthMessage::Unprovisioned { device_id }, ClientState::Begun { .. }) => {
                self.state = ClientState::Done(AuthStatus::Unprovisioned {
                    device_id: device_id.clone(),
                });
                None
            }
            (
                AuthMessage::Result {
                    authenticated: true,
                    ..
                },
                ClientState::Responded { device_id, session },
            ) => {
                self.state = ClientState::Authenticated {
                    device_id,
                    session,
                    next_seq: 1,
                };
                None
            }
            (AuthMessage::Result { reason, .. }, ClientState::Begun { .. })
            | (AuthMessage::Result { reason, .. }, ClientState::Responded { .. }) => {
                self.state = ClientState::Done(AuthStatus::Failed {
                    reason: reason.clone(),
                });
                None
            }
            (_, state) => {
                // Unexpected in this state (or Key/KeyCleared, handled elsewhere).
                self.state = state;
                None
            }
        }
    }

    /// Signs `command` if the session is authenticated; `None` otherwise (send it
    /// plain: a device without a key accepts it, a keyed one rejects it).
    pub fn sign(&mut self, command: &IncomingCommand) -> Option<SignedCommand> {
        let ClientState::Authenticated {
            session, next_seq, ..
        } = &mut self.state
        else {
            return None;
        };
        let signed = serde_json::to_string(command).ok()?;
        let seq = *next_seq;
        *next_seq = next_seq.checked_add(1)?;
        Some(SignedCommand {
            seq,
            mac: command_mac(session, seq, &signed),
            signed,
        })
    }
}

fn system_command(command: SystemCommand) -> IncomingCommand {
    IncomingCommand {
        id: SYSTEM_COMMAND_ID.to_string(),
        command: ModuleCommand::System(command),
    }
}

/// Re-exported for callers that build a [`CommandLine`] by hand in tests.
pub fn plain(command: IncomingCommand) -> CommandLine {
    CommandLine::Plain(command)
}

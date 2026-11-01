//! Protocol v2 authentication (audit Phase 2B, PIN-TPL-003 / PIN-TPL-013): HMAC test
//! vectors, the full handshake between `ClientSession` and `DeviceSession`, and every
//! rejection path. Run with `cargo test --features auth` (or `--all-features`).

use pinora_protocol::auth::*;
use pinora_protocol::*;

const DEVICE: &str = "a1b2c3d4e5f6";

fn key(byte: u8) -> Key {
    Key::from_bytes([byte; KEY_BYTES])
}

fn led(state: u32) -> IncomingCommand {
    IncomingCommand {
        id: "led-uuid".into(),
        command: ModuleCommand::Led(LedCommandPayload::SetState { state }),
    }
}

fn system(command: SystemCommand) -> IncomingCommand {
    IncomingCommand {
        id: SYSTEM_COMMAND_ID.into(),
        command: ModuleCommand::System(command),
    }
}

/// Runs the handshake; returns both sides and the device's replies.
fn handshake(device_key: Option<&Key>, client_key: Option<Key>) -> (DeviceSession, ClientSession) {
    let mut device = DeviceSession::new(DEVICE);
    let mut client = ClientSession::new();
    let begin = client.begin([7; NONCE_BYTES]);
    let reply = match device.handle(plain(begin), device_key, || [9; NONCE_BYTES]) {
        Decision::Reply(message) => message,
        other => panic!("unexpected {other:?}"),
    };
    let response = client.on_message(&reply, |id| {
        assert_eq!(id, DEVICE);
        client_key.clone()
    });
    if let Some(response) = response {
        match device.handle(plain(response), device_key, || unreachable!()) {
            Decision::Reply(result) => {
                assert!(client.on_message(&result, |_| None).is_none());
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    (device, client)
}

#[test]
fn hmac_sha256_matches_rfc_4231() {
    // Test case 1.
    assert_eq!(
        to_hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
    );
    // Test case 2.
    assert_eq!(
        to_hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
}

#[test]
fn hex_round_trips_and_rejects_bad_input() {
    let bytes = [0x00, 0x7f, 0xff, 0xa5];
    assert_eq!(to_hex(&bytes), "007fffa5");
    assert_eq!(from_hex::<4>("007FFFA5"), Some(bytes));
    assert_eq!(from_hex::<4>("007fffa"), None);
    assert_eq!(from_hex::<4>("007fffa5aa"), None);
    assert_eq!(from_hex::<4>("007fffgz"), None);
    let k = key(0x42);
    assert_eq!(Key::from_hex(&k.to_hex()), Some(k.clone()));
    assert_eq!(format!("{k:?}"), "Key(<redacted>)");
}

#[test]
fn a_full_handshake_authenticates_both_sides_and_signed_commands_pass() {
    let k = key(1);
    let (mut device, mut client) = handshake(Some(&k), Some(k.clone()));
    assert!(device.is_authenticated());
    assert_eq!(
        client.status(),
        AuthStatus::Authenticated {
            device_id: DEVICE.into()
        }
    );

    for state in [10, 20, 30] {
        let signed = client.sign(&led(state)).expect("authenticated");
        assert_eq!(
            device.handle(CommandLine::Signed(signed), Some(&k), || unreachable!()),
            Decision::Forward(led(state))
        );
    }
}

#[test]
fn signed_commands_survive_the_wire_unchanged() {
    // The MAC covers the exact text, including f32 values.
    let k = key(1);
    let (mut device, mut client) = handshake(Some(&k), Some(k.clone()));
    let command = IncomingCommand {
        id: "stepper".into(),
        command: ModuleCommand::StepperMotor(StepperMotorCommandPayload::MoveToAngle {
            angle: 0.1,
        }),
    };
    let line = serde_json::to_vec(&client.sign(&command).unwrap()).unwrap();
    let parsed = parse_command_line(&line).unwrap();
    assert_eq!(
        device.handle(parsed, Some(&k), || unreachable!()),
        Decision::Forward(command)
    );
}

#[test]
fn a_tampered_command_or_mac_is_rejected() {
    let k = key(1);
    let (mut device, mut client) = handshake(Some(&k), Some(k.clone()));
    let mut signed = client.sign(&led(10)).unwrap();
    signed.signed = signed.signed.replace("10", "99");
    assert!(matches!(
        device.handle(CommandLine::Signed(signed), Some(&k), || unreachable!()),
        Decision::Reject(reason) if reason.contains("bad MAC")
    ));

    let mut signed = client.sign(&led(11)).unwrap();
    let flipped = if signed.mac.starts_with('0') {
        "1"
    } else {
        "0"
    };
    signed.mac.replace_range(0..1, flipped);
    assert!(matches!(
        device.handle(CommandLine::Signed(signed), Some(&k), || unreachable!()),
        Decision::Reject(reason) if reason.contains("bad MAC")
    ));
}

#[test]
fn replayed_and_out_of_order_commands_are_rejected() {
    let k = key(1);
    let (mut device, mut client) = handshake(Some(&k), Some(k.clone()));
    let first = client.sign(&led(1)).unwrap();
    let second = client.sign(&led(2)).unwrap();
    assert!(matches!(
        device.handle(
            CommandLine::Signed(second.clone()),
            Some(&k),
            || unreachable!()
        ),
        Decision::Forward(_)
    ));
    for stale in [first, second] {
        assert!(matches!(
            device.handle(CommandLine::Signed(stale), Some(&k), || unreachable!()),
            Decision::Reject(reason) if reason.contains("replayed")
        ));
    }
}

#[test]
fn a_keyed_device_is_read_only_until_authenticated() {
    let k = key(1);
    let mut device = DeviceSession::new(DEVICE);
    assert!(matches!(
        device.handle(plain(led(1)), Some(&k), || unreachable!()),
        Decision::Reject(reason) if reason.contains("not authenticated")
    ));
    // Discovery is still allowed.
    let rediscover = system(SystemCommand::Rediscover {});
    assert_eq!(
        device.handle(plain(rediscover.clone()), Some(&k), || unreachable!()),
        Decision::Forward(rediscover)
    );
    // A signed command before the handshake is rejected too.
    let forged = SignedCommand {
        seq: 1,
        mac: "00".repeat(MAC_BYTES),
        signed: serde_json::to_string(&led(1)).unwrap(),
    };
    assert!(matches!(
        device.handle(CommandLine::Signed(forged), Some(&k), || unreachable!()),
        Decision::Reject(_)
    ));
}

#[test]
fn an_authenticated_session_still_rejects_plain_commands() {
    let k = key(1);
    let (mut device, _client) = handshake(Some(&k), Some(k.clone()));
    assert!(matches!(
        device.handle(plain(led(1)), Some(&k), || unreachable!()),
        Decision::Reject(reason) if reason.contains("sign")
    ));
}

#[test]
fn a_desktop_with_the_wrong_key_rejects_the_device_and_sends_nothing() {
    let (device, client) = handshake(Some(&key(1)), Some(key(2)));
    assert!(!device.is_authenticated());
    assert_eq!(
        client.status(),
        AuthStatus::DeviceRejected {
            device_id: DEVICE.into()
        }
    );
}

#[test]
fn a_desktop_without_the_key_is_told_no_key() {
    let (device, client) = handshake(Some(&key(1)), None);
    assert!(!device.is_authenticated());
    assert_eq!(
        client.status(),
        AuthStatus::NoKey {
            device_id: DEVICE.into()
        }
    );
}

#[test]
fn a_forged_desktop_proof_is_rejected_by_the_device() {
    let k = key(1);
    let mut device = DeviceSession::new(DEVICE);
    let mut client = ClientSession::new();
    let begin = client.begin([7; NONCE_BYTES]);
    let Decision::Reply(AuthMessage::Challenge { device_nonce, .. }) =
        device.handle(plain(begin), Some(&k), || [9; NONCE_BYTES])
    else {
        panic!("expected a challenge")
    };
    // An attacker who saw the challenge but has another key.
    let device_nonce = from_hex::<NONCE_BYTES>(&device_nonce).unwrap();
    let proof = desktop_proof(&key(2), DEVICE, &[7; NONCE_BYTES], &device_nonce);
    let decision = device.handle(
        plain(system(SystemCommand::AuthResponse { proof })),
        Some(&k),
        || unreachable!(),
    );
    assert_eq!(
        decision,
        Decision::Reply(AuthMessage::Result {
            authenticated: false,
            reason: "authentication failed".into()
        })
    );
    assert!(!device.is_authenticated());
}

#[test]
fn a_response_cannot_be_replayed_against_a_new_challenge() {
    let k = key(1);
    let mut device = DeviceSession::new(DEVICE);
    let mut client = ClientSession::new();
    let begin = client.begin([7; NONCE_BYTES]);
    let Decision::Reply(challenge) =
        device.handle(plain(begin.clone()), Some(&k), || [9; NONCE_BYTES])
    else {
        panic!()
    };
    let response = client.on_message(&challenge, |_| Some(k.clone())).unwrap();
    // The device issues a fresh nonce for a second AuthBegin; the old proof is stale.
    let _ = device.handle(plain(begin), Some(&k), || [10; NONCE_BYTES]);
    assert!(matches!(
        device.handle(plain(response.clone()), Some(&k), || unreachable!()),
        Decision::Reply(AuthMessage::Result {
            authenticated: false,
            ..
        })
    ));
    // And a response with no pending challenge is refused.
    assert!(matches!(
        device.handle(plain(response), Some(&k), || unreachable!()),
        Decision::Reply(AuthMessage::Result {
            authenticated: false,
            ..
        })
    ));
}

#[test]
fn an_unprovisioned_device_accepts_plain_commands_and_warns_once() {
    let (mut device, client) = handshake(None, None);
    assert_eq!(
        client.status(),
        AuthStatus::Unprovisioned {
            device_id: DEVICE.into()
        }
    );
    assert_eq!(
        device.handle(plain(led(1)), None, || unreachable!()),
        Decision::ForwardUnprovisioned(led(1))
    );
    assert_eq!(
        device.handle(plain(led(2)), None, || unreachable!()),
        Decision::Forward(led(2))
    );
    device.reset();
    assert_eq!(
        device.handle(plain(led(3)), None, || unreachable!()),
        Decision::ForwardUnprovisioned(led(3))
    );
}

#[test]
fn provisioning_is_never_accepted_by_a_radio_session() {
    let k = key(1);
    for command in [SystemCommand::ProvisionKey {}, SystemCommand::ClearKey {}] {
        for current in [None, Some(&k)] {
            let mut device = DeviceSession::new(DEVICE);
            assert!(matches!(
                device.handle(plain(system(command.clone())), current, || unreachable!()),
                Decision::Reject(reason) if reason.contains("USB")
            ));
        }
        // Not even inside a valid signed command.
        let (mut device, mut client) = handshake(Some(&k), Some(k.clone()));
        let signed = client.sign(&system(command)).unwrap();
        assert!(matches!(
            device.handle(CommandLine::Signed(signed), Some(&k), || unreachable!()),
            Decision::Reject(reason) if reason.contains("USB")
        ));
    }
}

#[test]
fn changing_the_device_key_ends_the_session() {
    let k = key(1);
    let (mut device, mut client) = handshake(Some(&k), Some(k.clone()));
    let signed = client.sign(&led(1)).unwrap();
    let new_key = key(3);
    assert!(matches!(
        device.handle(CommandLine::Signed(signed), Some(&new_key), || unreachable!()),
        Decision::Reject(reason) if reason.contains("not authenticated")
    ));
    assert!(!device.is_authenticated());
}

#[test]
fn a_malformed_client_nonce_is_refused() {
    let mut device = DeviceSession::new(DEVICE);
    let begin = system(SystemCommand::AuthBegin {
        client_nonce: "xyz".into(),
    });
    assert!(matches!(
        device.handle(plain(begin), Some(&key(1)), || unreachable!()),
        Decision::Reply(AuthMessage::Result {
            authenticated: false,
            ..
        })
    ));
}

#[test]
fn an_unauthenticated_client_does_not_sign() {
    let mut client = ClientSession::new();
    assert!(client.sign(&led(1)).is_none());
    let _ = client.begin([1; NONCE_BYTES]);
    assert_eq!(client.status(), AuthStatus::Pending);
    assert!(client.sign(&led(1)).is_none());
}

#[test]
fn status_serializes_for_the_ui_without_secrets() {
    let status = AuthStatus::Authenticated {
        device_id: DEVICE.into(),
    };
    assert_eq!(
        serde_json::to_string(&status).unwrap(),
        r#"{"state":"Authenticated","device_id":"a1b2c3d4e5f6"}"#
    );
    assert_eq!(
        serde_json::to_string(&AuthStatus::Pending).unwrap(),
        r#"{"state":"Pending"}"#
    );
}

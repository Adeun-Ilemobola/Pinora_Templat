//! `core/transport/auth_gate.rs`: what the radio transports do with each line, run
//! end to end against the desktop's `ClientSession`, and how the USB console routes
//! lines (audit Phase 2B, PIN-TPL-003).

use pinora_firmware_host_tests::core::transport::auth_gate::*;
use pinora_protocol::auth::*;
use pinora_protocol::*;

const DEVICE: &str = "a1b2c3d4e5f6";

fn key() -> Key {
    Key::from_bytes([5; KEY_BYTES])
}

fn led(state: u32) -> IncomingCommand {
    IncomingCommand {
        id: "led-uuid".into(),
        command: ModuleCommand::Led(LedCommandPayload::SetState { state }),
    }
}

fn line<T: serde::Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}

fn auth_reply(actions: Vec<Action>) -> AuthMessage {
    match actions.as_slice() {
        [Action::Send(ProtocolMessage::Auth(message))] => message.clone(),
        other => panic!("expected one Auth reply, got {other:?}"),
    }
}

fn syslog(actions: &[Action]) -> Vec<(String, LogPriority)> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::Send(ProtocolMessage::ModuleEvent(EventPackage {
                id,
                event: ModuleEvent::SysLog(log),
            })) => {
                assert_eq!(id, "log");
                Some((log.text.clone(), log.priority.clone()))
            }
            _ => None,
        })
        .collect()
}

fn authenticate(session: &mut DeviceSession, client: &mut ClientSession) {
    let k = key();
    let begin = client.begin([1; NONCE_BYTES]);
    let challenge =
        auth_reply(process_line(session, &line(&begin), Some(&k), || [2; NONCE_BYTES]).unwrap());
    let response = client.on_message(&challenge, |_| Some(key())).unwrap();
    let result =
        auth_reply(process_line(session, &line(&response), Some(&k), || unreachable!()).unwrap());
    assert!(client.on_message(&result, |_| None).is_none());
    assert_eq!(
        client.status(),
        AuthStatus::Authenticated {
            device_id: DEVICE.into()
        }
    );
}

#[test]
fn an_authenticated_desktop_drives_the_device_with_signed_lines() {
    let mut session = DeviceSession::new(DEVICE);
    let mut client = ClientSession::new();
    authenticate(&mut session, &mut client);
    let signed = client.sign(&led(40)).unwrap();
    assert_eq!(
        process_line(
            &mut session,
            &line(&signed),
            Some(&key()),
            || unreachable!()
        )
        .unwrap(),
        vec![Action::Deliver(led(40))]
    );
}

#[test]
fn a_keyed_device_rejects_unauthenticated_commands_with_a_high_syslog() {
    let mut session = DeviceSession::new(DEVICE);
    let actions = process_line(
        &mut session,
        &line(&led(1)),
        Some(&key()),
        || unreachable!(),
    )
    .unwrap();
    assert!(!actions.iter().any(|a| matches!(a, Action::Deliver(_))));
    let logs = syslog(&actions);
    assert_eq!(logs.len(), 1);
    assert!(logs[0].0.contains("not authenticated"));
    assert_eq!(logs[0].1, LogPriority::High);
}

#[test]
fn discovery_stays_available_to_unauthenticated_peers() {
    let mut session = DeviceSession::new(DEVICE);
    let rediscover = IncomingCommand {
        id: SYSTEM_COMMAND_ID.into(),
        command: ModuleCommand::System(SystemCommand::Rediscover {}),
    };
    assert_eq!(
        process_line(
            &mut session,
            &line(&rediscover),
            Some(&key()),
            || unreachable!()
        )
        .unwrap(),
        vec![Action::Deliver(rediscover)]
    );
}

#[test]
fn an_unprovisioned_device_warns_once_per_connection() {
    let mut session = DeviceSession::new(DEVICE);
    let first = process_line(&mut session, &line(&led(1)), None, || unreachable!()).unwrap();
    assert_eq!(
        syslog(&first),
        vec![(UNPROVISIONED_WARNING.to_string(), LogPriority::Medium)]
    );
    assert_eq!(first.last(), Some(&Action::Deliver(led(1))));
    let second = process_line(&mut session, &line(&led(2)), None, || unreachable!()).unwrap();
    assert_eq!(second, vec![Action::Deliver(led(2))]);
}

#[test]
fn provisioning_over_a_radio_session_is_refused() {
    let mut session = DeviceSession::new(DEVICE);
    let provision = IncomingCommand {
        id: SYSTEM_COMMAND_ID.into(),
        command: ModuleCommand::System(SystemCommand::ProvisionKey {}),
    };
    let actions = process_line(&mut session, &line(&provision), None, || unreachable!()).unwrap();
    assert!(syslog(&actions)[0].0.contains("USB"));
    assert!(!actions.iter().any(|a| matches!(a, Action::Deliver(_))));
}

#[test]
fn malformed_lines_are_errors() {
    let mut session = DeviceSession::new(DEVICE);
    assert!(process_line(&mut session, b"{not json", None, || unreachable!()).is_err());
}

#[test]
fn the_console_handles_provisioning_in_every_image() {
    for forwards in [false, true] {
        let provision = line(&IncomingCommand {
            id: SYSTEM_COMMAND_ID.into(),
            command: ModuleCommand::System(SystemCommand::ProvisionKey {}),
        });
        assert_eq!(
            route_console_line(&provision, forwards).unwrap(),
            ConsoleRoute::ProvisionKey
        );
        let clear = line(&IncomingCommand {
            id: SYSTEM_COMMAND_ID.into(),
            command: ModuleCommand::System(SystemCommand::ClearKey {}),
        });
        assert_eq!(
            route_console_line(&clear, forwards).unwrap(),
            ConsoleRoute::ClearKey
        );
    }
}

#[test]
fn the_console_forwards_commands_only_in_the_serial_image() {
    assert_eq!(
        route_console_line(&line(&led(3)), true).unwrap(),
        ConsoleRoute::Forward(led(3))
    );
    assert!(matches!(
        route_console_line(&line(&led(3)), false).unwrap(),
        ConsoleRoute::Ignore(_)
    ));
    let signed = SignedCommand {
        seq: 1,
        mac: "00".repeat(MAC_BYTES),
        signed: "{}".into(),
    };
    assert!(matches!(
        route_console_line(&line(&signed), true).unwrap(),
        ConsoleRoute::Ignore(_)
    ));
    assert!(route_console_line(b"garbage", true).is_err());
}

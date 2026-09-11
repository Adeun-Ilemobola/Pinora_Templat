use super::*;
use pinora_protocol::{ImuEvent, LedEvent, MpuDeviceMode, RemoteButtonEvent, SysLogEvent};

fn controller() -> ModuleController {
    ModuleController {
        collections: HashMap::new(),
        registered_module_ids: HashSet::new(),
        total_errors: 0,
        ui: slint::Weak::default(),
        t_command: Arc::new(|_| {}),
    }
}

fn receive(controller: &mut ModuleController, id: &str, event: ModuleEvent) {
    let message = ProtocolMessage::ModuleEvent(EventPackage {
        id: id.into(),
        event,
    });
    // Exercise the same newline-terminated bytes delivered by serial transport.
    let line = serde_json::to_string(&message).unwrap() + "\n";
    controller.incoming_event(line.into_bytes());
}

#[test]
fn package_id_routes_same_type_instances_and_preserves_ui_identity() {
    let mut c = controller();
    c.collections
        .insert("led-a".into(), ModuleState::Led(LedState::default()));
    c.collections
        .insert("led-b".into(), ModuleState::Led(LedState::default()));
    receive(
        &mut c,
        "led-b",
        ModuleEvent::Led(LedEvent::Brightness { level: 80 }),
    );
    let ModuleState::Led(a) = &c.collections["led-a"] else {
        panic!()
    };
    let ModuleState::Led(b) = &c.collections["led-b"] else {
        panic!()
    };
    assert_eq!(a.brightness, 0.0);
    assert_eq!(b.brightness, 80.0);
    assert_eq!(b.id.as_str(), "led-b");
    receive(
        &mut c,
        "unknown",
        ModuleEvent::Led(LedEvent::Brightness { level: 30 }),
    );
    receive(
        &mut c,
        "led-b",
        ModuleEvent::Button(pinora_protocol::ButtonEvent::Ckick {}),
    );
    let ModuleState::Led(b) = &c.collections["led-b"] else {
        panic!()
    };
    assert_eq!(b.brightness, 80.0);
    assert_eq!(c.collections.len(), 2);
}

#[test]
fn mode_only_event_populates_imu_identity() {
    let mut c = controller();
    c.collections
        .insert("imu".into(), ModuleState::Imu(ImuState::default()));
    receive(
        &mut c,
        "imu",
        ModuleEvent::Imu(ImuEvent::Mode {
            mode: MpuDeviceMode::Collecting,
        }),
    );
    let ModuleState::Imu(state) = &c.collections["imu"] else {
        panic!()
    };
    assert!(state.has_id);
    assert!(state.has_mode);
    assert_eq!(state.id.as_str(), "imu");
    assert_eq!(state.mode, crate::MpuDeviceMode::Collecting);
}

#[test]
fn diagnostics_do_not_require_a_syslog_or_source_registration() {
    let mut c = controller();
    c.collections
        .insert("rfid".into(), ModuleState::Rfid(RfidState::default()));
    for (source, priority) in [
        ("rfid", LogPriority::High),
        ("failed-imu", LogPriority::Critical),
        ("unknown", LogPriority::Low),
        ("unknown", LogPriority::Medium),
    ] {
        receive(
            &mut c,
            source,
            ModuleEvent::SysLog(SysLogEvent {
                text: "diagnostic".into(),
                raw_err: None,
                priority,
            }),
        );
    }
    assert_eq!(c.total_errors, 2);
    assert!(matches!(&c.collections["rfid"], ModuleState::Rfid(_)));
    assert_eq!(c.collections.len(), 1);
}

#[test]
fn remote_update_retains_source_and_key_for_publication() {
    let mut c = controller();
    c.collections.insert(
        "remote".into(),
        ModuleState::RemoteReceiver(RemoteReceiverState::default()),
    );
    receive(
        &mut c,
        "remote",
        ModuleEvent::RemoteReceiver(RemoteButtonEvent::Click {
            key: pinora_protocol::RemoteButton::Power,
        }),
    );
    let ModuleState::RemoteReceiver(state) = &c.collections["remote"] else {
        panic!()
    };
    assert_eq!(state.id.as_str(), "remote");
    assert_eq!(state.key, crate::RemoteButton::Power);
}

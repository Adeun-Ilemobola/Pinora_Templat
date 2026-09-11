use pinora_protocol::*;
use serde_json::{Value, json};

fn round_trip(event: ModuleEvent, payload: Value) {
    let message = ProtocolMessage::ModuleEvent(EventPackage {
        id: "module-uuid".into(),
        event,
    });
    let expected = json!({"ModuleEvent": {"id": "module-uuid", "event": payload}});
    let actual = serde_json::to_value(&message).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(
        serde_json::from_value::<ProtocolMessage>(actual).unwrap(),
        message
    );
    assert_eq!(
        serde_json::from_slice::<ProtocolMessage>(
            serde_json::to_string(&message).unwrap().as_bytes()
        )
        .unwrap(),
        message
    );
}

#[test]
fn led_package_preserves_internal_tag() {
    round_trip(
        ModuleEvent::Led(LedEvent::Brightness { level: 80 }),
        json!({"Led": {"event_type": "Brightness", "level": 80}}),
    );
}

#[test]
fn button_package_preserves_empty_struct_variant() {
    round_trip(
        ModuleEvent::Button(ButtonEvent::Ckick {}),
        json!({"Button": {"event_type": "Ckick"}}),
    );
}

#[test]
fn stepper_package_preserves_external_tag() {
    round_trip(
        ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            angle: 45.0,
            step: 512.0,
        }),
        json!({"StepperMotor": {"GetAngle": {"angle": 45.0, "step": 512.0}}}),
    );
}

#[test]
fn imu_mode_package_has_source_identity() {
    round_trip(
        ModuleEvent::Imu(ImuEvent::Mode {
            mode: MpuDeviceMode::Collecting,
        }),
        json!({"Imu": {"event_type": "Mode", "mode": "Collecting"}}),
    );
}

#[test]
fn rfid_package_retains_card_identity() {
    round_trip(
        ModuleEvent::Rfid(RfidEvent::GetCard {
            card_uid: "card-uid".into(),
            card_data: "card-data".into(),
        }),
        json!({"Rfid": {"GetCard": {"card_uid": "card-uid", "card_data": "card-data"}}}),
    );
}

#[test]
fn remote_package_preserves_key() {
    round_trip(
        ModuleEvent::RemoteReceiver(RemoteButtonEvent::Click {
            key: RemoteButton::Power,
        }),
        json!({"RemoteReceiver": {"Click": {"key": "Power"}}}),
    );
}

#[test]
fn syslog_package_has_source_identity() {
    round_trip(
        ModuleEvent::SysLog(SysLogEvent {
            text: "diagnostic".into(),
            raw_err: None,
            priority: LogPriority::High,
        }),
        json!({"SysLog": {"text": "diagnostic", "raw_err": null, "priority": "High"}}),
    );
}

#[test]
fn legacy_bare_event_is_rejected() {
    let legacy = json!({"ModuleEvent": {"Led": {"event_type": "Brightness", "id": "module-uuid", "level": 80}}});
    assert!(serde_json::from_value::<ProtocolMessage>(legacy).is_err());
}

#[test]
fn standalone_event_families_have_no_routing_ids() {
    assert_eq!(
        serde_json::to_value(ServoEvent::GetAngle { angle: 45 }).unwrap(),
        json!({"GetAngle": {"angle": 45}})
    );
    assert_eq!(
        serde_json::to_value(LidarEvent::Target {
            point: Point { x: 1, y: 2 }
        })
        .unwrap(),
        json!({"Target": {"point": {"x": 1, "y": 2}}})
    );
    assert_eq!(
        serde_json::to_value(RangefinderEvent::Range { millimeters: 100 }).unwrap(),
        json!({"event_type": "Range", "millimeters": 100})
    );
}

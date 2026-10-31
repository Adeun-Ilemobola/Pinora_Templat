//! Characterization tests for `Firmware_Templates/src/core/transport/discovery_cache.rs`
//! (the reconnect replay cache used by the Bluetooth and Wi-Fi transports).

use pinora_firmware_host_tests::core::transport::discovery_cache::remember;
use pinora_protocol::*;

fn registration(id: &str, name: &str) -> ProtocolMessage {
    ProtocolMessage::Registration(Registration {
        id: id.into(),
        module_type: ModuleType::Led,
        lool_up_id: name.into(),
        parent_id: String::new(),
    })
}

fn system(version: &str) -> ProtocolMessage {
    ProtocolMessage::System(SystemInfo {
        protocol_version: PROTOCOL_VERSION,
        esp_idf_version: version.into(),
        total_heap: String::new(),
        current_free_heap: String::new(),
        lowest_free_heap: String::new(),
        largest_allocation: String::new(),
        maximum_app_slot: String::new(),
        flash: String::new(),
    })
}

#[test]
fn keeps_only_the_latest_system_snapshot() {
    let (mut system_slot, mut registrations) = (None, Vec::new());
    remember(&system("a"), &mut system_slot, &mut registrations);
    remember(&system("b"), &mut system_slot, &mut registrations);
    assert_eq!(system_slot, Some(system("b")));
    assert!(registrations.is_empty());
}

#[test]
fn keeps_one_registration_per_module_in_first_seen_order() {
    let (mut system_slot, mut registrations) = (None, Vec::new());
    remember(
        &registration("1", "led1"),
        &mut system_slot,
        &mut registrations,
    );
    remember(
        &registration("2", "led2"),
        &mut system_slot,
        &mut registrations,
    );
    remember(
        &registration("1", "renamed"),
        &mut system_slot,
        &mut registrations,
    );
    assert_eq!(
        registrations,
        vec![registration("1", "renamed"), registration("2", "led2")]
    );
    assert_eq!(system_slot, None);
}

#[test]
fn ignores_module_events() {
    // PIN-TPL-012: runtime values are not cached, so they are not replayed.
    let (mut system_slot, mut registrations) = (None, Vec::new());
    let event = ProtocolMessage::ModuleEvent(EventPackage {
        id: "1".into(),
        event: ModuleEvent::Led(LedEvent::Brightness { level: 5 }),
    });
    remember(&event, &mut system_slot, &mut registrations);
    assert_eq!(system_slot, None);
    assert!(registrations.is_empty());
}

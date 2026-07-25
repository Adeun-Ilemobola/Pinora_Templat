use pinora_shared::protocol::{
    command::{IncomingCommand, ModuleCommand, RangefinderCommandPayload, RangefinderDistanceMode},
    global_definitions::RangPoint,
    module_event::{LidarEvent, ModuleEvent},
    registration::{ProtocolMessage, SystemInfo},
};
use serde_json::json;

#[test]
fn command_serialization_matches_the_wire_contract() {
    let command = IncomingCommand {
        id: "rangefinder".to_owned(),
        command: ModuleCommand::Rangefinder(RangefinderCommandPayload::SetDistanceMode {
            mode: RangefinderDistanceMode::Long,
        }),
    };

    let encoded = serde_json::to_value(&command).expect("command should serialize");

    assert_eq!(
        encoded,
        json!({
            "id": "rangefinder",
            "module_type": "Rangefinder",
            "payload": {
                "command": "SetDistanceMode",
                "mode": "Long"
            }
        })
    );
    assert_eq!(
        serde_json::from_value::<IncomingCommand>(encoded)
            .expect("serialized command should deserialize"),
        command
    );
}

#[test]
fn module_event_round_trips_through_the_shared_contract() {
    let message = ProtocolMessage::ModuleEvent(ModuleEvent::Lidar(LidarEvent::PointMap {
        id: "lidar".to_owned(),
        max_chunk: 2,
        curr_chunk: 1,
        map: vec![RangPoint {
            x: -12,
            y: 7,
            distant: 1_250,
        }],
    }));

    let encoded = serde_json::to_string(&message).expect("event should serialize");
    assert!(encoded.contains(r#""event_type":"RangPoint""#));

    let decoded =
        serde_json::from_str::<ProtocolMessage>(&encoded).expect("event should deserialize");

    assert_eq!(decoded, message);
}

#[test]
fn system_info_serialization_matches_the_wire_contract() {
    let message = ProtocolMessage::System(SystemInfo {
        esp_idf_version: "v5.5.1".to_owned(),
        total_heap: "320.00 KiB (327680 bytes)".to_owned(),
        current_free_heap: "200.00 KiB (204800 bytes)".to_owned(),
        lowest_free_heap: "180.00 KiB (184320 bytes)".to_owned(),
        largest_allocation: "128.00 KiB (131072 bytes)".to_owned(),
        maximum_app_slot: "1.88 MiB (1966080 bytes)".to_owned(),
        flash: "4.00 MiB (4194304 bytes)".to_owned(),
    });

    let encoded = serde_json::to_value(&message).expect("system info should serialize");

    assert_eq!(
        encoded,
        json!({
            "type": "System",
            "payload": {
                "esp_idf_version": "v5.5.1",
                "total_heap": "320.00 KiB (327680 bytes)",
                "current_free_heap": "200.00 KiB (204800 bytes)",
                "lowest_free_heap": "180.00 KiB (184320 bytes)",
                "largest_allocation": "128.00 KiB (131072 bytes)",
                "maximum_app_slot": "1.88 MiB (1966080 bytes)",
                "flash": "4.00 MiB (4194304 bytes)"
            }
        })
    );
    assert_eq!(
        serde_json::from_value::<ProtocolMessage>(encoded)
            .expect("system info should deserialize"),
        message
    );
}

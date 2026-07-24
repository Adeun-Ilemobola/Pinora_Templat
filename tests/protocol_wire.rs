use pinora_shared::protocol::{
    command::{IncomingCommand, ModuleCommand, RangefinderCommandPayload, RangefinderDistanceMode},
    global_definitions::RangPoint,
    module_event::{LidarEvent, ModuleEvent},
    registration::ProtocolMessage,
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

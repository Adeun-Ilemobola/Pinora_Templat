//! Characterization tests for the `Module` contract in
//! `Firmware_Templates/src/core/module.rs`, using a hardware-free stub module.

use std::collections::HashMap;

use pinora_firmware_host_tests::core::module::{
    Module, ModuleCore, ModuleError, ModuleHandle, ModuleRegistry,
};
use pinora_firmware_host_tests::core::transport::emitter::TransportEmitter;
use pinora_protocol::*;

struct StubModule {
    core: ModuleCore,
    ticks: u32,
    commands: Vec<ModuleCommand>,
}

impl StubModule {
    fn new(name: &str, parent: Option<String>, emitter: TransportEmitter) -> Self {
        Self {
            core: ModuleCore::new(ModuleType::Led, name, parent, emitter),
            ticks: 0,
            commands: Vec::new(),
        }
    }
}

impl Module for StubModule {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn tick(&mut self) -> Result<(), ModuleError> {
        self.ticks += 1;
        Ok(())
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        self.commands.push(command.clone());
        Ok(())
    }
}

#[test]
fn module_ids_are_fresh_uuid_v4_values() {
    let (emitter, _rx) = TransportEmitter::new();
    let a = StubModule::new("led1", None, emitter.clone());
    let b = StubModule::new("led1", None, emitter);
    let parsed = uuid::Uuid::parse_str(a.id()).expect("id is a UUID");
    assert_eq!(parsed.get_version_num(), 4);
    assert_ne!(a.id(), b.id(), "ids are regenerated per construction");
}

#[test]
fn register_sends_the_registration_with_lookup_name_and_empty_parent() {
    let (emitter, rx) = TransportEmitter::new();
    let module = StubModule::new("led1", None, emitter);
    module.register().unwrap();
    assert_eq!(
        rx.try_recv().unwrap(),
        ProtocolMessage::Registration(Registration {
            id: module.id().to_string(),
            module_type: ModuleType::Led,
            lool_up_id: "led1".into(),
            parent_id: String::new(),
        })
    );
    assert!(rx.try_recv().is_err(), "register sends exactly one message");
}

#[test]
fn register_carries_the_parent_id_for_children() {
    let (emitter, rx) = TransportEmitter::new();
    let module = StubModule::new("servo_x", Some("parent-uuid".into()), emitter);
    module.register().unwrap();
    match rx.try_recv().unwrap() {
        ProtocolMessage::Registration(r) => assert_eq!(r.parent_id, "parent-uuid"),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn default_first_emit_sends_nothing() {
    let (emitter, rx) = TransportEmitter::new();
    let mut module = StubModule::new("led1", None, emitter);
    module.first_emit().unwrap();
    assert!(rx.try_recv().is_err());
}

#[test]
fn emit_and_log_wrap_events_with_the_module_id() {
    let (emitter, rx) = TransportEmitter::new();
    let module = StubModule::new("led1", None, emitter);
    module.emit(ModuleEvent::Led(LedEvent::Brightness { level: 7 }));
    let log = SysLogEvent {
        text: "t".into(),
        raw_err: None,
        priority: LogPriority::Low,
    };
    module.log(log.clone());
    assert_eq!(
        rx.try_iter().collect::<Vec<_>>(),
        vec![
            ProtocolMessage::ModuleEvent(EventPackage {
                id: module.id().to_string(),
                event: ModuleEvent::Led(LedEvent::Brightness { level: 7 }),
            }),
            ProtocolMessage::ModuleEvent(EventPackage {
                id: module.id().to_string(),
                event: ModuleEvent::SysLog(log),
            }),
        ]
    );
}

#[test]
fn unit_error_converts_to_operation_failed() {
    assert_eq!(ModuleError::from(()), ModuleError::OperationFailed);
}

#[test]
fn registry_routes_by_uuid() {
    let (emitter, _rx) = TransportEmitter::new();
    let module = StubModule::new("led1", None, emitter);
    let id = module.id().to_string();
    let mut registry: ModuleRegistry<'_> = HashMap::new();
    let handle: ModuleHandle<'_> = Box::new(module);
    registry.insert(id.clone(), handle);
    let command = ModuleCommand::Led(LedCommandPayload::Toggle {});
    registry
        .get_mut(&id)
        .unwrap()
        .handle_command(&command)
        .unwrap();
    registry.get_mut(&id).unwrap().tick().unwrap();
    assert!(
        !registry.contains_key("led1"),
        "lookup names are not registry keys"
    );
}

#[test]
fn duplicate_lookup_names_are_detected() {
    use pinora_firmware_host_tests::core::module::duplicate_lookup_names;
    let (emitter, _rx) = TransportEmitter::new();
    let mut registry: ModuleRegistry<'_> = HashMap::new();
    for name in ["led1", "led2", "led1", "led3", "led2"] {
        let module = StubModule::new(name, None, emitter.clone());
        registry.insert(module.id().to_string(), Box::new(module));
    }
    assert_eq!(duplicate_lookup_names(&registry), vec!["led1", "led2"]);
    registry.clear();
    let module = StubModule::new("only", None, emitter);
    registry.insert(module.id().to_string(), Box::new(module));
    assert!(duplicate_lookup_names(&registry).is_empty());
}

#[test]
fn modules_have_no_children_by_default() {
    let (emitter, _rx) = TransportEmitter::new();
    let mut module = StubModule::new("led1", None, emitter);
    let id = module.id().to_string();
    assert!(module.child_mut(&id).is_none());
}

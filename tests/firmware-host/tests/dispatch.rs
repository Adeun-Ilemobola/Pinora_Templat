//! Tests for `Firmware_Templates/src/runtime/dispatch.rs` (Phase 1B, PIN-TPL-001):
//! a failing command is reported and does not stop command processing.

use std::sync::mpsc::Receiver;

use pinora_firmware_host_tests::core::module::{Module, ModuleCore, ModuleError, ModuleRegistry};
use pinora_firmware_host_tests::core::transport::emitter::TransportEmitter;
use pinora_firmware_host_tests::runtime::dispatch::{dispatch, DispatchOutcome, RESERVED_SSI_ID};
use pinora_protocol::*;

/// Fails on `Toggle`, succeeds on `SetState`, and counts what it handled.
struct FlakyModule {
    core: ModuleCore,
    handled: u32,
}

impl Module for FlakyModule {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn tick(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Led(LedCommandPayload::Toggle {}) => anyhow::bail!("I2C NACK"),
            _ => {
                self.handled += 1;
                Ok(())
            }
        }
    }
}

fn setup() -> (ModuleRegistry<'static>, String, Receiver<ProtocolMessage>) {
    let (emitter, rx) = TransportEmitter::new();
    let module = FlakyModule {
        core: ModuleCore::new(ModuleType::Led, "led1", None, emitter),
        handled: 0,
    };
    let id = module.id().to_string();
    let mut registry: ModuleRegistry<'static> = ModuleRegistry::new();
    registry.insert(id.clone(), Box::new(module));
    (registry, id, rx)
}

fn command(id: &str, command: ModuleCommand) -> IncomingCommand {
    IncomingCommand {
        id: id.to_string(),
        command,
    }
}

fn set_state(id: &str) -> IncomingCommand {
    command(
        id,
        ModuleCommand::Led(LedCommandPayload::SetState { state: 1 }),
    )
}

fn toggle(id: &str) -> IncomingCommand {
    command(id, ModuleCommand::Led(LedCommandPayload::Toggle {}))
}

#[test]
fn successful_command_is_handled_without_messages() {
    let (mut registry, id, rx) = setup();
    assert_eq!(
        dispatch(&mut registry, &set_state(&id)),
        DispatchOutcome::Handled
    );
    assert!(rx.try_recv().is_err());
}

#[test]
fn failing_command_is_reported_as_a_high_syslog_from_the_module() {
    let (mut registry, id, rx) = setup();
    assert_eq!(
        dispatch(&mut registry, &toggle(&id)),
        DispatchOutcome::Failed
    );
    match rx.try_recv().unwrap() {
        ProtocolMessage::ModuleEvent(EventPackage {
            id: source,
            event: ModuleEvent::SysLog(log),
        }) => {
            assert_eq!(source, id);
            assert_eq!(log.priority, LogPriority::High);
            assert_eq!(log.text, "Command failed: I2C NACK");
            assert!(log.raw_err.unwrap().contains("I2C NACK"));
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(rx.try_recv().is_err(), "exactly one report");
}

#[test]
fn commands_after_a_failure_are_still_processed() {
    // The PIN-TPL-001 regression: previously the first failure ended `main`.
    let (mut registry, id, _rx) = setup();
    assert_eq!(
        dispatch(&mut registry, &toggle(&id)),
        DispatchOutcome::Failed
    );
    assert_eq!(
        dispatch(&mut registry, &set_state(&id)),
        DispatchOutcome::Handled
    );
    assert_eq!(
        dispatch(&mut registry, &toggle(&id)),
        DispatchOutcome::Failed
    );
    assert_eq!(
        dispatch(&mut registry, &set_state(&id)),
        DispatchOutcome::Handled
    );
}

#[test]
fn unknown_module_is_not_reported_to_the_desktop() {
    let (mut registry, _id, rx) = setup();
    assert_eq!(
        dispatch(&mut registry, &set_state("no-such-module")),
        DispatchOutcome::UnknownModule
    );
    assert!(rx.try_recv().is_err());
}

#[test]
fn reserved_ssi_id_is_accepted_and_ignored() {
    let (mut registry, _id, rx) = setup();
    assert_eq!(
        dispatch(&mut registry, &set_state(RESERVED_SSI_ID)),
        DispatchOutcome::Reserved
    );
    assert!(rx.try_recv().is_err());
}

// ---- Phase 2A: system commands and child routing (PIN-TPL-012, PIN-TPL-017) ----

#[test]
fn rediscover_on_the_reserved_id_is_returned_to_the_main_loop() {
    let (mut registry, _id, rx) = setup();
    let rediscover = command(
        RESERVED_SSI_ID,
        ModuleCommand::System(SystemCommand::Rediscover {}),
    );
    assert_eq!(
        dispatch(&mut registry, &rediscover),
        DispatchOutcome::Rediscover
    );
    assert!(
        rx.try_recv().is_err(),
        "the loop, not dispatch, sends discovery"
    );
}

#[test]
fn system_commands_sent_to_a_module_are_ignored_by_it() {
    let (mut registry, id, rx) = setup();
    let system = command(&id, ModuleCommand::System(SystemCommand::Rediscover {}));
    assert_eq!(dispatch(&mut registry, &system), DispatchOutcome::Handled);
    assert!(rx.try_recv().is_err());
}

/// A composite module that owns one `FlakyModule` child.
struct Parent {
    core: ModuleCore,
    child: FlakyModule,
}

impl Module for Parent {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn tick(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn handle_command(&mut self, _command: &ModuleCommand) -> anyhow::Result<()> {
        anyhow::bail!("the parent should not receive child commands")
    }
    fn child_mut(&mut self, id: &str) -> Option<&mut dyn Module> {
        (self.child.id() == id).then_some(&mut self.child as &mut dyn Module)
    }
}

fn setup_parent() -> (ModuleRegistry<'static>, String, Receiver<ProtocolMessage>) {
    let (emitter, rx) = TransportEmitter::new();
    let parent = Parent {
        core: ModuleCore::new(ModuleType::Lidar, "lidar", None, emitter.clone()),
        child: FlakyModule {
            core: ModuleCore::new(ModuleType::Servo, "servo_x", None, emitter),
            handled: 0,
        },
    };
    let child_id = parent.child.id().to_string();
    let mut registry: ModuleRegistry<'static> = ModuleRegistry::new();
    registry.insert(parent.id().to_string(), Box::new(parent));
    (registry, child_id, rx)
}

#[test]
fn commands_to_a_child_id_reach_the_child() {
    // Before Phase 2A this was "No top-level module found".
    let (mut registry, child_id, _rx) = setup_parent();
    assert_eq!(
        dispatch(&mut registry, &set_state(&child_id)),
        DispatchOutcome::Handled
    );
}

#[test]
fn a_failing_child_command_is_reported_from_the_child() {
    let (mut registry, child_id, rx) = setup_parent();
    assert_eq!(
        dispatch(&mut registry, &toggle(&child_id)),
        DispatchOutcome::Failed
    );
    match rx.try_recv().unwrap() {
        ProtocolMessage::ModuleEvent(EventPackage {
            id,
            event: ModuleEvent::SysLog(_),
        }) => {
            assert_eq!(id, child_id)
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn ids_that_match_neither_a_module_nor_a_child_are_unknown() {
    let (mut registry, _child_id, _rx) = setup_parent();
    assert_eq!(
        dispatch(&mut registry, &set_state("nope")),
        DispatchOutcome::UnknownModule
    );
}

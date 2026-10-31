//! Tests for `Firmware_Templates/src/runtime/dispatch.rs` (Phase 1B, PIN-TPL-001):
//! a failing command is reported and does not stop command processing.

use std::collections::HashMap;
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
    let mut registry: ModuleRegistry<'static> = HashMap::new();
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

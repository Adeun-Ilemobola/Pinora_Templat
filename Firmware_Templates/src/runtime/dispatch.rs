//! Routes one `IncomingCommand` to its module.
//!
//! Routing order (since Phase 2A):
//! 1. The reserved id `"SSI"` carries system commands; `Rediscover` is returned to the
//!    main loop, which re-sends discovery and current state. Anything else sent to
//!    `"SSI"` is ignored.
//! 2. A top-level module with this id.
//! 3. A child of a composite module with this id ([`Module::child_mut`], PIN-TPL-017).
//!
//! A handler error is logged and reported to the desktop as a `SysLog` from the
//! failing module. It does **not** stop the firmware (PIN-TPL-001, Phase 1B).
//!
//! Pure (no ESP-IDF), so it is tested on the host (`tests/firmware-host`).

use pinora_protocol::{
    IncomingCommand, LogPriority, ModuleCommand, SysLogEvent, SystemCommand, SYSTEM_COMMAND_ID,
};

use crate::core::module::{Module, ModuleRegistry};

/// Reserved command id: system commands only (contract C-05).
pub const RESERVED_SSI_ID: &str = SYSTEM_COMMAND_ID;

/// What happened to a dispatched command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// The module (top-level or child) handled the command.
    Handled,
    /// The module's handler returned an error, which was reported as a `SysLog`.
    Failed,
    /// `Rediscover` was requested; the caller re-sends discovery and current state.
    Rediscover,
    /// Something other than a system command was sent to the reserved id; ignored.
    Reserved,
    /// No module or child has this id; logged on the console only.
    UnknownModule,
}

pub fn dispatch(modules: &mut ModuleRegistry<'_>, command: &IncomingCommand) -> DispatchOutcome {
    if command.id == RESERVED_SSI_ID {
        return match command.command {
            ModuleCommand::System(SystemCommand::Rediscover {}) => DispatchOutcome::Rediscover,
            _ => DispatchOutcome::Reserved,
        };
    }

    if let Some(module) = modules.get_mut(&command.id) {
        return handle(module.as_mut(), command);
    }

    for module in modules.values_mut() {
        if let Some(child) = module.child_mut(&command.id) {
            return handle(child, command);
        }
    }

    log::error!(
        "No module found for command id={} command={:?}",
        command.id,
        command.command
    );
    DispatchOutcome::UnknownModule
}

fn handle(module: &mut dyn Module, command: &IncomingCommand) -> DispatchOutcome {
    match module.handle_command(&command.command) {
        Ok(()) => DispatchOutcome::Handled,
        Err(error) => {
            log::error!(
                "Command failed for module id={} command={:?}: {error:#}",
                command.id,
                command.command
            );
            module.log(SysLogEvent {
                text: format!("Command failed: {error}"),
                raw_err: Some(format!("{error:?}")),
                priority: LogPriority::High,
            });
            DispatchOutcome::Failed
        }
    }
}

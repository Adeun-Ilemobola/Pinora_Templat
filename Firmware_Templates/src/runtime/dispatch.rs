//! Routes one `IncomingCommand` to its module.
//!
//! A handler error is logged and reported to the desktop as a `SysLog` from the
//! failing module. It does **not** stop the firmware (PIN-TPL-001, Phase 1B).
//! Before Phase 1B the error was returned with `?`, which ended `main` and silently
//! stopped all command processing.
//!
//! Pure (no ESP-IDF), so it is tested on the host (`tests/firmware-host`).

use pinora_protocol::{IncomingCommand, LogPriority, SysLogEvent};

use crate::core::module::ModuleRegistry;

/// Reserved command id: accepted and ignored (contract C-05).
pub const RESERVED_SSI_ID: &str = "SSI";

/// What happened to a dispatched command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// The module handled the command.
    Handled,
    /// The module's handler returned an error, which was reported as a `SysLog`.
    Failed,
    /// The command used the reserved id and was ignored.
    Reserved,
    /// No top-level module has this id; logged on the console only.
    UnknownModule,
}

pub fn dispatch(modules: &mut ModuleRegistry<'_>, command: &IncomingCommand) -> DispatchOutcome {
    if let Some(module) = modules.get_mut(&command.id) {
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
    } else if command.id == RESERVED_SSI_ID {
        DispatchOutcome::Reserved
    } else {
        log::error!(
            "No top-level module found for command id={} command={:?}",
            command.id,
            command.command
        );
        DispatchOutcome::UnknownModule
    }
}

//! The firmware main loop: registration, then tick and dispatch forever.
//!
//! Scheduling is cooperative busy-polling. Each iteration ticks every module (in
//! `HashMap` order), handles at most one command, and yields to FreeRTOS for 1 ms
//! only after at least 650 ms without a yield (PIN-TPL-009).
//!
//! Error semantics (contract C-16, documented here and not changed in Phase 0):
//! * `tick()` errors are discarded.
//! * A `handle_command()` error is returned with `?`, which ends `main` and so stops
//!   all command processing without rebooting (PIN-TPL-001).

use std::sync::mpsc::Receiver;

use pinora_protocol::IncomingCommand;

use crate::core::hardware::{now_us, rtos_sleep_ms};
use crate::core::module::ModuleRegistry;

/// Sends every module's registration, then its `first_emit` events, in registry order.
pub fn register_all(modules: &mut ModuleRegistry<'_>) -> anyhow::Result<()> {
    for (_id, module) in modules.iter_mut() {
        module.register()?;
        module.first_emit()?;
    }

    // for module in modules.values() {
    //     module.register()?;
    //     module.first_emit()?;
    // }

    Ok(())
}

/// Runs the main loop. `last_yield_us` is the timestamp taken in `main` before the
/// hardware was set up, so the first yield happens exactly when it did before.
///
/// Commands addressed to `"SSI"` are accepted and ignored (reserved, contract C-05).
pub fn run(
    modules: &mut ModuleRegistry<'_>,
    command_receiver: Receiver<IncomingCommand>,
    mut last_yield_us: f64,
) -> anyhow::Result<()> {
    loop {
        for module in modules.values_mut() {
            let _ = module.tick();
        }

        if let Ok(command) = command_receiver.try_recv() {
            if let Some(module) = modules.get_mut(&command.id) {
                module.handle_command(&command.command)?;
            } else if command.id == "SSI" {
            } else {
                log::error!(
                    "No top-level module found for command id={} command={:?}",
                    command.id,
                    command.command
                );
            }
        }
        let now = now_us();

        if now - last_yield_us >= 650_000.0 {
            rtos_sleep_ms(1);
            last_yield_us = now_us();
        }
    }
}

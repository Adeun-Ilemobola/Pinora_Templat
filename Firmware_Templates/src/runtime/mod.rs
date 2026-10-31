//! The firmware main loop: registration, then tick and dispatch forever.
//!
//! Scheduling is cooperative busy-polling. Each iteration ticks every module (in
//! `HashMap` order), handles at most one command, and yields to FreeRTOS for 1 ms
//! only after at least 650 ms without a yield (PIN-TPL-009).
//!
//! Error semantics (contract C-16, changed in Phase 1B, PIN-TPL-001):
//! * `tick()` errors are reported as rate-limited `SysLog`s ([`tick_errors`]).
//! * A `handle_command()` error is reported as a `SysLog` and the loop continues
//!   ([`dispatch`]). Previously it ended `main` and stopped all command processing.

pub mod dispatch;
pub mod tick_errors;

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

/// Runs the main loop forever. `last_yield_us` is the timestamp taken in `main`
/// before the hardware was set up.
///
/// Commands addressed to `"SSI"` are accepted and ignored (reserved, contract C-05).
pub fn run(
    modules: &mut ModuleRegistry<'_>,
    command_receiver: Receiver<IncomingCommand>,
    mut last_yield_us: f64,
) -> ! {
    let mut tick_errors = tick_errors::TickErrorReporter::new();

    loop {
        for module in modules.values_mut() {
            if let Err(error) = module.tick() {
                // The clock is read only when an error occurs.
                tick_errors.record(module.as_ref(), error, now_us());
            }
        }

        if let Ok(command) = command_receiver.try_recv() {
            dispatch::dispatch(modules, &command);
        }
        let now = now_us();

        if now - last_yield_us >= 650_000.0 {
            rtos_sleep_ms(1);
            last_yield_us = now_us();
        }
    }
}

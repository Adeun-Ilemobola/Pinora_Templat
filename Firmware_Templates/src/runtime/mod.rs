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
//!
//! Rediscovery (Phase 2A, PIN-TPL-012): a `Rediscover` system command makes the loop
//! re-send a fresh system snapshot, every registration and every module's current
//! state (`first_emit`). The desktop sends it after each connect, so even the
//! Serial transport (which has no replay cache) is rediscovered.

pub mod dispatch;
pub mod tick_errors;

use std::sync::mpsc::Receiver;

use pinora_protocol::IncomingCommand;

use pinora_protocol::{LogPriority, SysLogEvent};

use crate::core::hardware::{now_us, print_esp_system_info, rtos_sleep_ms};
use crate::core::module::{duplicate_lookup_names, ModuleRegistry};
use crate::core::transport::emitter::TransportEmitter;

/// Sends every module's registration, then its `first_emit` events, in registry order.
/// Modules sharing a lookup name are reported, because the desktop maps lookup names
/// to modules (PIN-TPL-017).
pub fn register_all(modules: &mut ModuleRegistry<'_>) -> anyhow::Result<()> {
    let duplicates = duplicate_lookup_names(modules);
    for (_id, module) in modules.iter_mut() {
        module.register()?;
        module.first_emit()?;
        let name = &module.core().lookup_name;
        if duplicates.contains(name) {
            log::warn!("Duplicate module lookup name {name:?}");
            module.log(SysLogEvent {
                text: format!("Duplicate module lookup name \"{name}\""),
                raw_err: None,
                priority: LogPriority::Medium,
            });
        }
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
/// `"SSI"` is reserved for system commands (contract C-05); `Rediscover` triggers
/// [`rediscover`] and anything else sent there is ignored.
pub fn run(
    modules: &mut ModuleRegistry<'_>,
    command_receiver: Receiver<IncomingCommand>,
    mut last_yield_us: f64,
    emitter: TransportEmitter,
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
            if dispatch::dispatch(modules, &command) == dispatch::DispatchOutcome::Rediscover {
                rediscover(modules, &emitter);
            }
        }
        let now = now_us();

        if now - last_yield_us >= 650_000.0 {
            rtos_sleep_ms(1);
            last_yield_us = now_us();
        }
    }
}

/// Re-sends a fresh system snapshot, every registration and every module's current
/// state. Failures are logged; the loop continues.
fn rediscover(modules: &mut ModuleRegistry<'_>, emitter: &TransportEmitter) {
    log::info!("Rediscovery requested");
    if let Err(error) = print_esp_system_info(emitter.clone()) {
        log::error!("Rediscovery: could not send the system snapshot: {error}");
    }
    if let Err(error) = register_all(modules) {
        log::error!("Rediscovery: could not send registrations: {error}");
    }
}

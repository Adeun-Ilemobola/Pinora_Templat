//! The firmware main loop: registration, then tick and dispatch forever.
//!
//! Scheduling (since Phase 2C, PIN-TPL-009): a fixed 1 ms period ([`schedule`]).
//! Each iteration ticks every module in construction order (PIN-TPL-027), handles the
//! pending commands, then blocks on the command channel until the next deadline, so
//! the idle task runs and a command wakes the loop at once. Before 2C the loop
//! busy-polled and yielded 1 ms only every 650 ms. Loop period and work time are
//! logged on the console every 10 s.
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
pub mod schedule;
pub mod tick_errors;

use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::Duration;

use pinora_protocol::IncomingCommand;

use pinora_protocol::{LogPriority, SysLogEvent};

use crate::core::hardware::{now_us, print_esp_system_info, rtos_sleep_ms};
use crate::core::module::{duplicate_lookup_names, ModuleRegistry};
use crate::core::transport::emitter::TransportEmitter;
use schedule::{
    next_deadline, plan_wait, wait_us, LoopStats, Wait, MAX_COMMANDS_PER_PERIOD, PERIOD_US,
};

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

/// Runs the main loop forever (see the module docs for the schedule).
///
/// `"SSI"` is reserved for system commands (contract C-05); `Rediscover` triggers
/// [`rediscover`] and anything else sent there is ignored.
pub fn run(
    modules: &mut ModuleRegistry<'_>,
    command_receiver: Receiver<IncomingCommand>,
    emitter: TransportEmitter,
) -> ! {
    let mut tick_errors = tick_errors::TickErrorReporter::new();
    let mut stats = LoopStats::default();
    let clock = || now_us() as u64;
    let mut deadline = clock() + PERIOD_US;

    loop {
        let started = clock();
        stats.iteration_started(started);

        for module in modules.values_mut() {
            if let Err(error) = module.tick() {
                // The clock is read only when an error occurs.
                tick_errors.record(module.as_ref(), error, now_us());
            }
        }
        for _ in 0..MAX_COMMANDS_PER_PERIOD {
            match command_receiver.try_recv() {
                Ok(command) => handle(modules, &command, &emitter),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        stats.work_done(clock().saturating_sub(started));
        if let Some(report) = stats.take_report(clock()) {
            log::info!("{report}");
        }

        // Wait for the next period; while blocking, a command wakes the loop early.
        loop {
            match plan_wait(wait_us(deadline, clock())) {
                Wait::None => break,
                Wait::NextTick => {
                    rtos_sleep_ms(1);
                    break;
                }
                Wait::Block(micros) => {
                    match command_receiver.recv_timeout(Duration::from_micros(micros)) {
                        Ok(command) => handle(modules, &command, &emitter),
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => {
                            // No command source left: just keep the period.
                            rtos_sleep_ms(1);
                            break;
                        }
                    }
                }
            }
        }
        deadline = next_deadline(deadline, clock(), PERIOD_US);
    }
}

fn handle(modules: &mut ModuleRegistry<'_>, command: &IncomingCommand, emitter: &TransportEmitter) {
    if dispatch::dispatch(modules, command) == dispatch::DispatchOutcome::Rediscover {
        rediscover(modules, emitter);
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

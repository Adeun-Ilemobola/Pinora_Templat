//! Rate-limited reporting of `tick()` errors (PIN-TPL-001, Phase 1B).
//!
//! Before Phase 1B tick errors were discarded silently. Now the first error from a
//! module is reported at once as a `SysLog`. After that, a module is reported at most
//! once per [`REPORT_INTERVAL_US`], and each report says how many errors were
//! suppressed since the previous one, so a sensor failing on every tick cannot flood
//! the transport queues.
//!
//! Pure (no ESP-IDF): the caller passes the current time, so this is tested on the
//! host (`tests/firmware-host`).

use std::collections::HashMap;

use pinora_protocol::{LogPriority, SysLogEvent};

use crate::core::module::{Module, ModuleError};

/// Minimum time between two reports for the same module (5 s).
pub const REPORT_INTERVAL_US: f64 = 5_000_000.0;

struct ModuleState {
    last_report_us: f64,
    suppressed: u32,
}

/// Per-module error bookkeeping. Allocates only once a module has failed.
#[derive(Default)]
pub struct TickErrorReporter {
    per_module: HashMap<String, ModuleState>,
}

impl TickErrorReporter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one tick error from `module` at time `now_us`. Returns `true` when a
    /// report was sent.
    pub fn record(&mut self, module: &dyn Module, error: ModuleError, now_us: f64) -> bool {
        match self.per_module.get_mut(module.id()) {
            Some(state) if now_us - state.last_report_us < REPORT_INTERVAL_US => {
                state.suppressed = state.suppressed.saturating_add(1);
                false
            }
            Some(state) => {
                let suppressed = state.suppressed;
                state.last_report_us = now_us;
                state.suppressed = 0;
                report(module, error, suppressed);
                true
            }
            None => {
                self.per_module.insert(
                    module.id().to_string(),
                    ModuleState {
                        last_report_us: now_us,
                        suppressed: 0,
                    },
                );
                report(module, error, 0);
                true
            }
        }
    }
}

fn report(module: &dyn Module, error: ModuleError, suppressed: u32) {
    let text = if suppressed == 0 {
        format!("Tick failed: {error:?}")
    } else {
        format!("Tick failed: {error:?} ({suppressed} more since last report)")
    };
    log::warn!("Module id={}: {text}", module.id());
    module.log(SysLogEvent {
        text,
        raw_err: None,
        priority: LogPriority::Medium,
    });
}

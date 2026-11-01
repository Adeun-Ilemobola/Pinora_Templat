//! Counts runtime messages dropped at each outbound stage and summarizes them
//! periodically (audit Phase 2B, PIN-TPL-010).
//!
//! Before 2B every drop logged its own warning on the console, which itself added to
//! UART pressure under load, and the desktop was never told. Now each stage only
//! increments a counter; the sender thread reports the totals as one SysLog every
//! [`REPORT_INTERVAL`] when anything was dropped.
//!
//! Pure (atomics only), so it is tested on the host.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// How often non-zero drop counts are reported.
pub const REPORT_INTERVAL: Duration = Duration::from_secs(5);

/// The outbound stage where a message was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// The [`super::emitter::TransportEmitter`] queue (all transports).
    Emitter,
    /// The selected transport's work queue (Bluetooth, Wi-Fi).
    Transport,
    /// The link itself: the Bluetooth pending-frame queue, or a frame over the size
    /// limit.
    Link,
}

#[derive(Debug, Default)]
pub struct DropCounters {
    emitter: AtomicU32,
    transport: AtomicU32,
    link: AtomicU32,
}

/// Drops since the previous report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DropReport {
    pub emitter: u32,
    pub transport: u32,
    pub link: u32,
}

impl DropCounters {
    pub fn record(&self, stage: Stage) {
        let counter = match stage {
            Stage::Emitter => &self.emitter,
            Stage::Transport => &self.transport,
            Stage::Link => &self.link,
        };
        // Saturating, so a long flood cannot wrap to a small number.
        let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
            Some(count.saturating_add(1))
        });
    }

    /// Returns and resets the counts; `None` if nothing was dropped.
    pub fn take(&self) -> Option<DropReport> {
        let report = DropReport {
            emitter: self.emitter.swap(0, Ordering::Relaxed),
            transport: self.transport.swap(0, Ordering::Relaxed),
            link: self.link.swap(0, Ordering::Relaxed),
        };
        (report.total() > 0).then_some(report)
    }
}

impl DropReport {
    pub fn total(&self) -> u64 {
        u64::from(self.emitter) + u64::from(self.transport) + u64::from(self.link)
    }

    /// The SysLog text.
    pub fn describe(&self) -> String {
        format!(
            "Telemetry dropped under load: {} message(s) (emitter queue {}, transport queue {}, link {})",
            self.total(),
            self.emitter,
            self.transport,
            self.link
        )
    }
}

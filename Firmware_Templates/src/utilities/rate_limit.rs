//! Per-source emission rate limiting (audit Phase 2B, PIN-TPL-010).
//!
//! A [`RateLimiter`] admits at most one event per interval and counts what it
//! suppressed, so a module can say "N similar errors suppressed". For sampled values,
//! call [`RateLimiter::admit`] only when there is something new to send and keep the
//! latest value: the next admitted emission then carries the current value
//! (coalescing), and nothing is lost except intermediate samples.
//!
//! Pure; times are microseconds from the caller's clock, so it is tested on the host.

/// Outcome of [`RateLimiter::admit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// Emit now. `suppressed` events were refused since the last admitted one.
    Admit {
        suppressed: u32,
    },
    Suppress,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    interval_us: u64,
    next_us: Option<u64>,
    suppressed: u32,
}

impl RateLimiter {
    /// At most one event per `interval_us` microseconds.
    pub fn new(interval_us: u64) -> Self {
        Self {
            interval_us,
            next_us: None,
            suppressed: 0,
        }
    }

    /// At most `per_second` events per second (`per_second` ≥ 1).
    pub fn per_second(per_second: u32) -> Self {
        Self::new(1_000_000 / u64::from(per_second.max(1)))
    }

    pub fn admit(&mut self, now_us: u64) -> Admission {
        match self.next_us {
            Some(next) if now_us < next => {
                self.suppressed = self.suppressed.saturating_add(1);
                Admission::Suppress
            }
            _ => {
                self.next_us = Some(now_us.saturating_add(self.interval_us));
                Admission::Admit {
                    suppressed: std::mem::take(&mut self.suppressed),
                }
            }
        }
    }
}

/// Appends " (N similar suppressed)" when `suppressed` > 0.
pub fn with_suppressed(text: &str, suppressed: u32) -> String {
    if suppressed == 0 {
        text.to_string()
    } else {
        format!("{text} ({suppressed} similar suppressed)")
    }
}

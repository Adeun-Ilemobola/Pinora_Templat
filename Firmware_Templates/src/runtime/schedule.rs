//! The main loop's fixed-rate schedule and its measurements (audit Phase 2C,
//! PIN-TPL-009).
//!
//! The loop runs one iteration per [`PERIOD_US`]: tick every module, handle pending
//! commands, then block on the command channel until the next deadline. A command
//! that arrives while the loop waits wakes it at once. A late iteration does not
//! replay missed periods: the next deadline moves to one period after "now".
//!
//! [`LoopStats`] records the period between iteration starts and the time spent
//! ticking, as a 100 µs histogram, and summarizes it every [`REPORT_INTERVAL_US`].
//!
//! Pure; times are microseconds from the caller's clock, so it is tested on the host.

/// Loop period: 1 ms (needs `CONFIG_FREERTOS_HZ=1000` for the blocking wait).
pub const PERIOD_US: u64 = 1_000;
/// Most commands handled before ticking again, so a command flood cannot starve ticks.
pub const MAX_COMMANDS_PER_PERIOD: usize = 16;
/// How often [`LoopStats`] is summarized.
pub const REPORT_INTERVAL_US: u64 = 10_000_000;

/// The deadline after `deadline`, given that the iteration finished at `now`.
pub fn next_deadline(deadline: u64, now: u64, period: u64) -> u64 {
    let next = deadline.saturating_add(period);
    if next <= now {
        // Overran by more than a period: skip the missed periods instead of bursting.
        now.saturating_add(period)
    } else {
        next
    }
}

/// Microseconds to wait before `deadline` (0 when it has passed).
pub fn wait_us(deadline: u64, now: u64) -> u64 {
    deadline.saturating_sub(now)
}

/// FreeRTOS tick at `CONFIG_FREERTOS_HZ=1000`.
pub const TICK_US: u64 = 1_000;

/// How the loop waits for the next deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wait {
    /// The deadline has passed.
    None,
    /// Block on the command channel for this long (a command wakes the loop).
    Block(u64),
    /// Sleep until the next tick boundary (`vTaskDelay(1)`).
    NextTick,
}

/// Plans the wait. ESP-IDF's timed waits round up to whole ticks and then add one
/// tick (`pthread_cond_timedwait`), so a wait of `w` µs lasts up to `w` + 2 ticks.
/// Measured on the board: a 1 ms loop that blocked for the full remaining time ran at
/// a 1.83 ms mean period. So a blocking wait asks for one tick less, and less than one
/// tick left is spent sleeping to the next tick boundary, which keeps iterations on
/// the tick grid; a command then waits at most one tick.
pub fn plan_wait(wait_us: u64) -> Wait {
    match wait_us {
        0 => Wait::None,
        w if w > TICK_US => Wait::Block(w - TICK_US),
        _ => Wait::NextTick,
    }
}

const BUCKET_US: u64 = 100;
const BUCKETS: usize = 200; // 0 … 20 ms; the last bucket holds everything longer.

#[derive(Debug, Clone)]
pub struct Histogram {
    counts: [u32; BUCKETS],
    total: u64,
    sum_us: u64,
    max_us: u64,
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            counts: [0; BUCKETS],
            total: 0,
            sum_us: 0,
            max_us: 0,
        }
    }
}

impl Histogram {
    pub fn record(&mut self, micros: u64) {
        let bucket = ((micros / BUCKET_US) as usize).min(BUCKETS - 1);
        self.counts[bucket] = self.counts[bucket].saturating_add(1);
        self.total += 1;
        self.sum_us = self.sum_us.saturating_add(micros);
        self.max_us = self.max_us.max(micros);
    }

    pub fn count(&self) -> u64 {
        self.total
    }

    pub fn mean_us(&self) -> u64 {
        self.sum_us.checked_div(self.total).unwrap_or(0)
    }

    pub fn max_us(&self) -> u64 {
        self.max_us
    }

    /// Upper bound of the bucket holding the `percent`-th percentile (100 µs
    /// resolution).
    pub fn percentile_us(&self, percent: u64) -> u64 {
        if self.total == 0 {
            return 0;
        }
        let wanted = (self.total * percent).div_ceil(100).max(1);
        let mut seen = 0u64;
        for (bucket, &count) in self.counts.iter().enumerate() {
            seen += u64::from(count);
            if seen >= wanted {
                return (bucket as u64 + 1) * BUCKET_US;
            }
        }
        self.max_us
    }
}

/// Period and work-time statistics of the main loop.
#[derive(Debug, Clone, Default)]
pub struct LoopStats {
    pub period: Histogram,
    pub work: Histogram,
    /// Iterations whose work took longer than one period.
    pub overruns: u64,
    last_start: Option<u64>,
    window_start: Option<u64>,
}

impl LoopStats {
    /// Call at the start of each iteration.
    pub fn iteration_started(&mut self, now: u64) {
        if let Some(last) = self.last_start {
            self.period.record(now.saturating_sub(last));
        }
        self.last_start = Some(now);
        self.window_start.get_or_insert(now);
    }

    /// Call when the iteration's ticks and commands are done.
    pub fn work_done(&mut self, work_us: u64) {
        self.work.record(work_us);
        if work_us > PERIOD_US {
            self.overruns += 1;
        }
    }

    /// The summary line every [`REPORT_INTERVAL_US`], which also resets the
    /// statistics; `None` before that.
    pub fn take_report(&mut self, now: u64) -> Option<String> {
        let start = self.window_start?;
        if now.saturating_sub(start) < REPORT_INTERVAL_US {
            return None;
        }
        let line = self.summary();
        let last_start = self.last_start;
        *self = Self::default();
        self.last_start = last_start;
        self.window_start = Some(now);
        Some(line)
    }

    pub fn summary(&self) -> String {
        format!(
            "Loop: {} iterations; period mean {} µs, p50 ≤{} µs, p99 ≤{} µs, max {} µs; work mean {} µs, p99 ≤{} µs, max {} µs; {} overruns",
            self.period.count(),
            self.period.mean_us(),
            self.period.percentile_us(50),
            self.period.percentile_us(99),
            self.period.max_us(),
            self.work.mean_us(),
            self.work.percentile_us(99),
            self.work.max_us(),
            self.overruns
        )
    }
}

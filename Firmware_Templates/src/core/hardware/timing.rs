//! Time helpers used by the cooperative main loop.
//!
//! There is no scheduler: modules call [`TimerState::ready`] from `tick()` and do
//! their work when a deadline has passed. Actual timing therefore depends on how
//! fast the main loop iterates (see audit finding PIN-TPL-009).

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::sys;

pub fn rtos_sleep_ms(ms: u32) {
    FreeRtos::delay_ms(ms);
}

/// Microseconds since boot from `esp_timer`, as `f64` (exact for ~285 years).
pub fn now_us() -> f64 {
    // SAFETY: `esp_timer_get_time` has no preconditions once ESP-IDF has started.
    unsafe { sys::esp_timer_get_time() as f64 }
}

/// A polled periodic deadline.
pub struct TimerState {
    period_us: f64,
    next_deadline_us: f64,
}

impl TimerState {
    pub fn from_ms(period_ms: f64) -> Self {
        let period_us = period_ms * 1_000.0;
        let now = now_us();

        Self {
            period_us,
            next_deadline_us: now + period_us,
        }
    }

    pub fn ready(&mut self) -> bool {
        let now = now_us();

        if now < self.next_deadline_us {
            return false;
        }

        self.next_deadline_us += self.period_us;

        // Do not rapidly replay hundreds of missed timer intervals.
        if self.next_deadline_us <= now {
            self.next_deadline_us = now + self.period_us;
        }

        true
    }

    pub fn reset(&mut self) {
        self.next_deadline_us = now_us() + self.period_us;
    }
}

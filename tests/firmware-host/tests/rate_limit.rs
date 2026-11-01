//! `utilities/rate_limit.rs`: one event per interval, with a suppressed count
//! (audit Phase 2B, PIN-TPL-010).

use pinora_firmware_host_tests::utilities::rate_limit::*;

#[test]
fn the_first_event_is_admitted() {
    assert_eq!(
        RateLimiter::per_second(1).admit(0),
        Admission::Admit { suppressed: 0 }
    );
}

#[test]
fn events_inside_the_interval_are_suppressed_and_counted() {
    let mut limiter = RateLimiter::new(1_000);
    assert_eq!(limiter.admit(10), Admission::Admit { suppressed: 0 });
    assert_eq!(limiter.admit(500), Admission::Suppress);
    assert_eq!(limiter.admit(1_009), Admission::Suppress);
    assert_eq!(limiter.admit(1_010), Admission::Admit { suppressed: 2 });
    assert_eq!(limiter.admit(5_000), Admission::Admit { suppressed: 0 });
}

#[test]
fn a_flood_is_cut_to_the_configured_rate() {
    // 1 kHz of events for one second at 20/s: exactly 20 admitted.
    let mut limiter = RateLimiter::per_second(20);
    let admitted = (0..1_000u64)
        .filter(|ms| matches!(limiter.admit(ms * 1_000), Admission::Admit { .. }))
        .count();
    assert_eq!(admitted, 20);
}

#[test]
fn per_second_zero_is_treated_as_one() {
    let mut limiter = RateLimiter::per_second(0);
    assert!(matches!(limiter.admit(0), Admission::Admit { .. }));
    assert_eq!(limiter.admit(999_999), Admission::Suppress);
    assert!(matches!(limiter.admit(1_000_000), Admission::Admit { .. }));
}

#[test]
fn the_clock_may_not_wrap_the_deadline() {
    // The deadline saturates at u64::MAX instead of wrapping to a small value, which
    // would admit everything.
    let mut limiter = RateLimiter::new(10);
    assert!(matches!(
        limiter.admit(u64::MAX - 5),
        Admission::Admit { .. }
    ));
    assert_eq!(limiter.admit(u64::MAX - 1), Admission::Suppress);
}

#[test]
fn suppressed_text() {
    assert_eq!(with_suppressed("Failed", 0), "Failed");
    assert_eq!(
        with_suppressed("Failed", 3),
        "Failed (3 similar suppressed)"
    );
}

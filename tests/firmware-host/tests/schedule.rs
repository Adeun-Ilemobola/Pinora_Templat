//! `runtime/schedule.rs`: the 1 ms fixed-rate schedule and loop statistics (audit
//! Phase 2C, PIN-TPL-009).

use pinora_firmware_host_tests::runtime::schedule::*;

#[test]
fn on_time_iterations_keep_a_fixed_grid() {
    let mut deadline = 1_000;
    for (finished, expected) in [(400, 2_000), (1_900, 3_000), (2_999, 4_000)] {
        deadline = next_deadline(deadline, finished, PERIOD_US);
        assert_eq!(deadline, expected);
    }
}

#[test]
fn a_long_overrun_skips_missed_periods_instead_of_bursting() {
    // Finished 5.5 periods late: the next deadline is one period after "now".
    assert_eq!(next_deadline(1_000, 6_500, PERIOD_US), 7_500);
    // Late by less than a period: stay on the grid.
    assert_eq!(next_deadline(1_000, 1_800, PERIOD_US), 2_000);
}

#[test]
fn waiting_never_underflows() {
    assert_eq!(wait_us(5_000, 4_200), 800);
    assert_eq!(wait_us(5_000, 5_000), 0);
    assert_eq!(wait_us(5_000, 9_000), 0);
}

#[test]
fn a_simulated_loop_runs_at_the_period_and_waits_between_ticks() {
    // 10 s of a loop whose work takes 150 µs.
    let (mut now, mut deadline) = (0u64, PERIOD_US);
    let mut stats = LoopStats::default();
    let mut waited = 0u64;
    let mut report = None;
    while now < 10_100_000 {
        stats.iteration_started(now);
        now += 150;
        stats.work_done(150);
        if let Some(r) = stats.take_report(now) {
            report = Some(r);
        }
        let wait = wait_us(deadline, now);
        waited += wait;
        now += wait;
        deadline = next_deadline(deadline, now, PERIOD_US);
    }
    assert!(
        waited > 8_000_000,
        "the loop must block most of the time, waited {waited}"
    );
    let report = report.expect("a report after 10 s");
    assert!(report.contains("period mean 1000 µs"), "{report}");
    assert!(report.contains("0 overruns"), "{report}");
}

#[test]
fn histogram_percentiles_have_100_us_resolution() {
    let mut h = Histogram::default();
    for _ in 0..99 {
        h.record(1_000);
    }
    h.record(4_200);
    assert_eq!(h.count(), 100);
    assert_eq!(h.percentile_us(50), 1_100);
    assert_eq!(h.percentile_us(99), 1_100);
    assert_eq!(h.percentile_us(100), 4_300);
    assert_eq!(h.max_us(), 4_200);
    assert_eq!(h.mean_us(), 1_032);
    let mut long = Histogram::default();
    long.record(60_000);
    assert_eq!(long.percentile_us(99), 20_000, "overflow bucket");
    assert_eq!(Histogram::default().percentile_us(99), 0);
}

#[test]
fn overruns_are_counted_and_reports_reset() {
    let mut stats = LoopStats::default();
    stats.iteration_started(0);
    stats.work_done(PERIOD_US + 1);
    stats.work_done(PERIOD_US);
    assert_eq!(stats.overruns, 1);
    assert!(stats.take_report(REPORT_INTERVAL_US - 1).is_none());
    let report = stats.take_report(REPORT_INTERVAL_US).unwrap();
    assert!(report.contains("1 overruns"), "{report}");
    assert_eq!(stats.overruns, 0);
    assert_eq!(stats.work.count(), 0);
}

#[test]
fn waits_ask_for_one_tick_less_and_short_waits_sleep_to_the_next_tick() {
    assert_eq!(plan_wait(0), Wait::None);
    assert_eq!(plan_wait(1), Wait::NextTick);
    assert_eq!(plan_wait(TICK_US), Wait::NextTick);
    assert_eq!(plan_wait(TICK_US + 1), Wait::Block(1));
    assert_eq!(plan_wait(5_000), Wait::Block(4_000));
}

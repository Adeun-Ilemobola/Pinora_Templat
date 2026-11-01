//! `core/transport/drop_stats.rs`: per-stage drop counting and the periodic summary
//! (audit Phase 2B, PIN-TPL-010).

use pinora_firmware_host_tests::core::transport::drop_stats::*;

#[test]
fn nothing_dropped_means_no_report() {
    assert_eq!(DropCounters::default().take(), None);
}

#[test]
fn drops_are_counted_per_stage_and_reset_by_take() {
    let counters = DropCounters::default();
    counters.record(Stage::Emitter);
    counters.record(Stage::Emitter);
    counters.record(Stage::Transport);
    counters.record(Stage::Link);
    let report = counters.take().unwrap();
    assert_eq!(
        report,
        DropReport {
            emitter: 2,
            transport: 1,
            link: 1
        }
    );
    assert_eq!(report.total(), 4);
    assert_eq!(
        report.describe(),
        "Telemetry dropped under load: 4 message(s) (emitter queue 2, transport queue 1, link 1)"
    );
    assert_eq!(counters.take(), None);
}

#[test]
fn counters_are_shared_across_threads() {
    let counters = std::sync::Arc::new(DropCounters::default());
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let counters = counters.clone();
            std::thread::spawn(move || (0..1000).for_each(|_| counters.record(Stage::Transport)))
        })
        .collect();
    workers.into_iter().for_each(|w| w.join().unwrap());
    assert_eq!(counters.take().unwrap().transport, 4000);
}

#[test]
fn the_report_interval_is_five_seconds() {
    assert_eq!(REPORT_INTERVAL.as_secs(), 5);
}

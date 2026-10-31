//! Tests for `Firmware_Templates/src/runtime/tick_errors.rs` (Phase 1B, PIN-TPL-001):
//! tick errors are reported, rate-limited per module.

use std::sync::mpsc::Receiver;

use pinora_firmware_host_tests::core::module::{Module, ModuleCore, ModuleError};
use pinora_firmware_host_tests::core::transport::emitter::TransportEmitter;
use pinora_firmware_host_tests::runtime::tick_errors::{TickErrorReporter, REPORT_INTERVAL_US};
use pinora_protocol::*;

struct Sensor {
    core: ModuleCore,
}

impl Module for Sensor {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn tick(&mut self) -> Result<(), ModuleError> {
        Err(ModuleError::OperationFailed)
    }
    fn handle_command(&mut self, _command: &ModuleCommand) -> anyhow::Result<()> {
        Ok(())
    }
}

fn sensor(emitter: &TransportEmitter) -> Sensor {
    Sensor {
        core: ModuleCore::new(ModuleType::Rangefinder, "ranger", None, emitter.clone()),
    }
}

fn texts(rx: &Receiver<ProtocolMessage>) -> Vec<(String, String, LogPriority)> {
    rx.try_iter()
        .map(|message| match message {
            ProtocolMessage::ModuleEvent(EventPackage {
                id,
                event: ModuleEvent::SysLog(log),
            }) => (id, log.text, log.priority),
            other => panic!("unexpected {other:?}"),
        })
        .collect()
}

#[test]
fn first_error_is_reported_immediately() {
    let (emitter, rx) = TransportEmitter::new();
    let module = sensor(&emitter);
    let mut reporter = TickErrorReporter::new();
    assert!(reporter.record(&module, ModuleError::OperationFailed, 0.0));
    assert_eq!(
        texts(&rx),
        vec![(
            module.id().to_string(),
            "Tick failed: OperationFailed".to_string(),
            LogPriority::Medium
        )]
    );
}

#[test]
fn repeated_errors_are_suppressed_until_the_interval_elapses() {
    let (emitter, rx) = TransportEmitter::new();
    let module = sensor(&emitter);
    let mut reporter = TickErrorReporter::new();
    assert!(reporter.record(&module, ModuleError::OperationFailed, 0.0));
    // A module failing every millisecond for just under 5 s.
    for i in 1..5_000 {
        assert!(!reporter.record(&module, ModuleError::SensorNotReady, i as f64 * 1_000.0));
    }
    assert!(reporter.record(&module, ModuleError::SensorNotReady, REPORT_INTERVAL_US));
    let reports = texts(&rx);
    assert_eq!(reports.len(), 2);
    assert_eq!(
        reports[1].1,
        "Tick failed: SensorNotReady (4999 more since last report)"
    );
}

#[test]
fn modules_are_rate_limited_independently() {
    let (emitter, rx) = TransportEmitter::new();
    let a = sensor(&emitter);
    let b = sensor(&emitter);
    let mut reporter = TickErrorReporter::new();
    assert!(reporter.record(&a, ModuleError::OperationFailed, 0.0));
    assert!(reporter.record(&b, ModuleError::OperationFailed, 1.0));
    assert!(!reporter.record(&a, ModuleError::OperationFailed, 2.0));
    let ids: Vec<_> = texts(&rx).into_iter().map(|(id, _, _)| id).collect();
    assert_eq!(ids, vec![a.id().to_string(), b.id().to_string()]);
}

#[test]
fn suppressed_count_resets_after_each_report() {
    let (emitter, rx) = TransportEmitter::new();
    let module = sensor(&emitter);
    let mut reporter = TickErrorReporter::new();
    reporter.record(&module, ModuleError::OperationFailed, 0.0);
    reporter.record(&module, ModuleError::OperationFailed, 1.0);
    reporter.record(&module, ModuleError::OperationFailed, REPORT_INTERVAL_US);
    reporter.record(
        &module,
        ModuleError::OperationFailed,
        2.0 * REPORT_INTERVAL_US,
    );
    let reports: Vec<_> = texts(&rx).into_iter().map(|(_, text, _)| text).collect();
    assert_eq!(
        reports,
        vec![
            "Tick failed: OperationFailed",
            "Tick failed: OperationFailed (1 more since last report)",
            "Tick failed: OperationFailed",
        ]
    );
}

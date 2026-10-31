//! Characterization tests for `Firmware_Templates/src/core/transport/emitter.rs`:
//! the reliable (blocking) vs lossy (dropping) emission rule.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use pinora_firmware_host_tests::core::transport::emitter::{EmitterError, TransportEmitter};
use pinora_protocol::*;

const QUEUE_CAPACITY: usize = 128;

fn event(n: u32) -> ProtocolMessage {
    ProtocolMessage::ModuleEvent(EventPackage {
        id: format!("m{n}"),
        event: ModuleEvent::Led(LedEvent::Brightness { level: n }),
    })
}

fn system() -> SystemInfo {
    SystemInfo {
        esp_idf_version: "v5.5.3".into(),
        total_heap: "a".into(),
        current_free_heap: "b".into(),
        lowest_free_heap: "c".into(),
        largest_allocation: "d".into(),
        maximum_app_slot: "e".into(),
        flash: "f".into(),
    }
}

#[test]
fn emit_reliable_delivers_in_order() {
    let (emitter, rx) = TransportEmitter::new();
    emitter.emit_reliable(event(1)).unwrap();
    emitter.emit_reliable(event(2)).unwrap();
    assert_eq!(rx.try_recv().unwrap(), event(1));
    assert_eq!(rx.try_recv().unwrap(), event(2));
}

#[test]
fn try_emit_drops_without_blocking_when_the_queue_is_full() {
    let (emitter, rx) = TransportEmitter::new();
    for n in 0..(QUEUE_CAPACITY as u32 + 10) {
        emitter.try_emit(event(n)); // must return immediately even when full
    }
    let received: Vec<_> = rx.try_iter().collect();
    assert_eq!(received.len(), QUEUE_CAPACITY);
    assert_eq!(received.first(), Some(&event(0)));
    assert_eq!(received.last(), Some(&event(QUEUE_CAPACITY as u32 - 1)));
}

#[test]
fn emit_reliable_blocks_while_the_queue_is_full() {
    let (emitter, rx) = TransportEmitter::new();
    for n in 0..QUEUE_CAPACITY as u32 {
        emitter.try_emit(event(n));
    }
    let (done_tx, done_rx) = mpsc::channel();
    let sender = emitter.clone();
    let worker = thread::spawn(move || {
        let result = sender.emit_reliable(event(999));
        done_tx.send(result.is_ok()).unwrap();
    });
    assert!(
        done_rx.recv_timeout(Duration::from_millis(100)).is_err(),
        "emit_reliable returned while the queue was full"
    );
    rx.recv().unwrap(); // free one slot
    assert_eq!(done_rx.recv_timeout(Duration::from_secs(5)), Ok(true));
    worker.join().unwrap();
    assert_eq!(rx.try_iter().last(), Some(event(999)));
}

#[test]
fn emit_reliable_reports_disconnection() {
    let (emitter, rx) = TransportEmitter::new();
    drop(rx);
    let error = emitter.emit_reliable(event(1)).unwrap_err();
    assert!(matches!(error, EmitterError::Disconnected));
    assert_eq!(error.to_string(), "event emitter is disconnected");
}

#[test]
fn try_emit_on_a_disconnected_queue_does_not_panic() {
    let (emitter, rx) = TransportEmitter::new();
    drop(rx);
    emitter.try_emit(event(1));
    emitter.any(event(2));
}

#[test]
fn system_info_is_sent_reliably_as_a_system_message() {
    let (emitter, rx) = TransportEmitter::new();
    emitter.system_info(system()).unwrap();
    assert_eq!(rx.try_recv().unwrap(), ProtocolMessage::System(system()));
}

#[test]
fn log_uses_the_reserved_source_id_log() {
    // Contract C-05: emitter-level logs are attributed to the pseudo-module "log".
    let (emitter, rx) = TransportEmitter::new();
    emitter.log("boot".into(), LogPriority::High, Some("raw".into()));
    assert_eq!(
        rx.try_recv().unwrap(),
        ProtocolMessage::ModuleEvent(EventPackage {
            id: "log".into(),
            event: ModuleEvent::SysLog(SysLogEvent {
                text: "boot".into(),
                priority: LogPriority::High,
                raw_err: Some("raw".into()),
            }),
        })
    );
}

#[test]
fn clones_share_one_queue() {
    let (emitter, rx) = TransportEmitter::new();
    let other = emitter.clone();
    emitter.try_emit(event(1));
    other.try_emit(event(2));
    assert_eq!(rx.try_iter().count(), 2);
}

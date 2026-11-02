//! The queue every module emits into (capacity 128), drained by the sender thread in
//! `manager`.
//!
//! * `emit_reliable` (registration, system snapshot) blocks until there is room, so
//!   discovery messages are never dropped here. Do not call it from `tick()`.
//! * `try_emit` (runtime events, logs) drops the message when the queue is full, so
//!   module ticks never block on telemetry. Drops are counted in [`DropCounters`]
//!   and reported periodically (Phase 2B, PIN-TPL-010), not logged one by one.
//!
//! Enqueueing is not delivery: there is no acknowledgement or retry.

use std::{
    fmt,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender, TrySendError},
        Arc,
    },
};

use crate::core::transport::drop_stats::{DropCounters, Stage};

use pinora_protocol::{
    EventPackage, LogPriority, ModuleEvent, ProtocolMessage, SysLogEvent, SystemInfo,
};

#[derive(Debug)]
pub enum EmitterError {
    Disconnected,
}
impl std::error::Error for EmitterError {}

impl fmt::Display for EmitterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("event emitter is disconnected")
    }
}
#[derive(Debug, Clone)]
pub struct TransportEmitter {
    tx: SyncSender<ProtocolMessage>,
    drops: Arc<DropCounters>,
}

impl TransportEmitter {
    pub fn new() -> (Self, Receiver<ProtocolMessage>) {
        let (tx, rx) = sync_channel::<ProtocolMessage>(128);
        (
            Self {
                tx,
                drops: Arc::new(DropCounters::default()),
            },
            rx,
        )
    }

    /// The drop counters shared by every outbound stage of this emitter's transport.
    pub fn drop_counters(&self) -> Arc<DropCounters> {
        Arc::clone(&self.drops)
    }

    pub fn system_info(&self, data: SystemInfo) -> Result<(), EmitterError> {
        self.emit_reliable(ProtocolMessage::System(data))
    }

    pub fn any(&self, data: ProtocolMessage) {
        self.try_emit(data);
    }
    pub fn emit_reliable(&self, message: ProtocolMessage) -> Result<(), EmitterError> {
        // Registration/system messages wait for queue space so new connections can discover them.
        self.tx
            .send(message)
            .map_err(|_| EmitterError::Disconnected)?;
        Ok(())
    }
    pub fn try_emit(&self, message: ProtocolMessage) {
        // Avoid stalling module ticks for runtime telemetry when the transport is backed up.
        match self.tx.try_send(message) {
            Ok(()) => {}

            Err(TrySendError::Full(_)) => self.drops.record(Stage::Emitter),

            Err(TrySendError::Disconnected(_)) => {
                log::error!("Event emitter is disconnected");
            }
        }
    }
    pub fn log(&self, message: String, priority: LogPriority, raw_err: Option<String>) {
        self.try_emit(ProtocolMessage::ModuleEvent(EventPackage {
            id: "log".to_string(),
            event: ModuleEvent::SysLog(SysLogEvent {
                text: message,
                priority,
                raw_err,
            }),
        }));
    }
}

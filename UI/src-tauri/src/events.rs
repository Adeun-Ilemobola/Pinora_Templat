//! Where transports send their Tauri events. Production code uses the `AppHandle`;
//! tests record the events instead (audit Phase 2B).

use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub(crate) trait EventSink: Clone + Send + Sync + 'static {
    fn emit_text(&self, event: &str, payload: String);
    fn emit_json(&self, event: &str, payload: serde_json::Value);
}

impl EventSink for AppHandle {
    fn emit_text(&self, event: &str, payload: String) {
        if let Err(error) = self.emit(event, payload) {
            log::error!("Failed to emit {event} event: {error:?}");
        }
    }

    fn emit_json(&self, event: &str, payload: serde_json::Value) {
        if let Err(error) = self.emit(event, payload) {
            log::error!("Failed to emit {event} event: {error:?}");
        }
    }
}

/// Serializes `payload` and emits it as JSON.
pub(crate) fn emit_serialized<S: EventSink, T: Serialize>(sink: &S, event: &str, payload: &T) {
    match serde_json::to_value(payload) {
        Ok(value) => sink.emit_json(event, value),
        Err(error) => log::error!("Could not serialize the {event} payload: {error}"),
    }
}

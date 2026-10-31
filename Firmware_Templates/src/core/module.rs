//! The module contract every hardware module implements.
//!
//! # Lifecycle
//!
//! 1. **Construction** in `board` acquires the hardware. Some actuators currently
//!    *move* here (PIN-TPL-004).
//! 2. Insertion into the [`ModuleRegistry`], keyed by runtime UUID.
//! 3. [`Module::register`], then [`Module::first_emit`], once each.
//! 4. [`Module::tick`] on every main-loop iteration, forever.
//! 5. [`Module::handle_command`] whenever a command's `id` equals this module's UUID.
//!
//! There is no shutdown or safe-state hook. Modules are dropped only if `main` returns.
//!
//! # Emission rule
//!
//! Registration uses the *reliable* path (blocks until queued, and is cached for
//! reconnect replay). Runtime events and logs use the *lossy* path (dropped when the
//! queue is full, so `tick()` never blocks on telemetry).

use std::collections::HashMap;

use uuid::Uuid;

use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::{EventPackage, ModuleEvent, SysLogEvent},
    registration::{ProtocolMessage, Registration},
};

use crate::core::transport::emitter::{EmitterError, TransportEmitter};

/// A boxed module as stored in the registry.
pub type ModuleHandle<'a> = Box<dyn Module + 'a>;

/// Top-level modules keyed by runtime UUID. Only these are routable (PIN-TPL-017).
pub type ModuleRegistry<'a> = HashMap<String, ModuleHandle<'a>>;

/// Identity and emitter shared by every module.
#[derive(Debug, Clone)]
pub struct ModuleCore {
    /// UUID v4, regenerated on every boot.
    pub id: String,
    pub module_type: ModuleType,
    /// Caller-chosen human name (e.g. `"led1"`); sent on the wire as `lool_up_id`.
    pub lookup_name: String,
    /// Parent module UUID, or `""` for top-level modules (wire contract C-01).
    pub parent_id: String,
    emitter: TransportEmitter,
}

impl ModuleCore {
    pub fn new(
        module_type: ModuleType,
        lookup_name: &str,
        parent_id: Option<String>,
        emitter: TransportEmitter,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            module_type: module_type,
            lookup_name: lookup_name.to_string(),
            parent_id: parent_id.unwrap_or_default(),
            emitter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleError {
    OperationFailed,
    SensorNotReady,
}

impl From<()> for ModuleError {
    fn from(_: ()) -> Self {
        Self::OperationFailed
    }
}

pub trait Module {
    fn core(&self) -> &ModuleCore;

    fn id(&self) -> &str {
        &self.core().id
    }
    /// Called every main-loop iteration. Errors are currently discarded by the loop.
    fn tick(&mut self) -> Result<(), ModuleError>;

    fn register(&self) -> Result<(), EmitterError> {
        self.emit_registration()
    }
    fn first_emit(&mut self) -> Result<(), EmitterError> {
        Ok(())
    }

    fn emit_registration(&self) -> Result<(), EmitterError> {
        let registration = Registration {
            id: self.id().to_string(),
            module_type: self.get_module_type().clone(),
            lool_up_id: self.core().lookup_name.clone(),
            parent_id: self.core().parent_id.clone(),
        };

        self.core()
            .emitter
            .emit_reliable(ProtocolMessage::Registration(registration))
    }

    fn get_module_type(&self) -> &ModuleType {
        &self.core().module_type
    }

    fn emit(&self, event: ModuleEvent) {
        self.core()
            .emitter
            .try_emit(ProtocolMessage::ModuleEvent(EventPackage {
                id: self.id().to_owned(),
                event,
            }));
    }

    fn log(&self, data: SysLogEvent) {
        self.emit(ModuleEvent::SysLog(data));
    }
    /// Receives every command family; modules ignore families that are not theirs.
    /// An `Err` here currently stops the main loop (PIN-TPL-001).
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()>;
}

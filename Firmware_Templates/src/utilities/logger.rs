use crate::core::modulecore::emit;
use crate::protocol::module_event::{LogPriority, ModuleEvent, SysLogEvent};

pub use crate::protocol::module_event::LogPriority as Priority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SysLog {}
impl SysLog {
    pub fn emit(text: String, raw_err: Option<String>, priority: LogPriority) {
        emit::event(ModuleEvent::SysLog(SysLogEvent {
            text,
            raw_err,
            priority,
        }));
    }

    pub fn info(text: String, raw_err: Option<String>) {
        Self::emit(text, raw_err, Priority::Low);
    }

    pub fn error(text: String, raw_err: Option<String>) {
        Self::emit(text, raw_err, Priority::Critical);
    }
}

use crate::core::modulecore::emit;
use crate::protocol::module_event::{LogPriority, ModuleEvent, SysLogEvent};

pub use crate::protocol::module_event::LogPriority as Priority;
use crate::protocol::registration::ProtocolMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SysLog {}
impl SysLog {
    pub fn send(text: String, raw_err: Option<String>, priority: LogPriority) {
        let event = SysLogEvent {
            text,
            raw_err,
            priority,
        };
        emit::event(ProtocolMessage::ModuleEvent(ModuleEvent::SysLog(event)));
    }

    pub fn info(text: String, raw_err: Option<String>) {
        Self::send(text, raw_err, Priority::Low);
    }

    pub fn error(text: String, raw_err: Option<String>) {
        Self::send(text, raw_err, Priority::Critical);
    }
}

use pinora_protocol::{LogPriority as ProtocolLogPriority, SysLogEvent};

use crate::{LogPriority, SysLogState};

impl From<ProtocolLogPriority> for LogPriority {
    fn from(priority: ProtocolLogPriority) -> Self {
        match priority {
            ProtocolLogPriority::Low => Self::Low,
            ProtocolLogPriority::Medium => Self::Medium,
            ProtocolLogPriority::High => Self::High,
            ProtocolLogPriority::Critical => Self::Critical,
        }
    }
}

impl From<LogPriority> for ProtocolLogPriority {
    fn from(priority: LogPriority) -> Self {
        match priority {
            LogPriority::Low => Self::Low,
            LogPriority::Medium => Self::Medium,
            LogPriority::High => Self::High,
            LogPriority::Critical => Self::Critical,
        }
    }
}

impl SysLogState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: SysLogEvent) {
        self.text = event.text.into();
        match event.raw_err {
            Some(raw_err) => {
                self.has_raw_err = true;
                self.raw_err = raw_err.into();
            }
            None => {
                self.has_raw_err = false;
                self.raw_err = Default::default();
            }
        }
        self.priority = event.priority.into();
    }
}

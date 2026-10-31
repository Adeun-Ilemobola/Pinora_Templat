//! Classifies lines read from a COM port (USB serial or Bluetooth SPP).
//!
//! Only lines that parse as a Rust `ProtocolMessage` are forwarded to the frontend
//! as protocol traffic (`espState`). Anything else, such as ESP-IDF console output,
//! partial or malformed JSON, or unknown message shapes, is forwarded separately as
//! `espConsole`. This is the same validation the Wi-Fi transport already applies,
//! and it keeps unexpected input out of the frontend's protocol handling
//! (PIN-TPL-006, Phase 1C).

use pinora_protocol::ProtocolMessage;

/// Tauri event for validated protocol lines (contract C-07).
pub(crate) const PROTOCOL_EVENT: &str = "espState";
/// Tauri event for every other line (added in Phase 1C).
pub(crate) const CONSOLE_EVENT: &str = "espConsole";

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ComLine {
    /// A valid protocol message. The text is forwarded unchanged.
    Protocol(String),
    /// Not a protocol message.
    Console(String),
}

impl ComLine {
    pub(crate) fn event_name(&self) -> &'static str {
        match self {
            ComLine::Protocol(_) => PROTOCOL_EVENT,
            ComLine::Console(_) => CONSOLE_EVENT,
        }
    }

    pub(crate) fn into_text(self) -> String {
        match self {
            ComLine::Protocol(text) | ComLine::Console(text) => text,
        }
    }
}

/// Classifies one newline-terminated line. Invalid UTF-8 is replaced lossily, as
/// before.
pub(crate) fn classify(line: &[u8]) -> ComLine {
    let text = String::from_utf8_lossy(line).into_owned();
    if serde_json::from_str::<ProtocolMessage>(text.trim()).is_ok() {
        ComLine::Protocol(text)
    } else {
        ComLine::Console(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRATION: &str =
        r#"{"Registration":{"id":"u","module_type":"Led","lool_up_id":"led1","parent_id":""}}"#;
    const LED_EVENT: &str =
        r#"{"ModuleEvent":{"event":{"Led":{"Brightness":{"level":80}}},"id":"u"}}"#;

    #[test]
    fn valid_protocol_lines_are_forwarded_unchanged_as_esp_state() {
        for line in [
            format!("{REGISTRATION}\n"),
            format!("{LED_EVENT}\r\n"),
            format!("  {LED_EVENT}  \n"),
        ] {
            let classified = classify(line.as_bytes());
            assert_eq!(classified.event_name(), "espState");
            assert_eq!(classified.into_text(), line, "text forwarded unchanged");
        }
    }

    #[test]
    fn console_output_goes_to_esp_console() {
        let line = "I (1234) Firmware_Templates::runtime: started\n";
        assert_eq!(
            classify(line.as_bytes()),
            ComLine::Console(line.to_string())
        );
    }

    #[test]
    fn json_that_is_not_a_protocol_message_goes_to_esp_console() {
        // The inputs that crashed the desktop log view before Phase 1C.
        for line in [
            "{}\n",
            "{\"Foo\":1}\n",
            "{\"ModuleEvent\":{\"id\":\"x\",\"event\":{}}}\n",
            "{\"ModuleEvent\":{\"id\":\"x\",\"event\":{\"Led\":{\"Brightness\":{}}}}}\n",
            "[1,2,3]\n",
            "null\n",
        ] {
            assert_eq!(
                classify(line.as_bytes()).event_name(),
                "espConsole",
                "{line}"
            );
        }
    }

    #[test]
    fn truncated_or_garbled_lines_go_to_esp_console() {
        let half = &LED_EVENT[..LED_EVENT.len() / 2];
        assert_eq!(classify(half.as_bytes()).event_name(), "espConsole");
        assert_eq!(classify(&[0xff, 0xfe, b'\n']).event_name(), "espConsole");
        assert_eq!(classify(b"\n").event_name(), "espConsole");
    }
}

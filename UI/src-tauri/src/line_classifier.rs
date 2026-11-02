//! Classifies lines read from a COM port (USB serial or Bluetooth SPP).
//!
//! Only lines that parse as a Rust `ProtocolMessage` are forwarded to the frontend
//! as protocol traffic (`espState`). Anything else, such as ESP-IDF console output,
//! partial or malformed JSON, or unknown message shapes, is forwarded separately as
//! `espConsole`. This is the same validation the Wi-Fi transport already applies,
//! and it keeps unexpected input out of the frontend's protocol handling
//! (PIN-TPL-006, Phase 1C).
//!
//! Protocol v2 `Auth` messages (Phase 2B) are classified separately and have **no**
//! WebView event: the backend consumes them, so a device key can never be forwarded
//! by accident. Both the COM reader and the Wi-Fi listener use this classifier.

use pinora_protocol::{AuthMessage, ProtocolMessage};

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
    /// An authentication message: handled by the backend, never forwarded.
    Auth(AuthMessage),
}

impl ComLine {
    /// The WebView event for this line; `None` for backend-only lines.
    pub(crate) fn event_name(&self) -> Option<&'static str> {
        match self {
            ComLine::Protocol(_) => Some(PROTOCOL_EVENT),
            ComLine::Console(_) => Some(CONSOLE_EVENT),
            ComLine::Auth(_) => None,
        }
    }

    /// The text to forward; `None` for backend-only lines.
    pub(crate) fn into_text(self) -> Option<String> {
        match self {
            ComLine::Protocol(text) | ComLine::Console(text) => Some(text),
            ComLine::Auth(_) => None,
        }
    }
}

/// Classifies one newline-terminated line. Invalid UTF-8 is replaced lossily, as
/// before.
pub(crate) fn classify(line: &[u8]) -> ComLine {
    let text = String::from_utf8_lossy(line).into_owned();
    match serde_json::from_str::<ProtocolMessage>(text.trim()) {
        Ok(ProtocolMessage::Auth(message)) => ComLine::Auth(message),
        Ok(_) => ComLine::Protocol(text),
        Err(_) => ComLine::Console(text),
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
            assert_eq!(classified.event_name(), Some("espState"));
            assert_eq!(
                classified.into_text().as_deref(),
                Some(line.as_str()),
                "text forwarded unchanged"
            );
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
                Some("espConsole"),
                "{line}"
            );
        }
    }

    #[test]
    fn truncated_or_garbled_lines_go_to_esp_console() {
        let half = &LED_EVENT[..LED_EVENT.len() / 2];
        assert_eq!(classify(half.as_bytes()).event_name(), Some("espConsole"));
        assert_eq!(
            classify(&[0xff, 0xfe, b'\n']).event_name(),
            Some("espConsole")
        );
        assert_eq!(classify(b"\n").event_name(), Some("espConsole"));
    }

    #[test]
    fn auth_lines_are_never_forwarded_to_the_webview() {
        // Phase 2B: a key line must not reach `espState`, whatever the transport.
        let key_line = r#"{"Auth":{"Key":{"device_id":"a1b2c3d4e5f6","key":"11"}}}"#;
        let classified = classify(key_line.as_bytes());
        assert!(matches!(classified, ComLine::Auth(AuthMessage::Key { .. })));
        assert_eq!(classified.event_name(), None);
        assert_eq!(classified.into_text(), None);
    }
}

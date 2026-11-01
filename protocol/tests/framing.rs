//! The shared line framer and limit table (audit Phase 2B, PIN-TPL-016).

use pinora_protocol::framing::*;
use pinora_protocol::{
    CommandLine, IncomingCommand, ModuleCommand, SystemCommand, parse_command_line,
};

fn lines(frames: Vec<Frame>) -> Vec<String> {
    frames
        .into_iter()
        .map(|frame| match frame {
            Frame::Line(line) => String::from_utf8(line).unwrap(),
            Frame::Overflow => "<overflow>".into(),
        })
        .collect()
}

#[test]
fn reads_may_split_or_merge_lines() {
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    assert!(framer.push(b"{\"a\"").is_empty());
    assert_eq!(
        lines(framer.push(b":1}\n{\"b\":2}\n{\"c\"")),
        ["{\"a\":1}", "{\"b\":2}"]
    );
    assert_eq!(framer.pending(), 4);
    assert_eq!(lines(framer.push(b":3}\n")), ["{\"c\":3}"]);
}

#[test]
fn crlf_endings_and_blank_lines() {
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    assert_eq!(lines(framer.push(b"one\r\n\r\n  \ntwo\n")), ["one", "two"]);
}

#[test]
fn the_limit_is_inclusive_and_overflow_resynchronizes() {
    let mut framer = LineFramer::new(4);
    assert_eq!(lines(framer.push(b"abcd\n")), ["abcd"]);
    assert_eq!(lines(framer.push(b"abcd\r\n")), ["abcd"]);
    assert_eq!(lines(framer.push(b"abcde")), ["<overflow>"]);
    // Reported once; the rest of the oversized line is discarded.
    assert!(framer.push(b"fghij").is_empty());
    assert_eq!(lines(framer.push(b"\nok\n")), ["ok"]);
}

#[test]
fn clear_drops_a_partial_line() {
    let mut framer = LineFramer::new(MAX_LINE_BYTES);
    framer.push(b"partial");
    framer.clear();
    assert_eq!(lines(framer.push(b"next\n")), ["next"]);
}

#[test]
fn one_limit_table() {
    assert_eq!(MAX_LINE_BYTES, 8192);
    assert_eq!(MAX_AUTH_LINE_BYTES, 512);
}

#[test]
fn encode_line_appends_a_newline_and_enforces_the_limit() {
    assert_eq!(encode_line(&"x").unwrap(), b"\"x\"\n");
    let big = "y".repeat(MAX_LINE_BYTES);
    assert!(matches!(encode_line(&big), Err(EncodeError::TooLong(_))));
    let fits = "y".repeat(MAX_LINE_BYTES - 2);
    assert_eq!(encode_line(&fits).unwrap().len(), MAX_LINE_BYTES + 1);
}

#[test]
fn command_lines_are_plain_or_signed() {
    let plain = br#"{"id":"SSI","command":{"System":{"Rediscover":{}}}}"#;
    assert_eq!(
        parse_command_line(plain).unwrap(),
        CommandLine::Plain(IncomingCommand {
            id: "SSI".into(),
            command: ModuleCommand::System(SystemCommand::Rediscover {}),
        })
    );
    let signed = br#"{"seq":1,"mac":"00","signed":"{}"}"#;
    assert!(matches!(parse_command_line(signed).unwrap(), CommandLine::Signed(s) if s.seq == 1));
    // Neither form: the plain-form error is reported.
    let error = parse_command_line(br#"{"seq":1}"#).unwrap_err().to_string();
    assert!(error.contains("missing field `id`"), "{error}");
    // Extra fields make a signed line invalid.
    assert!(parse_command_line(br#"{"seq":1,"mac":"00","signed":"{}","x":1}"#).is_err());
}

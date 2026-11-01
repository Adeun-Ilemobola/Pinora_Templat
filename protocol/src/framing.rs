//! Newline-delimited framing shared by every transport, with one table of limits
//! (audit Phase 2B, PIN-TPL-016). Before 2B each transport had its own copy with
//! different limits (firmware 8 KiB, desktop Wi-Fi 256 KiB, desktop COM unbounded).
//!
//! A line is the bytes before `\n`, with one trailing `\r` removed. Blank lines are
//! skipped. A line longer than the limit is reported once as [`Frame::Overflow`] and
//! its bytes are discarded up to the next newline, so the stream resynchronizes. What
//! to do on overflow (disconnect, log) is the caller's decision.

use serde::Serialize;

use crate::bluetooth_config::MAX_MESSAGE_BYTES;

/// Longest line content, in bytes, on every transport and in both directions.
pub const MAX_LINE_BYTES: usize = MAX_MESSAGE_BYTES;

/// Longest line of the Wi-Fi pre-protocol token exchange (contract C-03).
pub const MAX_AUTH_LINE_BYTES: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// One complete line, without its line ending.
    Line(Vec<u8>),
    /// A line exceeded the limit; its content was discarded.
    Overflow,
}

/// Splits a byte stream into lines. Reads may split or merge lines arbitrarily.
#[derive(Debug, Clone)]
pub struct LineFramer {
    buffer: Vec<u8>,
    max: usize,
    discarding: bool,
}

impl LineFramer {
    pub fn new(max: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max,
            discarding: false,
        }
    }

    /// Feeds bytes and returns the frames they complete, in order.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Frame> {
        let mut frames = Vec::new();
        for &byte in bytes {
            if byte == b'\n' {
                if self.discarding {
                    self.discarding = false;
                } else {
                    if self.buffer.last() == Some(&b'\r') {
                        self.buffer.pop();
                    }
                    if !self.buffer.iter().all(u8::is_ascii_whitespace) {
                        frames.push(Frame::Line(std::mem::take(&mut self.buffer)));
                    }
                }
                self.buffer.clear();
            } else if self.discarding {
                // Still inside an oversized line.
            } else if self.buffer.len() < self.max {
                self.buffer.push(byte);
            } else if byte == b'\r' && self.buffer.len() == self.max {
                // A CRLF ending right at the limit is still a valid line.
                self.buffer.push(byte);
            } else {
                self.buffer.clear();
                self.discarding = true;
                frames.push(Frame::Overflow);
            }
        }
        frames
    }

    /// Drops any partial line, for example after a disconnect.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.discarding = false;
    }

    /// Bytes of the current partial line.
    pub fn pending(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Debug)]
pub enum EncodeError {
    Serialize(serde_json::Error),
    /// The serialized message is longer than the limit; it was not framed.
    TooLong(usize),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::Serialize(error) => write!(f, "serialization failed: {error}"),
            EncodeError::TooLong(length) => {
                write!(
                    f,
                    "message of {length} bytes exceeds the {MAX_LINE_BYTES}-byte line limit"
                )
            }
        }
    }
}

impl std::error::Error for EncodeError {}

/// Serializes `message` as one JSON line, including the trailing `\n`.
pub fn encode_line<T: Serialize>(message: &T) -> Result<Vec<u8>, EncodeError> {
    let mut frame = serde_json::to_vec(message).map_err(EncodeError::Serialize)?;
    if frame.len() > MAX_LINE_BYTES {
        return Err(EncodeError::TooLong(frame.len()));
    }
    frame.push(b'\n');
    Ok(frame)
}

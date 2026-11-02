//! The data format Pinora writes to MIFARE Classic cards (audit Phase 2C,
//! PIN-TPL-019). It is a persisted format on physical cards: change it only with a
//! new version byte.
//!
//! **Pinora record v1**, written to the card's 47 usable data blocks in order:
//!
//! | Bytes | Content |
//! |---|---|
//! | 0–1 | magic `"PN"` |
//! | 2 | version `1` |
//! | 3–4 | payload length `n`, big endian (≤ [`MAX_PAYLOAD`]) |
//! | 5… | payload, then zero padding to the end of the last block |
//!
//! A card whose first block has no magic is read as a **legacy UUID card**: its first
//! 16 bytes are a UUID, which is what the firmware expected before 2C.
//!
//! Pure, so it is tested on the host.

pub const BLOCK_BYTES: usize = 16;
/// Data blocks of a 1K MIFARE Classic card that Pinora uses (sector trailers and the
/// manufacturer block excluded).
pub const USABLE_BLOCKS: [u8; 47] = [
    1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16, 17, 18, 20, 21, 22, 24, 25, 26, 28, 29, 30, 32, 33,
    34, 36, 37, 38, 40, 41, 42, 44, 45, 46, 48, 49, 50, 52, 53, 54, 56, 57, 58, 60, 61, 62,
];
const MAGIC: [u8; 2] = *b"PN";
const VERSION: u8 = 1;
const HEADER_BYTES: usize = 5;
/// Largest payload that fits on a card.
pub const MAX_PAYLOAD: usize = USABLE_BLOCKS.len() * BLOCK_BYTES - HEADER_BYTES;

pub type Block = [u8; BLOCK_BYTES];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardData {
    Record(Vec<u8>),
    LegacyUuid([u8; 16]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    TooLong(usize),
    UnsupportedVersion(u8),
    /// The header announces more data than the blocks read.
    Truncated {
        expected: usize,
        available: usize,
    },
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::TooLong(n) => write!(f, "payload of {n} bytes exceeds {MAX_PAYLOAD}"),
            RecordError::UnsupportedVersion(v) => write!(f, "unsupported card record version {v}"),
            RecordError::Truncated {
                expected,
                available,
            } => {
                write!(f, "card record needs {expected} bytes, {available} read")
            }
        }
    }
}

impl std::error::Error for RecordError {}

/// How many blocks to read in total, given the first block.
pub fn blocks_to_read(first: &Block) -> usize {
    if first[..2] != MAGIC {
        return 1;
    }
    let length = usize::from(u16::from_be_bytes([first[3], first[4]])).min(MAX_PAYLOAD);
    (HEADER_BYTES + length).div_ceil(BLOCK_BYTES)
}

/// Decodes the blocks read from a card (first block first).
pub fn decode(blocks: &[Block]) -> Result<CardData, RecordError> {
    let Some(first) = blocks.first() else {
        return Err(RecordError::Truncated {
            expected: BLOCK_BYTES,
            available: 0,
        });
    };
    if first[..2] != MAGIC {
        return Ok(CardData::LegacyUuid(*first));
    }
    if first[2] != VERSION {
        return Err(RecordError::UnsupportedVersion(first[2]));
    }
    let length = usize::from(u16::from_be_bytes([first[3], first[4]]));
    if length > MAX_PAYLOAD {
        return Err(RecordError::TooLong(length));
    }
    let bytes: Vec<u8> = blocks.iter().flatten().copied().collect();
    let end = HEADER_BYTES + length;
    if bytes.len() < end {
        return Err(RecordError::Truncated {
            expected: end,
            available: bytes.len(),
        });
    }
    Ok(CardData::Record(bytes[HEADER_BYTES..end].to_vec()))
}

/// Encodes a payload as the blocks to write, in [`USABLE_BLOCKS`] order.
pub fn encode(payload: &[u8]) -> Result<Vec<Block>, RecordError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(RecordError::TooLong(payload.len()));
    }
    let mut bytes = Vec::with_capacity(HEADER_BYTES + payload.len());
    bytes.extend_from_slice(&MAGIC);
    bytes.push(VERSION);
    bytes.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    bytes.extend_from_slice(payload);
    Ok(bytes
        .chunks(BLOCK_BYTES)
        .map(|chunk| {
            let mut block = [0u8; BLOCK_BYTES];
            block[..chunk.len()].copy_from_slice(chunk);
            block
        })
        .collect())
}

/// The `card_data` text of a `GetCard` event: a record's payload as UTF-8 when it is
/// valid UTF-8, otherwise `hex:` and lowercase hex; a legacy card as its UUID.
pub fn render(data: &CardData) -> String {
    match data {
        CardData::Record(payload) => match std::str::from_utf8(payload) {
            Ok(text) => text.to_string(),
            Err(_) => format!(
                "hex:{}",
                payload
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            ),
        },
        CardData::LegacyUuid(bytes) => uuid::Uuid::from_bytes(*bytes).to_string(),
    }
}

/// The pending write payload. `WritePayload` commands append to it; it is bounded so
/// commands cannot exhaust memory (before 2C it grew without limit).
#[derive(Debug, Default, Clone)]
pub struct WriteBuffer(Vec<u8>);

impl WriteBuffer {
    /// Appends `data`, or leaves the buffer unchanged if the result would not fit.
    pub fn append(&mut self, data: &[u8]) -> Result<(), RecordError> {
        let total = self.0.len() + data.len();
        if total > MAX_PAYLOAD {
            return Err(RecordError::TooLong(total));
        }
        self.0.extend_from_slice(data);
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// MFRC522 version register values accepted at start-up: genuine 1.0 / 2.0 and the
/// common FM17522 and clone values. Anything else means no reader is responding.
pub fn is_known_reader_version(version: u8) -> bool {
    matches!(version, 0x88 | 0x91 | 0x92 | 0xB2)
}

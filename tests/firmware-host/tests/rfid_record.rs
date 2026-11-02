//! `modules/sensors/rfid_record.rs`: the card data format and the bounded write
//! buffer (audit Phase 2C, PIN-TPL-019).

use pinora_firmware_host_tests::modules::sensors::rfid_record::*;

#[test]
fn a_record_round_trips_through_the_blocks() {
    for length in [0, 1, 11, 12, 100, MAX_PAYLOAD] {
        let payload: Vec<u8> = (0..length).map(|i| (i % 251) as u8).collect();
        let blocks = encode(&payload).unwrap();
        assert!(blocks.len() <= USABLE_BLOCKS.len());
        assert_eq!(blocks_to_read(&blocks[0]), blocks.len(), "length {length}");
        assert_eq!(decode(&blocks).unwrap(), CardData::Record(payload));
    }
}

#[test]
fn the_maximum_payload_fills_the_card_exactly() {
    assert_eq!(MAX_PAYLOAD, 47 * 16 - 5);
    assert_eq!(encode(&vec![0; MAX_PAYLOAD]).unwrap().len(), 47);
    assert!(matches!(
        encode(&vec![0; MAX_PAYLOAD + 1]),
        Err(RecordError::TooLong(_))
    ));
}

#[test]
fn a_card_without_the_header_reads_as_a_legacy_uuid() {
    let uuid = *uuid::Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8")
        .unwrap()
        .as_bytes();
    assert_eq!(blocks_to_read(&uuid), 1);
    let data = decode(&[uuid]).unwrap();
    assert_eq!(data, CardData::LegacyUuid(uuid));
    assert_eq!(render(&data), "67e55044-10b1-426f-9247-bb680e5fe0c8");
}

#[test]
fn rendering_uses_text_for_utf8_and_hex_otherwise() {
    assert_eq!(
        render(&CardData::Record(b"hello card".to_vec())),
        "hello card"
    );
    assert_eq!(
        render(&CardData::Record(vec![0xff, 0x00, 0x7a])),
        "hex:ff007a"
    );
}

#[test]
fn truncated_or_unknown_records_are_errors() {
    let blocks = encode(&[7; 40]).unwrap();
    assert!(matches!(
        decode(&blocks[..1]),
        Err(RecordError::Truncated { .. })
    ));
    let mut future = blocks.clone();
    future[0][2] = 2;
    assert_eq!(decode(&future), Err(RecordError::UnsupportedVersion(2)));
    assert!(decode(&[]).is_err());
}

#[test]
fn a_corrupt_length_never_asks_for_more_than_the_card() {
    let mut first = encode(&[1; 3]).unwrap()[0];
    first[3] = 0xFF;
    first[4] = 0xFF;
    assert_eq!(blocks_to_read(&first), USABLE_BLOCKS.len());
    assert!(matches!(decode(&[first]), Err(RecordError::TooLong(_))));
}

#[test]
fn the_write_buffer_is_bounded() {
    let mut buffer = WriteBuffer::default();
    buffer.append(&vec![1; MAX_PAYLOAD - 10]).unwrap();
    assert!(buffer.append(&[2; 11]).is_err());
    assert_eq!(
        buffer.as_slice().len(),
        MAX_PAYLOAD - 10,
        "unchanged after a refused append"
    );
    buffer.append(&[2; 10]).unwrap();
    assert!(buffer.append(&[3]).is_err());
    buffer.clear();
    assert!(buffer.is_empty());
}

#[test]
fn reader_versions() {
    for ok in [0x88, 0x91, 0x92, 0xB2] {
        assert!(is_known_reader_version(ok));
    }
    for bad in [0x00, 0xFF, 0x12] {
        assert!(!is_known_reader_version(bad));
    }
}

use super::*;
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::io::Write;

const TEST_MAGIC: [u8; 4] = *b"TEST";

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
struct EvolvingDocument {
    health: f32,
    hunger: f32,
}

#[derive(Serialize)]
struct EarlierDocument {
    health: f32,
    removed_field: u32,
}

#[test]
fn named_document_uses_a_message_pack_map() {
    let encoded = encode_named(TEST_MAGIC, 1, &EvolvingDocument::default()).unwrap();
    let payload = decompress_limited(&encoded[HEADER_SIZE..], 1024).unwrap();

    assert!(matches!(payload.first(), Some(0x80..=0x8f)));
}

#[test]
fn missing_fields_use_default_and_unknown_fields_are_ignored() {
    let earlier = EarlierDocument {
        health: 12.0,
        removed_field: 99,
    };
    let encoded = encode_named(TEST_MAGIC, 1, &earlier).unwrap();
    let decoded: EvolvingDocument = decode_named(&encoded, TEST_MAGIC, 1, 1024).unwrap();

    assert_eq!(decoded.health, 12.0);
    assert_eq!(decoded.hunger, 0.0);
}

#[test]
fn unknown_outer_format_is_rejected() {
    let encoded = encode_named(TEST_MAGIC, 2, &EvolvingDocument::default()).unwrap();
    let error = decode_named::<EvolvingDocument>(&encoded, TEST_MAGIC, 1, 1024).unwrap_err();

    assert!(error.contains("不支持的文档格式 2"));
}

#[test]
fn decompressed_size_is_bounded() {
    let encoded = encode_named(
        TEST_MAGIC,
        1,
        &EarlierDocument {
            health: 12.0,
            removed_field: 99,
        },
    )
    .unwrap();
    let error = decode_named::<EvolvingDocument>(&encoded, TEST_MAGIC, 1, 1).unwrap_err();

    assert!(error.contains("解压后超过"));
}

#[test]
fn corrupted_compressed_payload_is_rejected() {
    let mut encoded = encode_named(TEST_MAGIC, 1, &EvolvingDocument::default()).unwrap();
    encoded.truncate(encoded.len() - 3);

    assert!(decode_named::<EvolvingDocument>(&encoded, TEST_MAGIC, 1, 1024).is_err());
}

#[test]
fn trailing_message_pack_bytes_are_rejected() {
    let encoded = encode_named(TEST_MAGIC, 1, &EvolvingDocument::default()).unwrap();
    let mut payload = decompress_limited(&encoded[HEADER_SIZE..], 1024).unwrap();
    payload.push(0);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&payload).unwrap();
    let compressed = encoder.finish().unwrap();
    let mut document = encoded[..HEADER_SIZE].to_vec();
    document.extend_from_slice(&compressed);

    let error = decode_named::<EvolvingDocument>(&document, TEST_MAGIC, 1, 1024).unwrap_err();
    assert!(error.contains("尾随字节"));
}

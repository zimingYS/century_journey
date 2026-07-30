use super::*;
use crate::shared::identifier::Identifier;

fn tree(root: IVec3) -> TreeInstance {
    TreeInstance::from_persisted(
        root,
        Identifier::new("century_journey", "oak"),
        91,
        TreeGrowthStage::Mature,
        100,
        100,
        1_000,
        120,
        Some(180),
    )
    .unwrap()
}

fn saved_chunk() -> SavedChunk {
    let mut data = ChunkData::new();
    data.voxels[0] = 12;
    SavedChunk {
        position: IVec3::ZERO,
        data,
        tree_instances: vec![tree(IVec3::new(1, 2, 3))],
        modified_time: 45.0,
    }
}

#[test]
fn current_record_round_trip_preserves_tree_identity_and_schedule() {
    let expected = saved_chunk();
    let record = encode_chunk_record(&expected).unwrap();
    let actual = decode_chunk_record(&record).unwrap();

    assert_eq!(actual.position, expected.position);
    assert_eq!(actual.data.voxels, expected.data.voxels);
    assert_eq!(actual.tree_instances, expected.tree_instances);
    assert_eq!(actual.modified_time, expected.modified_time);
}

#[test]
fn legacy_record_migrates_to_an_empty_tree_instance_list() {
    let expected = saved_chunk();
    let legacy = LegacySavedChunk {
        position: expected.position,
        data: expected.data,
        modified_time: expected.modified_time,
    };
    let record = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&legacy)
        .unwrap();

    let migrated = decode_chunk_record(&record).unwrap();
    assert!(migrated.tree_instances.is_empty());
    assert_eq!(migrated.data.voxels[0], 12);
}

#[test]
fn unknown_format_and_trailing_bytes_are_rejected() {
    let mut unknown = encode_chunk_record(&saved_chunk()).unwrap();
    unknown[CHUNK_RECORD_MAGIC.len()..CHUNK_RECORD_HEADER_LEN]
        .copy_from_slice(&(CURRENT_CHUNK_FORMAT_VERSION + 1).to_le_bytes());
    assert!(matches!(
        decode_chunk_record(&unknown),
        Err(SaveError::UnsupportedVersion { .. })
    ));

    let mut trailing = encode_chunk_record(&saved_chunk()).unwrap();
    trailing.push(0);
    assert!(decode_chunk_record(&trailing).is_err());
}

#[test]
fn tree_root_outside_saved_chunk_is_rejected() {
    let payload = CurrentChunkPayload {
        position: IVec3::ZERO,
        data: ChunkData::new(),
        tree_instances: vec![SavedTreeInstance::from(&tree(IVec3::new(16, 2, 3)))],
        modified_time: 1.0,
    };
    let encoded = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&payload)
        .unwrap();
    let mut record = Vec::new();
    record.extend_from_slice(&CHUNK_RECORD_MAGIC);
    record.extend_from_slice(&CURRENT_CHUNK_FORMAT_VERSION.to_le_bytes());
    record.extend_from_slice(&encoded);

    assert!(decode_chunk_record(&record).is_err());
}

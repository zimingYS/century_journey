use super::*;
use crate::shared::identifier::Identifier;
use bincode::Options;

fn tree(root: bevy::math::IVec3) -> crate::game::world::TreeInstance {
    crate::game::world::TreeInstance::from_persisted(
        root,
        Identifier::new("century_journey", "oak"),
        91,
        crate::game::world::TreeGrowthStage::Mature,
        100,
        100,
        1_000,
        120,
        Some(180),
    )
    .unwrap()
}

fn saved_chunk() -> SavedChunk {
    let mut data = crate::game::world::chunk::ChunkData::new();
    data.voxels[0] = 12;
    SavedChunk {
        position: bevy::math::IVec3::ZERO,
        data,
        tree_instances: vec![tree(bevy::math::IVec3::new(1, 2, 3))],
        modified_time: 45.0,
    }
}

#[test]
fn section_record_round_trip_preserves_tree_identity_and_schedule() {
    let expected = saved_chunk();
    let record = encode_chunk_record(&expected).unwrap();
    let actual = decode_chunk_record(&record).unwrap();

    assert_eq!(actual.position, expected.position);
    assert_eq!(actual.data.voxels, expected.data.voxels);
    assert_eq!(actual.tree_instances, expected.tree_instances);
    assert_eq!(actual.modified_time, expected.modified_time);
}

#[test]
fn field_codec_round_trips_all_named_growth_stages() {
    let root = bevy::math::IVec3::new(1, 2, 3);
    let stages = [
        (crate::game::world::TreeGrowthStage::Sapling, Some(130)),
        (crate::game::world::TreeGrowthStage::Young, Some(180)),
        (crate::game::world::TreeGrowthStage::Mature, None),
    ];

    for (stage, next_update) in stages {
        let instance = crate::game::world::TreeInstance::from_persisted(
            root,
            Identifier::new("century_journey", "oak"),
            91,
            stage,
            100,
            100,
            1_000,
            120,
            next_update,
        )
        .unwrap();
        let payload = tree_fields::encode(std::slice::from_ref(&instance)).unwrap();

        assert_eq!(tree_fields::decode(&payload).unwrap(), vec![instance]);
    }
}

#[test]
fn missing_tree_section_defaults_to_empty_and_unknown_section_is_skipped() {
    let expected = saved_chunk();
    let mut sections = section::build_sections(&expected).unwrap();
    sections.retain(|section| section.name != "tree_instances");
    sections.insert(
        1,
        section::Section {
            name: "weather_state".into(),
            payload: vec![9, 8, 7],
        },
    );
    sections.reverse();

    let decoded = decode_chunk_record(&section::encode_container(&sections).unwrap()).unwrap();
    assert!(decoded.tree_instances.is_empty());
    assert_eq!(decoded.data.voxels[0], 12);
}

#[test]
fn unknown_tree_field_is_skipped_without_losing_known_fields() {
    let expected = saved_chunk();
    let mut sections = section::build_sections(&expected).unwrap();
    let trees = sections
        .iter_mut()
        .find(|section| section.name == "tree_instances")
        .unwrap();
    let old_record_len = u32::from_le_bytes(trees.payload[4..8].try_into().unwrap()) as usize;
    trees.payload.extend_from_slice(&999u16.to_le_bytes());
    trees.payload.extend_from_slice(&3u32.to_le_bytes());
    trees.payload.extend_from_slice(&[1, 2, 3]);
    trees.payload[4..8].copy_from_slice(&(old_record_len as u32 + 9).to_le_bytes());

    let decoded = decode_chunk_record(&section::encode_container(&sections).unwrap()).unwrap();
    assert_eq!(decoded.tree_instances, expected.tree_instances);
}

#[test]
fn positional_and_bare_bincode_layouts_remain_readable() {
    let expected = saved_chunk();
    let positional = legacy::PositionalChunkPayload {
        position: expected.position,
        data: expected.data.clone(),
        tree_instances: expected
            .tree_instances
            .iter()
            .map(legacy::PositionalTreeRecord::from)
            .collect(),
        modified_time: expected.modified_time,
    };
    let payload = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&positional)
        .unwrap();
    let mut record = CHUNK_RECORD_MAGIC.to_vec();
    record.extend_from_slice(&POSITIONAL_CHUNK_FORMAT.to_le_bytes());
    record.extend_from_slice(&payload);
    let decoded = decode_chunk_record(&record).unwrap();
    assert_eq!(decoded.tree_instances, expected.tree_instances);

    let bare = legacy::BareChunkPayload {
        position: expected.position,
        data: expected.data,
        modified_time: expected.modified_time,
    };
    let record = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&bare)
        .unwrap();
    let decoded = decode_chunk_record(&record).unwrap();
    assert!(decoded.tree_instances.is_empty());
    assert_eq!(decoded.data.voxels[0], 12);
}

#[test]
fn duplicate_or_missing_required_sections_are_rejected() {
    let expected = saved_chunk();
    let sections = section::build_sections(&expected).unwrap();
    let mut duplicate = sections.clone();
    duplicate.push(sections[0].clone());
    assert!(decode_chunk_record(&section::encode_container(&duplicate).unwrap()).is_err());

    for missing in ["metadata", "blocks"] {
        let remaining: Vec<_> = sections
            .iter()
            .filter(|section| section.name != missing)
            .cloned()
            .collect();
        assert!(decode_chunk_record(&section::encode_container(&remaining).unwrap()).is_err());
    }
}

#[test]
fn malformed_lengths_counts_and_trailing_bytes_are_rejected() {
    let expected = saved_chunk();
    let mut truncated = encode_chunk_record(&expected).unwrap();
    truncated.pop();
    assert!(decode_chunk_record(&truncated).is_err());

    let mut trailing = encode_chunk_record(&expected).unwrap();
    trailing.push(0);
    assert!(decode_chunk_record(&trailing).is_err());

    let mut sections = section::build_sections(&expected).unwrap();
    let blocks = sections
        .iter_mut()
        .find(|section| section.name == "blocks")
        .unwrap();
    blocks.payload[7..11].copy_from_slice(&1u32.to_le_bytes());
    assert!(decode_chunk_record(&section::encode_container(&sections).unwrap()).is_err());
}

#[test]
fn unknown_top_level_format_is_rejected() {
    let mut record = encode_chunk_record(&saved_chunk()).unwrap();
    record[CHUNK_RECORD_MAGIC.len()..CHUNK_RECORD_HEADER_LEN]
        .copy_from_slice(&(CURRENT_CHUNK_FORMAT + 1).to_le_bytes());
    assert!(matches!(
        decode_chunk_record(&record),
        Err(SaveError::UnsupportedVersion { .. })
    ));
}

#[test]
fn restored_tree_must_belong_to_metadata_chunk() {
    let expected = saved_chunk();
    let mut sections = section::build_sections(&expected).unwrap();
    let metadata = sections
        .iter_mut()
        .find(|section| section.name == "metadata")
        .unwrap();
    metadata.payload[0..4].copy_from_slice(&1i32.to_le_bytes());

    assert!(decode_chunk_record(&section::encode_container(&sections).unwrap()).is_err());
}

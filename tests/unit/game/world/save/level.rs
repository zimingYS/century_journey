use super::*;

fn sample_level() -> LevelData {
    LevelData {
        version: LevelData::CURRENT_VERSION,
        game_version: LevelData::GAME_VERSION.to_string(),
        seed: 42,
        generation_version: CURRENT_GENERATION_VERSION,
        simulation_tick: 123,
        game_minute: 480,
        subminute_tick: 0,
        spawn_position: [1.0, 70.0, -2.0],
        time_of_day: 8.0,
        block_id_map: vec![(1, "century_journey:stone".into())],
    }
}

#[test]
fn current_level_round_trip_keeps_time_and_game_version() {
    let decoded = decode_level(&encode_level(&sample_level()).unwrap()).unwrap();
    assert_eq!(decoded.version, LevelData::CURRENT_VERSION);
    assert_eq!(decoded.game_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(decoded.seed, 42);
    assert_eq!(decoded.generation_version, CURRENT_GENERATION_VERSION);
    assert_eq!(decoded.simulation_tick, 123);
    assert_eq!(decoded.game_minute, 480);
    assert_eq!(decoded.subminute_tick, 0);
    assert_eq!(decoded.time_of_day, 8.0);
    assert_eq!(decoded.spawn_position, [1.0, 70.0, -2.0]);
}

#[test]
fn legacy_float_version_has_an_explicit_migration() {
    let legacy = LegacyLevelDataV0 {
        seed: 7,
        spawn_position: [0.0, 70.0, 0.0],
        time_of_day: 12.0,
        block_id_map: Vec::new(),
        version: 0.1,
    };
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&legacy)
        .unwrap();
    let decoded = decode_level(&compress(&serialized).unwrap()).unwrap();

    assert_eq!(decoded.version, LevelData::CURRENT_VERSION);
    assert_eq!(decoded.game_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(decoded.seed, 7);
    assert_eq!(decoded.generation_version, LEGACY_GENERATION_VERSION);
    assert_eq!(decoded.time_of_day, 12.0);
}

#[test]
fn v1_level_file_migrates_to_the_legacy_generation_version() {
    let legacy = LegacyLevelDataV1 {
        version: 1,
        game_version: "0.2.0".into(),
        seed: 99,
        spawn_position: [0.0, 70.0, 0.0],
        time_of_day: 18.0,
        block_id_map: Vec::new(),
    };
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&legacy)
        .unwrap();
    let mut encoded = LEVEL_MAGIC.to_vec();
    encoded.extend(compress(&serialized).unwrap());

    let decoded = decode_level(&encoded).unwrap();

    assert_eq!(decoded.version, LevelData::CURRENT_VERSION);
    assert_eq!(decoded.seed, 99);
    assert_eq!(decoded.generation_version, LEGACY_GENERATION_VERSION);
}

#[test]
fn v2_level_file_migrates_legacy_float_time_to_simulation_clock() {
    let legacy = LegacyLevelDataV2 {
        version: 2,
        game_version: "0.3.0".into(),
        seed: 101,
        generation_version: CURRENT_GENERATION_VERSION,
        spawn_position: [0.0, 70.0, 0.0],
        time_of_day: 13.5,
        block_id_map: Vec::new(),
    };
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&legacy)
        .unwrap();
    let mut encoded = LEVEL_MAGIC.to_vec();
    encoded.extend(compress(&serialized).unwrap());

    let decoded = decode_level(&encoded).unwrap();

    assert_eq!(decoded.version, LevelData::CURRENT_VERSION);
    assert_eq!(decoded.generation_version, CURRENT_GENERATION_VERSION);
    assert_eq!(decoded.game_minute, 810);
    assert_eq!(decoded.simulation_tick, 0);
    assert_eq!(decoded.time_of_day, 13.5);
}

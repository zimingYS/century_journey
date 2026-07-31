use super::*;
use crate::game::save::world::metadata::legacy_bincode::{
    FloatVersionLevel, GameVersionLevel, GenerationLevel, SimulationClockLevel,
};
use serde::Serialize;

fn sample_level() -> LevelData {
    LevelData {
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

fn legacy_envelope<T: Serialize>(value: &T) -> Vec<u8> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(value)
        .unwrap();
    let mut encoded = LEGACY_LEVEL_MAGIC.to_vec();
    encoded.extend(compress(&serialized).unwrap());
    encoded
}

#[test]
fn current_level_round_trip_keeps_time_and_game_version() {
    let decoded = decode_level(&encode_level(&sample_level()).unwrap()).unwrap();
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
fn named_document_defaults_missing_fields_and_ignores_unknown_fields() {
    #[derive(Serialize)]
    struct EarlierFields {
        seed: u64,
        generation_version: u32,
        retired_weather_code: u32,
    }

    let bytes = document::encode_named(
        LEVEL_MAGIC,
        LEVEL_DOCUMENT_FORMAT,
        &EarlierFields {
            seed: 77,
            generation_version: CURRENT_GENERATION_VERSION,
            retired_weather_code: 9,
        },
    )
    .unwrap();
    let decoded = decode_level(&bytes).unwrap();

    assert_eq!(decoded.seed, 77);
    assert_eq!(decoded.game_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(decoded.spawn_position, [0.0, 70.0, 0.0]);
    assert_eq!(decoded.time_of_day, 8.0);
}

#[test]
fn unknown_outer_document_format_is_rejected() {
    let mut bytes = encode_level(&sample_level()).unwrap();
    bytes[4..8].copy_from_slice(&(LEVEL_DOCUMENT_FORMAT + 1).to_le_bytes());
    assert!(matches!(
        decode_level(&bytes),
        Err(SaveError::Serialize(message)) if message.contains("不支持的文档格式")
    ));
}

#[test]
fn float_time_layout_without_magic_is_migrated() {
    let legacy = FloatVersionLevel {
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

    assert_eq!(decoded.seed, 7);
    assert_eq!(decoded.generation_version, LEGACY_GENERATION_VERSION);
    assert_eq!(decoded.time_of_day, 12.0);
}

#[test]
fn game_version_layout_uses_legacy_generation_algorithm() {
    let decoded = decode_level(&legacy_envelope(&GameVersionLevel {
        version: 1,
        game_version: "0.2.0".into(),
        seed: 99,
        spawn_position: [0.0, 70.0, 0.0],
        time_of_day: 18.0,
        block_id_map: Vec::new(),
    }))
    .unwrap();

    assert_eq!(decoded.seed, 99);
    assert_eq!(decoded.generation_version, LEGACY_GENERATION_VERSION);
    assert_eq!(decoded.time_of_day, 18.0);
}

#[test]
fn generation_layout_converts_float_time_to_simulation_clock() {
    let decoded = decode_level(&legacy_envelope(&GenerationLevel {
        version: 2,
        game_version: "0.3.0".into(),
        seed: 101,
        generation_version: CURRENT_GENERATION_VERSION,
        spawn_position: [0.0, 70.0, 0.0],
        time_of_day: 13.5,
        block_id_map: Vec::new(),
    }))
    .unwrap();

    assert_eq!(decoded.generation_version, CURRENT_GENERATION_VERSION);
    assert_eq!(decoded.game_minute, 810);
    assert_eq!(decoded.simulation_tick, 0);
    assert_eq!(decoded.time_of_day, 13.5);
}

#[test]
fn final_positional_layout_restores_authoritative_clock() {
    let decoded = decode_level(&legacy_envelope(&SimulationClockLevel {
        version: 3,
        game_version: "0.4.0".into(),
        seed: 202,
        generation_version: CURRENT_GENERATION_VERSION,
        simulation_tick: 444,
        game_minute: 600,
        subminute_tick: 3,
        spawn_position: [2.0, 72.0, 4.0],
        time_of_day: 0.0,
        block_id_map: Vec::new(),
    }))
    .unwrap();

    assert_eq!(decoded.simulation_tick, 444);
    assert_eq!(decoded.game_minute, 600);
    assert_eq!(decoded.subminute_tick, 3);
    assert!((decoded.time_of_day - 10.0025).abs() < 0.000_01);
}

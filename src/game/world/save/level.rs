use crate::content::block::registry::BlockRegistry;
use crate::engine::persistence;
use crate::game::world::save::format::LevelData;
use crate::game::world::save::region::{RegionManager, SaveError};
use bevy::prelude::*;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

const LEVEL_MAGIC: &[u8; 4] = b"CJLV";

#[derive(serde::Serialize, serde::Deserialize)]
struct LegacyLevelDataV0 {
    seed: u64,
    spawn_position: [f32; 3],
    time_of_day: f32,
    block_id_map: Vec<(u16, String)>,
    version: f32,
}

/// 检测存档是否存在
pub fn world_exists(world_name: &str) -> bool {
    let path = RegionManager::level_path(world_name);
    path.exists() || persistence::backup_path(&path).exists()
}

/// 保存世界配置数据到level.dat
pub fn save_level(
    world_name: &str,
    seed: u64,
    spawn_pos: Vec3,
    time_of_day: f32,
    block_registry: &BlockRegistry,
) -> Result<(), SaveError> {
    // 确保当前世界文件存在
    RegionManager::ensure_dirs(world_name)?;

    // 构建ID映射表(将动态ID转换为方块标识符)
    let block_id_map = block_registry.build_save_id_map();

    let level = LevelData {
        version: LevelData::CURRENT_VERSION,
        game_version: LevelData::GAME_VERSION.to_string(),
        seed,
        spawn_position: [spawn_pos.x, spawn_pos.y, spawn_pos.z],
        time_of_day,
        block_id_map,
    };

    let path = RegionManager::level_path(world_name);
    let encoded = encode_level(&level)?;
    persistence::atomic_write_verified(&path, &encoded, validate_level_bytes)?;
    Ok(())
}

/// 从level.dat加载世界元数据
pub fn load_level(world_name: &str) -> Result<LevelData, SaveError> {
    let path = RegionManager::level_path(world_name);
    let bytes = persistence::read_verified(&path, validate_level_bytes)?;
    decode_level(&bytes)
}

/// 从最近有效备份读取世界元数据，但不修改主文件。
pub fn load_level_backup(world_name: &str) -> Result<LevelData, SaveError> {
    let path = RegionManager::level_path(world_name);
    let bytes = persistence::read_backup_verified(&path, validate_level_bytes)?;
    decode_level(&bytes)
}

/// 判断世界元数据是否存在有效备份。
pub fn level_backup_available(world_name: &str) -> bool {
    let path = RegionManager::level_path(world_name);
    persistence::has_valid_backup(&path, validate_level_bytes)
}

/// 用最近有效备份恢复世界元数据。
pub fn restore_level_backup(world_name: &str) -> Result<(), SaveError> {
    let path = RegionManager::level_path(world_name);
    persistence::restore_backup(&path, validate_level_bytes)?;
    Ok(())
}

fn encode_level(level: &LevelData) -> Result<Vec<u8>, SaveError> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(level)?;
    let compressed = compress(&serialized)?;
    let mut encoded = Vec::with_capacity(LEVEL_MAGIC.len() + compressed.len());
    encoded.extend_from_slice(LEVEL_MAGIC);
    encoded.extend_from_slice(&compressed);
    Ok(encoded)
}

fn decode_level(bytes: &[u8]) -> Result<LevelData, SaveError> {
    if let Some(compressed) = bytes.strip_prefix(LEVEL_MAGIC) {
        let decompressed = decompress(compressed)?;
        let current = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .reject_trailing_bytes()
            .deserialize::<LevelData>(&decompressed)?;
        return migrate_level_data(current);
    }

    let decompressed = decompress(bytes)?;
    let legacy = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize::<LegacyLevelDataV0>(&decompressed)?;
    migrate_legacy_level_data(legacy)
}

fn migrate_level_data(mut level: LevelData) -> Result<LevelData, SaveError> {
    match level.version {
        0 => {
            level.version = 1;
            level.game_version = LevelData::GAME_VERSION.to_string();
        }
        1 => {}
        found => {
            return Err(SaveError::UnsupportedVersion {
                found,
                supported: LevelData::CURRENT_VERSION,
            });
        }
    }

    if !level.time_of_day.is_finite() {
        level.time_of_day = crate::shared::time::NEW_WORLD_START_TIME;
    } else {
        level.time_of_day = level.time_of_day.rem_euclid(24.0);
    }
    Ok(level)
}

fn migrate_legacy_level_data(legacy: LegacyLevelDataV0) -> Result<LevelData, SaveError> {
    if !legacy.version.is_finite() || legacy.version > 0.1 {
        return Err(SaveError::Serialize(format!(
            "无法迁移旧世界格式版本 {}",
            legacy.version
        )));
    }
    migrate_level_data(LevelData {
        version: LevelData::CURRENT_VERSION,
        game_version: LevelData::GAME_VERSION.to_string(),
        seed: legacy.seed,
        spawn_position: legacy.spawn_position,
        time_of_day: legacy.time_of_day,
        block_id_map: legacy.block_id_map,
    })
}

fn validate_level_bytes(bytes: &[u8]) -> Result<(), String> {
    decode_level(bytes)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn compress(data: &[u8]) -> Result<Vec<u8>, SaveError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data)?;
    encoder.finish().map_err(SaveError::Io)
}

fn decompress(data: &[u8]) -> Result<Vec<u8>, SaveError> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

// 根据存档中的ID映射表，将区块数据中的方块ID重映射到当前运行时ID
pub fn remap_chunk_block_ids(
    chunk_data: &mut crate::game::world::chunk::ChunkData,
    saved_id_map: &[(u16, String)],
    current_registry: &BlockRegistry,
) {
    // 构建映射：保存时的 runtime_id -> 当前的 runtime_id
    let remap = current_registry.build_id_remap_table(saved_id_map);

    // 重映射每个方块
    for voxel in chunk_data.voxels.iter_mut() {
        if let Some(&new_id) = remap.get(voxel) {
            *voxel = new_id;
        } else {
            // 未知方块 ID，回退为空气
            log::warn!("未知的方块 ID {}，替换为空气", voxel);
            *voxel = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_level() -> LevelData {
        LevelData {
            version: LevelData::CURRENT_VERSION,
            game_version: LevelData::GAME_VERSION.to_string(),
            seed: 42,
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
        assert_eq!(decoded.time_of_day, 12.0);
    }
}

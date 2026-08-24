//! 编解码世界元数据命名文档，并在边界处读取冻结的历史 bincode 布局。

use crate::content::block::registry::BlockRegistry;
use crate::engine::{document, persistence};
use crate::game::save::world::chunk::region::{RegionManager, SaveError};
use crate::game::save::world::metadata::legacy_bincode::{
    FloatVersionLevel, GameVersionLevel, GenerationLevel, SimulationClockLevel,
};
use crate::game::save::world::metadata::model::LevelData;
use crate::game::world::generation::pipeline::{
    CURRENT_GENERATION_VERSION, LEGACY_GENERATION_VERSION,
};
use crate::game::world::time::WorldSimulationClock;
use bevy::math::Vec3;
use bevy::prelude;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

/// 当前世界元数据命名文档的格式标识。
pub const LEVEL_MAGIC: [u8; 4] = *b"CJLM";
/// 当前外层文档编码格式；普通业务字段变化不得递增此值。
pub const LEVEL_DOCUMENT_FORMAT: u32 = 1;
/// 历史顺序式 bincode 世界元数据的格式标识。
pub(super) const LEGACY_LEVEL_MAGIC: &[u8; 4] = b"CJLV";
const MAX_LEVEL_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;
const LAST_LEGACY_LAYOUT_VERSION: u32 = 3;

/// 检测世界主元数据或其备份是否存在。
pub fn world_exists(world_name: &str) -> bool {
    let path = RegionManager::level_path(world_name);
    path.exists() || persistence::backup_path(&path).exists()
}

/// 将世界元数据原子写入 `level.dat`，并保留可恢复备份。
pub fn save_level(
    world_name: &str,
    seed: u64,
    generation_version: u32,
    clock: &WorldSimulationClock,
    spawn_pos: Vec3,
    block_registry: &BlockRegistry,
) -> prelude::Result<(), SaveError> {
    RegionManager::ensure_dirs(world_name)?;
    let level = LevelData {
        game_version: LevelData::GAME_VERSION.to_string(),
        seed,
        generation_version,
        simulation_tick: clock.simulation_tick(),
        game_minute: clock.total_game_minutes(),
        subminute_tick: clock.subminute_tick(),
        spawn_position: [spawn_pos.x, spawn_pos.y, spawn_pos.z],
        time_of_day: clock.visual_hour(0.0),
        block_id_map: block_registry.build_save_id_map(),
    };

    let path = RegionManager::level_path(world_name);
    let encoded = encode_level(&level)?;
    persistence::atomic_write_verified(&path, &encoded, validate_level_bytes)?;
    Ok(())
}

/// 从 `level.dat` 加载世界元数据并执行必要的语义规范化。
pub fn load_level(world_name: &str) -> prelude::Result<LevelData, SaveError> {
    let path = RegionManager::level_path(world_name);
    let bytes = persistence::read_verified(&path, validate_level_bytes)?;
    decode_level(&bytes)
}

/// 从最近有效备份读取世界元数据，但不修改主文件。
pub fn load_level_backup(world_name: &str) -> prelude::Result<LevelData, SaveError> {
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
pub fn restore_level_backup(world_name: &str) -> prelude::Result<(), SaveError> {
    let path = RegionManager::level_path(world_name);
    persistence::restore_backup(&path, validate_level_bytes)?;
    Ok(())
}

/// 将当前世界元数据编码为带字段名的 MessagePack+gzip 文档。
pub fn encode_level(level: &LevelData) -> prelude::Result<Vec<u8>, SaveError> {
    document::encode_named(LEVEL_MAGIC, LEVEL_DOCUMENT_FORMAT, level).map_err(SaveError::Serialize)
}

/// 识别当前命名文档或冻结的历史 bincode 布局。
pub fn decode_level(bytes: &[u8]) -> prelude::Result<LevelData, SaveError> {
    let level = if document::has_magic(bytes, LEVEL_MAGIC) {
        document::decode_named(
            bytes,
            LEVEL_MAGIC,
            LEVEL_DOCUMENT_FORMAT,
            MAX_LEVEL_DOCUMENT_BYTES,
        )
        .map_err(SaveError::Serialize)?
    } else {
        decode_legacy_level(bytes)?
    };
    normalize_level(level)
}

fn decode_legacy_level(bytes: &[u8]) -> prelude::Result<LevelData, SaveError> {
    if let Some(compressed) = bytes.strip_prefix(LEGACY_LEVEL_MAGIC) {
        let payload = decompress(compressed)?;

        if let Ok(legacy) = legacy_options().deserialize::<SimulationClockLevel>(&payload) {
            if legacy.version != LAST_LEGACY_LAYOUT_VERSION {
                return Err(SaveError::UnsupportedVersion {
                    found: legacy.version,
                    supported: LAST_LEGACY_LAYOUT_VERSION,
                });
            }
            return Ok(LevelData {
                game_version: legacy.game_version,
                seed: legacy.seed,
                generation_version: legacy.generation_version,
                simulation_tick: legacy.simulation_tick,
                game_minute: legacy.game_minute,
                subminute_tick: legacy.subminute_tick,
                spawn_position: legacy.spawn_position,
                time_of_day: legacy.time_of_day,
                block_id_map: legacy.block_id_map,
            });
        }

        if let Ok(legacy) = legacy_options().deserialize::<GenerationLevel>(&payload) {
            if legacy.version != 2 {
                return Err(SaveError::UnsupportedVersion {
                    found: legacy.version,
                    supported: LAST_LEGACY_LAYOUT_VERSION,
                });
            }
            let clock = WorldSimulationClock::from_legacy_time_of_day(legacy.time_of_day);
            return Ok(level_from_legacy_clock(
                legacy.game_version,
                legacy.seed,
                legacy.generation_version,
                legacy.spawn_position,
                legacy.block_id_map,
                &clock,
            ));
        }

        let legacy = legacy_options().deserialize::<GameVersionLevel>(&payload)?;
        if legacy.version > 1 {
            return Err(SaveError::UnsupportedVersion {
                found: legacy.version,
                supported: LAST_LEGACY_LAYOUT_VERSION,
            });
        }
        let clock = WorldSimulationClock::from_legacy_time_of_day(legacy.time_of_day);
        return Ok(level_from_legacy_clock(
            legacy.game_version,
            legacy.seed,
            LEGACY_GENERATION_VERSION,
            legacy.spawn_position,
            legacy.block_id_map,
            &clock,
        ));
    }

    let payload = decompress(bytes)?;
    let legacy = legacy_options().deserialize::<FloatVersionLevel>(&payload)?;
    if !legacy.version.is_finite() || legacy.version > 0.1 {
        return Err(SaveError::Serialize(format!(
            "无法迁移旧世界格式版本 {}",
            legacy.version
        )));
    }
    let clock = WorldSimulationClock::from_legacy_time_of_day(legacy.time_of_day);
    Ok(level_from_legacy_clock(
        LevelData::GAME_VERSION.to_string(),
        legacy.seed,
        LEGACY_GENERATION_VERSION,
        legacy.spawn_position,
        legacy.block_id_map,
        &clock,
    ))
}

fn legacy_options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
}

fn level_from_legacy_clock(
    game_version: String,
    seed: u64,
    generation_version: u32,
    spawn_position: [f32; 3],
    block_id_map: Vec<(u16, String)>,
    clock: &WorldSimulationClock,
) -> LevelData {
    LevelData {
        game_version,
        seed,
        generation_version,
        simulation_tick: clock.simulation_tick(),
        game_minute: clock.total_game_minutes(),
        subminute_tick: clock.subminute_tick(),
        spawn_position,
        time_of_day: clock.visual_hour(0.0),
        block_id_map,
    }
}

fn normalize_level(mut level: LevelData) -> prelude::Result<LevelData, SaveError> {
    if !(LEGACY_GENERATION_VERSION..=CURRENT_GENERATION_VERSION).contains(&level.generation_version)
    {
        return Err(SaveError::Serialize(format!(
            "不支持的基础地形生成版本 {}，当前支持 {}..={}",
            level.generation_version, LEGACY_GENERATION_VERSION, CURRENT_GENERATION_VERSION
        )));
    }
    if level.game_version.is_empty() {
        level.game_version = LevelData::GAME_VERSION.to_string();
    }

    let clock = WorldSimulationClock::from_persisted(
        level.simulation_tick,
        level.game_minute,
        level.subminute_tick,
    );
    level.simulation_tick = clock.simulation_tick();
    level.game_minute = clock.total_game_minutes();
    level.subminute_tick = clock.subminute_tick();
    level.time_of_day = clock.visual_hour(0.0);
    Ok(level)
}

fn validate_level_bytes(bytes: &[u8]) -> prelude::Result<(), String> {
    decode_level(bytes)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// 为历史兼容测试和只读适配压缩 bincode 载荷。
pub fn compress(data: &[u8]) -> prelude::Result<Vec<u8>, SaveError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data)?;
    encoder.finish().map_err(SaveError::Io)
}

fn decompress(data: &[u8]) -> prelude::Result<Vec<u8>, SaveError> {
    let mut decoder = GzDecoder::new(data).take((MAX_LEVEL_DOCUMENT_BYTES + 1) as u64);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    if decompressed.len() > MAX_LEVEL_DOCUMENT_BYTES {
        return Err(SaveError::Serialize(format!(
            "世界元数据解压后超过 {} 字节上限",
            MAX_LEVEL_DOCUMENT_BYTES
        )));
    }
    Ok(decompressed)
}

/// 根据存档标识映射把区块方块编号重映射到当前运行时编号。
///
/// 未知 ID 的体素替换为空气；按出现次数汇总告警，避免大面积未知时刷屏。
pub fn remap_chunk_block_ids(
    chunk_data: &mut crate::game::world::chunk::ChunkData,
    saved_id_map: &[(u16, String)],
    current_registry: &BlockRegistry,
) {
    let remap = current_registry.build_id_remap_table(saved_id_map);
    let mut unknown_counts: std::collections::BTreeMap<u16, usize> = Default::default();
    for voxel in chunk_data.voxels.iter_mut() {
        if let Some(&new_id) = remap.get(voxel) {
            *voxel = new_id;
        } else {
            *unknown_counts.entry(*voxel).or_default() += 1;
            *voxel = 0;
        }
    }
    for (unknown_id, count) in unknown_counts {
        log::warn!(
            "[存档系统] 未知的方块 ID {} 出现 {count} 次，已替换为空气",
            unknown_id
        );
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/world/metadata/io.rs"]
mod tests;

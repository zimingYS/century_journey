//! 执行玩家存档的原子读写、备份检测和恢复。

use crate::engine::persistence;
use crate::game::save::path::world_save_root;
use crate::game::save::player::data::migration;
use crate::game::save::player::data::migration::{
    LegacyPlayerSaveDataV3, LegacyPlayerSaveDataV4, LegacyPlayerSaveDataV5, LegacyPlayerSaveDataV6,
};
#[cfg(test)]
use crate::game::save::player::data::migration::{LegacySaveItemStack, LegacySaveItemStackV6};
use crate::game::save::player::data::model::PlayerSaveData;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

const PLAYER_MAGIC: &[u8; 4] = b"CJPL";
const PLAYER_DIRECTORY_NAME: &str = "players";
const SINGLE_PLAYER_FILE_NAME: &str = "singleplayer.dat";

/// 序列化并压缩写入玩家数据
pub fn write_player_data(data: &PlayerSaveData, path: &std::path::Path) -> Result<(), String> {
    let compressed = encode_player_data(data)?;
    persistence::atomic_write_verified(path, &compressed, validate_player_bytes)
        .map_err(|error| error.to_string())
}

fn encode_player_data(data: &PlayerSaveData) -> Result<Vec<u8>, String> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(data)
        .map_err(|e| format!("bincode serialize: {e}"))?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(&serialized)
        .map_err(|e| format!("gzip write: {e}"))?;
    let compressed = encoder.finish().map_err(|e| format!("gzip finish: {e}"))?;
    let mut encoded = Vec::with_capacity(PLAYER_MAGIC.len() + compressed.len());
    encoded.extend_from_slice(PLAYER_MAGIC);
    encoded.extend_from_slice(&compressed);
    Ok(encoded)
}

/// 读取并解压反序列化玩家数据
pub fn read_player_data(path: &std::path::Path) -> Result<PlayerSaveData, String> {
    let bytes = persistence::read_verified(path, validate_player_bytes)
        .map_err(|error| error.to_string())?;
    decode_player_data(&bytes)
}

/// 从最近一次有效备份读取玩家数据，但不修改主文件。
pub fn read_player_backup(path: &std::path::Path) -> Result<PlayerSaveData, String> {
    let bytes = persistence::read_backup_verified(path, validate_player_bytes)
        .map_err(|error| error.to_string())?;
    decode_player_data(&bytes)
}

/// 判断指定玩家存档是否存在可通过完整性校验的备份。
pub fn player_backup_available(path: &std::path::Path) -> bool {
    persistence::has_valid_backup(path, validate_player_bytes)
}

/// 使用最近一次有效备份原子恢复玩家主存档。
pub fn restore_player_backup(path: &std::path::Path) -> Result<(), String> {
    persistence::restore_backup(path, validate_player_bytes).map_err(|error| error.to_string())
}

fn decode_player_data(bytes: &[u8]) -> Result<PlayerSaveData, String> {
    let (compressed, current_format) = match bytes.strip_prefix(PLAYER_MAGIC) {
        Some(data) => (data, true),
        None => (bytes, false),
    };
    let mut decoder = GzDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("gzip decompress: {e}"))?;
    if current_format {
        if let Ok(data) = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .reject_trailing_bytes()
            .deserialize::<PlayerSaveData>(&decompressed)
        {
            return migration::migrate_player_data(data);
        }
        if let Ok(legacy) = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .reject_trailing_bytes()
            .deserialize::<LegacyPlayerSaveDataV6>(&decompressed)
        {
            return Ok(legacy.into());
        }
        let legacy = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .reject_trailing_bytes()
            .deserialize::<LegacyPlayerSaveDataV5>(&decompressed)
            .map_err(|error| format!("bincode deserialize v7/v6/v5: {error}"))?;
        return Ok(legacy.into());
    }

    if let Ok(legacy) = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize::<LegacyPlayerSaveDataV4>(&decompressed)
    {
        return Ok(legacy.into());
    }
    let legacy = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize::<LegacyPlayerSaveDataV3>(&decompressed)
        .map_err(|e| format!("bincode deserialize (v5/v4/v3): {e}"))?;
    Ok(legacy.into())
}

fn validate_player_bytes(bytes: &[u8]) -> Result<(), String> {
    decode_player_data(bytes).map(|_| ())
}

/// 返回指定世界内部的单机玩家存档路径。
///
/// 玩家数据必须位于世界根目录内，才能与世界创建、删除和备份生命周期保持一致。
pub fn player_save_path(world_name: &str) -> std::path::PathBuf {
    world_save_root(world_name)
        .join(PLAYER_DIRECTORY_NAME)
        .join(SINGLE_PLAYER_FILE_NAME)
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/save/player/io.rs"]
mod tests;

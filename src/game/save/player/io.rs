//! 执行玩家存档的原子读写、备份检测和恢复。

use crate::engine::{document, persistence};
use crate::game::save::path::world_save_root;
use crate::game::save::player::data::legacy_bincode;
#[cfg(test)]
use crate::game::save::player::data::legacy_bincode::{
    DurabilityLayout, DurabilityStack, EquipmentInventoryLayout, ExpandedInventoryLayout,
    GameBuildLayout, IdentifierCountStack, RuntimeIdMapLayout, RuntimeMappedStack,
};
use crate::game::save::player::data::model::PlayerSaveData;

const PLAYER_DOCUMENT_MAGIC: [u8; 4] = *b"CJPM";
const LEGACY_PLAYER_MAGIC: &[u8; 4] = b"CJPL";
const PLAYER_DOCUMENT_FORMAT: u32 = 1;
const PLAYER_MAX_DECOMPRESSED_BYTES: usize = 4 * 1024 * 1024;
const PLAYER_DIRECTORY_NAME: &str = "players";
const SINGLE_PLAYER_FILE_NAME: &str = "singleplayer.dat";

/// 使用命名 MessagePack 文档原子写入当前玩家数据。
pub fn write_player_data(data: &PlayerSaveData, path: &std::path::Path) -> Result<(), String> {
    let compressed = encode_player_data(data)?;
    persistence::atomic_write_verified(path, &compressed, validate_player_bytes)
        .map_err(|error| error.to_string())
}

fn encode_player_data(data: &PlayerSaveData) -> Result<Vec<u8>, String> {
    document::encode_named(PLAYER_DOCUMENT_MAGIC, PLAYER_DOCUMENT_FORMAT, data)
}

/// 读取玩家数据；当前命名文档按字段兼容，历史 bincode 文件通过冻结布局只读升级。
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
    if document::has_magic(bytes, PLAYER_DOCUMENT_MAGIC) {
        return document::decode_named(
            bytes,
            PLAYER_DOCUMENT_MAGIC,
            PLAYER_DOCUMENT_FORMAT,
            PLAYER_MAX_DECOMPRESSED_BYTES,
        );
    }

    if let Some(compressed) = bytes.strip_prefix(LEGACY_PLAYER_MAGIC) {
        return legacy_bincode::decode(compressed, true);
    }
    legacy_bincode::decode(bytes, false)
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

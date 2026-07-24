use super::player_model::{PlayerSaveData, SAVE_VERSION, SaveItemStack};
use crate::engine::persistence;
use crate::game::inventory::container::survival::SurvivalInventory;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const PLAYER_MAGIC: &[u8; 4] = b"CJPL";

#[derive(Serialize, Deserialize, Clone)]
struct LegacySaveItemStack {
    item: String,
    count: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct LegacySaveItemStackV6 {
    item: String,
    count: u32,
    durability: Option<u32>,
}

impl From<LegacySaveItemStack> for SaveItemStack {
    fn from(legacy: LegacySaveItemStack) -> Self {
        Self {
            runtime_id: None,
            item: legacy.item,
            count: legacy.count,
            durability: None,
        }
    }
}

impl From<LegacySaveItemStackV6> for SaveItemStack {
    fn from(legacy: LegacySaveItemStackV6) -> Self {
        Self {
            runtime_id: None,
            item: legacy.item,
            count: legacy.count,
            durability: legacy.durability,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LegacyPlayerSaveDataV6 {
    version: u32,
    game_version: String,
    position: [f32; 3],
    rotation: [f32; 4],
    camera_pitch: f32,
    gamemode: String,
    health: f32,
    hunger: f32,
    saturation: f32,
    respawn_point: [f32; 3],
    hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    hotbar: [LegacySaveItemStackV6; 9],
    #[serde(with = "serde_arrays")]
    backpack: [LegacySaveItemStackV6; 27],
    #[serde(with = "serde_arrays")]
    equipment: [LegacySaveItemStackV6; SurvivalInventory::EQUIPMENT_SIZE],
    accessories: Vec<LegacySaveItemStackV6>,
}

#[derive(Serialize, Deserialize)]
struct LegacyPlayerSaveDataV5 {
    version: u32,
    game_version: String,
    position: [f32; 3],
    rotation: [f32; 4],
    camera_pitch: f32,
    gamemode: String,
    health: f32,
    hunger: f32,
    hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    hotbar: [LegacySaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    backpack: [LegacySaveItemStack; 27],
    #[serde(with = "serde_arrays")]
    equipment: [LegacySaveItemStack; SurvivalInventory::EQUIPMENT_SIZE],
    accessories: Vec<LegacySaveItemStack>,
}

#[derive(Serialize, Deserialize)]
struct LegacyPlayerSaveDataV4 {
    version: u32,
    position: [f32; 3],
    rotation: [f32; 4],
    camera_pitch: f32,
    gamemode: String,
    health: f32,
    hunger: f32,
    hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    hotbar: [LegacySaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    backpack: [LegacySaveItemStack; 27],
    #[serde(with = "serde_arrays")]
    equipment: [LegacySaveItemStack; SurvivalInventory::EQUIPMENT_SIZE],
    accessories: Vec<LegacySaveItemStack>,
}

#[derive(Serialize, Deserialize)]
struct LegacyPlayerSaveDataV3 {
    version: u32,
    position: [f32; 3],
    rotation: [f32; 4],
    #[serde(default)]
    camera_pitch: f32,
    gamemode: String,
    #[serde(default)]
    health: f32,
    #[serde(default)]
    hunger: f32,
    hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    hotbar: [LegacySaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    backpack: [LegacySaveItemStack; 36],
    #[serde(with = "serde_arrays")]
    armor: [LegacySaveItemStack; 4],
    #[serde(with = "serde_arrays")]
    accessories: [LegacySaveItemStack; 6],
}

impl From<LegacyPlayerSaveDataV3> for PlayerSaveData {
    fn from(legacy: LegacyPlayerSaveDataV3) -> Self {
        let backpack = std::array::from_fn(|i| legacy.backpack[i].clone().into());
        let equipment = std::array::from_fn(|i| {
            legacy
                .armor
                .get(i)
                .cloned()
                .map(Into::into)
                .unwrap_or_else(SaveItemStack::air)
        });
        let legacy_backpack_overflow = legacy.backpack[27..]
            .iter()
            .cloned()
            .map(Into::into)
            .collect();

        Self {
            version: SAVE_VERSION,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            saturation: 5.0,
            respawn_point: [0.0, 70.0, 0.0],
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar.map(Into::into),
            backpack,
            equipment,
            accessories: legacy.accessories.map(Into::into).to_vec(),
            item_id_map: Vec::new(),
            legacy_backpack_overflow,
        }
    }
}

impl From<LegacyPlayerSaveDataV4> for PlayerSaveData {
    fn from(legacy: LegacyPlayerSaveDataV4) -> Self {
        Self {
            version: SAVE_VERSION,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            saturation: 5.0,
            respawn_point: [0.0, 70.0, 0.0],
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar.map(Into::into),
            backpack: legacy.backpack.map(Into::into),
            equipment: legacy.equipment.map(Into::into),
            accessories: legacy.accessories.into_iter().map(Into::into).collect(),
            item_id_map: Vec::new(),
            legacy_backpack_overflow: Vec::new(),
        }
    }
}

impl From<LegacyPlayerSaveDataV5> for PlayerSaveData {
    fn from(legacy: LegacyPlayerSaveDataV5) -> Self {
        Self {
            version: SAVE_VERSION,
            game_version: legacy.game_version,
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            saturation: 5.0,
            respawn_point: [0.0, 70.0, 0.0],
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar.map(Into::into),
            backpack: legacy.backpack.map(Into::into),
            equipment: legacy.equipment.map(Into::into),
            accessories: legacy.accessories.into_iter().map(Into::into).collect(),
            item_id_map: Vec::new(),
            legacy_backpack_overflow: Vec::new(),
        }
    }
}

impl From<LegacyPlayerSaveDataV6> for PlayerSaveData {
    fn from(legacy: LegacyPlayerSaveDataV6) -> Self {
        Self {
            version: SAVE_VERSION,
            game_version: legacy.game_version,
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            saturation: legacy.saturation,
            respawn_point: legacy.respawn_point,
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar.map(Into::into),
            backpack: legacy.backpack.map(Into::into),
            equipment: legacy.equipment.map(Into::into),
            accessories: legacy.accessories.into_iter().map(Into::into).collect(),
            item_id_map: Vec::new(),
            legacy_backpack_overflow: Vec::new(),
        }
    }
}

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

pub fn read_player_backup(path: &std::path::Path) -> Result<PlayerSaveData, String> {
    let bytes = persistence::read_backup_verified(path, validate_player_bytes)
        .map_err(|error| error.to_string())?;
    decode_player_data(&bytes)
}

pub fn player_backup_available(path: &std::path::Path) -> bool {
    persistence::has_valid_backup(path, validate_player_bytes)
}

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
            return migrate_player_data(data);
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

fn migrate_player_data(mut data: PlayerSaveData) -> Result<PlayerSaveData, String> {
    match data.version {
        0..=6 => {
            data.version = SAVE_VERSION;
            data.game_version = env!("CARGO_PKG_VERSION").to_string();
        }
        SAVE_VERSION => {}
        found => {
            return Err(format!(
                "玩家存档版本 {found} 高于当前支持版本 {SAVE_VERSION}"
            ));
        }
    }
    Ok(data)
}

fn validate_player_bytes(bytes: &[u8]) -> Result<(), String> {
    decode_player_data(bytes).map(|_| ())
}

/// 获取玩家存档文件路径
pub fn player_save_path(world_name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("saves")
        .join(world_name)
        .join("players")
        .join("singleplayer.dat")
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/world/save/player/player_io.rs"]
mod tests;

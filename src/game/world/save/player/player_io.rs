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
    hotbar: [SaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    backpack: [SaveItemStack; 27],
    #[serde(with = "serde_arrays")]
    equipment: [SaveItemStack; SurvivalInventory::EQUIPMENT_SIZE],
    accessories: Vec<SaveItemStack>,
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
    hotbar: [SaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    backpack: [SaveItemStack; 36],
    #[serde(with = "serde_arrays")]
    armor: [SaveItemStack; 4],
    #[serde(with = "serde_arrays")]
    accessories: [SaveItemStack; 6],
}

impl From<LegacyPlayerSaveDataV3> for PlayerSaveData {
    fn from(legacy: LegacyPlayerSaveDataV3) -> Self {
        let backpack = std::array::from_fn(|i| legacy.backpack[i].clone());
        let equipment = std::array::from_fn(|i| {
            legacy
                .armor
                .get(i)
                .cloned()
                .unwrap_or_else(SaveItemStack::air)
        });
        let legacy_backpack_overflow = legacy.backpack[27..].to_vec();

        Self {
            version: SAVE_VERSION,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar,
            backpack,
            equipment,
            accessories: legacy.accessories.to_vec(),
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
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar,
            backpack: legacy.backpack,
            equipment: legacy.equipment,
            accessories: legacy.accessories,
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
        let data = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .reject_trailing_bytes()
            .deserialize::<PlayerSaveData>(&decompressed)
            .map_err(|error| format!("bincode deserialize v5: {error}"))?;
        return migrate_player_data(data);
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
        0..=4 => {
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
mod tests {
    use super::*;
    use crate::game::inventory::container::InventoryContainer;

    #[test]
    fn v3_layout_migrates_armor_accessories_and_overflow() {
        let mut legacy = LegacyPlayerSaveDataV3 {
            version: 3,
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            camera_pitch: 0.25,
            gamemode: "survival".into(),
            health: 18.0,
            hunger: 12.0,
            hotbar_active: 2,
            hotbar: std::array::from_fn(|_| SaveItemStack::air()),
            backpack: std::array::from_fn(|_| SaveItemStack::air()),
            armor: std::array::from_fn(|_| SaveItemStack::air()),
            accessories: std::array::from_fn(|_| SaveItemStack::air()),
        };
        legacy.armor[0] = SaveItemStack {
            item: "century_journey:test_helmet".into(),
            count: 1,
        };
        legacy.accessories[0] = SaveItemStack {
            item: "century_journey:test_ring".into(),
            count: 1,
        };
        legacy.backpack[27] = SaveItemStack {
            item: "century_journey:legacy_overflow".into(),
            count: 3,
        };

        let migrated: PlayerSaveData = legacy.into();
        let inventory = migrated.restore_inventory();

        assert_eq!(
            inventory
                .survival
                .get_stack(
                    crate::game::inventory::container::survival::SurvivalInventory::equipment_index(
                        0
                    )
                )
                .map(|stack| stack.count),
            Some(1)
        );
        assert_eq!(
            inventory
                .survival
                .get_stack(
                    crate::game::inventory::container::survival::SurvivalInventory::accessory_index(
                        0
                    )
                )
                .map(|stack| stack.count),
            Some(1)
        );
        assert_eq!(
            inventory.survival.get_stack(0).map(|stack| stack.count),
            Some(3)
        );
    }

    #[test]
    fn current_player_file_round_trip_keeps_game_version_and_stats() {
        let mut data = PlayerSaveData {
            health: 11.5,
            hunger: 6.25,
            ..PlayerSaveData::default()
        };
        data.hotbar[0] = SaveItemStack {
            item: "century_journey:test_item".into(),
            count: 9,
        };

        let decoded = decode_player_data(&encode_player_data(&data).unwrap()).unwrap();
        assert_eq!(decoded.version, SAVE_VERSION);
        assert_eq!(decoded.game_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(decoded.health, 11.5);
        assert_eq!(decoded.hunger, 6.25);
        assert_eq!(decoded.hotbar[0].count, 9);
    }

    #[test]
    fn v4_layout_has_an_explicit_migration() {
        let mut legacy = LegacyPlayerSaveDataV4 {
            version: 4,
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            camera_pitch: 0.1,
            gamemode: "survival".into(),
            health: 9.0,
            hunger: 8.0,
            hotbar_active: 0,
            hotbar: std::array::from_fn(|_| SaveItemStack::air()),
            backpack: std::array::from_fn(|_| SaveItemStack::air()),
            equipment: std::array::from_fn(|_| SaveItemStack::air()),
            accessories: vec![SaveItemStack::air(); 6],
        };
        legacy.equipment[6] = SaveItemStack {
            item: "century_journey:test_backpack".into(),
            count: 1,
        };
        let serialized = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .serialize(&legacy)
            .unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&serialized).unwrap();
        let decoded = decode_player_data(&encoder.finish().unwrap()).unwrap();

        assert_eq!(decoded.version, SAVE_VERSION);
        assert_eq!(decoded.health, 9.0);
        assert_eq!(decoded.hunger, 8.0);
        assert_eq!(decoded.equipment[6].count, 1);
    }
}

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

#[cfg(test)]
impl LegacySaveItemStack {
    fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
        }
    }
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

#[cfg(test)]
impl LegacySaveItemStackV6 {
    fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
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
            hotbar: std::array::from_fn(|_| LegacySaveItemStack::air()),
            backpack: std::array::from_fn(|_| LegacySaveItemStack::air()),
            armor: std::array::from_fn(|_| LegacySaveItemStack::air()),
            accessories: std::array::from_fn(|_| LegacySaveItemStack::air()),
        };
        legacy.armor[0] = LegacySaveItemStack {
            item: "century_journey:test_helmet".into(),
            count: 1,
        };
        legacy.accessories[0] = LegacySaveItemStack {
            item: "century_journey:test_ring".into(),
            count: 1,
        };
        legacy.backpack[27] = LegacySaveItemStack {
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
            runtime_id: None,
            item: "century_journey:test_item".into(),
            count: 9,
            durability: Some(17),
        };

        let decoded = decode_player_data(&encode_player_data(&data).unwrap()).unwrap();
        assert_eq!(decoded.version, SAVE_VERSION);
        assert_eq!(decoded.game_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(decoded.health, 11.5);
        assert_eq!(decoded.hunger, 6.25);
        assert_eq!(decoded.hotbar[0].count, 9);
        assert_eq!(decoded.hotbar[0].durability, Some(17));
    }

    #[test]
    fn v6_player_file_migrates_to_v7_identifier_mapping() {
        let mut legacy = LegacyPlayerSaveDataV6 {
            version: 6,
            game_version: "0.2.0".into(),
            position: [2.0, 71.0, 4.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            camera_pitch: 0.1,
            gamemode: "survival".into(),
            health: 17.0,
            hunger: 12.0,
            saturation: 4.0,
            respawn_point: [0.0, 70.0, 0.0],
            hotbar_active: 0,
            hotbar: std::array::from_fn(|_| LegacySaveItemStackV6::air()),
            backpack: std::array::from_fn(|_| LegacySaveItemStackV6::air()),
            equipment: std::array::from_fn(|_| LegacySaveItemStackV6::air()),
            accessories: vec![LegacySaveItemStackV6::air(); 6],
        };
        legacy.hotbar[0] = LegacySaveItemStackV6 {
            item: "century_journey:wooden_axe".into(),
            count: 1,
            durability: Some(31),
        };
        let serialized = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .serialize(&legacy)
            .unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&serialized).unwrap();
        let mut encoded = PLAYER_MAGIC.to_vec();
        encoded.extend(encoder.finish().unwrap());

        let decoded = decode_player_data(&encoded).unwrap();

        assert_eq!(decoded.version, SAVE_VERSION);
        assert!(decoded.item_id_map.is_empty());
        assert_eq!(decoded.hotbar[0].runtime_id, None);
        assert_eq!(decoded.hotbar[0].durability, Some(31));
    }

    #[test]
    fn stage_seven_v5_player_file_migrates_to_v6_defaults() {
        let mut legacy = LegacyPlayerSaveDataV5 {
            version: 5,
            game_version: "0.2.0".into(),
            position: [4.0, 70.0, -3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            camera_pitch: 0.2,
            gamemode: "survival".into(),
            health: 15.0,
            hunger: 11.0,
            hotbar_active: 0,
            hotbar: std::array::from_fn(|_| LegacySaveItemStack::air()),
            backpack: std::array::from_fn(|_| LegacySaveItemStack::air()),
            equipment: std::array::from_fn(|_| LegacySaveItemStack::air()),
            accessories: vec![LegacySaveItemStack::air(); 6],
        };
        legacy.hotbar[0] = LegacySaveItemStack {
            item: "century_journey:wooden_axe".into(),
            count: 1,
        };
        let serialized = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .serialize(&legacy)
            .unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&serialized).unwrap();
        let mut encoded = PLAYER_MAGIC.to_vec();
        encoded.extend(encoder.finish().unwrap());

        let decoded = decode_player_data(&encoded).unwrap();

        assert_eq!(decoded.version, SAVE_VERSION);
        assert_eq!(decoded.saturation, 5.0);
        assert_eq!(decoded.respawn_point, [0.0, 70.0, 0.0]);
        assert_eq!(decoded.hotbar[0].durability, None);
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
            hotbar: std::array::from_fn(|_| LegacySaveItemStack::air()),
            backpack: std::array::from_fn(|_| LegacySaveItemStack::air()),
            equipment: std::array::from_fn(|_| LegacySaveItemStack::air()),
            accessories: vec![LegacySaveItemStack::air(); 6],
        };
        legacy.equipment[6] = LegacySaveItemStack {
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

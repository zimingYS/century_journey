use super::player_model::{PlayerSaveData, SAVE_VERSION, SaveItemStack};
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

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

/// 序列化并压缩写入玩家数据
pub fn write_player_data(data: &PlayerSaveData, path: &std::path::Path) -> Result<(), String> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(data)
        .map_err(|e| format!("bincode serialize: {e}"))?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(&serialized)
        .map_err(|e| format!("gzip write: {e}"))?;
    let compressed = encoder.finish().map_err(|e| format!("gzip finish: {e}"))?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, compressed).map_err(|e| format!("file write: {e}"))?;
    Ok(())
}

/// 读取并解压反序列化玩家数据
pub fn read_player_data(path: &std::path::Path) -> Result<PlayerSaveData, String> {
    let compressed = std::fs::read(path).map_err(|e| format!("file read: {e}"))?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("gzip decompress: {e}"))?;
    if let Ok(data) = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .deserialize::<PlayerSaveData>(&decompressed)
        && data.version >= SAVE_VERSION
    {
        return Ok(data);
    }

    let legacy = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .deserialize::<LegacyPlayerSaveDataV3>(&decompressed)
        .map_err(|e| format!("bincode deserialize (v4/v3): {e}"))?;
    Ok(legacy.into())
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
}

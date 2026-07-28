use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::save::player::{PlayerSaveData, SAVE_VERSION, SaveItemStack};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LegacySaveItemStack {
    pub(super) item: String,
    pub(super) count: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LegacySaveItemStackV6 {
    pub(super) item: String,
    pub(super) count: u32,
    pub(super) durability: Option<u32>,
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
pub struct LegacyPlayerSaveDataV6 {
    pub(super) version: u32,
    pub(super) game_version: String,
    pub(super) position: [f32; 3],
    pub(super) rotation: [f32; 4],
    pub(super) camera_pitch: f32,
    pub(super) gamemode: String,
    pub(super) health: f32,
    pub(super) hunger: f32,
    pub(super) saturation: f32,
    pub(super) respawn_point: [f32; 3],
    pub(super) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(super) hotbar: [LegacySaveItemStackV6; 9],
    #[serde(with = "serde_arrays")]
    pub(super) backpack: [LegacySaveItemStackV6; 27],
    #[serde(with = "serde_arrays")]
    pub(super) equipment: [LegacySaveItemStackV6; SurvivalInventory::EQUIPMENT_SIZE],
    pub(super) accessories: Vec<LegacySaveItemStackV6>,
}

#[derive(Serialize, Deserialize)]
pub struct LegacyPlayerSaveDataV5 {
    pub(super) version: u32,
    pub(super) game_version: String,
    pub(super) position: [f32; 3],
    pub(super) rotation: [f32; 4],
    pub(super) camera_pitch: f32,
    pub(super) gamemode: String,
    pub(super) health: f32,
    pub(super) hunger: f32,
    pub(super) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(super) hotbar: [LegacySaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    pub(super) backpack: [LegacySaveItemStack; 27],
    #[serde(with = "serde_arrays")]
    pub(super) equipment: [LegacySaveItemStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub(super) accessories: Vec<LegacySaveItemStack>,
}

#[derive(Serialize, Deserialize)]
pub struct LegacyPlayerSaveDataV4 {
    pub(super) version: u32,
    pub(super) position: [f32; 3],
    pub(super) rotation: [f32; 4],
    pub(super) camera_pitch: f32,
    pub(super) gamemode: String,
    pub(super) health: f32,
    pub(super) hunger: f32,
    pub(super) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(super) hotbar: [LegacySaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    pub(super) backpack: [LegacySaveItemStack; 27],
    #[serde(with = "serde_arrays")]
    pub(super) equipment: [LegacySaveItemStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub(super) accessories: Vec<LegacySaveItemStack>,
}

#[derive(Serialize, Deserialize)]
pub struct LegacyPlayerSaveDataV3 {
    pub(super) version: u32,
    pub(super) position: [f32; 3],
    pub(super) rotation: [f32; 4],
    #[serde(default)]
    pub(super) camera_pitch: f32,
    pub(super) gamemode: String,
    #[serde(default)]
    pub(super) health: f32,
    #[serde(default)]
    pub(super) hunger: f32,
    pub(super) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(super) hotbar: [LegacySaveItemStack; 9],
    #[serde(with = "serde_arrays")]
    pub(super) backpack: [LegacySaveItemStack; 36],
    #[serde(with = "serde_arrays")]
    pub(super) armor: [LegacySaveItemStack; 4],
    #[serde(with = "serde_arrays")]
    pub(super) accessories: [LegacySaveItemStack; 6],
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

pub(in crate::game) fn migrate_player_data(
    mut data: PlayerSaveData,
) -> Result<PlayerSaveData, String> {
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

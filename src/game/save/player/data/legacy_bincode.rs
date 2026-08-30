//! 只读解析历史玩家 bincode 位置布局。
//!
//! bincode 不保存字段名，因此既有布局必须按当时的字段顺序冻结。这里的类型按数据
//! 能力命名且永不继续添加字段；读取后立即转换为当前 `PlayerSaveData`，后续写回
//! 自然升级为命名 MessagePack 文档。

use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::save::player::{PlayerSaveData, SaveItemStack};
use bincode::Options;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::io::Read;

const LEGACY_MAX_DECOMPRESSED_BYTES: usize = 4 * 1024 * 1024;

/// 早期布局中只保存稳定标识和数量的物品记录。
#[derive(Serialize, Deserialize, Clone)]
pub(in crate::game::save::player) struct IdentifierCountStack {
    pub(in crate::game::save::player) item: String,
    pub(in crate::game::save::player) count: u32,
}

/// 加入耐久度、但尚未保存运行时 ID 的物品记录。
#[derive(Serialize, Deserialize, Clone)]
pub(in crate::game::save::player) struct DurabilityStack {
    pub(in crate::game::save::player) item: String,
    pub(in crate::game::save::player) count: u32,
    pub(in crate::game::save::player) durability: Option<u32>,
}

/// 加入运行时 ID 映射后的历史物品记录。
#[derive(Serialize, Deserialize, Clone)]
pub(in crate::game::save::player) struct RuntimeMappedStack {
    pub(in crate::game::save::player) runtime_id: Option<u32>,
    pub(in crate::game::save::player) item: String,
    pub(in crate::game::save::player) count: u32,
    pub(in crate::game::save::player) durability: Option<u32>,
}

impl From<IdentifierCountStack> for SaveItemStack {
    fn from(legacy: IdentifierCountStack) -> Self {
        Self {
            runtime_id: None,
            item: legacy.item,
            count: legacy.count,
            durability: None,
        }
    }
}

impl From<DurabilityStack> for SaveItemStack {
    fn from(legacy: DurabilityStack) -> Self {
        Self {
            runtime_id: None,
            item: legacy.item,
            count: legacy.count,
            durability: legacy.durability,
        }
    }
}

impl From<RuntimeMappedStack> for SaveItemStack {
    fn from(legacy: RuntimeMappedStack) -> Self {
        Self {
            runtime_id: legacy.runtime_id,
            item: legacy.item,
            count: legacy.count,
            durability: legacy.durability,
        }
    }
}

/// 使用 36 格背包、独立护甲数组和固定饰品数组的历史位置布局。
#[derive(Serialize, Deserialize)]
pub(in crate::game::save::player) struct ExpandedInventoryLayout {
    pub(in crate::game::save::player) layout_marker: u32,
    pub(in crate::game::save::player) position: [f32; 3],
    pub(in crate::game::save::player) rotation: [f32; 4],
    pub(in crate::game::save::player) camera_pitch: f32,
    pub(in crate::game::save::player) gamemode: String,
    pub(in crate::game::save::player) health: f32,
    pub(in crate::game::save::player) hunger: f32,
    pub(in crate::game::save::player) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) hotbar: [IdentifierCountStack; 9],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) backpack: [IdentifierCountStack; 36],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) armor: [IdentifierCountStack; 4],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) accessories: [IdentifierCountStack; 6],
}

/// 把背包、装备与饰品调整到当前容量形态后的历史位置布局。
#[derive(Serialize, Deserialize)]
pub(in crate::game::save::player) struct EquipmentInventoryLayout {
    pub(in crate::game::save::player) layout_marker: u32,
    pub(in crate::game::save::player) position: [f32; 3],
    pub(in crate::game::save::player) rotation: [f32; 4],
    pub(in crate::game::save::player) camera_pitch: f32,
    pub(in crate::game::save::player) gamemode: String,
    pub(in crate::game::save::player) health: f32,
    pub(in crate::game::save::player) hunger: f32,
    pub(in crate::game::save::player) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) hotbar: [IdentifierCountStack; 9],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) backpack: [IdentifierCountStack; 27],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) equipment:
        [IdentifierCountStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub(in crate::game::save::player) accessories: Vec<IdentifierCountStack>,
}

/// 加入游戏构建标识后的历史位置布局。
#[derive(Serialize, Deserialize)]
pub(in crate::game::save::player) struct GameBuildLayout {
    pub(in crate::game::save::player) layout_marker: u32,
    pub(in crate::game::save::player) game_version: String,
    pub(in crate::game::save::player) position: [f32; 3],
    pub(in crate::game::save::player) rotation: [f32; 4],
    pub(in crate::game::save::player) camera_pitch: f32,
    pub(in crate::game::save::player) gamemode: String,
    pub(in crate::game::save::player) health: f32,
    pub(in crate::game::save::player) hunger: f32,
    pub(in crate::game::save::player) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) hotbar: [IdentifierCountStack; 9],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) backpack: [IdentifierCountStack; 27],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) equipment:
        [IdentifierCountStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub(in crate::game::save::player) accessories: Vec<IdentifierCountStack>,
}

/// 加入耐久度、生存饱和度和重生点后的历史位置布局。
#[derive(Serialize, Deserialize)]
pub(in crate::game::save::player) struct DurabilityLayout {
    pub(in crate::game::save::player) layout_marker: u32,
    pub(in crate::game::save::player) game_version: String,
    pub(in crate::game::save::player) position: [f32; 3],
    pub(in crate::game::save::player) rotation: [f32; 4],
    pub(in crate::game::save::player) camera_pitch: f32,
    pub(in crate::game::save::player) gamemode: String,
    pub(in crate::game::save::player) health: f32,
    pub(in crate::game::save::player) hunger: f32,
    pub(in crate::game::save::player) saturation: f32,
    pub(in crate::game::save::player) respawn_point: [f32; 3],
    pub(in crate::game::save::player) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) hotbar: [DurabilityStack; 9],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) backpack: [DurabilityStack; 27],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) equipment:
        [DurabilityStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub(in crate::game::save::player) accessories: Vec<DurabilityStack>,
}

/// 加入物品运行时 ID 映射后的最后一种历史 bincode 位置布局。
#[derive(Serialize, Deserialize)]
pub(in crate::game::save::player) struct RuntimeIdMapLayout {
    pub(in crate::game::save::player) layout_marker: u32,
    pub(in crate::game::save::player) game_version: String,
    pub(in crate::game::save::player) position: [f32; 3],
    pub(in crate::game::save::player) rotation: [f32; 4],
    pub(in crate::game::save::player) camera_pitch: f32,
    pub(in crate::game::save::player) gamemode: String,
    pub(in crate::game::save::player) health: f32,
    pub(in crate::game::save::player) hunger: f32,
    pub(in crate::game::save::player) saturation: f32,
    pub(in crate::game::save::player) respawn_point: [f32; 3],
    pub(in crate::game::save::player) hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) hotbar: [RuntimeMappedStack; 9],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) backpack: [RuntimeMappedStack; 27],
    #[serde(with = "serde_arrays")]
    pub(in crate::game::save::player) equipment:
        [RuntimeMappedStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub(in crate::game::save::player) accessories: Vec<RuntimeMappedStack>,
    pub(in crate::game::save::player) item_id_map: Vec<(u32, String)>,
}

/// 解压并读取带旧 magic 或无 magic 的冻结位置布局。
pub(in crate::game::save::player) fn decode(
    compressed: &[u8],
    had_legacy_magic: bool,
) -> Result<PlayerSaveData, String> {
    let decompressed = decompress_limited(compressed)?;
    if had_legacy_magic {
        decode_magic_layout(&decompressed)
    } else {
        decode_bare_layout(&decompressed)
    }
}

fn decode_magic_layout(bytes: &[u8]) -> Result<PlayerSaveData, String> {
    if let Ok(layout) = deserialize::<RuntimeIdMapLayout>(bytes)
        && layout.layout_marker == 7
    {
        return Ok(layout.into());
    }
    if let Ok(layout) = deserialize::<DurabilityLayout>(bytes)
        && layout.layout_marker == 6
    {
        return Ok(layout.into());
    }
    let layout = deserialize::<GameBuildLayout>(bytes)
        .map_err(|error| format!("无法识别带 magic 的历史玩家布局: {error}"))?;
    if layout.layout_marker != 5 {
        return Err(format!("不支持的历史玩家布局标记 {}", layout.layout_marker));
    }
    Ok(layout.into())
}

fn decode_bare_layout(bytes: &[u8]) -> Result<PlayerSaveData, String> {
    if let Ok(layout) = deserialize::<EquipmentInventoryLayout>(bytes)
        && layout.layout_marker == 4
    {
        return Ok(layout.into());
    }
    let layout = deserialize::<ExpandedInventoryLayout>(bytes)
        .map_err(|error| format!("无法识别无 magic 的历史玩家布局: {error}"))?;
    if layout.layout_marker != 3 {
        return Err(format!("不支持的历史玩家布局标记 {}", layout.layout_marker));
    }
    Ok(layout.into())
}

fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> bincode::Result<T> {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize(bytes)
}

fn decompress_limited(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder =
        GzDecoder::new(bytes).take((LEGACY_MAX_DECOMPRESSED_BYTES.saturating_add(1)) as u64);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|error| format!("历史玩家存档 gzip 解压失败: {error}"))?;
    if decompressed.len() > LEGACY_MAX_DECOMPRESSED_BYTES {
        return Err(format!(
            "历史玩家存档解压后超过 {LEGACY_MAX_DECOMPRESSED_BYTES} 字节上限"
        ));
    }
    Ok(decompressed)
}

impl From<ExpandedInventoryLayout> for PlayerSaveData {
    fn from(legacy: ExpandedInventoryLayout) -> Self {
        let backpack = std::array::from_fn(|index| legacy.backpack[index].clone().into());
        let equipment = std::array::from_fn(|index| {
            legacy
                .armor
                .get(index)
                .cloned()
                .map(Into::into)
                .unwrap_or_else(SaveItemStack::air)
        });
        // 背包容量回到 36 后整组可直接容纳，仅在容量缩小时才产生溢出。
        let overflow_start = SurvivalInventory::BACKPACK_SIZE.min(legacy.backpack.len());
        let legacy_backpack_overflow = legacy.backpack[overflow_start..]
            .iter()
            .cloned()
            .map(Into::into)
            .collect();
        PlayerSaveData {
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar.map(Into::into),
            backpack,
            equipment,
            accessories: legacy.accessories.map(Into::into).to_vec(),
            legacy_backpack_overflow,
            ..Default::default()
        }
    }
}

/// 把历史 27 格背包扩容为当前 36 格，新增槽位以空气填充。
fn expand_backpack<T: Into<SaveItemStack>, const N: usize>(
    legacy: [T; N],
) -> [SaveItemStack; SurvivalInventory::BACKPACK_SIZE] {
    let mut stacks = legacy.into_iter().map(Into::into);
    std::array::from_fn(|_| stacks.next().unwrap_or_else(SaveItemStack::air))
}

impl From<EquipmentInventoryLayout> for PlayerSaveData {
    fn from(legacy: EquipmentInventoryLayout) -> Self {
        PlayerSaveData {
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar.map(Into::into),
            backpack: expand_backpack(legacy.backpack),
            equipment: legacy.equipment.map(Into::into),
            accessories: legacy.accessories.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<GameBuildLayout> for PlayerSaveData {
    fn from(legacy: GameBuildLayout) -> Self {
        PlayerSaveData {
            game_version: legacy.game_version,
            position: legacy.position,
            rotation: legacy.rotation,
            camera_pitch: legacy.camera_pitch,
            gamemode: legacy.gamemode,
            health: legacy.health,
            hunger: legacy.hunger,
            hotbar_active: legacy.hotbar_active,
            hotbar: legacy.hotbar.map(Into::into),
            backpack: expand_backpack(legacy.backpack),
            equipment: legacy.equipment.map(Into::into),
            accessories: legacy.accessories.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<DurabilityLayout> for PlayerSaveData {
    fn from(legacy: DurabilityLayout) -> Self {
        PlayerSaveData {
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
            backpack: expand_backpack(legacy.backpack),
            equipment: legacy.equipment.map(Into::into),
            accessories: legacy.accessories.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<RuntimeIdMapLayout> for PlayerSaveData {
    fn from(legacy: RuntimeIdMapLayout) -> Self {
        PlayerSaveData {
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
            backpack: expand_backpack(legacy.backpack),
            equipment: legacy.equipment.map(Into::into),
            accessories: legacy.accessories.into_iter().map(Into::into).collect(),
            item_id_map: legacy.item_id_map,
            ..Default::default()
        }
    }
}

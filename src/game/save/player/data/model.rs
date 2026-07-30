//! 定义当前版本玩家存档，并负责权威运行时状态的双向转换。

use crate::content::item::ItemRegistry;
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::save::player::data::item_codec;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 当前玩家存档格式版本。
pub const SAVE_VERSION: u32 = 7;

/// 可序列化物品堆叠
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SaveItemStack {
    pub runtime_id: Option<u32>,
    pub item: String,
    pub count: u32,
    #[serde(default)]
    pub durability: Option<u32>,
}

impl SaveItemStack {
    /// 创建表示空槽位的稳定存档值。
    pub(crate) fn air() -> Self {
        Self {
            runtime_id: None,
            item: "century_journey:air".into(),
            count: 0,
            durability: None,
        }
    }

    /// 判断该记录是否应恢复为空槽位。
    pub fn is_air(&self) -> bool {
        self.item == "century_journey:air" || self.count == 0
    }
}

/// 可序列化玩家存档数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSaveData {
    pub version: u32,
    pub game_version: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    #[serde(default)]
    pub camera_pitch: f32,
    pub gamemode: String,
    #[serde(default)]
    pub health: f32,
    #[serde(default)]
    pub hunger: f32,
    #[serde(default = "default_saturation")]
    pub saturation: f32,
    #[serde(default = "default_respawn_point")]
    pub respawn_point: [f32; 3],
    pub hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub hotbar: [SaveItemStack; HOTBAR_SIZE],
    #[serde(with = "serde_arrays")]
    pub backpack: [SaveItemStack; SurvivalInventory::BACKPACK_SIZE],
    #[serde(with = "serde_arrays")]
    pub equipment: [SaveItemStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub accessories: Vec<SaveItemStack>,
    /// 保存时的动态 ID 到唯一标识符映射，用于跨内容版本重映射。
    pub item_id_map: Vec<(u32, String)>,
    #[serde(skip)]
    pub(crate) legacy_backpack_overflow: Vec<SaveItemStack>,
}

impl Default for PlayerSaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            position: [0.0, 70.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            camera_pitch: 0.0,
            gamemode: "survival".into(),
            health: 20.0,
            hunger: 20.0,
            saturation: default_saturation(),
            respawn_point: default_respawn_point(),
            hotbar_active: 0,
            hotbar: std::array::from_fn(|_| SaveItemStack::air()),
            backpack: std::array::from_fn(|_| SaveItemStack::air()),
            equipment: std::array::from_fn(|_| SaveItemStack::air()),
            accessories: vec![SaveItemStack::air(); 6],
            item_id_map: Vec::new(),
            legacy_backpack_overflow: Vec::new(),
        }
    }
}

/// 为缺少饱和度字段的旧存档提供迁移默认值。
pub(crate) fn default_saturation() -> f32 {
    5.0
}

/// 为缺少重生点字段的旧存档提供迁移默认值。
pub(crate) fn default_respawn_point() -> [f32; 3] {
    [0.0, 70.0, 0.0]
}

// ─── 序列化辅助函数 ──────────────────────────────────

fn gamemode_to_string(mode: GameMode) -> String {
    match mode {
        GameMode::Survival => "survival".into(),
        GameMode::Creative => "creative".into(),
    }
}

fn string_to_gamemode(s: &str) -> GameMode {
    match s {
        "creative" => GameMode::Creative,
        _ => GameMode::Survival,
    }
}

// ─── PlayerSaveData 方法 ──────────────────────────────

impl PlayerSaveData {
    /// 从玩家权威运行时状态收集当前版本存档快照。
    /// 参数逐项对应存档字段，保持显式可避免遗漏版本化状态。
    #[allow(clippy::too_many_arguments)]
    pub fn from_runtime(
        position: Vec3,
        rotation: Quat,
        camera_pitch: f32,
        gamemode: &PlayerGameMode,
        inventory: &InventoryState,
        item_registry: &ItemRegistry,
        health: f32,
        hunger: f32,
        saturation: f32,
        respawn_point: Vec3,
    ) -> Self {
        let hotbar = std::array::from_fn(|i| {
            item_codec::optional_stack_to_save(inventory.hotbar.get_stack(i), item_registry)
        });
        let backpack = std::array::from_fn(|i| {
            item_codec::optional_stack_to_save(inventory.survival.get_stack(i), item_registry)
        });
        let equipment = std::array::from_fn(|i| {
            item_codec::optional_stack_to_save(
                inventory
                    .survival
                    .get_stack(SurvivalInventory::equipment_index(i)),
                item_registry,
            )
        });
        let accessories = (0..inventory.survival.accessories.len())
            .map(|i| {
                item_codec::optional_stack_to_save(
                    inventory
                        .survival
                        .get_stack(SurvivalInventory::accessory_index(i)),
                    item_registry,
                )
            })
            .collect();

        Self {
            version: SAVE_VERSION,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            position: [position.x, position.y, position.z],
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
            camera_pitch,
            gamemode: gamemode_to_string(gamemode.mode),
            health,
            hunger,
            saturation,
            respawn_point: respawn_point.to_array(),
            hotbar_active: inventory.hotbar.active_index,
            hotbar,
            backpack,
            equipment,
            accessories,
            item_id_map: item_registry.build_save_id_map(),
            legacy_backpack_overflow: Vec::new(),
        }
    }

    /// 将存档字符串恢复为受支持的游戏模式。
    pub fn restore_gamemode(&self) -> PlayerGameMode {
        PlayerGameMode {
            mode: string_to_gamemode(&self.gamemode),
        }
    }

    /// 不依赖当前内容注册表恢复背包，主要供旧格式迁移和测试使用。
    pub fn restore_inventory(&self) -> InventoryState {
        self.restore_inventory_resolving(item_codec::save_to_optional_stack)
    }

    /// 按稳定标识和动态编号映射恢复背包，并清除已删除内容。
    pub fn restore_inventory_with_registry(&self, item_registry: &ItemRegistry) -> InventoryState {
        let remap = item_registry.build_id_remap_table(&self.item_id_map);
        self.restore_inventory_resolving(|slot| {
            item_codec::save_to_optional_stack_with_registry(slot, item_registry, &remap)
        })
    }

    fn restore_inventory_resolving(
        &self,
        mut resolve: impl FnMut(&SaveItemStack) -> Option<ItemStack>,
    ) -> InventoryState {
        let mut state = InventoryState::default();
        for (i, slot) in self.hotbar.iter().enumerate() {
            if let Some(stack) = resolve(slot) {
                state.hotbar.set_stack(i, stack);
            }
        }
        state.hotbar.active_index = self.hotbar_active.min(HOTBAR_SIZE - 1);
        for (i, slot) in self.backpack.iter().enumerate() {
            if let Some(stack) = resolve(slot) {
                state.survival.set_stack(i, stack);
            }
        }
        for (i, slot) in self.equipment.iter().enumerate() {
            if let Some(stack) = resolve(slot) {
                state
                    .survival
                    .set_stack(SurvivalInventory::equipment_index(i), stack);
            }
        }
        state
            .survival
            .ensure_accessory_slots(self.accessories.len());
        for (i, slot) in self.accessories.iter().enumerate() {
            if let Some(stack) = resolve(slot) {
                state
                    .survival
                    .set_stack(SurvivalInventory::accessory_index(i), stack);
            }
        }
        for slot in &self.legacy_backpack_overflow {
            if let Some(stack) = resolve(slot) {
                restore_legacy_stack(&mut state, stack);
            }
        }
        state
    }

    /// 从存档位置和旋转恢复玩家权威变换。
    pub fn restore_transform(&self) -> Transform {
        let [x, y, z] = self.position;
        let [rx, ry, rz, rw] = self.rotation;
        Transform {
            translation: Vec3::new(x, y, z),
            rotation: Quat::from_xyzw(rx, ry, rz, rw),
            scale: Vec3::ONE,
        }
    }

    /// 返回独立保存的第一人称相机俯仰角。
    pub fn camera_pitch(&self) -> f32 {
        self.camera_pitch
    }

    /// 返回存档中的玩家重生点。
    pub fn respawn_point(&self) -> Vec3 {
        Vec3::from_array(self.respawn_point)
    }
}

fn restore_legacy_stack(state: &mut InventoryState, mut stack: ItemStack) {
    for index in 0..SurvivalInventory::BACKPACK_SIZE {
        if stack.is_empty() {
            return;
        }
        if let Some(existing) = state.survival.get_stack_mut(index)
            && existing.is_same_item(&stack)
        {
            existing.merge_from(&mut stack);
        }
    }
    for index in 0..SurvivalInventory::BACKPACK_SIZE {
        if state.survival.get_stack(index).is_none() {
            state.survival.set_stack(index, stack);
            return;
        }
    }
    for index in 0..HOTBAR_SIZE {
        if stack.is_empty() {
            return;
        }
        if let Some(existing) = state.hotbar.get_stack_mut(index)
            && existing.is_same_item(&stack)
        {
            existing.merge_from(&mut stack);
        }
    }
    for index in 0..HOTBAR_SIZE {
        if state.hotbar.get_stack(index).is_none() {
            state.hotbar.set_stack(index, stack);
            return;
        }
    }
    log::warn!("[存档系统] 旧版背包容量迁移后空间不足，无法恢复物品: {stack:?}");
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/player/data/model.rs"]
mod tests;

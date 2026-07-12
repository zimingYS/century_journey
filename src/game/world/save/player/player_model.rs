use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const SAVE_VERSION: u32 = 4;

/// 可序列化物品堆叠
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SaveItemStack {
    pub item: String,
    pub count: u32,
}

impl SaveItemStack {
    pub(crate) fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
        }
    }

    pub fn is_air(&self) -> bool {
        self.item == "century_journey:air" || self.count == 0
    }
}

/// 可序列化玩家存档数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSaveData {
    pub version: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    #[serde(default)]
    pub camera_pitch: f32,
    pub gamemode: String,
    #[serde(default)]
    pub health: f32,
    #[serde(default)]
    pub hunger: f32,
    pub hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub hotbar: [SaveItemStack; HOTBAR_SIZE],
    #[serde(with = "serde_arrays")]
    pub backpack: [SaveItemStack; SurvivalInventory::BACKPACK_SIZE],
    #[serde(with = "serde_arrays")]
    pub equipment: [SaveItemStack; SurvivalInventory::EQUIPMENT_SIZE],
    pub accessories: Vec<SaveItemStack>,
    #[serde(skip)]
    pub(crate) legacy_backpack_overflow: Vec<SaveItemStack>,
}

impl Default for PlayerSaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            position: [0.0, 70.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            camera_pitch: 0.0,
            gamemode: "survival".into(),
            health: 20.0,
            hunger: 20.0,
            hotbar_active: 0,
            hotbar: std::array::from_fn(|_| SaveItemStack::air()),
            backpack: std::array::from_fn(|_| SaveItemStack::air()),
            equipment: std::array::from_fn(|_| SaveItemStack::air()),
            accessories: vec![SaveItemStack::air(); 6],
            legacy_backpack_overflow: Vec::new(),
        }
    }
}

// ─── 序列化辅助函数 ──────────────────────────────────

fn item_id_to_string(id: &ItemId) -> String {
    id.to_string()
}

fn string_to_item_id(s: &str) -> ItemId {
    if let Some(rest) = s.strip_prefix("item:") {
        ItemId::item(rest)
    } else if let Some(rest) = s.strip_prefix("block:") {
        ItemId::block(rest)
    } else {
        ItemId::block(s)
    }
}

fn optional_stack_to_save(opt: Option<&ItemStack>) -> SaveItemStack {
    match opt {
        Some(s) if !s.is_empty() => SaveItemStack {
            item: item_id_to_string(&s.item),
            count: s.count,
        },
        _ => SaveItemStack::air(),
    }
}

fn save_to_optional_stack(slot: &SaveItemStack) -> Option<ItemStack> {
    if slot.is_air() {
        None
    } else {
        Some(ItemStack::new(string_to_item_id(&slot.item), slot.count))
    }
}

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
    pub fn from_runtime(
        position: Vec3,
        rotation: Quat,
        camera_pitch: f32,
        gamemode: &PlayerGameMode,
        inventory: &InventoryState,
        health: f32,
        hunger: f32,
    ) -> Self {
        let hotbar = std::array::from_fn(|i| optional_stack_to_save(inventory.hotbar.get_stack(i)));
        let backpack =
            std::array::from_fn(|i| optional_stack_to_save(inventory.survival.get_stack(i)));
        let equipment = std::array::from_fn(|i| {
            optional_stack_to_save(
                inventory
                    .survival
                    .get_stack(SurvivalInventory::equipment_index(i)),
            )
        });
        let accessories = (0..inventory.survival.accessories.len())
            .map(|i| {
                optional_stack_to_save(
                    inventory
                        .survival
                        .get_stack(SurvivalInventory::accessory_index(i)),
                )
            })
            .collect();

        Self {
            version: SAVE_VERSION,
            position: [position.x, position.y, position.z],
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
            camera_pitch,
            gamemode: gamemode_to_string(gamemode.mode),
            health,
            hunger,
            hotbar_active: inventory.hotbar.active_index,
            hotbar,
            backpack,
            equipment,
            accessories,
            legacy_backpack_overflow: Vec::new(),
        }
    }

    pub fn restore_gamemode(&self) -> PlayerGameMode {
        PlayerGameMode {
            mode: string_to_gamemode(&self.gamemode),
        }
    }

    pub fn restore_inventory(&self) -> InventoryState {
        let mut state = InventoryState::default();
        for (i, slot) in self.hotbar.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state.hotbar.set_stack(i, stack);
            }
        }
        state.hotbar.active_index = self.hotbar_active.min(HOTBAR_SIZE - 1);
        for (i, slot) in self.backpack.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state.survival.set_stack(i, stack);
            }
        }
        for (i, slot) in self.equipment.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state
                    .survival
                    .set_stack(SurvivalInventory::equipment_index(i), stack);
            }
        }
        state
            .survival
            .ensure_accessory_slots(self.accessories.len());
        for (i, slot) in self.accessories.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state
                    .survival
                    .set_stack(SurvivalInventory::accessory_index(i), stack);
            }
        }
        for slot in &self.legacy_backpack_overflow {
            if let Some(stack) = save_to_optional_stack(slot) {
                restore_legacy_stack(&mut state, stack);
            }
        }
        state
    }

    pub fn restore_transform(&self) -> Transform {
        let [x, y, z] = self.position;
        let [rx, ry, rz, rw] = self.rotation;
        Transform {
            translation: Vec3::new(x, y, z),
            rotation: Quat::from_xyzw(rx, ry, rz, rw),
            scale: Vec3::ONE,
        }
    }

    pub fn camera_pitch(&self) -> f32 {
        self.camera_pitch
    }
}

/// 存档数据健康检查与自动修复
pub fn validate_player_data(data: &PlayerSaveData) -> PlayerSaveData {
    let mut data = data.clone();
    let mut repaired = false;

    if data.position.iter().any(|v| v.is_nan() || v.is_infinite()) {
        log::warn!("[存档系统] 无效位置{:?}，已重置为世界原点", data.position);
        data.position = [0.0, 70.0, 0.0];
        repaired = true;
    }
    if data.rotation.iter().any(|v| v.is_nan() || v.is_infinite()) {
        log::warn!("[存档系统] 旋转无效 {:?}, 已重置为恒等矩阵", data.rotation);
        data.rotation = [0.0, 0.0, 0.0, 1.0];
        repaired = true;
    }
    if data.camera_pitch.is_nan() || data.camera_pitch.is_infinite() {
        log::warn!(
            "[存档系统] 相机俯仰角{}无效, 已重置为0.0",
            data.camera_pitch
        );
        data.camera_pitch = 0.0;
        repaired = true;
    }
    if !matches!(data.gamemode.as_str(), "survival" | "creative") {
        log::warn!(
            "[存档系统] 未知游戏模式: '{}', 已重置为生存模式",
            data.gamemode
        );
        data.gamemode = "survival".into();
        repaired = true;
    }
    if data.hotbar_active >= HOTBAR_SIZE {
        log::warn!(
            "[存档系统] 快捷栏索引 {} 超出索引范围,已重置为0",
            data.hotbar_active
        );
        data.hotbar_active = 0;
        repaired = true;
    }
    for (slot, kind) in data
        .hotbar
        .iter_mut()
        .map(|s| (s, "hotbar"))
        .chain(data.backpack.iter_mut().map(|s| (s, "backpack")))
        .chain(data.equipment.iter_mut().map(|s| (s, "equipment")))
        .chain(data.accessories.iter_mut().map(|s| (s, "accessories")))
    {
        if slot.is_air() {
            continue;
        }
        if slot.item.is_empty() || !slot.item.contains(':') {
            log::warn!(
                "[存档系统] '{}'中的物品{}无效,已替换为空气",
                slot.item,
                kind
            );
            *slot = SaveItemStack::air();
            repaired = true;
        }
    }

    if repaired {
        log::warn!("[存档系统] 保存数据出现问题 — 已自动修复");
    }
    data
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

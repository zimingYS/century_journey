use crate::content::item::registry::registry::ItemRegistry;
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::item::stack::{ItemInstanceData, ItemStack};
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub(crate) fn air() -> Self {
        Self {
            runtime_id: None,
            item: "century_journey:air".into(),
            count: 0,
            durability: None,
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

fn default_saturation() -> f32 {
    5.0
}

fn default_respawn_point() -> [f32; 3] {
    [0.0, 70.0, 0.0]
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

fn optional_stack_to_save(opt: Option<&ItemStack>, item_registry: &ItemRegistry) -> SaveItemStack {
    match opt {
        Some(s) if !s.is_empty() => SaveItemStack {
            runtime_id: item_registry.runtime_id(&s.item),
            item: item_id_to_string(&s.item),
            count: s.count,
            durability: s.instance.durability,
        },
        _ => SaveItemStack::air(),
    }
}

fn save_to_optional_stack(slot: &SaveItemStack) -> Option<ItemStack> {
    if slot.is_air() {
        None
    } else {
        Some(ItemStack::with_instance(
            string_to_item_id(&slot.item),
            slot.count,
            ItemInstanceData {
                durability: slot.durability,
            },
        ))
    }
}

fn save_to_optional_stack_with_registry(
    slot: &SaveItemStack,
    item_registry: &ItemRegistry,
    remap: &std::collections::HashMap<u32, u32>,
) -> Option<ItemStack> {
    if slot.is_air() {
        return None;
    }
    let item = string_to_item_id(&slot.item);
    if !item_registry.contains(&item) {
        log::warn!(
            "[存档系统] 物品 {} 在当前内容版本中不存在，已将槽位清空",
            slot.item
        );
        return None;
    }
    if let Some(saved_runtime_id) = slot.runtime_id {
        let current_runtime_id = item_registry.runtime_id(&item);
        match (remap.get(&saved_runtime_id), current_runtime_id) {
            (Some(mapped), Some(current)) if *mapped == current => {
                if saved_runtime_id != current {
                    log::info!(
                        "[存档系统] 物品 {} 动态 ID 已从 {} 重映射为 {}",
                        slot.item,
                        saved_runtime_id,
                        current
                    );
                }
            }
            _ => log::warn!(
                "[存档系统] 物品 {} 的旧动态 ID {} 无法可信重映射，改用唯一标识符恢复",
                slot.item,
                saved_runtime_id
            ),
        }
    }
    Some(ItemStack::with_instance(
        item,
        slot.count,
        ItemInstanceData {
            durability: slot.durability,
        },
    ))
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
        item_registry: &ItemRegistry,
        health: f32,
        hunger: f32,
        saturation: f32,
        respawn_point: Vec3,
    ) -> Self {
        let hotbar = std::array::from_fn(|i| {
            optional_stack_to_save(inventory.hotbar.get_stack(i), item_registry)
        });
        let backpack = std::array::from_fn(|i| {
            optional_stack_to_save(inventory.survival.get_stack(i), item_registry)
        });
        let equipment = std::array::from_fn(|i| {
            optional_stack_to_save(
                inventory
                    .survival
                    .get_stack(SurvivalInventory::equipment_index(i)),
                item_registry,
            )
        });
        let accessories = (0..inventory.survival.accessories.len())
            .map(|i| {
                optional_stack_to_save(
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

    pub fn restore_gamemode(&self) -> PlayerGameMode {
        PlayerGameMode {
            mode: string_to_gamemode(&self.gamemode),
        }
    }

    pub fn restore_inventory(&self) -> InventoryState {
        self.restore_inventory_resolving(save_to_optional_stack)
    }

    pub fn restore_inventory_with_registry(&self, item_registry: &ItemRegistry) -> InventoryState {
        let remap = item_registry.build_id_remap_table(&self.item_id_map);
        self.restore_inventory_resolving(|slot| {
            save_to_optional_stack_with_registry(slot, item_registry, &remap)
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

    pub fn respawn_point(&self) -> Vec3 {
        Vec3::from_array(self.respawn_point)
    }
}

/// 存档数据健康检查与自动修复
pub fn validate_player_data(data: &PlayerSaveData) -> PlayerSaveData {
    let mut data = data.clone();
    let mut repaired = false;

    data.item_id_map.sort_by_key(|(runtime_id, _)| *runtime_id);
    let mut seen_runtime_ids = std::collections::HashSet::new();
    let mut seen_identifiers = std::collections::HashSet::new();
    data.item_id_map.retain(|(runtime_id, identifier)| {
        let valid = ItemId::parse(identifier).is_ok()
            && seen_runtime_ids.insert(*runtime_id)
            && seen_identifiers.insert(identifier.clone());
        if !valid {
            log::warn!(
                "[存档系统] 无效或重复的物品 ID 映射: {} -> {}，已移除",
                runtime_id,
                identifier
            );
            repaired = true;
        }
        valid
    });

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
    if !data.health.is_finite() {
        data.health = 20.0;
        repaired = true;
    } else {
        data.health = data.health.clamp(0.0, 20.0);
    }
    if !data.hunger.is_finite() {
        data.hunger = 20.0;
        repaired = true;
    } else {
        data.hunger = data.hunger.clamp(0.0, 20.0);
    }
    if !data.saturation.is_finite() {
        data.saturation = default_saturation();
        repaired = true;
    } else {
        data.saturation = data.saturation.clamp(0.0, data.hunger);
    }
    if data.respawn_point.iter().any(|value| !value.is_finite()) {
        data.respawn_point = default_respawn_point();
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
            continue;
        }
        if let Some(runtime_id) = slot.runtime_id
            && !data
                .item_id_map
                .iter()
                .any(|(mapped_id, identifier)| *mapped_id == runtime_id && identifier == &slot.item)
        {
            log::warn!(
                "[存档系统] {kind} 中物品 {} 的动态 ID {} 与映射表不一致，将按唯一标识符恢复",
                slot.item,
                runtime_id
            );
            slot.runtime_id = None;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_seven_reload_keeps_inventory_durability_stats_and_respawn_point() {
        let mut inventory = InventoryState::default();
        let mut tool = ItemStack::single(ItemId::item("century_journey:test_tool"));
        tool.instance.durability = Some(23);
        inventory.hotbar.set_stack(2, tool);
        inventory
            .survival
            .set_stack(4, ItemStack::new(ItemId::block("century_journey:dirt"), 17));
        inventory.hotbar.active_index = 2;
        let item_registry = ItemRegistry::default();

        let data = PlayerSaveData::from_runtime(
            Vec3::new(3.0, 72.0, -5.0),
            Quat::from_rotation_y(0.5),
            -0.25,
            &PlayerGameMode {
                mode: GameMode::Survival,
            },
            &inventory,
            &item_registry,
            13.5,
            7.25,
            3.0,
            Vec3::new(8.0, 71.0, 4.0),
        );
        let restored = data.restore_inventory();

        assert_eq!(data.game_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(data.health, 13.5);
        assert_eq!(data.hunger, 7.25);
        assert_eq!(data.saturation, 3.0);
        assert_eq!(data.respawn_point(), Vec3::new(8.0, 71.0, 4.0));
        assert_eq!(data.position, [3.0, 72.0, -5.0]);
        assert_eq!(data.camera_pitch, -0.25);
        assert_eq!(restored.hotbar.active_index, 2);
        assert_eq!(
            restored.hotbar.get_stack(2).map(|stack| stack.count),
            Some(1)
        );
        assert_eq!(
            restored.hotbar.get_stack(2).and_then(ItemStack::durability),
            Some(23)
        );
        assert_eq!(
            restored.survival.get_stack(4).map(|stack| stack.count),
            Some(17)
        );
    }

    #[test]
    fn item_runtime_ids_are_remapped_by_unique_identifier() {
        let wood = crate::shared::identifier::Identifier::new("century_journey", "wood");
        let stone = crate::shared::identifier::Identifier::new("century_journey", "stone");
        let mut saved_registry = ItemRegistry::default();
        saved_registry
            .register(crate::content::item::definition::ItemDefinition::from_block(&wood, "Wood"));
        saved_registry.register(
            crate::content::item::definition::ItemDefinition::from_block(&stone, "Stone"),
        );

        let mut inventory = InventoryState::default();
        inventory
            .hotbar
            .set_stack(0, ItemStack::new(ItemId::new(stone.clone()), 5));
        let data = PlayerSaveData::from_runtime(
            Vec3::ZERO,
            Quat::IDENTITY,
            0.0,
            &PlayerGameMode::default(),
            &inventory,
            &saved_registry,
            20.0,
            20.0,
            5.0,
            Vec3::ZERO,
        );
        assert_eq!(data.hotbar[0].runtime_id, Some(1));

        let mut current_registry = ItemRegistry::default();
        current_registry.register(
            crate::content::item::definition::ItemDefinition::from_block(&stone, "Stone"),
        );
        current_registry
            .register(crate::content::item::definition::ItemDefinition::from_block(&wood, "Wood"));
        let restored = data.restore_inventory_with_registry(&current_registry);

        let stack = restored.hotbar.get_stack(0).unwrap();
        assert_eq!(stack.item, ItemId::new(stone));
        assert_eq!(stack.count, 5);
        assert_eq!(current_registry.runtime_id(&stack.item), Some(0));
    }
}

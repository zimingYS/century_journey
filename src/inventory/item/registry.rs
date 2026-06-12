use std::collections::HashMap;
use bevy::prelude::*;
use crate::inventory::item::definition::{ItemCategory, ItemDefinition};
use crate::inventory::item::id::ItemId;

/// 物品注册表
#[derive(Resource, Default)]
pub struct ItemRegistry {
    /// ItemId → ItemDefinition
    entries: HashMap<ItemId, ItemDefinition>,
    /// 按分类索引
    by_category: HashMap<ItemCategory, Vec<ItemId>>,
}

impl ItemRegistry {
    /// 注册一个物品定义
    pub fn register(&mut self, def: ItemDefinition){
        self.by_category
            .entry(def.category)
            .or_default()
            .push(def.id.clone());

        self.entries.insert(def.id.clone(), def);
    }

    /// 获取物品定义
    pub fn get(&self, id: &ItemId) -> Option<&ItemDefinition> {
        self.entries.get(id)
    }

    /// 检查物品是否已经注册
    pub fn contains(&self, id: &ItemId) -> bool {
        self.entries.contains_key(id)
    }

    /// 获取所有已经注册的物品
    pub fn all_items(&self) -> impl Iterator<Item = &ItemDefinition> {
        self.entries.values()
    }

    /// 获取制定分类下的所有的物品ID
    pub fn items_by_category(&self, category: &ItemCategory) -> &[ItemId]  {
        self.by_category
            .get(&category)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// 已注册物品总数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 从 BlockRegistry 自动生成 ItemDefinition 并注册到 ItemRegistry
pub fn bridge_block_registry_system(
    block_registry: Option<Res<crate::voxel::registry::BlockRegistry>>,
    mut item_registry: ResMut<ItemRegistry>,
) {
    let Some(reg) = block_registry else { return };

    // 只在首次构建时运行
    if !item_registry.is_empty() {
        return;
    }

    let mut count = 0usize;

    for identifier in reg.identifier_to_id.keys() {
        // 跳过空气
        if identifier == "century_journey:air" {
            continue;
        }

        // 从 BlockProperty 读取 display_name，找不到时用 identifier 兜底
        let display_name = reg
            .get_id_by_identifier(identifier)
            .and_then(|id| reg.get(id))
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| identifier.clone());

        let def = ItemDefinition::from_block(identifier, &display_name);
        item_registry.register(def);
        count += 1;
    }

    info!(
        "[ItemRegistry] 从 BlockRegistry 自动注册 {} 个方块物品",
        count
    );
}
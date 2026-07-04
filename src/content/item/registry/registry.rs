use crate::content::item::definition::{ItemCategory, ItemDefinition};
use crate::engine::asset::manager::AssetManager;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;
use std::collections::HashMap;

/// 物品注册表
///
/// Registry 是 RuntimeId (ItemId) 的唯一拥有者。
/// 负责：Identifier -> ItemId -> ItemDefinition 的映射和查询。
/// 不负责：JSON 加载、纹理加载、模型加载。
#[derive(Resource, Default)]
pub struct ItemRegistry {
    /// ItemId → ItemDefinition
    entries: HashMap<ItemId, ItemDefinition>,
    /// 按分类索引
    by_category: HashMap<ItemCategory, Vec<ItemId>>,
}

impl ItemRegistry {
    /// 注册一个物品定义。
    /// 自动从 ItemDefinition::identifier 和 category 推导 ItemId。
    /// 返回分配的 ItemId (RuntimeId)。
    pub fn register(&mut self, def: ItemDefinition) -> ItemId {
        let id = match def.category {
            ItemCategory::Block => ItemId::new(def.identifier.clone()),
            _ => ItemId::new(def.identifier.clone()),
        };
        self.by_category
            .entry(def.category)
            .or_default()
            .push(id.clone());

        self.entries.insert(id.clone(), def);
        id
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

    /// 获取指定分类下的所有的物品ID
    pub fn items_by_category(&self, category: &ItemCategory) -> &[ItemId] {
        self.by_category
            .get(category)
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

    /// 该物品是否为方块类物品
    pub fn is_block(&self, id: &ItemId) -> bool {
        self.get(id)
            .map(|def| def.category == ItemCategory::Block)
            .unwrap_or(false)
    }

    /// 若为方块类物品，返回对应的方块标识符（用于查 BlockRegistry）
    pub fn block_identifier(&self, id: &ItemId) -> Option<&Identifier> {
        self.get(id)?.placeable_block.as_ref()
    }
}

/// 从 JSON 文件加载物品定义到 Registry。
/// Loader 负责读取 JSON，Registry 负责分配 ItemId (RuntimeId)。
pub fn load_item_definitions_system(
    mut item_registry: ResMut<ItemRegistry>,
    asset: Res<AssetManager>,
) {
    let mut count = 0usize;
    for (_path, def) in asset.read_json_dir_recursive_sync::<ItemDefinition>("definitions/items") {
        // Registry 自动从 definition 推导 ItemId (RuntimeId)
        item_registry.register(def);
        count += 1;
    }
    info!("[物品注册] 从 JSON 加载 {} 个物品定义", count);
}

/// 从 BlockRegistry 自动生成 ItemDefinition (Block→Item Bridge)
/// 桥接逻辑属于 Content 层，不属于 Game 层。
pub fn auto_generate_block_items_system(
    block_registry: Option<Res<crate::content::block::registry::BlockRegistry>>,
    mut item_registry: ResMut<ItemRegistry>,
) {
    let Some(reg) = block_registry else { return };

    // 只在首次构建时运行
    let existing_blocks = item_registry.items_by_category(&ItemCategory::Block).len();
    if existing_blocks > 0 {
        return;
    }

    let mut count = 0usize;
    for identifier in reg.identifiers() {
        if identifier == "century_journey:air" {
            continue;
        }

        let display_name = reg
            .get_id_by_identifier(&identifier.to_string())
            .and_then(|id| reg.get(id))
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| identifier.path().to_string());

        let def = ItemDefinition::from_block(identifier, &display_name);
        item_registry.register(def);
        count += 1;
    }

    info!("[物品注册] 从BlockRegistry自动生成{}个方块物品", count);
}

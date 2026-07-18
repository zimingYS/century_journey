use crate::content::item::definition::{ItemCategory, ItemDefinition};
use crate::content::validation::ContentCompilation;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;
use std::collections::HashMap;

/// 物品注册表
///
/// Registry 同时拥有稳定 ItemId 与本次运行的紧凑 u32 RuntimeId。
/// 负责：Identifier -> ItemId -> RuntimeId / ItemDefinition 的映射和查询。
/// 不负责：JSON 加载、纹理加载、模型加载。
#[derive(Resource, Default)]
pub struct ItemRegistry {
    /// ItemId → ItemDefinition
    entries: HashMap<ItemId, ItemDefinition>,
    /// 按分类索引
    by_category: HashMap<ItemCategory, Vec<ItemId>>,
    /// 本次运行内的紧凑动态 ID；只由稳定排序后的注册顺序决定。
    runtime_to_item: Vec<ItemId>,
    item_to_runtime: HashMap<ItemId, u32>,
}

impl ItemRegistry {
    /// 注册一个物品定义。
    /// 自动从 ItemDefinition::identifier 和 category 推导 ItemId。
    /// 返回物品的稳定 ItemId；紧凑 RuntimeId 可通过 `runtime_id` 查询。
    pub fn register(&mut self, def: ItemDefinition) -> ItemId {
        let id = match def.category {
            ItemCategory::Block => ItemId::new(def.identifier.clone()),
            _ => ItemId::new(def.identifier.clone()),
        };
        if self.entries.contains_key(&id) {
            log::error!("[物品注册] 拒绝重复物品标识符: {id}");
            return id;
        }
        let runtime_id = u32::try_from(self.runtime_to_item.len())
            .expect("item registry exceeds u32 runtime ID space");
        self.by_category
            .entry(def.category)
            .or_default()
            .push(id.clone());

        self.entries.insert(id.clone(), def);
        self.item_to_runtime.insert(id.clone(), runtime_id);
        self.runtime_to_item.push(id.clone());
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

    pub fn runtime_id(&self, id: &ItemId) -> Option<u32> {
        self.item_to_runtime.get(id).copied()
    }

    pub fn item_by_runtime_id(&self, runtime_id: u32) -> Option<&ItemId> {
        self.runtime_to_item.get(runtime_id as usize)
    }

    pub fn build_save_id_map(&self) -> Vec<(u32, String)> {
        self.runtime_to_item
            .iter()
            .enumerate()
            .map(|(runtime_id, item)| (runtime_id as u32, item.to_string()))
            .collect()
    }

    pub fn build_id_remap_table(&self, saved_map: &[(u32, String)]) -> HashMap<u32, u32> {
        saved_map
            .iter()
            .filter_map(|(saved_id, identifier)| {
                let item = ItemId::parse(identifier).ok()?;
                self.runtime_id(&item)
                    .map(|current_id| (*saved_id, current_id))
            })
            .collect()
    }
}

/// 从 JSON 文件加载物品定义到 Registry。
/// Loader 负责读取编译产物，Registry 负责分配紧凑 RuntimeId。
pub fn load_item_definitions_system(
    mut item_registry: ResMut<ItemRegistry>,
    compilation: Res<ContentCompilation>,
) {
    if item_registry
        .all_items()
        .any(|definition| definition.category != ItemCategory::Block)
    {
        return;
    }
    let mut count = 0usize;
    for def in compilation.content.items.iter().cloned() {
        // Registry 自动从 definition 推导稳定 ItemId，并按编译顺序分配 RuntimeId。
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

    let mut identifiers = reg.identifiers().cloned().collect::<Vec<_>>();
    identifiers.sort();
    let mut count = 0usize;
    for identifier in &identifiers {
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

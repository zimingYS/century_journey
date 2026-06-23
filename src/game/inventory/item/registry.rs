use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::manager::AssetManager;
use crate::game::inventory::item::definition::{ItemCategory, ItemDefinition};
use crate::game::inventory::item::id::ItemId;
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub fn register(&mut self, def: ItemDefinition) {
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

    /// 获取指定分类下的所有的物品ID
    pub fn items_by_category(&self, category: &ItemCategory) -> &[ItemId] {
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

pub fn load_item_definitions_system(
    mut item_registry: ResMut<ItemRegistry>,
    asset: Res<AssetManager>,
) {
    let items_dir = PathBuf::from("assets/definitions/items");
    if !items_dir.exists() {
        info!("[ItemRegistry] items/ 目录不存在，跳过 JSON 加载");
        return;
    }

    let mut count = 0usize;
    scan_and_load(
        &asset,
        &items_dir,
        &items_dir,
        &mut item_registry,
        &mut count,
    );

    info!("[物品注册] 从 JSON 加载 {} 个物品定义", count);
}

fn scan_and_load(
    asset: &AssetManager,
    base: &PathBuf,
    current: &PathBuf,
    registry: &mut ItemRegistry,
    count: &mut usize,
) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_and_load(asset, base, &path, registry, count);
            continue;
        }

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // 通过 AssetManager 读取 JSON
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let asset_path = format!(
            "definitions/items/{}",
            relative.to_str().unwrap_or("").replace('\\', "/")
        );
        let id = AssetId::default_namespace(&asset_path);
        match asset.read_json_sync::<ItemDefinition>(&id) {
            Ok(mut def) => {
                def.finalize_id();
                registry.register(def);
                *count += 1;
            }
            Err(e) => {
                error!("[物品注册] JSON加载失败 {:?}: {}", path, e);
            }
        }
    }
}

/// 从物品注册表自动生成ItemDefinition
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
    for identifier in reg.identifier_to_id.keys() {
        if identifier == "century_journey:air" {
            continue;
        }

        let display_name = reg
            .get_id_by_identifier(identifier)
            .and_then(|id| reg.get(id))
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| identifier.clone());

        let def = ItemDefinition::from_block(identifier, &display_name);
        item_registry.register(def);
        count += 1;
    }

    info!("[物品注册] 从BlockRegistry自动生成{}个方块物品", count);
}

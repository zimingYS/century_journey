use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use crate::tag::identifier::{TagId, TagRegistryType};

/// 某一注册表类型的标签数据
#[derive(Default)]
pub struct TypedTagRegistry{
    ///  所有实体标识符集合
    tags : HashMap<TagId, HashSet<String>>,
    /// 该实体所属的所有标签集合
    reverse: HashMap<String, HashSet<TagId>>,
}

impl TypedTagRegistry{
    /// 查询某个实体是否拥有指定标签
    pub fn is_tagged(&self, entry_id: &str, tag: &TagId) -> bool{
        self.reverse.get(entry_id).map_or(false, |x| x.contains(tag))
    }

    /// 获取标签包含的所有实体标识符
    pub fn get_tag_entries(&self, tag: &TagId) -> Option<&HashSet<String>> {
        self.tags.get(tag)
    }

    /// 获取实体拥有的所有标签
    pub fn get_entry_tags(&self, entry_id: &str) -> Option<&HashSet<TagId>> {
        self.reverse.get(entry_id)
    }

    /// 获取所有已注册的标签ID
    pub fn all_tag_ids(&self) -> impl Iterator<Item = &TagId> {
        self.tags.keys()
    }

    /// 标签数量
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    /// 添加实体到标签
    pub(crate) fn insert(&mut self, tag: TagId, entry: String) {
        self.tags
            .entry(tag.clone())
            .or_default()
            .insert(entry.clone());
        self.reverse
            .entry(entry)
            .or_default()
            .insert(tag);
    }

    /// 替换标签的所有实体
    pub(crate) fn replace_tag(&mut self, tag: TagId, entries: HashSet<String>) {
        // 清除旧的反向映射
        if let Some(old) = self.tags.remove(&tag) {
            for entry in &old {
                if let Some(tag_set) = self.reverse.get_mut(entry) {
                    tag_set.remove(&tag);
                    if tag_set.is_empty() {
                        self.reverse.remove(entry);
                    }
                }
            }
        }
        // 插入新的
        for entry in entries {
            self.insert(tag.clone(), entry);
        }
    }

    /// 追加实体到标签
    pub(crate) fn append_to_tag(&mut self, tag: TagId, entries: HashSet<String>) {
        for entry in entries {
            self.insert(tag.clone(), entry);
        }
    }
}

/// 全局标签注册表
#[derive(Resource, Default)]
pub struct TagRegistry{
    /// 注册表类型
    pub(crate) registries: HashMap<TagRegistryType, TypedTagRegistry>,
}

/// 统一管理方块、生物群系等不同类型对象的标签系统
/// 支持查询、判断、获取列表、统计等
impl TagRegistry{
    /// 获取指定类型的标签注册表
    pub fn get_registry(&self, registry_type: &TagRegistryType) -> Option<&TypedTagRegistry> {
        self.registries.get(registry_type)
    }

    /// 获取或创建指定类型的标签注册表
    pub fn get_or_create_registry(&mut self, registry_type: TagRegistryType, )
    -> &mut TypedTagRegistry {
        self.registries.entry(registry_type).or_default()
    }

    /// 查询方块是否拥有指定标签
     pub fn is_block_tagged(&self, block_id: &str, tag: &TagId) -> bool {
        self.registries
            .get(&TagRegistryType::Block)
            .map_or(false, |r| r.is_tagged(block_id, tag))
    }

    /// 通过方块运行时ID查询是否拥有指定标签
    pub fn is_block_id_tagged(
        &self, block_runtime_id: u16,
        tag: &TagId,
        block_registry: &crate::voxel::registry::BlockRegistry,
    ) -> bool {
        block_registry
            .get_identifier_by_id(block_runtime_id)
            .map(|id| self.is_block_tagged(id, tag))
            .unwrap_or(false)
    }

    /// 获取方块标签包含的所有方块标识符
    pub fn get_block_tag_entries(&self, tag: &TagId) -> Vec<String> {
        self.registries
            .get(&TagRegistryType::Block)
            .and_then(|r| r.get_tag_entries(tag))
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取方块标签包含的所有方块运行时ID
    pub fn get_block_tag_ids(
        &self,
        tag: &TagId,
        block_registry: &crate::voxel::registry::BlockRegistry,
    ) -> Vec<u16> {
        self.get_block_tag_entries(tag)
            .iter()
            .filter_map(|id| block_registry.get_id_by_identifier(id))
            .collect()
    }

    /// 获取方块拥有的所有标签
    pub fn get_block_tags(&self, block_id: &str) -> Vec<&TagId> {
        self.registries
            .get(&TagRegistryType::Block)
            .and_then(|r| r.get_entry_tags(block_id))
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }

    /// 查询生物群系是否拥有指定标签
    pub fn is_biome_tagged(&self, biome_id: &str, tag: &TagId) -> bool {
        self.registries
            .get(&TagRegistryType::Biome)
            .map_or(false, |r| r.is_tagged(biome_id, tag))
    }

    /// 获取生物群系标签包含的所有群系标识符
    pub fn get_biome_tag_entries(&self, tag: &TagId) -> Vec<String> {
        self.registries
            .get(&TagRegistryType::Biome)
            .and_then(|r| r.get_tag_entries(tag))
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取生物群系拥有的所有标签
    pub fn get_biome_tags(&self, biome_id: &str) -> Vec<&TagId> {
        self.registries
            .get(&TagRegistryType::Biome)
            .and_then(|r| r.get_entry_tags(biome_id))
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }

    /// 检查标签是否存在
    pub fn tag_exists(&self, registry_type: &TagRegistryType, tag: &TagId) -> bool {
        self.registries
            .get(registry_type)
            .map_or(false, |r| r.get_tag_entries(tag).is_some())
    }

    /// 获取某一类型的所有标签ID
    pub fn all_tags(&self, registry_type: &TagRegistryType) -> Vec<&TagId> {
        self.registries
            .get(registry_type)
            .map(|r| r.all_tag_ids().collect())
            .unwrap_or_default()
    }

    /// 获取某一类型注册表的标签总数
    pub fn tag_count(&self, registry_type: &TagRegistryType) -> usize {
        self.registries
            .get(registry_type)
            .map(|r| r.tag_count())
            .unwrap_or(0)
    }

    /// 从 BlockProperty.tags 自动填充 TagRegistry
    ///
    /// 在加载所有方块 JSON 后调用，根据每个方块的 tags 字段自动创建/追加标签条目。
    /// 与 assets/definitions/tags/ 目录中已有的 JSON Tag 合并，不覆盖。
    pub fn auto_populate_from_block_tags(
        &mut self,
        block_registry: &crate::voxel::registry::BlockRegistry,
    ) {
        let typed = self.get_or_create_registry(TagRegistryType::Block);
        let mut added = 0usize;

        for (identifier, _runtime_id) in &block_registry.identifier_to_id {
            if identifier == "century_journey:air" { continue; }

            let tags = block_registry
                .get_id_by_identifier(identifier)
                .and_then(|id| block_registry.get(id))
                .map(|prop| prop.tags.as_slice())
                .unwrap_or(&[]);

            if tags.is_empty() { continue; }

            for tag_str in tags {
                // "mineable/pickaxe" → TagId { ns: "mineable", path: "pickaxe" }
                // "stone_like" → TagId { ns: "century_journey", path: "stone_like" }
                let tag_id = if tag_str.contains('/') {
                    let parts: Vec<&str> = tag_str.split('/').collect();
                    if parts.len() == 2 {
                        TagId::new(parts[0], parts[1])
                    } else {
                        TagId::new("century_journey", tag_str)
                    }
                } else {
                    TagId::new("century_journey", tag_str)
                };
                typed.insert(tag_id, identifier.clone());
                added += 1;
            }
        }

        info!(
            "[Tag] 从 BlockProperty.tags 自动构建 {} 条标签映射",
            added
        );
    }
}
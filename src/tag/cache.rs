use std::collections::HashSet;
use bevy::prelude::*;
use crate::tag::registry::TagRegistry;
use crate::voxel::registry::BlockRegistry;

/// 缓存的标签数据
#[derive(Resource, Clone, Default)]
pub struct CachedTagCache(pub TagCache);

/// 异步标签缓存
#[derive(Clone, Default)]
pub struct TagCache {
    /// 方块标签缓存
    block_tags: std::collections::HashMap<String, HashSet<u16>>,
}

impl TagCache {
    /// 从TagRegistry BlockRegistry构建缓存
    pub fn build(tag_registry: &TagRegistry, block_registry: &BlockRegistry) -> Self {
        let mut cache = Self::default();

        // 缓存所有方块标签
        for tag_id in tag_registry.all_tags(&crate::tag::identifier::TagRegistryType::Block) {
            let entries = tag_registry.get_block_tag_entries(tag_id);
            let runtime_ids: HashSet<u16> = entries
                .iter()
                .filter_map(|id| block_registry.get_id_by_identifier(id))
                .collect();

            cache.block_tags.insert(tag_id.to_full(), runtime_ids);
        }

        log::info!(
            "[标签缓存] 已缓存 {} 个方块标签",
            cache.block_tags.len()
        );

        cache
    }

    /// 查询方块运行时ID是否拥有指定标签
    pub fn is_block_in_tag(&self, block_runtime_id: u16, tag_full: &str) -> bool {
        self.block_tags
            .get(tag_full)
            .map_or(false, |ids| ids.contains(&block_runtime_id))
    }

    /// 获取指定标签下的所有方块运行时ID集合
    pub fn get_block_tag_ids(&self, tag_full: &str) -> Option<&HashSet<u16>> {
        self.block_tags.get(tag_full)
    }

    /// 检查标签是否已缓存
    pub fn has_tag(&self, tag_full: &str) -> bool {
        self.block_tags.contains_key(tag_full)
    }

    /// 获取已缓存的所有标签名
    pub fn cached_tag_names(&self) -> impl Iterator<Item = &String> {
        self.block_tags.keys()
    }
}
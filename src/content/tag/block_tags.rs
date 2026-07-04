use crate::content::block::registry::BlockRegistry;
use crate::shared::tag::identifier::{TagId, TagRegistryType};
use crate::shared::tag::registry::TagRegistry;
use log::info;
use std::collections::HashSet;

/// 通过方块运行时ID查询是否拥有指定标签
pub fn is_block_id_tagged(
    tag_registry: &TagRegistry,
    block_runtime_id: u16,
    tag: &TagId,
    block_registry: &BlockRegistry,
) -> bool {
    block_registry
        .get_identifier_by_id(block_runtime_id)
        .map(|id| tag_registry.is_block_tagged(&id.to_string(), tag))
        .unwrap_or(false)
}

/// 获取方块标签包含的所有方块运行时ID
pub fn get_block_tag_ids(
    tag_registry: &TagRegistry,
    tag: &TagId,
    block_registry: &BlockRegistry,
) -> Vec<u16> {
    tag_registry
        .get_block_tag_entries(tag)
        .iter()
        .filter_map(|id| block_registry.get_id_by_identifier(id))
        .collect()
}

/// 从 BlockProperty.tags 自动填充 TagRegistry
///
/// 在加载所有方块 JSON 后调用，根据每个方块的 tags 字段自动创建/追加标签条目。
/// 与 assets/definitions/tags/ 目录中已有的 JSON Tag 合并，不覆盖。
pub fn auto_populate_from_block_tags(
    tag_registry: &mut TagRegistry,
    block_registry: &BlockRegistry,
) {
    let typed = tag_registry.get_or_create_registry(TagRegistryType::Block);
    let mut added = 0usize;

    for identifier in block_registry.identifiers() {
        if identifier == "century_journey:air" {
            continue;
        }

        let tags = block_registry
            .get_id_by_identifier(&identifier.to_string())
            .and_then(|id| block_registry.get(id))
            .map(|prop| prop.tags.as_slice())
            .unwrap_or(&[]);

        if tags.is_empty() {
            continue;
        }

        for tag_str in tags {
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
            typed.insert(tag_id, identifier.to_string());
            added += 1;
        }
    }

    info!("[Tag] 从 BlockProperty.tags 自动构建 {} 条标签映射", added);
}

/// 验证标签中的条目是否在方块注册表中存在
pub fn validate_tags_against_block_registry(
    tag_registry: &TagRegistry,
    block_registry: &BlockRegistry,
) {
    let Some(typed) = tag_registry.get_registry(&TagRegistryType::Block) else {
        return;
    };

    let mut missing_count = 0usize;
    for tag_id in typed.all_tag_ids() {
        if let Some(entries) = typed.get_tag_entries(tag_id) {
            for entry in entries {
                if block_registry.get_id_by_identifier(entry).is_none() {
                    log::warn!(
                        "[标签验证] 标签 {} 中的条目 '{}' 在方块注册表中不存在",
                        tag_id,
                        entry
                    );
                    missing_count += 1;
                }
            }
        }
    }

    if missing_count == 0 {
        log::info!("[标签验证] 所有标签条目验证通过");
    } else {
        log::warn!("[标签验证] 共发现 {} 个不匹配的条目", missing_count);
    }
}

/// 缓存方块标签数据
#[derive(Clone, Default)]
pub struct TagCache {
    block_tags: std::collections::HashMap<String, HashSet<u16>>,
}

impl TagCache {
    pub fn build(tag_registry: &TagRegistry, block_registry: &BlockRegistry) -> Self {
        let mut cache = Self::default();
        for tag_id in tag_registry.all_tags(&TagRegistryType::Block) {
            let entries = tag_registry.get_block_tag_entries(tag_id);
            let runtime_ids: HashSet<u16> = entries
                .iter()
                .filter_map(|id| block_registry.get_id_by_identifier(id))
                .collect();
            cache.block_tags.insert(tag_id.to_full(), runtime_ids);
        }
        log::info!("[标签缓存] 已缓存 {} 个方块标签", cache.block_tags.len());
        cache
    }

    pub fn is_block_in_tag(&self, block_runtime_id: u16, tag_full: &str) -> bool {
        self.block_tags
            .get(tag_full)
            .map_or(false, |ids| ids.contains(&block_runtime_id))
    }

    pub fn has_tag(&self, tag_full: &str) -> bool {
        self.block_tags.contains_key(tag_full)
    }

    pub fn get_block_tag_ids(&self, tag_full: &str) -> Option<&HashSet<u16>> {
        self.block_tags.get(tag_full)
    }
}

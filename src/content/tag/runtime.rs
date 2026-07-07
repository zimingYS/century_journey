use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagId;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// 运行时标签注册表 — 标签系统的唯一真值（Single Source of Truth）
///
/// 所有 Definition / TagDefinition 经过 Compiler 编译后，最终生成此结构。
/// Gameplay 永远只依赖 RuntimeTagRegistry，不依赖任何 Definition。
///
/// 内部存储:
/// - `tags`: HashMap<TagId, HashSet<u16>> — 每个标签包含的运行时 ID 集合
/// - `tags_of`: HashMap<u16, HashSet<TagId>> — 反向索引（每个 ID 属于哪些标签）
#[derive(Resource, Default, Clone)]
pub struct RuntimeTagRegistry {
    /// TagId → 运行时 ID 集合
    tags: HashMap<TagId, HashSet<u16>>,
    /// 运行时 ID → TagId 集合 (反向缓存)
    reverse: HashMap<u16, HashSet<TagId>>,
}

impl RuntimeTagRegistry {
    // ─── 构建 (Compiler 使用) ──────────────────────────

    /// 插入一个标签及其运行时 ID 集合
    pub(crate) fn insert(&mut self, tag: TagId, ids: HashSet<u16>) {
        for &id in &ids {
            self.reverse.entry(id).or_default().insert(tag.clone());
        }
        self.tags.insert(tag, ids);
    }

    // ─── 查询 API ──────────────────────────────────────

    /// 检查指定运行时 ID 是否属于某个标签
    pub fn contains(&self, tag: &TagId, runtime_id: u16) -> bool {
        self.tags
            .get(tag)
            .is_some_and(|ids| ids.contains(&runtime_id))
    }

    /// 获取某个标签的所有运行时 ID 引用（带生命周期）
    pub fn iter(&self, tag: &TagId) -> Option<impl Iterator<Item = &u16>> {
        self.tags.get(tag).map(|ids| ids.iter())
    }

    /// 获取标签包含的 ID 数量
    pub fn len(&self, tag: &TagId) -> usize {
        self.tags.get(tag).map_or(0, |ids| ids.len())
    }

    /// 标签是否为空
    pub fn is_empty(&self, tag: &TagId) -> bool {
        self.len(tag) == 0
    }

    /// 获取所有标签 ID
    pub fn all_tags(&self) -> impl Iterator<Item = &TagId> {
        self.tags.keys()
    }

    /// 获取某个运行时 ID 所属的所有标签
    pub fn tags_of(&self, runtime_id: u16) -> Option<&HashSet<TagId>> {
        self.reverse.get(&runtime_id)
    }

    /// 标签总数
    pub fn total_tags(&self) -> usize {
        self.tags.len()
    }

    /// 获取标签的完整 ID 集合 (用于批量操作)
    pub fn get_ids(&self, tag: &TagId) -> Option<&HashSet<u16>> {
        self.tags.get(tag)
    }
}

/// 物品标签索引 — 与 RuntimeTagRegistry 平行存在。
///
/// RuntimeTagRegistry 面向方块调色板 (u16)，服务于 chunk 存储优化。
/// 物品没有紧凑数字 ID 的需求，ItemId 本身已经是 HashMap key，
/// 所以物品标签直接用 Identifier 索引，不走 u16 转换。
#[derive(Resource, Default, Clone)]
pub struct ItemTagIndex {
    tags: HashMap<TagId, HashSet<ItemId>>,
}

impl ItemTagIndex {
    pub(crate) fn insert(&mut self, tag: TagId, items: HashSet<ItemId>) {
        self.tags.insert(tag, items);
    }

    /// 检查物品是否属于某个标签
    pub fn contains(&self, tag: &TagId, item: &ItemId) -> bool {
        self.tags.get(tag).is_some_and(|set| set.contains(item))
    }

    /// 获取标签下所有物品
    pub fn get_items(&self, tag: &TagId) -> Option<&HashSet<ItemId>> {
        self.tags.get(tag)
    }

    pub fn total_tags(&self) -> usize {
        self.tags.len()
    }
}

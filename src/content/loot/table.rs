use crate::shared::item_id::ItemId;
use crate::shared::random::RandomSource;
use serde::{Deserialize, Serialize};

// LootTable 返回 tokens (ItemId, count) 而非 ItemStack，
// 以避免 Content 层依赖 Game 层。ItemStack 转换由 Game 层负责。

/// 掉落结果：物品ID和数量
pub type LootDrop = (ItemId, u32);

/// 单个掉落条目（JSON 可序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootEntry {
    /// 掉落物品
    pub item: ItemId,
    /// 掉落数量（最小）
    #[serde(default = "default_min_count")]
    pub min_count: u32,
    /// 掉落数量（最大）
    #[serde(default = "default_max_count")]
    pub max_count: u32,
    /// 掉落概率 (0.0 ~ 1.0)
    #[serde(default = "default_chance")]
    pub chance: f32,
}

impl LootEntry {
    /// 创建必定掉落固定数量的条目
    pub fn guaranteed(item: ItemId, count: u32) -> Self {
        Self {
            item,
            min_count: count,
            max_count: count,
            chance: 1.0,
        }
    }

    /// 创建必定掉落范围数量的条目
    pub fn ranged(item: ItemId, min: u32, max: u32) -> Self {
        Self {
            item,
            min_count: min,
            max_count: max,
            chance: 1.0,
        }
    }
}

/// 掉落表 — 一组掉落条目（JSON 可序列化）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LootTable {
    #[serde(default)]
    pub entries: Vec<LootEntry>,
}

impl LootTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 单物品掉落
    pub fn single(item: ItemId, count: u32) -> Self {
        Self {
            entries: vec![LootEntry::guaranteed(item, count)],
        }
    }

    /// 添加条目
    pub fn with(mut self, entry: LootEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// 根据概率计算实际掉落的物品列表
    /// 返回 (ItemId, count) tokens，由 Game 层转换为 ItemStack
    pub fn roll(&self, rng: &mut dyn RandomSource) -> Vec<LootDrop> {
        let mut results = Vec::new();
        for entry in &self.entries {
            if rng.next_f32() < entry.chance {
                let count = if entry.min_count == entry.max_count {
                    entry.min_count
                } else {
                    rng.range_u32_inclusive(entry.min_count, entry.max_count)
                };
                if count > 0 {
                    results.push((entry.item.clone(), count));
                }
            }
        }
        results
    }
}

// ─── serde 默认值 ────────────────────────────────────
fn default_min_count() -> u32 {
    1
}
fn default_max_count() -> u32 {
    1
}
fn default_chance() -> f32 {
    1.0
}

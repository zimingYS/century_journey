use crate::game::inventory::item::id::ItemId;
use crate::game::inventory::item::stack::ItemStack;

/// 单个掉落条目
#[derive(Debug, Clone)]
pub struct LootEntry {
    /// 掉落物品
    pub item: ItemId,
    /// 掉落数量（最小）
    pub min_count: u32,
    /// 掉落数量（最大）
    pub max_count: u32,
    /// 掉落概率 (0.0 ~ 1.0)
    pub chance: f32,
}

impl LootEntry {
    /// 创建必定掉落固定数量的条目
    pub fn guaranteed(item: ItemId, count: u32) -> Self {
        Self { item, min_count: count, max_count: count, chance: 1.0 }
    }

    /// 创建必定掉落范围数量的条目
    pub fn ranged(item: ItemId, min: u32, max: u32) -> Self {
        Self { item, min_count: min, max_count: max, chance: 1.0 }
    }
}

/// 掉落表 — 一组掉落条目
#[derive(Debug, Clone, Default)]
pub struct LootTable {
    pub entries: Vec<LootEntry>,
}

impl LootTable {
    pub fn new() -> Self { Self { entries: Vec::new() } }

    /// 单物品掉落
    pub fn single(item: ItemId, count: u32) -> Self {
        Self { entries: vec![LootEntry::guaranteed(item, count)] }
    }

    /// 添加条目
    pub fn with(mut self, entry: LootEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// 根据概率计算实际掉落的物品列表
    pub fn roll(&self) -> Vec<ItemStack> {
        let mut results = Vec::new();
        for entry in &self.entries {
            if rand::random::<f32>() < entry.chance {
                let count = if entry.min_count == entry.max_count {
                    entry.min_count
                } else {
                    entry.min_count + (rand::random::<u32>() % (entry.max_count - entry.min_count + 1))
                };
                if count > 0 {
                    results.push(ItemStack::new(entry.item.clone(), count));
                }
            }
        }
        results
    }
}
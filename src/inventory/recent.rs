use crate::inventory::item::id::ItemId;

/// 最近使用的物品
#[derive(Debug, Clone)]
pub struct RecentItems {
    pub items: Vec<ItemId>,
    pub max_count: usize,
}

impl Default for RecentItems {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            max_count: 9,
        }
    }
}

impl RecentItems {
    /// 添加一个物品到最近使用
    pub fn push(&mut self, item_id: ItemId) {
        if item_id.is_air() { return; }
        // 去重
        self.items.retain(|id| id != &item_id);
        // 头部插入
        self.items.insert(0, item_id);
        // 截断
        self.items.truncate(self.max_count);
    }
}
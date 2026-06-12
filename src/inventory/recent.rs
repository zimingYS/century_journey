use crate::inventory::item::id::ItemId;
use crate::inventory::item::stack::ItemStack;

#[derive(Debug, Clone)]
pub struct RecentItems {
    /// 最近使用的物品堆叠
    pub items: Vec<ItemStack>,
    /// 最大保留数量
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
    pub fn push_stack(&mut self, stack: ItemStack) {
        if stack.is_empty() {
            return;
        }
        // 去重（按物品类型）
        self.items.retain(|s| s.item != stack.item);
        // 头部插入
        self.items.insert(0, stack);
        // 截断
        self.items.truncate(self.max_count);
    }

    /// 添加一个物品到最近使用（兼容旧 API，count=1）
    pub fn push(&mut self, item_id: ItemId) {
        if item_id.is_air() {
            return;
        }
        // 去重
        self.items.retain(|s| s.item != item_id);
        // 头部插入
        self.items.insert(0, ItemStack::single(item_id));
        // 截断
        self.items.truncate(self.max_count);
    }
}
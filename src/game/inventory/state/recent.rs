//! 记录最近使用物品的确定顺序，供客户端展示而不反向拥有规则。

use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;

#[derive(Debug, Clone)]
/// 按最近使用顺序保存去重后的物品堆快照。
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
        self.items.retain(|s| s.item != stack.item);
        self.items.insert(0, stack);
        self.items.truncate(self.max_count);
    }

    /// 添加一个物品到最近使用（兼容旧 API，count=1）
    pub fn push(&mut self, item_id: ItemId) {
        if item_id.is_air() {
            return;
        }
        self.items.retain(|s| s.item != item_id);
        self.items.insert(0, ItemStack::single(item_id));
        self.items.truncate(self.max_count);
    }
}

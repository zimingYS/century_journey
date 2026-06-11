use crate::inventory::item::id::ItemId;

/// 物品堆叠
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack{
    pub item_id: ItemId,
    pub count: u32,
}

impl ItemStack {
    // 由数量创建物品
    pub fn new(item_id: ItemId, count: u32) -> Self {
        Self{ item_id, count }
    }

    // 创建一个物品
    pub fn single(item_id: ItemId) -> Self {
        Self::new(item_id, 1)
    }

    // 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.item_id.is_air()
    }
}

// 初始化物品堆叠为空气
impl Default for ItemStack {
    fn default() -> Self {
        Self {
            item_id: ItemId::air(),
            count: 0,
        }
    }
}
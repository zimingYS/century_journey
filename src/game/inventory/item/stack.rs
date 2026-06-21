use crate::game::inventory::item::id::ItemId;

// 这边先临时使用全部物品最大堆叠64个
// 以后会根据物品种类进行最大堆叠分类
// 故现在先写到这边
/// 物品最大堆叠数
pub const MAX_STACK_SIZE: u32 = 64;

/// 物品堆叠
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack{
    /// 物品类型
    pub item: ItemId,
    /// 堆叠数量
    pub count: u32,
}

impl ItemStack {
    // 由数量创建物品
    pub fn new(item: ItemId, count: u32) -> Self {
        Self{ item, count }
    }

    // 创建一个物品
    pub fn single(item_id: ItemId) -> Self {
        Self::new(item_id, 1)
    }


    /// 创建空堆叠
    pub fn empty() -> Self {
        Self {
            item: ItemId::air(),
            count: 0,
        }
    }

    // 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.item.is_air()
    }

    /// 判断是否与另一个堆叠同种物品
    pub fn is_same_item(&self, other: &ItemStack) -> bool {
        self.item == other.item
    }

    /// 判断是否可以与另一个堆叠合并
    pub fn can_merge(&self, other: &ItemStack) -> bool {
        // 必须是相同物品且小于最大可堆叠物品上限
        if !self.is_same_item(other) {
            return false;
        }
        self.count + other.count <= MAX_STACK_SIZE
    }

    /// 从另一个堆叠合并尽可能多的物品到自身
    pub fn merge_from(&mut self, other: &mut ItemStack) -> u32 {
        if !self.is_same_item(other) || other.is_empty() {
            return 0;
        }
        let available = MAX_STACK_SIZE - self.count;
        let to_move = available.min(other.count);
        self.count += to_move;
        other.count -= to_move;
        if other.count == 0 {
            *other = ItemStack::empty();
        }
        to_move
    }

    /// 将自身对半拆分，返回拆分出的新堆叠
    pub fn split_half(&mut self) -> Option<ItemStack>{
        if self.count <= 1 {
            return None;
        }
        let half = self.count / 2;
        self.count -= half;
        Some(ItemStack::new(self.item.clone(), half))
    }

    /// 从自身取走指定数量，返回取走的堆叠
    pub fn take(&mut self, amount: u32) -> Option<ItemStack> {
        if amount == 0 || self.is_empty() {
            return None;
        }
        let taken = amount.min(self.count);
        self.count -= taken;
        let result = ItemStack::new(self.item.clone(), taken);
        if self.count == 0 {
            *self = ItemStack::empty();
        }
        Some(result)
    }

    /// 获取物品类型引用（兼容旧代码中的 `.item_id` 字段访问）
    pub fn item_id(&self) -> &ItemId {
        &self.item
    }

    /// 获取方块标识符引用
    pub fn as_block_id(&self) -> Option<&str> {
        self.item.as_block_id()
    }

    /// 检查剩余空间（还能合并多少个同种物品）
    pub fn remaining_space(&self) -> u32 {
        MAX_STACK_SIZE.saturating_sub(self.count)
    }

    /// 是否已达最大堆叠数
    pub fn is_full(&self) -> bool {
        self.count >= MAX_STACK_SIZE
    }

    /// 返回显示用的数量文本（空/1 时不显示）
    pub fn count_text(&self) -> Option<String> {
        if self.is_empty() || self.count <= 1 {
            None
        } else {
            Some(self.count.to_string())
        }
    }

    /// 获取最大堆叠数
    pub const MAX_STACK_SIZE: u32 = 64;
}

// 初始化物品堆叠为空气
impl Default for ItemStack {
    fn default() -> Self {
        Self::empty()
    }
}